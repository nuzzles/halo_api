//! Stateful native direct readers and scan-frame context ownership.
use super::*;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NativeReaderProfileError {
    #[error("native profile value cannot be represented by this reader: {0}")]
    Width(&'static str),
    #[error("native traversal policy not supported by this entry point: {0}")]
    Policy(&'static str),
}
impl NativeScanProfile {
    // Widths are consumed from NativeComponentWidths only when reached on wire.
    fn component_switch_encoding(&self) -> Result<PositionEncoding, NativeReaderProfileError> {
        let mut switches = self.clone();
        switches.movement.world_object.index_bits = 0;
        switches.movement.world_object.axis_bits = [0; 3];
        switches.movement.traversal.index_bits = 0;
        switches.movement.traversal.axis_bits = [0; 3];
        switches.movement.delta_axis_width = 0;
        switches.component_encoding()
    }
    /// Adapt raw metadata to the existing component reader's address-sized widths.
    /// The profile retains original u64 values if this conversion is refused.
    pub fn component_encoding(&self) -> Result<PositionEncoding, NativeReaderProfileError> {
        let width =
            |w: u64, name| usize::try_from(w).map_err(|_| NativeReaderProfileError::Width(name));
        let axes = |w: [u64; 3], name| -> Result<[usize; 3], NativeReaderProfileError> {
            Ok([width(w[0], name)?, width(w[1], name)?, width(w[2], name)?])
        };
        let world = &self.movement.world_object;
        let traversal = &self.movement.traversal;
        let world_axes = axes(world.axis_bits, "world axes")?;
        Ok(PositionEncoding {
            bodies: ComponentBodyPolicy {
                ability_anchor: self.grammar.ability_anchor_body,
                mobility: self.grammar.mobility_action_body,
                mobility_extra_bits: self.movement.mobility_action_extra_bits,
            },
            index_bits: width(world.index_bits, "world index")?,
            world_axis_bits: Some(world_axes),
            handle_bits: width(traversal.index_bits, "traversal index")?,
            traversal_axis_bits: axes(traversal.axis_bits, "traversal axes")?,
            default_axis_bits: [22; 3],
            region_axis_bits: [(world.region, world_axes)].into(),
            // Native deltaAxisW treats zero as absence of an override and
            // selects the corresponding axis from the traversal descriptor.
            delta_axis_bits: if self.movement.delta_axis_width == 0 {
                axes(traversal.axis_bits, "delta traversal axes")?
            } else {
                [width(self.movement.delta_axis_width, "delta axes")?; 3]
            },
            calibrated_skip: self.movement.calibrated_skip,
            baseline_scope: self.grammar.baseline_scope,
            full_precision: self.movement.full_precision,
            writer_absolute: self.grammar.writer_absolute,
            delta_handle_tail: self.movement.delta_has_handle_tail,
        })
    }
}
#[derive(Debug, Clone, Default)]
pub struct NativeReaderContext {
    pub profile: NativeScanProfile,
    pub observer: Option<NativeFilmObserver>,
}
#[derive(Debug, Clone)]
pub struct NativeFrameConfig {
    pub extra_fields: bool,
    pub id_low_bits: i64,
    pub id_base: u32,
    pub new_default_state_bits: i64,
    pub packet_preamble_bits: i64,
    pub context: NativeReaderContext,
}
impl Default for NativeFrameConfig {
    fn default() -> Self {
        Self {
            extra_fields: false,
            id_low_bits: 13,
            id_base: 0,
            new_default_state_bits: 0,
            packet_preamble_bits: 2,
            context: NativeReaderContext::default(),
        }
    }
}
impl NativeFrameConfig {
    /// Install only the frame's profile/observer on a reader. Record metadata
    /// remains for a frame traversal; it is not a direct component parameter.
    pub fn reader<'a>(&self, data: &'a [u8]) -> NativeFilmReader<'a> {
        NativeFilmReader::with_context(data, self.context.clone())
    }
}
#[derive(Debug)]
pub struct NativeFilmReader<'a> {
    data: &'a [u8],
    cursor: NativeFilmBits<'a>,
    context: NativeReaderContext,
    slot: u32,
    accumulator: Option<&'a mut FilmWorld>,
}
impl<'a> NativeFilmReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self::with_context(data, NativeReaderContext::default())
    }
    pub fn with_context(data: &'a [u8], context: NativeReaderContext) -> Self {
        Self {
            data,
            cursor: NativeFilmBits::new(data),
            context,
            slot: 0,
            accumulator: None,
        }
    }
    pub fn context(&self) -> NativeReaderContext {
        self.context.clone()
    }
    pub fn replace_context(&mut self, context: NativeReaderContext) -> NativeReaderContext {
        std::mem::replace(&mut self.context, context)
    }
    /// Copy the native scan profile. Allocated calibration maps retain their
    /// shared identity; use NativeScanProfile::snapshot for an independent copy.
    pub fn profile(&self) -> NativeScanProfile {
        self.context.profile.clone()
    }
    /// Install only the profile and return its predecessor. Observer, cursor,
    /// capture slot and optional position accumulator are preserved.
    pub fn replace_profile(&mut self, profile: NativeScanProfile) -> NativeScanProfile {
        std::mem::replace(&mut self.context.profile, profile)
    }
    /// Install only the observer and return its predecessor, retaining shared
    /// receiver identity. None disables delivery without resetting reader state.
    pub fn replace_observer(
        &mut self,
        observer: Option<NativeFilmObserver>,
    ) -> Option<NativeFilmObserver> {
        std::mem::replace(&mut self.context.observer, observer)
    }
    /// Address-sized projection for callers using component source ranges.
    /// Panics if the signed cursor is negative or exceeds this target's address
    /// space; `native_bit_position` always preserves the original value.
    pub fn bit_position(&self) -> usize {
        usize::try_from(self.native_bit_position()).expect("native cursor outside address space")
    }
    pub fn native_bit_position(&self) -> i64 {
        self.cursor.position()
    }
    pub fn remaining_bits(&self) -> i64 {
        self.cursor.remaining()
    }
    /// Set a native signed cursor without resetting context or accumulated state.
    pub fn set_native_bit_position(&mut self, bit: i64) {
        self.cursor.set_position(bit);
    }
    /// Set an address-sized cursor using the pinned Go int conversion. Values
    /// above i64::MAX wrap; use native_bit_position to inspect them afterwards.
    pub fn set_bit_position(&mut self, bit: usize) {
        self.set_native_bit_position(bit as i64);
    }
    /// Native Skip, including negative movement and signed wrapping.
    pub fn skip_signed(&mut self, width: i64) {
        self.cursor.skip(width);
    }
    /// Attach or detach the optional native position accumulator. The reader
    /// borrows the world; profile/observer replacement does not reset it. Without
    /// an accumulator delta publications remain relative and baselines are silent.
    /// Frame reads use this separately from their entity traversal world.
    pub fn replace_position_accumulator(
        &mut self,
        world: Option<&'a mut FilmWorld>,
    ) -> Option<&'a mut FilmWorld> {
        std::mem::replace(&mut self.accumulator, world)
    }
    pub fn position_accumulator(&self) -> Option<&FilmWorld> {
        self.accumulator.as_deref()
    }
    pub fn source_bits(&self) -> usize {
        self.data.len() * 8
    }
    pub fn padded_bits(&self) -> usize {
        super::bits::padded_from_native(self.native_bit_position(), self.source_bits())
    }
    pub fn capture_slot(&self) -> u32 {
        self.slot
    }
    pub fn set_capture_slot(&mut self, slot: u32) {
        self.slot = slot;
    }
    /// Native zero-tail primitive. See NativeFilmBits::read_wide for signed
    /// endpoint wrapping and panic semantics. The Option return is retained for
    /// compatibility; successful native reads always return Some.
    pub fn read_bits(&mut self, width: usize) -> Option<u64> {
        Some(self.read_bits_wide(width as u64))
    }
    /// Full pinned Go uint width domain, including on wasm32.
    pub fn read_bits_wide(&mut self, width: u64) -> u64 {
        self.cursor.read_wide(width)
    }
    pub fn skip(&mut self, width: usize) -> Option<()> {
        self.skip_signed(width as i64);
        Some(())
    }
    /// Engine signed variable-width integer, using this same signed cursor.
    /// Selector/payload consumption remains visible if a native read panics.
    pub fn read_signed_variable(&mut self) -> Option<i32> {
        let width = 8 << self.read_bits_wide(2);
        let value = self.read_bits_wide(width) as u32;
        Some(match width {
            8 => value as i8 as i32,
            16 => value as i16 as i32,
            _ => value as i32,
        })
    }
    /// Native ReadQuantizedVec3 on this reader's current signed cursor. Unlike
    /// the bounded helper, successfully consumed axes remain consumed if a later
    /// native read panics. This primitive emits no component observer callbacks.
    pub fn read_quantized_vec3(&mut self, width: u64, bounds: FilmQuantizationRange) -> [f32; 3] {
        let scale = if width < 64 {
            (1_u64 << width) as f32
        } else {
            0.0
        };
        bounds.map(|[min, max]| {
            let q = self.read_bits_wide(width) as f32;
            let step = (max - min) / scale;
            q.mul_add(step, min) + step * 0.5
        })
    }
    /// Direct consumeByName semantics. Retained fields include callback candidates;
    /// only installed observer hooks receive publications. An attached world
    /// seeds absolutes, accumulates seeded deltas and re-emits saved baselines.
    /// Accumulation still runs when the position hook is absent.
    pub fn read_component(
        &mut self,
        name: &str,
        level: u32,
        archetype: u32,
    ) -> Result<(Option<bool>, DecodedComponent), NativeReaderProfileError> {
        self.read_component_with_width_limit(name, level, archetype, u64::MAX)
    }

    /// Read ti37 defaults with native padding and live MPP widths. Callers must
    /// independently check the resulting cursor against their record boundary.
    pub fn read_equipment_default_state(
        &mut self,
    ) -> Result<(Option<bool>, DecodedComponent), NativeReaderProfileError> {
        self.read_world_default_state(37)
    }
    /// Native ti42 defaults, including raw magazine fields and MPP callbacks.
    /// This does not read the later weapon-ammo component.
    pub fn read_ground_weapon_default_state(
        &mut self,
    ) -> Result<(Option<bool>, DecodedComponent), NativeReaderProfileError> {
        self.read_world_default_state(42)
    }
    /// Native ti40 defaults, including map-dependent media fields and raw MPP identity.
    pub fn read_vehicle_default_state(
        &mut self,
    ) -> Result<(Option<bool>, DecodedComponent), NativeReaderProfileError> {
        self.read_world_default_state(40)
    }
    fn read_world_default_state(
        &mut self,
        archetype: u32,
    ) -> Result<(Option<bool>, DecodedComponent), NativeReaderProfileError> {
        let encoding = self.context.profile.component_switch_encoding()?;
        let bit = self.native_bit_position();
        let position = std::cell::Cell::new(bit);
        struct Restore<'r, 'd> {
            cursor: &'r mut NativeFilmBits<'d>,
            position: &'r std::cell::Cell<i64>,
        }
        impl Drop for Restore<'_, '_> {
            fn drop(&mut self) {
                self.cursor.set_position(self.position.get());
            }
        }
        let (status, read, error) = {
            let _restore = Restore {
                cursor: &mut self.cursor,
                position: &position,
            };
            components::decode_native_world_default(
                self.data,
                bit,
                archetype,
                &encoding,
                &self.context,
                &position,
            )
        };
        if let Some(field) = error {
            return Err(NativeReaderProfileError::Width(field));
        }
        Ok((status, read))
    }

    fn read_component_with_width_limit(
        &mut self,
        name: &str,
        level: u32,
        archetype: u32,
        maximum: u64,
    ) -> Result<(Option<bool>, DecodedComponent), NativeReaderProfileError> {
        let profile = &self.context.profile;
        // Only switches go through the legacy encoding adapter. Raw widths are
        // supplied separately and converted exactly when their field is consumed.
        let encoding = profile.component_switch_encoding()?;
        let map = FilmMapBounds {
            module: String::new(),
            min: profile.movement.range.map(|r| r[0]),
            max: profile.movement.range.map(|r| r[1]),
            axis_widths: encoding.world_axis_bits.unwrap(),
            region: profile.movement.world_object.region,
            region_index_bits: encoding.index_bits,
        };
        let bit = self.native_bit_position();
        let position = std::cell::Cell::new(bit);
        let capture = NativeComponentCapture {
            position: Some(NativePositionCapture {
                map: &map,
                quantum: profile.movement.delta_quantum,
                slot: self.slot,
                world: self.accumulator.as_deref_mut(),
                emit: true,
            }),
            movement_slot: Some(self.slot),
            unit_references: true,
        };
        let (status, read, width_error) = {
            struct Restore<'r, 'd> {
                cursor: &'r mut NativeFilmBits<'d>,
                position: &'r std::cell::Cell<i64>,
            }
            impl Drop for Restore<'_, '_> {
                fn drop(&mut self) {
                    self.cursor.set_position(self.position.get());
                }
            }
            let _restore = Restore {
                cursor: &mut self.cursor,
                position: &position,
            };
            components::decode_native_component_with_widths(
                self.data,
                bit,
                name,
                level,
                archetype,
                NativeComponentReadPolicy {
                    encoding: Some(&encoding),
                    capture,
                    simulation_complete: profile.grammar.simulation_complete,
                },
                self.context.observer.clone(),
                Some(components::NativeComponentWidths {
                    movement: &profile.movement,
                    mpp: profile.mpp,
                    maximum,
                }),
                Some(&position),
            )
        };
        if let Some(field) = width_error {
            return Err(NativeReaderProfileError::Width(field));
        }
        Ok((status, read))
    }
}

