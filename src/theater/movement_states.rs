//! Production movement transitions and jumps derived from held vertical velocity.
use super::*;
use crate::clients::hi::models::FilmChunkData;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Native movement publication labels. A jump remains explicitly derived.
pub const MOVEMENT_CROUCH: &str = "crouch";
pub const MOVEMENT_SLIDE: &str = "slide";
pub const MOVEMENT_CLAMBER: &str = "clamber";
pub const MOVEMENT_JUMP_DERIVED: &str = "jumpDerived";
pub const MOVEMENT_SPRINT: &str = "sprint";
/// Native measured height used to classify a velocity-derived jump, in metres.
pub const SPARTAN_JUMP_HEIGHT_M: f64 = 0.85;
/// Fractional half-width around the native reference jump height.
pub const SPARTAN_JUMP_HEIGHT_TOLERANCE: f64 = 0.10;
/// Minimum vertical rise speed in metres per second (native RiseMinMS).
pub const SPARTAN_JUMP_RISE_MIN_MPS: f64 = 0.5;
/// Maximum duration represented by a held vertical-velocity sample, microseconds.
pub const SPARTAN_JUMP_HOLD_MAX_US: u64 = 250_000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MovementStateRead {
    pub slot: u32,
    pub kind: String,
    #[serde(rename = "TimestampUS")]
    pub timestamp_us: u64,
    pub chunk: i64,
    pub packet_index: usize,
    pub on: bool,
    pub progress: u32,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MovementStateStats {
    pub records: usize,
    pub read: usize,
    pub absent: bool,
    pub scanned: bool,
    pub packets: usize,
    pub event_packets: usize,
    pub event_packets_located: usize,
    pub event_packets_unlocated: usize,
    pub desyncs: usize,
    pub slot_unbound: usize,
    pub datum_bindings: usize,
    pub datum_ambiguous: usize,
    pub duplicates: usize,
    pub velocity_reads: usize,
    pub jump_episodes: usize,
    pub jumps_derived: usize,
    pub map_widths: [u64; 3],
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MovementStateStream {
    /// Nonzero production-frame admission counters in source order. Some(empty)
    /// means the frame scan ran with no admissions counted; None means unavailable
    /// (older export or setup stopped before frame scanning).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub admission_diagnostics: Option<Vec<MovementFrameAdmission>>,
    /// Records whose native traversal consumed synthetic tail padding.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub padding: Vec<MovementPadding>,
    pub records: Vec<MovementStateRead>,
    pub stats: MovementStateStats,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MovementFrameAdmission {
    pub source: FilmPacket,
    /// Ordinal among all native packet types in this source chunk.
    pub packet_index: usize,
    pub diagnostics: ProductionAdmissionDiagnostics,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MovementPadding {
    pub source: FilmPacket,
    pub record_start_bit: i64,
    pub record_end_bit: i64,
    pub padded_bits: usize,
}
#[derive(Debug, Clone)]
struct VerticalVelocity {
    ts: u64,
    vz: f64,
    chunk: i64,
    packet: usize,
}
#[derive(Default)]
pub(super) struct Scanner {
    pub(super) stats: MovementStateStats,
    pub(super) readings: BTreeMap<(u64, u32, String), MovementStateRead>,
    velocities: BTreeMap<u32, Vec<VerticalVelocity>>,
}
impl Scanner {
    fn publish(&mut self, read: MovementStateRead) {
        use std::collections::btree_map::Entry;
        match self
            .readings
            .entry((read.timestamp_us, read.slot, read.kind.clone()))
        {
            Entry::Vacant(e) => {
                e.insert(read);
                self.stats.read += 1;
            }
            Entry::Occupied(_) => self.stats.duplicates += 1,
        }
    }
    fn observe(
        &mut self,
        record: &EntityRecord,
        world: &FilmWorld,
        source: FilmPacket,
        packet: usize,
    ) {
        for observation in &record.diagnostics.component_observations {
            let slot = match observation {
                FilmComponentObservation::MovementState { slot, .. } => *slot,
                _ => continue,
            };
            self.receive(
                observation,
                world.archetype(slot),
                source.timestamp_us,
                i64::from(source.chunk_index),
                packet,
            );
        }
    }
    pub(super) fn receive(
        &mut self,
        observation: &FilmComponentObservation,
        archetype: Option<u32>,
        timestamp_us: u64,
        chunk: i64,
        packet: usize,
    ) {
        let FilmComponentObservation::MovementState {
            component,
            slot,
            values,
        } = observation
        else {
            return;
        };
        let slot = *slot;
        let (kind, on, progress) = match component {
            NativeMovementComponent::Crouch => ("crouch", values[0] != 0, values[1] as u32),
            NativeMovementComponent::Slide => ("slide", values[0] != 0, 0),
            NativeMovementComponent::Mobility => ("clamber", values[0] != 0, 0),
            NativeMovementComponent::ActiveAbility => ("sprint", values[0] == 2, 0),
            NativeMovementComponent::Velocity => {
                if values[0] == 0 && values[1] == 0 && archetype == Some(35) {
                    self.velocities
                        .entry(slot)
                        .or_default()
                        .push(VerticalVelocity {
                            ts: timestamp_us,
                            vz: native_velocity(values[2] as u32, values[3] as u16)[2] as f64,
                            chunk,
                            packet,
                        });
                    self.stats.velocity_reads += 1;
                }
                return;
            }
            NativeMovementComponent::Posture | NativeMovementComponent::UnitControl => return,
        };
        if archetype != Some(35) {
            self.stats.slot_unbound += 1;
            return;
        }
        self.publish(MovementStateRead {
            slot,
            kind: kind.into(),
            timestamp_us,
            chunk,
            packet_index: packet,
            on,
            progress,
        });
    }
    pub(super) fn derive_jumps(&mut self) {
        let velocities = std::mem::take(&mut self.velocities);
        for (slot, mut samples) in velocities {
            samples.sort_by_key(|s| s.ts);
            samples.dedup_by_key(|s| s.ts);
            let mut current: Option<(VerticalVelocity, f64)> = None;
            for (i, sample) in samples.iter().enumerate() {
                if sample.vz >= SPARTAN_JUMP_RISE_MIN_MPS && current.is_none() {
                    current = Some((sample.clone(), 0.));
                } else if sample.vz < SPARTAN_JUMP_RISE_MIN_MPS
                    && let Some((start, height)) = current.take()
                {
                    self.stats.jump_episodes += 1;
                    if (height - SPARTAN_JUMP_HEIGHT_M).abs()
                        <= SPARTAN_JUMP_HEIGHT_M * SPARTAN_JUMP_HEIGHT_TOLERANCE
                    {
                        self.stats.jumps_derived += 1;
                        for (ts, on) in [(start.ts, true), (sample.ts, false)] {
                            self.publish(MovementStateRead {
                                slot,
                                kind: MOVEMENT_JUMP_DERIVED.into(),
                                timestamp_us: ts,
                                chunk: start.chunk,
                                packet_index: start.packet,
                                on,
                                progress: 0,
                            });
                        }
                    }
                    continue;
                }
                if let Some((_, height)) = &mut current {
                    let held = samples
                        .get(i + 1)
                        .map(|next| next.ts - sample.ts)
                        .filter(|&d| d <= SPARTAN_JUMP_HOLD_MAX_US)
                        .unwrap_or(0);
                    *height += sample.vz * (held as f64 / 1e6);
                }
            }
        }
    }
}

/// Reference cubemap and magnitude arithmetic, including its invalid-face sentinel.
/// Kept separate from the legacy observation decoder's stricter direction validation.
pub(super) fn native_velocity(code: u32, scale: u16) -> [f32; 3] {
    super::decode_native_velocity(u64::from(code), u64::from(scale))
}

pub fn scan_movement_states(
    chunks: &[FilmChunkData],
    registry: &FilmRegistry,
    encoding: &FrameEncoding,
) -> Result<MovementStateStream, DecodeError> {
    scan_movement_states_observed(chunks, registry, encoding, |_, _, _| {})
}

pub(crate) fn scan_movement_states_observed(
    chunks: &[FilmChunkData],
    registry: &FilmRegistry,
    encoding: &FrameEncoding,
    observe_frame: impl FnMut(FilmPacket, usize, &ProductionFrame),
) -> Result<MovementStateStream, DecodeError> {
    let (stream, error) =
        scan_movement_states_diagnostics_observed(chunks, registry, encoding, observe_frame);
    error.map_or(Ok(stream), Err)
}

/// Preserve raw native scan counters alongside failures. Replay consumers may
/// deliberately reset these counters; that does not erase the scanner result.
pub fn scan_movement_states_with_diagnostics(
    chunks: &[FilmChunkData],
    registry: &FilmRegistry,
    encoding: &FrameEncoding,
) -> (MovementStateStream, Option<DecodeError>) {
    scan_movement_states_diagnostics_observed(chunks, registry, encoding, |_, _, _| {})
}

fn scan_movement_states_diagnostics_observed(
    chunks: &[FilmChunkData],
    registry: &FilmRegistry,
    encoding: &FrameEncoding,
    mut observe_frame: impl FnMut(FilmPacket, usize, &ProductionFrame),
) -> (MovementStateStream, Option<DecodeError>) {
    let mut scanner = Scanner::default();
    // Native ScanMovementStates publishes MapWidths before source/registry lookup.
    scanner.stats.map_widths = encoding
        .position
        .as_ref()
        .and_then(|p| p.world_axis_bits)
        .unwrap_or_default()
        .map(|v| v as u64);
    let mut padding = vec![];
    let mut admission_diagnostics = None;
    let error = (|| -> Result<(), DecodeError> {
        if !encoding.valid() || encoding.extra_fields {
            return Err(DecodeError::Inconsistent(
                "unsupported production movement frame encoding".into(),
            ));
        }
        let selected = super::fire_events::native_chunk_prefix(chunks)?;
        let arch = registry
            .archetype(35)
            .ok_or(DecodeError::Missing("biped archetype"))?;
        scanner.stats.map_widths = encoding
            .position
            .as_ref()
            .and_then(|p| p.world_axis_bits)
            .ok_or(DecodeError::Missing("world position widths"))?
            .map(|v| v as u64);
        if [
            "unit-crouch",
            "biped-slide",
            "biped-mobility-action",
            "biped-spartan-ability",
        ]
        .iter()
        .all(|name| super::ability_states::component_index(arch, name).is_none())
        {
            scanner.stats.absent = true;
            scanner.stats.scanned = true;
            return Ok(());
        }
        admission_diagnostics = Some(Vec::new());
        let mut world = FilmWorld {
            anticipated: Some(build_native_anticipated_bindings(chunks)),
            ..Default::default()
        };
        for chunk in selected {
            let data = &chunk.data;
            world.current_chunk = i64::from(chunk.metadata.index);
            let packets = super::fire_events::native_chunk_packets(chunk);
            let chunk_packets: Vec<_> = packets.iter().collect();
            for p in chunk_packets.iter().filter(|p| p.packet_type == 2) {
                let payload = &data[p.payload_offset..p.payload_offset + p.payload_size];
                for a in recover_keyframe_anchors(payload) {
                    world.bind_keyframe(a.id >> 30, a.id & 0x3fff_ffff, a.archetype);
                }
            }
            for p in chunk_packets.iter().filter(|p| p.packet_type == 2) {
                let payload = &data[p.payload_offset..p.payload_offset + p.payload_size];
                let datums = recover_keyframe_datums(payload);
                scanner.stats.datum_bindings += datums.bind_missing(&mut world);
                scanner.stats.datum_ambiguous += datums.ambiguous;
            }
            for (index, p) in chunk_packets.into_iter().enumerate() {
                if p.packet_type != 0 || p.payload_size == 0 {
                    continue;
                }
                let payload = &data[p.payload_offset..p.payload_offset + p.payload_size];
                let start = if decode_packet_head_event(payload).is_some() {
                    scanner.stats.event_packets += 1;
                    let Some(start) =
                        locate_strict_entity_view(payload, registry, encoding, &world)
                    else {
                        scanner.stats.event_packets_unlocated += 1;
                        continue;
                    };
                    scanner.stats.event_packets_located += 1;
                    start
                } else {
                    2
                };
                scanner.stats.packets += 1;
                let frame = super::production_frame::decode_production_frame_observed(
                    payload,
                    start,
                    registry,
                    encoding,
                    &mut world,
                    |record, _, world| scanner.observe(record, world, *p, index),
                );
                if let Some(frame) = frame {
                    observe_frame(*p, index, &frame);
                    if let Some(diagnostics) = &frame.admission_diagnostics
                        && !diagnostics.is_empty()
                    {
                        admission_diagnostics
                            .as_mut()
                            .unwrap()
                            .push(MovementFrameAdmission {
                                source: *p,
                                packet_index: index,
                                diagnostics: diagnostics.clone(),
                            });
                    }
                    for record in frame.records {
                        if record.padded_bits != 0 {
                            padding.push(MovementPadding {
                                source: *p,
                                record_start_bit: record.header.start_bit,
                                record_end_bit: record.end_bit,
                                padded_bits: record.padded_bits,
                            });
                        }
                        if record.archetype == Some(35) {
                            scanner.stats.records += 1;
                            scanner.stats.desyncs +=
                                usize::from(record.stop != EntityViewStop::Complete);
                        }
                    }
                }
            }
        }
        scanner.derive_jumps();
        scanner.stats.scanned = true;
        Ok(())
    })()
    .err();
    (
        MovementStateStream {
            admission_diagnostics,
            records: scanner.readings.into_values().collect(),
            padding,
            stats: scanner.stats,
        },
        error,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn admission_counters_survive_source_scan_and_export() {
        let mut raw = String::new();
        flate2::read::ZlibDecoder::new(
            include_bytes!("fixtures/production-admission-v41.json.zlib").as_slice(),
        )
        .read_to_string(&mut raw)
        .unwrap();
        let cases: Vec<serde_json::Value> = serde_json::from_str(&raw).unwrap();
        let case = cases
            .iter()
            .find(|c| c["mode"] == 0 && c["start"] == 2)
            .unwrap();
        let hex = case["hex"].as_str().unwrap();
        let payload: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect();
        let mut data = super::super::fire_events::test_payload_after_keyframe(&payload, 123);
        data.extend(super::super::fire_events::test_payload_after_keyframe(
            &payload, 456,
        ));
        let chunk = FilmChunkData {
            metadata: crate::clients::hi::models::FilmChunk {
                index: 1,
                chunk_type: 2,
                start_time_offset_ms: 0,
                duration_ms: 0,
                size: data.len() as i64,
                file_relative_path: String::new(),
            },
            data,
        };
        let registry = FilmRegistry {
            archetypes: (0..36)
                .map(|index| FilmArchetype {
                    index,
                    components: if index == 35 {
                        vec!["unit-crouch-component".into()]
                    } else {
                        vec![]
                    },
                    levels: if index == 35 { vec![0] } else { vec![] },
                })
                .collect(),
            major_version: 41,
            format_version: 27,
            end_byte: 0,
            truncated: false,
        };
        let encoding = resolve_v41_profile(&registry, None, Some("Bazaar"))
            .unwrap()
            .frame_encoding(RecordIdLayout {
                low_bits: 5,
                base: 0,
            })
            .unwrap();
        let (stream, error) = scan_movement_states_with_diagnostics(
            std::slice::from_ref(&chunk),
            &registry,
            &encoding,
        );
        assert!(error.is_none(), "{error:?}");
        let entries = stream.admission_diagnostics.as_ref().unwrap();
        assert_eq!(entries.len(), 2);
        for (entry, (ordinal, time)) in entries.iter().zip([(1, 123), (3, 456)]) {
            assert_eq!(entry.packet_index, ordinal);
            assert_eq!(entry.source.chunk_index, 1);
            assert_eq!(entry.source.timestamp_us, time);
            assert_eq!(
                serde_json::to_value(&entry.diagnostics).unwrap(),
                case["diagnostics"]
            );
        }
        let mut bootstrap = Vec::new();
        flate2::read::ZlibDecoder::new(include_bytes!("fixtures/bootstrap-v41.zlib").as_slice())
            .read_to_end(&mut bootstrap)
            .unwrap();
        let mut bootstrap_chunk = chunk.clone();
        bootstrap_chunk.metadata.index = 0;
        bootstrap_chunk.metadata.chunk_type = 1;
        bootstrap_chunk.metadata.size = bootstrap.len() as i64;
        bootstrap_chunk.data = bootstrap;
        let film = LegacyFilm::try_from_chunks_with_map(
            &[bootstrap_chunk, chunk],
            DecodeOptions::v41(),
            "Bazaar",
            encoding.ids,
        )
        .unwrap();
        assert_eq!(
            film.movement_states
                .as_ref()
                .unwrap()
                .admission_diagnostics
                .as_ref(),
            Some(entries)
        );
        let restored: LegacyFilm =
            serde_json::from_slice(&serde_json::to_vec(&film).unwrap()).unwrap();
        assert_eq!(restored.movement_states, film.movement_states);
        let mut json = serde_json::to_value(&stream).unwrap();
        assert_eq!(
            serde_json::from_value::<MovementStateStream>(json.clone()).unwrap(),
            stream
        );
        json.as_object_mut()
            .unwrap()
            .remove("admission_diagnostics");
        assert!(
            serde_json::from_value::<MovementStateStream>(json)
                .unwrap()
                .admission_diagnostics
                .is_none()
        );
        let (failed, error) = scan_movement_states_with_diagnostics(&[], &registry, &encoding);
        assert!(error.is_some());
        assert!(failed.admission_diagnostics.is_none());
    }

    #[test]
    fn native_movement_source_selection_and_framing() {
        let mut bytes = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/movement-source-v41.json.zlib")[..],
        )
        .read_to_end(&mut bytes)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(rows.len(), 384);
        for (case, row) in rows.iter().enumerate() {
            let chunks: Vec<_> = row["inputs"]
                .as_array()
                .unwrap()
                .iter()
                .map(|input| {
                    let hex = input["hex"].as_str().unwrap();
                    let data: Vec<_> = (0..hex.len())
                        .step_by(2)
                        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                        .collect();
                    let index = input["index"].as_i64().unwrap() as i32;
                    FilmChunkData {
                        metadata: crate::clients::hi::models::FilmChunk {
                            index,
                            chunk_type: if index < 1 { 1 } else { 2 },
                            start_time_offset_ms: 0,
                            duration_ms: 0,
                            size: data.len() as i64,
                            file_relative_path: String::new(),
                        },
                        data,
                    }
                })
                .collect();
            let mut encoding: FrameEncoding = serde_json::from_value(serde_json::json!({
                "ids": {"low_bits": 13, "base": 0}, "mpp_widths": [9, 5],
                "extra_fields": false, "corruption_check": false,
                "position": row["encoding"]
            }))
            .unwrap();
            encoding.corruption_check = false;
            let mut registry = FilmRegistry {
                archetypes: (0..36)
                    .map(|index| FilmArchetype {
                        index,
                        components: vec![],
                        levels: vec![],
                    })
                    .collect(),
                major_version: 41,
                format_version: 27,
                end_byte: 0,
                truncated: false,
            };
            registry.archetypes[35]
                .components
                .push("unit-crouch-component".into());
            registry.archetypes[35].levels.push(0);
            match row["registryCase"].as_u64().unwrap() {
                0 => {}
                1 => registry.archetypes.truncate(35),
                2 => {
                    registry.archetypes[35].components.clear();
                    registry.archetypes[35].levels.clear();
                }
                _ => panic!("unknown fixture registry case"),
            }
            let (stream, error) =
                scan_movement_states_with_diagnostics(&chunks, &registry, &encoding);
            if error.is_none() && row["registryCase"] == 0 {
                assert_eq!(
                    stream.admission_diagnostics,
                    Some(vec![]),
                    "measured empty {case}"
                );
            } else {
                assert!(
                    stream.admission_diagnostics.is_none(),
                    "setup not scanned {case}"
                );
            }
            let error_kind = match error.as_ref() {
                None => "",
                Some(DecodeError::Missing("readable film chunks")) => "source",
                Some(DecodeError::Missing("biped archetype")) => "biped",
                other => panic!("unexpected setup error in case {case}: {other:?}"),
            };
            assert_eq!(
                error_kind,
                row["errorKind"].as_str().unwrap(),
                "case {case}"
            );
            assert_eq!(
                error.is_some(),
                row["error"].as_bool().unwrap(),
                "case {case}"
            );
            assert_eq!(
                serde_json::to_value(&stream.stats).unwrap(),
                row["stats"],
                "case {case}"
            );
            assert_eq!(
                serde_json::to_value(&stream.records).unwrap(),
                if row["reads"].is_null() {
                    serde_json::json!([])
                } else {
                    row["reads"].clone()
                },
                "case {case}"
            );
            let table = build_native_anticipated_bindings(&chunks);
            assert_eq!(
                table.declarations as u64,
                row["count"].as_u64().unwrap(),
                "case {case}"
            );
            assert_eq!(table.conflicts as u64, row["conflicts"].as_u64().unwrap());
            let mut got: Vec<_> = table
                .entries
                .iter()
                .flat_map(|(&id, ds)| {
                    ds.iter()
                        .map(move |d| [u64::from(id), d.chunk_index as u64, u64::from(d.archetype)])
                })
                .collect();
            let mut expected: Vec<[u64; 3]> =
                serde_json::from_value(row["declarations"].clone()).unwrap();
            got.sort();
            expected.sort();
            assert_eq!(got, expected, "case {case}");
        }
    }
    #[test]
    fn raw_failure_counters_precede_source_and_archetype_errors() {
        let mut encoding: FrameEncoding = serde_json::from_value(serde_json::json!({
            "ids": {"low_bits": 13, "base": 0}, "mpp_widths": [9, 5],
            "extra_fields": false, "corruption_check": false
        }))
        .unwrap();
        encoding.position = Some(
            FilmMapBounds {
                module: String::new(),
                min: [0.; 3],
                max: [100.; 3],
                axis_widths: [17, 18, 19],
                region: 0,
                region_index_bits: 2,
            }
            .position_encoding(),
        );
        let registry = FilmRegistry {
            archetypes: vec![],
            major_version: 41,
            format_version: 27,
            end_byte: 8,
            truncated: false,
        };
        let chunks = [FilmChunkData {
            metadata: crate::clients::hi::models::FilmChunk {
                index: 1,
                chunk_type: 2,
                start_time_offset_ms: 0,
                duration_ms: 0,
                size: 0,
                file_relative_path: String::new(),
            },
            data: vec![],
        }];
        for (input, expected) in [
            (&[][..], DecodeError::Missing("readable film chunks")),
            (&chunks[..], DecodeError::Missing("biped archetype")),
        ] {
            let (stream, error) =
                scan_movement_states_with_diagnostics(input, &registry, &encoding);
            assert_eq!(error.as_ref().unwrap().to_string(), expected.to_string());
            assert_eq!(
                stream.stats,
                MovementStateStats {
                    map_widths: [17, 18, 19],
                    ..Default::default()
                }
            );
            assert!(stream.records.is_empty() && stream.padding.is_empty());
            let restored: MovementStateStream =
                serde_json::from_value(serde_json::to_value(&stream).unwrap()).unwrap();
            assert_eq!(restored, stream);
            assert_eq!(
                scan_movement_states(input, &registry, &encoding)
                    .unwrap_err()
                    .to_string(),
                error.unwrap().to_string()
            );
        }
    }
    #[test]
    fn velocity_and_jump_derivation_match_native() {
        let mut json = String::new();
        flate2::read::ZlibDecoder::new(
            include_bytes!("fixtures/jump-derivation-v41.json.zlib").as_slice(),
        )
        .read_to_string(&mut json)
        .unwrap();
        let oracle: serde_json::Value = serde_json::from_str(&json).unwrap();
        let velocities = oracle["velocities"].as_array().unwrap();
        assert_eq!(velocities.len(), 4096);
        for row in velocities {
            let actual = native_velocity(
                row["dir"].as_u64().unwrap() as u32,
                row["scale"].as_u64().unwrap() as u16,
            );
            let expected: [f32; 3] = serde_json::from_value(row["vector"].clone()).unwrap();
            assert_eq!(actual, expected, "{row}");
        }
        let runs = oracle["runs"].as_array().unwrap();
        assert_eq!(runs.len(), 1024);
        let mut total = 0;
        for row in runs {
            let mut scanner = Scanner::default();
            for v in row["input"].as_array().unwrap() {
                scanner
                    .velocities
                    .entry(v["slot"].as_u64().unwrap() as u32)
                    .or_default()
                    .push(VerticalVelocity {
                        ts: v["ts"].as_u64().unwrap(),
                        vz: v["vz"].as_f64().unwrap(),
                        chunk: v["chunk"].as_i64().unwrap(),
                        packet: v["packet"].as_u64().unwrap() as usize,
                    });
            }
            scanner.derive_jumps();
            let expected: Vec<MovementStateRead> =
                serde_json::from_value(row["records"].clone()).unwrap();
            assert_eq!(scanner.readings.into_values().collect::<Vec<_>>(), expected);
            let stats: MovementStateStats = serde_json::from_value(row["stats"].clone()).unwrap();
            assert_eq!(scanner.stats, stats);
            total += stats.jumps_derived;
        }
        assert!(total > 0);
    }
}

#[cfg(test)]
mod native_contract_tests {
    use super::*;
    use std::io::Read;
    fn round_trip<T: serde::de::DeserializeOwned + serde::Serialize>(rows: &serde_json::Value) {
        assert_eq!(rows.as_array().unwrap().len(), 64);
        for row in rows.as_array().unwrap() {
            let typed: T = serde_json::from_value(row.clone()).unwrap();
            assert_eq!(&serde_json::to_value(typed).unwrap(), row);
        }
    }
    #[test]
    fn native_type_contracts_preserve_all_exported_fields() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            include_bytes!("fixtures/type-contracts-v41.json.zlib").as_slice(),
        )
        .read_to_end(&mut raw)
        .unwrap();
        let oracle: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        let t = &oracle["types"];
        assert_eq!(t.as_object().unwrap().len(), 12);
        round_trip::<StatborgDeathInstant>(&t["DeathInstant"]);
        round_trip::<FlagGrabSpan>(&t["FlagSpan"]);
        round_trip::<FlagGrabTrack>(&t["FlagTrack"]);
        round_trip::<FlagGrabCounts>(&t["FlagGrabsNetPlayer"]);
        round_trip::<StatborgPlayerLine>(&t["PlayerLine"]);
        round_trip::<StatborgScorePoint>(&t["ScorePoint"]);
        round_trip::<StatborgValue>(&t["StatValue"]);
        round_trip::<StatborgRecord>(&t["StatRecord"]);
        round_trip::<MovementStateRead>(&t["MovementStateRead"]);
        round_trip::<MovementStateStats>(&t["MovementStateStats"]);
        round_trip::<AbilityChargeStats>(&t["AbilityChargeStats"]);
        round_trip::<AbilityImpulseStats>(&t["AbilityImpulseStats"]);
        assert_eq!(
            serde_json::json!({"crouch":MOVEMENT_CROUCH,"slide":MOVEMENT_SLIDE,"clamber":MOVEMENT_CLAMBER,"jump":MOVEMENT_JUMP_DERIVED,"sprint":MOVEMENT_SPRINT,"height":SPARTAN_JUMP_HEIGHT_M,"tolerance":SPARTAN_JUMP_HEIGHT_TOLERANCE,"rise_speed":SPARTAN_JUMP_RISE_MIN_MPS,"hold_us":SPARTAN_JUMP_HOLD_MAX_US}),
            oracle["constants"]
        );
    }
}