/// Live raw-resync traversal with every speculative read retained separately from
/// actual callback delivery. Cumulative caller counters remain on its observer.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NativeResyncFrame {
    pub records: Vec<EntityRecord>,
    pub diagnostics: FilmReadDiagnostics,
    pub resync_bits: Vec<i64>,
    pub end_bit: i64,
    pub stop: InferenceFrameStop,
}

impl NativeFrameConfig {
    /// Sequential native raw resync, including scoped recovery scans and re-reads.
    /// Commits clean New/Delete records. Starts at zero without a packet preamble.
    pub fn decode_resync_frame(
        &self,
        data: &[u8],
        registry: &FilmRegistry,
        world: &mut FilmWorld,
        targets: &std::collections::BTreeSet<u32>,
        mut accept: impl FnMut(u32, [f32; 3], bool) -> bool,
    ) -> Result<NativeResyncFrame, NativeReaderProfileError> {
        let profile = KillWalkProfile {
            encoding: self.contextual_frame_encoding()?,
            simulation_complete: self.context.profile.grammar.simulation_complete,
        };
        let read = |header_bit: i64, header: &RecordHeader, world: &FilmWorld, slot| {
            inference_frame::read_bound_record_contextual(
                data,
                header_bit,
                header,
                registry,
                &profile,
                world,
                components::RecordCaptureSlots {
                    context: Some(self.context.clone()),
                    movement: Some(slot),
                    position: Some(slot),
                },
            )
        };
        let mut out = NativeResyncFrame {
            records: vec![],
            diagnostics: Default::default(),
            resync_bits: vec![],
            end_bit: 0,
            stop: InferenceFrameStop::PayloadBoundary,
        };
        let mut slot = 0;
        for _ in 0..4096 {
            if out.end_bit >= data.len().saturating_mul(8) as i64 {
                return Ok(out);
            }
            let start = out.end_bit;
            let header_bit = start.wrapping_add(if self.extra_fields { 32 } else { 0 });
            let Some(header) =
                records::decode_frame_header_signed(data, header_bit, &profile.encoding)
            else {
                if let Some(refusal) = out.diagnostics.width_refusals.last() {
                    out.end_bit = refusal.bit;
                }
                out.stop = InferenceFrameStop::Failure(EntityViewStop::InvalidEncoding);
                return Ok(out);
            };
            out.end_bit = header.end_bit;
            if header.kind == RecordKind::End {
                out.stop = InferenceFrameStop::End(header);
                return Ok(out);
            }
            if header.kind == RecordKind::Delta {
                slot = header.id.unwrap_or(0) & 0x3fff_ffff;
            }
            let record = read(header_bit, &header, world, slot);
            if let Some(r) = &record {
                out.diagnostics.merge(&r.diagnostics);
                out.end_bit = r.end_bit;
            }
            if record
                .as_ref()
                .is_some_and(|r| r.stop == EntityViewStop::Complete)
            {
                let record = record.unwrap();
                match header.kind {
                    RecordKind::New => {
                        world.bind_full(header.id.unwrap(), record.archetype.unwrap())
                    }
                    RecordKind::Delete => world.unbind(header.id.unwrap() & 0x3fff_ffff),
                    _ => {}
                }
                out.records.push(record);
                continue;
            }
            let scope = self
                .context
                .observer
                .as_ref()
                .map(NativeFilmObserver::neutralize_position_capture);
            let scanned = self.scan_for_target_delta(
                data,
                start.wrapping_add(1),
                registry,
                world,
                targets,
                &mut accept,
            )?;
            drop(scope);
            out.diagnostics.merge(&scanned.diagnostics);
            let Some((bit, candidate)) = scanned.result.filter(|(bit, _)| *bit >= 0) else {
                out.stop = InferenceFrameStop::Failure(
                    record.map_or(EntityViewStop::Truncated, |r| r.stop),
                );
                return Ok(out);
            };
            // Native re-reads even if callback mutation makes the accepted record fail.
            let reread = read(
                candidate.header.start_bit,
                &candidate.header,
                world,
                candidate.header.id.unwrap_or(0) & 0x3fff_ffff,
            );
            let Some(recovered) = reread else {
                out.stop = InferenceFrameStop::Failure(EntityViewStop::Truncated);
                return Ok(out);
            };
            out.diagnostics.merge(&recovered.diagnostics);
            out.end_bit = recovered.end_bit;
            out.resync_bits.push(bit);
            out.records.push(recovered);
            slot = 0;
        }
        out.stop = InferenceFrameStop::RecordLimit;
        Ok(out)
    }

    /// Scan for a clean target Delta, using native shallow observer ownership.
    /// The callback receives slot, first position and whether that position exists.
    /// When absent, the vector can retain a previous trial's value and must be ignored.
    /// This probe does not mutate World or confirm the successor chain.
    pub fn scan_for_target_delta(
        &self,
        data: &[u8],
        from: impl TryInto<i64>,
        registry: &FilmRegistry,
        world: &FilmWorld,
        targets: &std::collections::BTreeSet<u32>,
        mut accept: impl FnMut(u32, [f32; 3], bool) -> bool,
    ) -> Result<FilmReadAttempt<(i64, EntityRecord)>, NativeReaderProfileError> {
        use std::sync::{Arc, Mutex};
        let from = from
            .try_into()
            .ok()
            .ok_or(NativeReaderProfileError::Width("resync start"))?;
        let profile = KillWalkProfile {
            encoding: self.contextual_frame_encoding()?,
            simulation_complete: self.context.profile.grammar.simulation_complete,
        };
        let observer = self
            .context
            .observer
            .as_ref()
            .map(NativeFilmObserver::shallow_copy)
            .unwrap_or_default();
        let captured = Arc::new(Mutex::new(([0.0; 3], false)));
        observer.set_hook(
            NativeHookKind::Position,
            Some({
                let captured = captured.clone();
                Arc::new(move |publication| {
                    if let NativeHookPublication::Component(FilmComponentObservation::Position {
                        vector_bits,
                        ..
                    }) = publication
                    {
                        let mut state = captured.lock().unwrap_or_else(|e| e.into_inner());
                        if !state.1 {
                            *state = (vector_bits.map(f32::from_bits), true);
                        }
                    }
                })
            }),
        );
        let mut context = self.context.clone();
        context.observer = Some(observer);
        let mut diagnostics = FilmReadDiagnostics::default();
        let mut result = None;
        for start in from..(data.len() as i64 * 8).wrapping_sub(24) {
            if (data.len() as i64 * 8).wrapping_sub(start) < 24 {
                continue;
            }
            captured.lock().unwrap_or_else(|e| e.into_inner()).1 = false;
            let header_bit = start.wrapping_add(if self.extra_fields { 32 } else { 0 });
            let Some(header) =
                records::decode_frame_header_signed(data, header_bit, &profile.encoding)
            else {
                continue;
            };
            if header.kind != RecordKind::Delta {
                continue;
            }
            let slot = header.id.unwrap_or(0) & 0x3fff_ffff;
            let Some(record) = inference_frame::read_bound_record_contextual(
                data,
                header_bit,
                &header,
                registry,
                &profile,
                world,
                components::RecordCaptureSlots {
                    context: Some(context.clone()),
                    movement: Some(slot),
                    position: Some(slot),
                },
            ) else {
                continue;
            };
            diagnostics.merge(&record.diagnostics);
            if record.stop != EntityViewStop::Complete
                || record.components.is_empty()
                || !targets.contains(&slot)
            {
                continue;
            }
            let (position, present) = *captured.lock().unwrap_or_else(|e| e.into_inner());
            if accept(slot, position, present) {
                result = Some((start, record));
                break;
            }
        }
        Ok(FilmReadAttempt {
            result,
            diagnostics,
        })
    }

    /// Native harvest successor confirmation, including signed positions and flush tails.
    pub fn confirm_harvest_successor(
        &self,
        data: &[u8],
        position: impl TryInto<i64>,
        registry: &FilmRegistry,
        world: &FilmWorld,
    ) -> Result<FilmReadAttempt<bool>, NativeReaderProfileError> {
        let position = position
            .try_into()
            .ok()
            .ok_or(NativeReaderProfileError::Width("harvest position"))?;
        let profile = KillWalkProfile {
            encoding: self.contextual_frame_encoding()?,
            simulation_complete: self.context.profile.grammar.simulation_complete,
        };
        let mut diagnostics = FilmReadDiagnostics::default();
        let confirmed = frame_harvest::next_bound_clean(
            data,
            position,
            registry,
            &profile,
            world,
            &mut diagnostics,
            Some(&self.context),
        );
        Ok(FilmReadAttempt {
            result: Some(confirmed),
            diagnostics,
        })
    }

    /// Exhaustive native target harvesting with scoped trial capture and live rereads.
    pub fn harvest_targets(
        &self,
        data: &[u8],
        registry: &FilmRegistry,
        world: &FilmWorld,
        targets: &std::collections::BTreeSet<u32>,
        mode: HarvestConfirmation,
    ) -> Result<FilmReadAttempt<Vec<EntityRecord>>, NativeReaderProfileError> {
        let profile = KillWalkProfile {
            encoding: self.contextual_frame_encoding()?,
            simulation_complete: self.context.profile.grammar.simulation_complete,
        };
        Ok(frame_harvest::scan_frame_targets_contextual(
            data,
            registry,
            &profile,
            world,
            targets,
            mode,
            Some(&self.context),
        ))
    }

    /// Explicit native width repair. Preserves caller stub maps and world state;
    /// speculative callbacks use the shared observer with scoped capture receivers.
    pub fn repair_component(
        &self,
        data: &[u8],
        failed: &EntityRecord,
        registry: &FilmRegistry,
        world: &FilmWorld,
    ) -> Result<FilmReadAttempt<ChainComponentRepair>, NativeReaderProfileError> {
        let profile = KillWalkProfile {
            encoding: self.contextual_frame_encoding()?,
            simulation_complete: self.context.profile.grammar.simulation_complete,
        };
        Ok(chain_inference::repair_chain_component_contextual(
            data,
            failed,
            registry,
            &profile,
            world,
            &Default::default(),
            Some(&self.context),
        ))
    }

    /// Explicit speculative resync with shared counters and scoped capture hooks.
    /// Returns a landing to re-read; does not mutate the world or enable automatic resync.
    pub fn validated_resync(
        &self,
        data: &[u8],
        from: impl TryInto<i64>,
        registry: &FilmRegistry,
        world: &FilmWorld,
        targets: &std::collections::BTreeSet<u32>,
    ) -> Result<FilmReadAttempt<i64>, NativeReaderProfileError> {
        let from = from
            .try_into()
            .ok()
            .ok_or(NativeReaderProfileError::Width("validated resync position"))?;
        let profile = KillWalkProfile {
            encoding: self.contextual_frame_encoding()?,
            simulation_complete: self.context.profile.grammar.simulation_complete,
        };
        Ok(chain_inference::validated_chain_resync_contextual(
            data,
            from,
            registry,
            &profile,
            world,
            targets,
            Some(&self.context),
        ))
    }

    /// Speculative single-step inference using this reader context and observer.
    /// Successful endpoints retain signed native coordinates.
    pub fn infer_unbound(
        &self,
        data: &[u8],
        body_bit: impl TryInto<i64>,
        registry: &FilmRegistry,
        world: &FilmWorld,
    ) -> Result<FilmReadAttempt<(u32, i64)>, NativeReaderProfileError> {
        let body_bit = body_bit
            .try_into()
            .ok()
            .ok_or(NativeReaderProfileError::Width("inference body position"))?;
        let profile = KillWalkProfile {
            encoding: self.contextual_frame_encoding()?,
            simulation_complete: self.context.profile.grammar.simulation_complete,
        };
        let (result, diagnostics) = inference_frame::infer_unbound_archetype_contextual(
            data,
            body_bit,
            registry,
            &profile,
            world,
            Some(&self.context),
        );
        Ok(FilmReadAttempt {
            result,
            diagnostics,
        })
    }

    /// Speculative chain inference with native signed endpoints and shared counters.
    pub fn infer_chain(
        &self,
        data: &[u8],
        body_bit: impl TryInto<i64>,
        registry: &FilmRegistry,
        world: &FilmWorld,
    ) -> Result<ChainInference, NativeReaderProfileError> {
        let body_bit = body_bit
            .try_into()
            .ok()
            .ok_or(NativeReaderProfileError::Width("inference body position"))?;
        let encoding = self.contextual_frame_encoding()?;
        Ok(chain_inference::infer_chain_archetype_contextual(
            data,
            body_bit,
            registry,
            &encoding,
            world,
            self.context.profile.grammar.simulation_complete,
            Some(&self.context),
        ))
    }

    /// Entity-view inference loop with live observer scopes for single-step and
    /// recursive chain trials. The supplied grammar selects the native policy.
    pub fn decode_inference_view(
        &self,
        data: &[u8],
        start: impl TryInto<i64>,
        registry: &FilmRegistry,
        world: &mut FilmWorld,
    ) -> Result<InferenceFrame, NativeReaderProfileError> {
        let profile = KillWalkProfile {
            encoding: self.contextual_frame_encoding()?,
            simulation_complete: self.context.profile.grammar.simulation_complete,
        };
        inference_frame::decode_inference_frame_contextual(
            data,
            start,
            registry,
            &profile,
            world,
            InferenceFrameOptions {
                chain_inference: self.context.profile.grammar.chain_inference,
                view_tables: self.context.profile.grammar.view_tables,
            },
            Some(&self.context),
        )
        .ok_or(NativeReaderProfileError::Width(
            "inference start or encoding",
        ))
    }

    /// Native full-state keyframe read with live profile maps and hooks.
    pub fn read_keyframe_record(
        &self,
        data: &[u8],
        start: impl TryInto<i64>,
        registry: &FilmRegistry,
    ) -> Result<KeyframeRecord, NativeReaderProfileError> {
        components::decode_native_keyframe_record_contextual(
            data,
            start,
            registry,
            &self.contextual_frame_encoding()?,
            Some(&self.context),
        )
        .ok_or(NativeReaderProfileError::Width("keyframe layout or start"))
    }
    /// Native deterministic keyframe chain, retaining attempted records and diagnostics.
    pub fn chain_keyframes(
        &self,
        data: &[u8],
        registry: &FilmRegistry,
        from: i64,
        want: i64,
        previous_slot: i64,
    ) -> Result<KeyframeChainResult, NativeReaderProfileError> {
        Ok(components::chain_keyframe_records_contextual(
            data,
            registry,
            from,
            want,
            previous_slot,
            &self.contextual_frame_encoding()?,
            Some(&self.context),
        ))
    }
    /// Native sequential keyframe table, including its sentinel fallback behavior.
    pub fn read_keyframe_table(
        &self,
        data: &[u8],
        registry: &FilmRegistry,
    ) -> Result<NativeKeyframeTable, NativeReaderProfileError> {
        Ok(components::decode_native_keyframe_table_contextual(
            data,
            registry,
            &self.contextual_frame_encoding()?,
            Some(&self.context),
        ))
    }

    /// Native eight-view march. Slots are restored on normal return; a native
    /// panic retains mutations made before unwinding. Live hooks remain active.
    pub fn march_records(
        &self,
        data: &[u8],
        start: impl TryInto<i64>,
        registry: &FilmRegistry,
        world: &mut FilmWorld,
    ) -> Result<Vec<EntityRecord>, NativeReaderProfileError> {
        let start = start
            .try_into()
            .ok()
            .ok_or(NativeReaderProfileError::Width("march start"))?;
        let mut reader = NativeFilmReader::with_context(data, self.context.clone());
        reader.set_native_bit_position(start);
        let snapshot = world.snapshot();
        let mut records = Vec::new();
        for _ in 0..8 {
            if (data.len() as i64 * 8).wrapping_sub(reader.native_bit_position()) < 8 {
                break;
            }
            let view = reader.read_frame_records(registry, world, self)?;
            let complete = view.stop == EntityViewStop::Complete;
            records.extend(
                view.records
                    .into_iter()
                    .filter(|r| r.header.kind != RecordKind::End),
            );
            if !complete {
                break;
            }
        }
        world.restore(snapshot);
        Ok(records)
    }

    /// Walk native class or generic views with signed configuration and live context.
    pub fn decode_inference_views(
        &self,
        data: &[u8],
        skip_lead_bits: i64,
        view_count: i64,
        registry: &FilmRegistry,
        world: &mut FilmWorld,
    ) -> Result<InferenceViews, NativeReaderProfileError> {
        let profile = KillWalkProfile {
            encoding: self.contextual_frame_encoding()?,
            simulation_complete: self.context.profile.grammar.simulation_complete,
        };
        frame_harvest::decode_inference_views_contextual(
            data,
            registry,
            &profile,
            world,
            InferenceViewsOptions {
                inference: InferenceFrameOptions {
                    chain_inference: self.context.profile.grammar.chain_inference,
                    view_tables: self.context.profile.grammar.view_tables,
                },
                view_classes: self.context.profile.grammar.view_classes,
                view_count,
                skip_lead_bits,
                packet_preamble_bits: self.packet_preamble_bits,
            },
            Some(&self.context),
        )
        .ok_or(NativeReaderProfileError::Width("view encoding"))
    }

    /// Decode the native message/entity/control view classes using the default
    /// view-table admission policy. Optional inference without view tables uses
    /// a separate traversal and is refused here rather than silently ignored.
    pub fn decode_production_views(
        &self,
        data: &[u8],
        skip_lead_bits: impl TryInto<i64>,
        registry: &FilmRegistry,
        world: &mut FilmWorld,
    ) -> Result<ProductionFrame, NativeReaderProfileError> {
        if !self.context.profile.grammar.view_classes || !self.context.profile.grammar.view_tables {
            return Err(NativeReaderProfileError::Policy(
                "requires view classes and view tables",
            ));
        }
        let encoding = self.contextual_frame_encoding()?;
        let preamble_bits = self.packet_preamble_bits;
        production_frame::decode_production_frame_contextual(
            data,
            skip_lead_bits,
            registry,
            &encoding,
            world,
            production_frame::ProductionReaderContext {
                reader: Some(&self.context),
                preamble_bits,
            },
            |_, _, _| {},
        )
        .ok_or(NativeReaderProfileError::Width("frame reader"))
    }

    /// Live component reads obtain dimensions from the raw runtime profile.
    /// These representable dimensions carry only position policies through the
    /// checked legacy adapter; they must never be used as consumed widths.
    pub(crate) fn contextual_frame_encoding(
        &self,
    ) -> Result<FrameEncoding, NativeReaderProfileError> {
        let mut policies = self.clone();
        let movement = &mut policies.context.profile.movement;
        movement.world_object.index_bits = 1;
        movement.world_object.axis_bits = [1; 3];
        movement.traversal.index_bits = 1;
        movement.traversal.axis_bits = [1; 3];
        movement.delta_axis_width = 1;
        // The terminal tail is admitted only after a successful New body.
        policies.context.profile.grammar.new_record_tail_bits = 0;
        policies.context.profile.mpp = FilmMppWidths::default();
        // Keyframe consumers use the raw context, not this legacy placeholder.
        policies.context.profile.keyframe = NativeKeyframeLayout::default();
        policies.new_default_state_bits = 0;
        policies.id_low_bits = 0;
        policies.id_base = 0;
        let mut encoding = policies.frame_encoding()?;
        encoding.new_record.native_fallback_default_bits = Some(self.new_default_state_bits);
        encoding.native_id_low_bits = Some(self.id_low_bits);
        encoding.ids.base = self.id_base;
        Ok(encoding)
    }

    /// Adapt the native frame settings to the supported v41 record reader.
    /// Raw metadata remains on this configuration when widths are unsupported.
    pub(crate) fn frame_encoding(&self) -> Result<FrameEncoding, NativeReaderProfileError> {
        let width = |value: i64, field| {
            usize::try_from(value).map_err(|_| NativeReaderProfileError::Width(field))
        };
        let p = &self.context.profile;
        let position = p.component_encoding()?;
        let encoding = FrameEncoding {
            keyframe_layout: p
                .keyframe
                .bounded()
                .ok_or(NativeReaderProfileError::Width("keyframe layout"))?,
            keyframe_simulation_complete: Some(p.grammar.simulation_complete),
            native_id_low_bits: None,
            component_widths: ComponentWidthOverrides::default(),
            new_record: NewRecordEncoding {
                deserialize_defaults: p.grammar.default_state_by_archetype,
                fallback_default_bits: width(self.new_default_state_bits, "new default state")?,
                native_fallback_default_bits: None,
                // Native TraverseEntity only skips a strictly positive tail.
                terminal_bits: width(p.grammar.new_record_tail_bits.max(0), "new record tail")?,
            },
            position_capture: Some(PositionCaptureEncoding {
                min_bits: p.movement.range.map(|r| r[0].to_bits()),
                max_bits: p.movement.range.map(|r| r[1].to_bits()),
                quantum_bits: p.movement.delta_quantum.to_bits(),
                region: p.movement.world_object.region,
                axis_widths: position.world_axis_bits.unwrap(),
                region_index_bits: position.index_bits,
            }),
            ids: RecordIdLayout {
                low_bits: width(self.id_low_bits, "record ID")?,
                base: self.id_base,
            },
            mpp_widths: [
                width(p.mpp.lead, "MPP lead")?,
                width(p.mpp.index, "MPP index")?,
            ],
            position: Some(position),
            extra_fields: self.extra_fields,
            corruption_check: p.grammar.corruption_check,
        };
        if !encoding.valid() {
            return Err(NativeReaderProfileError::Width("frame encoding"));
        }
        Ok(encoding)
    }
}

impl NativeFilmReader<'_> {
    /// Native DecodeFrameRecords: install the frame context, consume a positive
    /// packet preamble only at bit zero, and commit clean creations/deletions.
    /// Retains End and failed attempts, unlike the native returned record list.
    /// This generic loop does not apply inference or view-class admission rules.
    pub fn read_frame_records(
        &mut self,
        registry: &FilmRegistry,
        world: &mut FilmWorld,
        config: &NativeFrameConfig,
    ) -> Result<DecodedEntityView, NativeReaderProfileError> {
        self.read_frame_records_impl(registry, world, config, false)
    }
    /// Use the supplied world for both entity lifetimes and position accumulation,
    /// matching a native reader whose accumulator points to its traversal world.
    /// A separately attached accumulator must first be detached.
    pub fn read_frame_records_accumulating(
        &mut self,
        registry: &FilmRegistry,
        world: &mut FilmWorld,
        config: &NativeFrameConfig,
    ) -> Result<DecodedEntityView, NativeReaderProfileError> {
        if self.accumulator.is_some() {
            return Err(NativeReaderProfileError::Policy(
                "two position accumulator worlds",
            ));
        }
        self.read_frame_records_impl(registry, world, config, true)
    }
    fn read_frame_records_impl(
        &mut self,
        registry: &FilmRegistry,
        world: &mut FilmWorld,
        config: &NativeFrameConfig,
        accumulate_in_world: bool,
    ) -> Result<DecodedEntityView, NativeReaderProfileError> {
        let encoding = config.contextual_frame_encoding()?;
        self.context = config.context.clone();
        if self.native_bit_position() == 0 && config.packet_preamble_bits > 0 {
            self.set_native_bit_position(config.packet_preamble_bits);
        }
        let bit = self.native_bit_position();
        let mut bindings = EntityBindings::default();
        for (&slot, state) in &world.slots {
            bindings.slots.insert(
                slot,
                EntityBinding {
                    id: state.full_id,
                    generation_any: state.generation_any,
                    archetype: state.archetype,
                    origin: BindingOrigin::Supplied,
                },
            );
        }
        let position = std::cell::Cell::new(self.native_bit_position());
        let slot = std::cell::Cell::new(self.slot);
        let (view, width_error) = {
            // Native callers retain both fields when component reads or hooks
            // unwind. World mutations and observer publications already occur live.
            struct Restore<'r, 'd> {
                cursor: &'r mut NativeFilmBits<'d>,
                slot: &'r mut u32,
                position: &'r std::cell::Cell<i64>,
                captured_slot: &'r std::cell::Cell<u32>,
            }
            impl Drop for Restore<'_, '_> {
                fn drop(&mut self) {
                    self.cursor.set_position(self.position.get());
                    *self.slot = self.captured_slot.get();
                }
            }
            let _restore = Restore {
                cursor: &mut self.cursor,
                slot: &mut self.slot,
                position: &position,
                captured_slot: &slot,
            };
            let result = components::decode_native_entity_view_live(
                self.data,
                bit,
                registry,
                &encoding,
                &mut bindings,
                components::NativeFrameRuntime {
                    context: self.context.clone(),
                    world: Some(world),
                    accumulator: self.accumulator.as_deref_mut(),
                    accumulate_in_world,
                    cursor_mirror: Some(&position),
                    slot_mirror: Some(&slot),
                },
            );
            // Header decoding still uses a legacy cursor; retain its endpoint
            // on ordinary returns as well as mirrored component read endpoints.
            position.set(result.0.end_bit);
            result
        };
        if let Some(field) = width_error {
            return Err(NativeReaderProfileError::Width(field));
        }
        Ok(view)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
    use std::{
        io::Read,
        sync::{Arc, Mutex},
    };

    #[test]
    fn native_unused_widths_are_lazy_in_32_bit_domain() {
        let mut bytes = Vec::new();
        std::io::Read::read_to_end(
            &mut flate2::read::ZlibDecoder::new(
                &include_bytes!("fixtures/lazy-widths-v41.json.zlib")[..],
            ),
            &mut bytes,
        )
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(rows.len(), 648);
        for (case, row) in rows.iter().enumerate() {
            let hex = row["hex"].as_str().unwrap();
            let data: Vec<_> = (0..hex.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                .collect();
            let width = row["width"].as_u64().unwrap();
            let mut profile = NativeScanProfile::default();
            profile.movement.world_object.index_bits = width;
            profile.movement.world_object.axis_bits = [width; 3];
            profile.movement.traversal.index_bits = width;
            profile.movement.traversal.axis_bits = [width; 3];
            profile.movement.delta_axis_width = width;
            profile.movement.full_precision = row["mode"] == 6;
            profile.grammar.baseline_scope = false;
            profile.grammar.writer_absolute = false;
            for maximum in [u64::from(u32::MAX), usize::MAX as u64] {
                let mut reader = NativeFilmReader::with_context(
                    &data,
                    NativeReaderContext {
                        profile: profile.clone(),
                        observer: None,
                    },
                );
                reader.set_bit_position(row["start"].as_u64().unwrap() as usize);
                let (status, read) = reader
                    .read_component_with_width_limit(row["name"].as_str().unwrap(), 0, 35, maximum)
                    .unwrap();
                assert_eq!(status, row["ok"].as_bool(), "status {case}");
                assert_eq!(
                    read.end_bit as u64,
                    row["end"].as_u64().unwrap(),
                    "end {case}"
                );
                assert_eq!(reader.read_bits(7), row["tail"].as_u64(), "tail {case}");
                assert_eq!(
                    reader.bit_position() as u64,
                    row["tail_end"].as_u64().unwrap()
                );
                assert_eq!(reader.profile(), profile, "retained metadata {case}");
            }
        }
    }

    #[test]
    fn native_unrepresentable_width_refuses_only_when_reached() {
        let mut profile = NativeScanProfile::default();
        profile.movement.world_object.index_bits = 1;
        profile.movement.world_object.axis_bits = [6, u64::from(u32::MAX) + 1, 8];
        let mut reader = NativeFilmReader::with_context(
            &[0; 16],
            NativeReaderContext {
                profile,
                observer: None,
            },
        );
        assert_eq!(
            reader.read_component_with_width_limit(
                "object-position-dynamic-precision-component",
                0,
                35,
                u64::from(u32::MAX)
            ),
            Err(NativeReaderProfileError::Width("world axes"))
        );
        // Four selectors, one region bit, and the first six-bit axis survived.
        assert_eq!(reader.bit_position(), 11);
        assert_eq!(reader.read_bits(7), Some(0));
    }

    #[test]
    fn native_delta_width_fallback_matches_reference() {
        check_native_position_widths(
            include_bytes!("fixtures/delta-width-fallback-v41.json.zlib"),
            3528,
            false,
        );
    }

    #[test]
    fn native_absolute_width_boundaries_match_reference() {
        check_native_position_widths(
            include_bytes!("fixtures/absolute-width-boundaries-v41.json.zlib"),
            720,
            true,
        );
    }

    fn check_native_position_widths(fixture: &[u8], count: usize, absolute: bool) {
        let mut bytes = Vec::new();
        flate2::read::ZlibDecoder::new(fixture)
            .read_to_end(&mut bytes)
            .unwrap();
        let rows: Vec<Value> = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(rows.len(), count);
        for (index, row) in rows.iter().enumerate() {
            let hex = row["hex"].as_str().unwrap();
            let data: Vec<u8> = (0..hex.len())
                .step_by(2)
                .map(|at| u8::from_str_radix(&hex[at..at + 2], 16).unwrap())
                .collect();
            let mut profile = NativeScanProfile::default();
            let axes = serde_json::from_value(row["axes"].clone()).unwrap();
            if absolute {
                profile.movement.world_object.axis_bits = axes;
                profile.movement.world_object.index_bits = 1;
                profile.movement.world_object.region = 0;
                profile.grammar.writer_absolute = false;
            } else {
                profile.movement.traversal.axis_bits = axes;
                profile.movement.delta_axis_width = row["override"].as_u64().unwrap();
            }
            profile.movement.delta_quantum = 0.25;
            profile.movement.full_precision = false;
            profile.grammar.baseline_scope = false;
            profile.movement.delta_has_handle_tail = false;
            let observer = NativeFilmObserver::default();
            let events = Arc::new(Mutex::new(Vec::<Value>::new()));
            let captured = events.clone();
            observer.set_hook(
                NativeHookKind::Position,
                Some(Arc::new(move |p| {
                    let NativeHookPublication::Component(FilmComponentObservation::Position {
                        position_kind,
                        vector_bits,
                        bit,
                        slot,
                    }) = p
                    else {
                        panic!("position callback");
                    };
                    let kind = match position_kind {
                        NativePositionKind::DeltaAxis => "dax",
                        NativePositionKind::Absolute => "abs",
                        other => panic!("unexpected position kind {other:?}"),
                    };
                    captured
                        .lock()
                        .unwrap()
                        .push(json!({"kind":kind,"vector":vector_bits,"bit":bit,"slot":slot}));
                })),
            );
            let mut world = FilmWorld::default();
            world.bind_full(42, 0);
            let mode = row["mode"].as_u64().unwrap();
            if mode == 2 {
                world.set_position(42, [1., 2., 3.]);
            }
            let mut delayed_world = world.clone();
            let map = FilmMapBounds {
                module: String::new(),
                min: profile.movement.range.map(|r| r[0]),
                max: profile.movement.range.map(|r| r[1]),
                axis_widths: profile.movement.world_object.axis_bits.map(|w| w as usize),
                region: profile.movement.world_object.region,
                region_index_bits: profile.movement.world_object.index_bits as usize,
            };
            let mut reader = NativeFilmReader::with_context(
                &data,
                NativeReaderContext {
                    profile: profile.clone(),
                    observer: Some(observer),
                },
            );
            if mode > 0 {
                reader.replace_position_accumulator(Some(&mut world));
            }
            reader.set_capture_slot(42);
            reader.set_bit_position(row["start"].as_u64().unwrap() as usize);
            let (status, read) = reader
                .read_component("object-position-dynamic-precision-component", 16, 0)
                .unwrap();
            assert_eq!(json!(status), row["ok"], "status {index}");
            assert_eq!(json!(read.end_bit), row["end"], "end {index}");
            if let Some(expected) = row["axis_fields"].as_array() {
                for field in expected {
                    let name = format!(
                        "{}[{}]",
                        if absolute { "position" } else { "delta" },
                        field["axis"].as_u64().unwrap()
                    );
                    let actual = read.fields.iter().find(|f| f.name == name).unwrap();
                    assert_eq!(json!(actual.bit), field["bit"], "field bit {index}");
                    assert_eq!(json!(actual.width), field["width"], "field width {index}");
                    assert_eq!(json!(actual.raw), field["raw"], "field raw {index}");
                    if actual.width > 64
                        && let Some(mut bounded) = bits::Cursor::new(&data, actual.bit)
                    {
                        assert_eq!(bounded.read(actual.width), None);
                        assert_eq!(
                            serde_json::json!(bounded.position),
                            serde_json::json!(actual.bit)
                        );
                    }
                    // A native u64 read discards high bits for widths above 64.
                    // Retained prefix fields must nevertheless cover every such
                    // source bit, including bits with value zero.
                    let prefix_end = actual.bit + actual.width.saturating_sub(64) as i64;
                    let source_end = prefix_end.min((data.len() * 8) as i64).max(actual.bit);
                    let mut next = actual.bit;
                    for part in read
                        .fields
                        .iter()
                        .filter(|f| f.name.starts_with(&format!("{name}.discarded[")))
                    {
                        assert_eq!(part.bit, next, "prefix gap {index}");
                        assert!(
                            part.width <= 64
                                && part.bit.wrapping_add(part.width as i64) <= source_end
                        );
                        let mut raw = 0u64;
                        for bit in part.bit..part.bit.wrapping_add(part.width as i64) {
                            raw = (raw << 1)
                                | u64::from(
                                    (data[crate::theater::bits::native_address(bit / 8)]
                                        >> (7 - bit % 8))
                                        & 1,
                                );
                        }
                        assert_eq!(part.raw, raw, "discarded source {index}");
                        next += part.width as i64;
                    }
                    assert_eq!(next, source_end, "unretained source {index}");
                }
            }
            assert_eq!(
                serde_json::from_value::<DecodedComponent>(json!(read)).unwrap(),
                read
            );
            assert_eq!(
                json!(*events.lock().unwrap()),
                row["samples"],
                "samples {index}"
            );
            assert_eq!(
                json!(reader.read_bits(7).unwrap()),
                row["tail"],
                "tail {index}"
            );
            assert_eq!(
                json!(reader.bit_position()),
                row["tail_end"],
                "tail end {index}"
            );
            assert_eq!(reader.profile(), profile, "raw profile {index}");
            reader.replace_position_accumulator(None);
            let position = world.position(42).map(|v| v.map(f32::to_bits));
            assert_eq!(json!(position), row["position"], "world {index}");
            let sample = capture_component_position(
                &read,
                42,
                &map,
                0.25,
                (mode > 0).then_some(&mut delayed_world),
            );
            let delayed: Vec<Value> = sample
                .into_iter()
                .map(|s| {
                    json!({
                        "kind": s.kind.native_name(), "bit": s.bit, "slot": s.slot,
                        "vector": s.vector.map(f32::to_bits),
                    })
                })
                .collect();
            assert_eq!(json!(delayed), row["samples"], "delayed samples {index}");
            assert_eq!(
                json!(delayed_world.position(42).map(|v| v.map(f32::to_bits))),
                row["position"],
                "delayed world {index}"
            );
        }
    }

    #[test]
    fn native_signed_variable_reader_sequences() {
        let mut bytes = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/signed-variable-reader-v41.json.zlib")[..],
        )
        .read_to_end(&mut bytes)
        .unwrap();
        let rows: Vec<Value> = serde_json::from_slice(&bytes).unwrap();
        let mut selectors = [0; 4];
        let mut padded = 0;
        let mut negative = 0;
        for (index, row) in rows.iter().enumerate() {
            let hex = row["hex"].as_str().unwrap();
            let data: Vec<u8> = (0..hex.len())
                .step_by(2)
                .map(|at| u8::from_str_radix(&hex[at..at + 2], 16).unwrap())
                .collect();
            let mut reader = NativeFilmReader::new(&data);
            reader.set_bit_position(row["start"].as_u64().unwrap() as usize);
            reader.set_capture_slot(42);
            let initial_profile = reader.context().profile;
            selectors[row["selector"].as_u64().unwrap() as usize] += 1;
            for step in row["steps"].as_array().unwrap() {
                let actual = reader.read_signed_variable().unwrap();
                assert_eq!(json!(actual), step["value"], "case {index}");
                assert_eq!(json!(reader.bit_position()), step["end"], "case {index}");
                negative += usize::from(actual < 0);
                padded += usize::from(reader.padded_bits() > 0);
            }
            assert_eq!(json!(reader.read_bits(7).unwrap()), row["tail"]);
            assert_eq!(json!(reader.bit_position()), row["end"]);
            assert_eq!(reader.capture_slot(), 42);
            assert_eq!(reader.context().profile, initial_profile);
        }
        assert!(selectors.into_iter().all(|count| count > 0));
        assert!(padded > 0);
        assert!(negative > 0);
    }

    #[test]
    fn native_reader_sequences_match_reference() {
        let mut bytes = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/context-reader-sequence-v41.json.zlib")[..],
        )
        .read_to_end(&mut bytes)
        .unwrap();
        let rows: Vec<Value> = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(rows.len(), 256);
        for row in rows {
            let i = row["case"].as_u64().unwrap() as usize;
            let data: Vec<u8> = (0..i % 25).map(|j| (i * 71 + j * 37) as u8).collect();
            let mut context = NativeFilmContext::new(None);
            let mut profile = NativeScanProfile::default();
            profile.movement.full_precision = i.is_multiple_of(2);
            profile.movement.mobility_action_extra_bits = 5;
            profile.grammar.mobility_action_body = !i.is_multiple_of(3);
            profile.grammar.simulation_complete = i.is_multiple_of(4);
            context.set_scan_profile(profile.clone()).unwrap();
            let observer = context.observation();
            assert!(observer.same_instance(&context.observation()));
            assert!(!observer.same_instance(&NativeFilmContext::new(None).observation()));
            let events = Arc::new(Mutex::new(Vec::<Value>::new()));
            for kind in [
                NativeHookKind::EmpTimer,
                NativeHookKind::WeaponRounds,
                NativeHookKind::DesiredWeaponSet,
                NativeHookKind::MobilityAction,
                NativeHookKind::MovementState,
            ] {
                let events = events.clone();
                observer.set_hook(
                    kind,
                    Some(Arc::new(move |p| {
                        let v = match p {
                            NativeHookPublication::Component(
                                FilmComponentObservation::EmpTimer { quantum },
                            ) => json!(["emp", quantum]),
                            NativeHookPublication::Component(
                                FilmComponentObservation::WeaponRounds { rounds },
                            ) => json!(["rounds", rounds]),
                            NativeHookPublication::Component(
                                FilmComponentObservation::DesiredWeaponSet { selection },
                            ) => json!(["desired", selection]),
                            NativeHookPublication::Component(
                                FilmComponentObservation::MovementState {
                                    component,
                                    slot,
                                    values,
                                },
                            ) => json!(["movement", component, slot, values]),
                            NativeHookPublication::MobilityAction([a, b]) => {
                                json!(["mobility", a, b])
                            }
                            _ => panic!("unexpected callback"),
                        };
                        events.lock().unwrap().push(v);
                    })),
                );
            }
            let mut reader = context.reader(&data).unwrap();
            let frame = context.scan_frame().unwrap();
            assert!(frame.context.observer.is_none());
            assert_eq!(row["frame_observer_nil"], true);
            let mut frame_reader = frame.reader(&data);
            let mut replacement = profile.clone();
            replacement.grammar.simulation_complete = !profile.grammar.simulation_complete;
            replacement.movement.full_precision = !profile.movement.full_precision;
            context.set_scan_profile(replacement).unwrap();
            assert_eq!(reader.context().profile, profile);
            reader.skip(i % 8).unwrap();
            frame_reader.skip(i % 8).unwrap();
            assert_eq!(json!(reader.read_bits(3).unwrap()), row["primitive"]);
            frame_reader.read_bits(3).unwrap();
            for (k, step) in row["steps"].as_array().unwrap().iter().enumerate() {
                if k == 3 {
                    let events = events.clone();
                    observer.set_hook(
                        NativeHookKind::EmpTimer,
                        Some(Arc::new(move |p| {
                            let NativeHookPublication::Component(
                                FilmComponentObservation::EmpTimer { quantum },
                            ) = p
                            else {
                                panic!("wrong hook")
                            };
                            events
                                .lock()
                                .unwrap()
                                .push(json!(["emp-replaced", quantum]));
                        })),
                    );
                }
                let name = step["name"].as_str().unwrap();
                let (status, _) = reader.read_component(name, (i % 4) as u32, 35).unwrap();
                let (frame_status, _) = frame_reader
                    .read_component(name, (i % 4) as u32, 35)
                    .unwrap();
                assert_eq!(json!(status), step["ok"], "case {i} step {k} {name}");
                assert_eq!(json!(frame_status), step["frame_ok"], "case {i} step {k}");
                assert_eq!(
                    json!(reader.bit_position()),
                    step["end"],
                    "case {i} step {k} {name}"
                );
                assert_eq!(
                    json!(frame_reader.bit_position()),
                    step["frame_end"],
                    "case {i} step {k}"
                );
                assert_eq!(
                    json!(*events.lock().unwrap()),
                    step["events"],
                    "case {i} step {k} {name}"
                );
                assert_eq!(
                    reader.padded_bits(),
                    reader.bit_position().saturating_sub(data.len() * 8)
                );
            }
        }
    }
}
