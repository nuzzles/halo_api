//! Native optional inference record loop. The default view-admission guard still
//! takes precedence over inference. Component repair and resync are disabled in
//! this native path; explicit repair remains available as a separate operation.
use super::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct InferenceFrameOptions {
    pub chain_inference: bool,
    pub view_tables: bool,
}
impl Default for InferenceFrameOptions {
    fn default() -> Self {
        Self {
            chain_inference: false,
            view_tables: true,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
// Keep the existing inline evidence API; callback-order metadata extends the
// heap-backed diagnostics without requiring a new allocation per inference.
#[allow(clippy::large_enum_variant)]
pub enum InferenceEvidence {
    SingleStep { archetype: u32, end_bit: i64 },
    Chain(ChainInference),
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InferenceFrameRecord {
    pub header: RecordHeader,
    pub archetype: Option<u32>,
    /// Inferred skips do not fabricate decoded components or values.
    pub decoded: Option<EntityRecord>,
    pub inference: Option<InferenceEvidence>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InferenceFrameStop {
    End(RecordHeader),
    ViewRejected {
        header: RecordHeader,
        reason: FilmViewAdmission,
    },
    Failure(EntityViewStop),
    PayloadBoundary,
    RecordLimit,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InferenceFrame {
    #[serde(default, skip_serializing_if = "FilmReadDiagnostics::is_empty")]
    pub diagnostics: FilmReadDiagnostics,
    pub records: Vec<InferenceFrameRecord>,
    pub inferred_count: usize,
    pub end_bit: i64,
    pub padded_bits: usize,
    pub stop: InferenceFrameStop,
}
impl InferenceFrame {
    pub fn hit_end(&self) -> bool {
        matches!(
            self.stop,
            InferenceFrameStop::End(_) | InferenceFrameStop::ViewRejected { .. }
        )
    }
}
pub(super) fn read_bound_record_with_movement(
    data: &[u8],
    header_bit: impl TryInto<i64>,
    header: &RecordHeader,
    registry: &FilmRegistry,
    profile: &KillWalkProfile,
    world: &FilmWorld,
    movement_slot: Option<u32>,
) -> Option<EntityRecord> {
    read_bound_record_contextual(
        data,
        header_bit,
        header,
        registry,
        profile,
        world,
        super::components::RecordCaptureSlots {
            context: None,
            movement: movement_slot,
            position: movement_slot,
        },
    )
}
pub(crate) fn read_bound_record_contextual(
    data: &[u8],
    header_bit: impl TryInto<i64>,
    header: &RecordHeader,
    registry: &FilmRegistry,
    profile: &KillWalkProfile,
    world: &FilmWorld,
    slots: super::components::RecordCaptureSlots,
) -> Option<EntityRecord> {
    let _movement_binding = slots
        .context
        .as_ref()
        .and_then(|c| c.observer.as_ref())
        .zip(slots.movement)
        .map(|(observer, slot)| observer.scope_movement_binding(slot, world.archetype(slot)));
    let mut encoding = profile.encoding.clone();
    // This loop's only extra field is the prefix already consumed by its caller.
    encoding.extra_fields = false;
    let mut bindings = EntityBindings::default();
    if let Some(id) = header.id
        && let Some(ti) = world.archetype(id & 0x3fff_ffff)
    {
        bindings.bind(id, ti);
    }
    super::components::decode_entity_record_with_capture_slots_and_generation_policy(
        data,
        header_bit,
        registry,
        &encoding,
        &bindings,
        profile.simulation_complete,
        slots,
        false,
    )
}
/// Native single-step inference accepts exactly one archetype, even when two
/// archetypes share an end. Unlike the chain path, soft-bound successors confirm.
pub fn infer_unbound_archetype(
    data: &[u8],
    body_bit: impl TryInto<i64>,
    registry: &FilmRegistry,
    profile: &KillWalkProfile,
    world: &FilmWorld,
) -> Option<(u32, i64)> {
    infer_unbound_archetype_observed(data, body_bit, registry, profile, world).0
}
fn infer_unbound_archetype_observed(
    data: &[u8],
    body_bit: impl TryInto<i64>,
    registry: &FilmRegistry,
    profile: &KillWalkProfile,
    world: &FilmWorld,
) -> (Option<(u32, i64)>, FilmReadDiagnostics) {
    infer_unbound_archetype_contextual(data, body_bit, registry, profile, world, None)
}
pub(crate) fn infer_unbound_archetype_contextual(
    data: &[u8],
    body_bit: impl TryInto<i64>,
    registry: &FilmRegistry,
    profile: &KillWalkProfile,
    world: &FilmWorld,
    context: Option<&NativeReaderContext>,
) -> (Option<(u32, i64)>, FilmReadDiagnostics) {
    let _captures = context
        .and_then(|c| c.observer.as_ref())
        .map(|o| o.neutralize_captures());
    let mut diagnostics = FilmReadDiagnostics::default();
    let result = (|| {
        let body_bit = body_bit.try_into().ok()?;
        let mut winner = None;
        for ti in 0..registry.archetypes.len() {
            let (trial, observed) = super::components::chain_delta_body_trial_contextual(
                data,
                body_bit,
                registry,
                ti as u32,
                &profile.encoding,
                profile.simulation_complete,
                context,
            );
            diagnostics.merge(&observed);
            let Some((end, _)) = trial else { continue };
            if (data.len() as i64 * 8).wrapping_sub(end) < 24 {
                continue;
            }
            let header_bit = end.wrapping_add(if profile.encoding.extra_fields { 32 } else { 0 });
            let Some(header) =
                super::records::decode_frame_header_signed(data, header_bit, &profile.encoding)
            else {
                continue;
            };
            if header.kind != RecordKind::Delta
                || !header
                    .id
                    .is_some_and(|id| world.archetype(id & 0x3fff_ffff).is_some())
            {
                continue;
            }
            let Some(record) = read_bound_record_contextual(
                data,
                header_bit,
                &header,
                registry,
                profile,
                world,
                super::components::RecordCaptureSlots {
                    context: context.cloned(),
                    movement: context.map(|_| header.id.unwrap() & 0x3fff_ffff),
                    position: context.map(|_| header.id.unwrap() & 0x3fff_ffff),
                },
            ) else {
                continue;
            };
            diagnostics.merge(&record.diagnostics);
            if record.stop != EntityViewStop::Complete {
                continue;
            }
            if winner.is_some() {
                return None;
            }
            winner = Some((ti as u32, end));
        }
        winner
    })();
    if context.is_none() {
        diagnostics.suppress_unit_references();
    }
    (result, diagnostics)
}
/// Walk a record view from an explicit start, retaining decoded data separately
/// from inferred skips. Unique chain results create soft bindings; single-step
/// results never bind. The supplied world retains committed NEW/DELETE changes.
pub fn decode_inference_frame(
    data: &[u8],
    start: impl TryInto<i64>,
    registry: &FilmRegistry,
    profile: &KillWalkProfile,
    world: &mut FilmWorld,
    options: InferenceFrameOptions,
) -> Option<InferenceFrame> {
    decode_inference_frame_contextual(data, start, registry, profile, world, options, None)
}
pub(crate) fn decode_inference_frame_contextual(
    data: &[u8],
    start: impl TryInto<i64>,
    registry: &FilmRegistry,
    profile: &KillWalkProfile,
    world: &mut FilmWorld,
    options: InferenceFrameOptions,
    context: Option<&NativeReaderContext>,
) -> Option<InferenceFrame> {
    decode_inference_frame_with_capture(
        data, start, registry, profile, world, options, context, &mut 0,
    )
}

/// Generic views share one native reader and therefore one inherited capture slot.
#[allow(clippy::too_many_arguments)]
pub(crate) fn decode_inference_frame_with_capture(
    data: &[u8],
    start: impl TryInto<i64>,
    registry: &FilmRegistry,
    profile: &KillWalkProfile,
    world: &mut FilmWorld,
    options: InferenceFrameOptions,
    context: Option<&NativeReaderContext>,
    capture_slot: &mut u32,
) -> Option<InferenceFrame> {
    let start = start.try_into().ok()?;
    if !profile.encoding.valid() {
        return None;
    }
    let mut out = InferenceFrame {
        diagnostics: Default::default(),
        records: vec![],
        inferred_count: 0,
        end_bit: start,
        padded_bits: 0,
        stop: InferenceFrameStop::PayloadBoundary,
    };
    while out.end_bit < (data.len() * 8) as i64 {
        if out.records.len() >= 8192 {
            out.stop = InferenceFrameStop::RecordLimit;
            break;
        }
        let header_bit =
            out.end_bit
                .wrapping_add(if profile.encoding.extra_fields { 32 } else { 0 });
        let mut cursor = NativeFilmBits::new(data);
        cursor.set_position(header_bit);
        let width = profile
            .encoding
            .native_id_low_bits
            .or_else(|| i64::try_from(profile.encoding.ids.low_bits).ok())?;
        let header = decode_native_record_header(&mut cursor, width, profile.encoding.ids.base);
        out.end_bit = header.end_bit;
        if header.kind == RecordKind::End {
            out.stop = InferenceFrameStop::End(header);
            break;
        }
        let id = header.id?;
        let slot = id & 0x3fff_ffff;
        if header.kind == RecordKind::Delta && options.view_tables {
            let reason = world.admit_delta(id, true);
            if let Some(observer) = context.and_then(|c| c.observer.as_ref()) {
                observer.record_admission(&reason);
            }
            match &reason {
                FilmViewAdmission::Unbound => out.diagnostics.rejected_unbound += 1,
                FilmViewAdmission::OtherView => out.diagnostics.rejected_other_view += 1,
                FilmViewAdmission::Anticipated(d) => {
                    *out.diagnostics
                        .anticipated_bindings
                        .entry(d.archetype)
                        .or_default() += 1
                }
                _ => {}
            }
            if matches!(
                reason,
                FilmViewAdmission::Unbound | FilmViewAdmission::OtherView
            ) {
                out.stop = InferenceFrameStop::ViewRejected { header, reason };
                break;
            }
        }
        let mut row = InferenceFrameRecord {
            header: header.clone(),
            archetype: None,
            decoded: None,
            inference: None,
        };
        if header.kind == RecordKind::Delta && world.archetype(slot).is_none() {
            let result = if options.chain_inference {
                let inference = chain_inference::infer_chain_archetype_contextual(
                    data,
                    header.end_bit,
                    registry,
                    &profile.encoding,
                    world,
                    profile.simulation_complete,
                    context,
                );
                out.diagnostics.merge(&inference.diagnostics);
                let result = inference
                    .archetype
                    .map(|ti| (ti, inference.end_bit, inference.unique_archetype));
                row.inference = Some(InferenceEvidence::Chain(inference));
                result
            } else {
                let (inferred, diagnostics) = infer_unbound_archetype_contextual(
                    data,
                    header.end_bit,
                    registry,
                    profile,
                    world,
                    context,
                );
                out.diagnostics.merge(&diagnostics);
                inferred.map(|(ti, end)| {
                    row.inference = Some(InferenceEvidence::SingleStep {
                        archetype: ti,
                        end_bit: end,
                    });
                    (ti, end, false)
                })
            };
            if let Some((ti, end, unique)) = result {
                if options.chain_inference && unique {
                    world.bind_soft(id, ti);
                }
                row.archetype = Some(ti);
                out.end_bit = end;
                out.inferred_count += 1;
                out.records.push(row);
                continue;
            }
            out.records.push(row);
            out.stop = InferenceFrameStop::Failure(EntityViewStop::MissingBinding { id });
            break;
        }
        if header.kind == RecordKind::Delta {
            *capture_slot = slot;
        }
        let Some(record) = read_bound_record_contextual(
            data,
            header_bit,
            &header,
            registry,
            profile,
            world,
            super::components::RecordCaptureSlots {
                context: context.cloned(),
                movement: Some(*capture_slot),
                position: Some(*capture_slot),
            },
        ) else {
            out.records.push(row);
            out.stop = InferenceFrameStop::Failure(EntityViewStop::Truncated);
            break;
        };
        out.diagnostics.merge(&record.diagnostics);
        out.end_bit = record.end_bit;
        row.archetype = record.archetype;
        let stop = record.stop.clone();
        row.decoded = Some(record);
        out.records.push(row);
        if stop != EntityViewStop::Complete {
            out.stop = InferenceFrameStop::Failure(stop);
            break;
        }
        match header.kind {
            RecordKind::New => {
                if let Some(refusal) =
                    world.bind_native_new(id, out.records.last()?.archetype?, header_bit)
                {
                    if let Some(observer) = context.and_then(|c| c.observer.as_ref()) {
                        observer.record_new_binding_refusal(&refusal);
                    }
                    out.diagnostics.new_binding_refusals.push(refusal);
                }
            }
            RecordKind::Delete => world.unbind(slot),
            _ => {}
        }
    }
    out.padded_bits = super::bits::padded_from_native(out.end_bit, data.len() * 8);
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::BTreeMap, io::Read};
    #[test]
    fn native_movement_hook_inference() {
        check_movement_frames(
            include_bytes!("fixtures/movement-hook-inference-v41.json.zlib"),
            0,
        );
    }
    #[test]
    fn native_movement_hook_recovery_slots() {
        check_movement_frames(
            include_bytes!("fixtures/movement-hook-recovery-slots-v41.json.zlib"),
            1,
        );
    }
    #[test]
    fn native_movement_hook_generic() {
        check_movement_frames(
            include_bytes!("fixtures/movement-hook-generic-v41.json.zlib"),
            2,
        );
    }
    #[test]
    fn native_position_hook_inference() {
        check_movement_frames(
            include_bytes!("fixtures/position-hook-inference-v41.json.zlib"),
            0,
        );
    }
    #[test]
    fn native_position_hook_generic() {
        check_movement_frames(
            include_bytes!("fixtures/position-hook-generic-v41.json.zlib"),
            2,
        );
    }
    #[test]
    fn native_position_hook_recovery_slots() {
        check_movement_frames(
            include_bytes!("fixtures/position-hook-recovery-slots-v41.json.zlib"),
            1,
        );
    }
    #[test]
    fn native_position_hook_march() {
        check_movement_frames(
            include_bytes!("fixtures/position-hook-march-v41.json.zlib"),
            3,
        );
    }
    #[test]
    fn native_movement_hook_march() {
        check_movement_frames(
            include_bytes!("fixtures/movement-hook-march-v41.json.zlib"),
            3,
        );
    }
    #[test]
    fn native_position_hook_locator() {
        check_movement_frames(
            include_bytes!("fixtures/position-hook-locator-v41.json.zlib"),
            4,
        );
    }
    #[test]
    fn native_body_profile_generic() {
        check_movement_frames(
            include_bytes!("fixtures/body-profile-generic-v41.json.zlib"),
            2,
        );
    }
    #[test]
    fn native_body_profile_inference() {
        check_movement_frames(
            include_bytes!("fixtures/body-profile-inference-v41.json.zlib"),
            0,
        );
    }
    #[test]
    fn native_calibrated_position_generic() {
        check_movement_frames(
            include_bytes!("fixtures/calibrated-position-generic-v41.json.zlib"),
            2,
        );
    }
    #[test]
    fn native_calibrated_position_inference() {
        check_movement_frames(
            include_bytes!("fixtures/calibrated-position-inference-v41.json.zlib"),
            0,
        );
    }
    #[test]
    fn native_new_record_profile_generic() {
        check_movement_frames(
            include_bytes!("fixtures/new-record-profile-generic-v41.json.zlib"),
            2,
        );
    }
    #[test]
    fn native_new_record_profile_inference() {
        check_movement_frames(
            include_bytes!("fixtures/new-record-profile-inference-v41.json.zlib"),
            0,
        );
    }
    #[test]
    fn native_width_override_generic() {
        check_movement_frames(
            include_bytes!("fixtures/width-override-generic-v41.json.zlib"),
            2,
        );
    }
    #[test]
    fn native_width_override_inference() {
        check_movement_frames(
            include_bytes!("fixtures/width-override-inference-v41.json.zlib"),
            0,
        );
    }
    #[test]
    fn native_baseline_scope_generic() {
        check_movement_frames(
            include_bytes!("fixtures/baseline-scope-generic-v41.json.zlib"),
            2,
        );
    }
    #[test]
    fn native_baseline_scope_inference() {
        check_movement_frames(
            include_bytes!("fixtures/baseline-scope-inference-v41.json.zlib"),
            0,
        );
    }
    fn check_movement_frames(fixture: &[u8], mode: u8) {
        #[derive(Deserialize)]
        struct Case {
            #[serde(default)]
            component_widths: ComponentWidthOverrides,
            #[serde(default)]
            corruption_check: bool,
            archetype: Option<usize>,
            #[serde(default)]
            new_record: NewRecordEncoding,
            #[serde(default)]
            located: i64,
            capture: Option<PositionCaptureEncoding>,
            name: String,
            level: u32,
            unbound: bool,
            hex: String,
            encoding: PositionEncoding,
            observations: Vec<FilmComponentObservation>,
            records: serde_json::Value,
            inferred: usize,
            hit: bool,
            tables: bool,
            chain: bool,
            end: usize,
        }
        let mut json = String::new();
        flate2::read::ZlibDecoder::new(fixture)
            .read_to_string(&mut json)
            .unwrap();
        let cases: Vec<Case> = serde_json::from_str(&json).unwrap();
        assert_eq!(cases.len(), 512);
        for (i, c) in cases.into_iter().enumerate() {
            let archetype = c.archetype.unwrap_or(35);
            let mut registry = FilmRegistry {
                archetypes: (0..50)
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
            registry.archetypes[archetype].components = vec![c.name];
            registry.archetypes[archetype].levels = vec![c.level];
            let data: Vec<_> = c
                .hex
                .as_bytes()
                .as_chunks::<2>()
                .0
                .iter()
                .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap())
                .collect();
            let encoding = FrameEncoding {
                keyframe_layout: Default::default(),
                keyframe_simulation_complete: None,
                native_id_low_bits: None,
                component_widths: c.component_widths,
                new_record: c.new_record,
                position_capture: c.capture,
                ids: RecordIdLayout {
                    low_bits: 11,
                    base: 0,
                },
                mpp_widths: [9, 5],
                position: Some(c.encoding),
                extra_fields: false,
                corruption_check: c.corruption_check,
            };
            let mut world = FilmWorld::default();
            world.bind_full(50, archetype as u32);
            if !c.unbound {
                world.bind_full(51, archetype as u32);
            }
            let profile = KillWalkProfile {
                encoding,
                simulation_complete: true,
            };
            if mode == 1 {
                let frame = decode_frame_resync(
                    &data,
                    &registry,
                    &profile,
                    &mut world,
                    &std::collections::BTreeSet::from([50, 51]),
                    |_| true,
                )
                .unwrap();
                assert_eq!(
                    frame.observations.caller_visible.component_observations, c.observations,
                    "resync observations {i}"
                );
                let records: Vec<_> = frame
                    .records
                    .iter()
                    .map(|r| {
                        serde_json::json!({
                            "id":r.header.id.unwrap(), "end":r.end_bit
                        })
                    })
                    .collect();
                assert_eq!(
                    serde_json::to_value(records).unwrap(),
                    c.records,
                    "resync records {i}"
                );
                continue;
            }
            if mode == 4 {
                world.bind_full(123, 35);
                let located = locate_march_event_records_observed(
                    &data,
                    &registry,
                    &profile.encoding,
                    &world,
                    true,
                    true,
                );
                assert_eq!(
                    located.start.map_or(-1, |s| s as i64),
                    c.located,
                    "locator offset {i}"
                );
                assert_eq!(
                    located.diagnostics.component_observations, c.observations,
                    "locator observations {i}"
                );
                assert_eq!(
                    locate_march_event_records(&data, &registry, &profile.encoding, &world, true),
                    located.start,
                    "fast locator {i}"
                );
                let restored: MarchLocatorObservations =
                    serde_json::from_slice(&serde_json::to_vec(&located).unwrap()).unwrap();
                assert_eq!(restored, located);
                continue;
            }
            if mode == 3 {
                let before = world.clone();
                let records =
                    walk_march_records(&data, 13, &registry, &profile.encoding, &world, true);
                assert_eq!(world, before, "march world restored {i}");
                let observations: Vec<_> = records
                    .iter()
                    .flat_map(|r| r.diagnostics.component_observations.iter().cloned())
                    .collect();
                assert_eq!(observations, c.observations, "march observations {i}");
                let records: Vec<_> = records
                    .iter()
                    .map(|r| serde_json::json!({"id":r.header.id.unwrap(), "end":r.end_bit}))
                    .collect();
                assert_eq!(
                    serde_json::to_value(records).unwrap(),
                    c.records,
                    "march records {i}"
                );
                continue;
            }
            if mode == 2 {
                let mut bindings = EntityBindings::default();
                bindings.bind(50, archetype as u32);
                if !c.unbound {
                    bindings.bind(51, archetype as u32);
                }
                let view =
                    decode_entity_view(&data, 13, &registry, &profile.encoding, &mut bindings);
                let observations: Vec<_> = view
                    .records
                    .iter()
                    .flat_map(|r| r.diagnostics.component_observations.iter().cloned())
                    .collect();
                assert_eq!(observations, c.observations, "generic observations {i}");
                assert_eq!(
                    serde_json::json!(view.end_bit),
                    serde_json::json!(c.end),
                    "generic end {i}"
                );
                assert_eq!(
                    view.stop == EntityViewStop::Complete,
                    c.hit,
                    "generic complete {i}"
                );
                let records: Vec<_> = view
                    .records
                    .iter()
                    .filter(|r| r.header.kind != RecordKind::End)
                    .map(|r| serde_json::json!({"id":r.header.id.unwrap(),"end":r.end_bit}))
                    .collect();
                assert_eq!(
                    serde_json::to_value(records).unwrap(),
                    c.records,
                    "generic records {i}"
                );
                continue;
            }
            let frame = decode_inference_frame(
                &data,
                13,
                &registry,
                &profile,
                &mut world,
                InferenceFrameOptions {
                    chain_inference: c.chain,
                    view_tables: c.tables,
                },
            )
            .unwrap();
            let observations = &frame.diagnostics.component_observations;
            assert_eq!(observations, &c.observations, "production observations {i}");
            assert_eq!(
                serde_json::json!(frame.end_bit),
                serde_json::json!(c.end),
                "end {i}"
            );
            let records: Vec<_> = frame
                .records
                .iter()
                .map(|r| {
                    serde_json::json!({
                        "id":r.header.id.unwrap(), "end":r.decoded.as_ref().map_or(0, |r| r.end_bit)
                    })
                })
                .collect();
            assert_eq!(
                serde_json::to_value(records).unwrap(),
                c.records,
                "records {i}"
            );
            assert_eq!(frame.inferred_count, c.inferred, "inferred {i}");
            assert_eq!(frame.hit_end(), c.hit, "hit {i}");
            let restored: InferenceFrame =
                serde_json::from_slice(&serde_json::to_vec(&frame).unwrap()).unwrap();
            assert_eq!(restored, frame);
        }
    }
    #[test]
    fn native_inference_frame_loop() {
        compare_frames(include_bytes!("fixtures/inference-frame-v41.json.zlib"));
    }
    #[test]
    fn native_frame_diagnostics() {
        compare_frames(include_bytes!("fixtures/frame-diagnostics-v41.json.zlib"));
    }
    fn compare_frames(fixture: &[u8]) {
        #[derive(Deserialize)]
        struct Case {
            #[serde(default)]
            anticipated: bool,
            diagnostics: Option<FilmReadDiagnostics>,
            hex: String,
            names: Vec<Vec<String>>,
            chain: bool,
            tables: bool,
            extra: bool,
            position: PositionEncoding,
            records: serde_json::Value,
            inferred: usize,
            hit: bool,
            end: usize,
            slots: serde_json::Value,
        }
        let mut json = String::new();
        flate2::read::ZlibDecoder::new(fixture)
            .read_to_string(&mut json)
            .unwrap();
        let cases: Vec<Case> = serde_json::from_str(&json).unwrap();
        assert_eq!(cases.len(), 1024);
        for (i, c) in cases.into_iter().enumerate() {
            let data: Vec<u8> = (0..c.hex.len())
                .step_by(2)
                .map(|n| u8::from_str_radix(&c.hex[n..n + 2], 16).unwrap())
                .collect();
            let registry = FilmRegistry {
                archetypes: c
                    .names
                    .into_iter()
                    .enumerate()
                    .map(|(index, components)| FilmArchetype {
                        index,
                        levels: vec![0; components.len()],
                        components,
                    })
                    .collect(),
                major_version: 41,
                format_version: 27,
                end_byte: 0,
                truncated: false,
            };
            let profile = KillWalkProfile {
                encoding: FrameEncoding {
                    keyframe_layout: Default::default(),
                    keyframe_simulation_complete: None,
                    native_id_low_bits: None,
                    component_widths: Default::default(),
                    new_record: Default::default(),
                    position_capture: None,
                    ids: RecordIdLayout {
                        low_bits: 11,
                        base: 0,
                    },
                    mpp_widths: [9, 5],
                    position: Some(c.position),
                    extra_fields: c.extra,
                    corruption_check: false,
                },
                simulation_complete: false,
            };
            let mut world = FilmWorld::default();
            world.bind_full(50, 0);
            world.bind_soft(51, 0);
            world.current_view = 1;
            world.bind_full(52, 0);
            world.current_view = 0;
            if c.anticipated {
                world.current_chunk = 1;
                world.anticipated = Some(AnticipatedBindings::from_declarations(
                    [53, 54, 100].map(|id| (id, 2, 0)),
                ));
            }
            let frame = decode_inference_frame(
                &data,
                0,
                &registry,
                &profile,
                &mut world,
                InferenceFrameOptions {
                    chain_inference: c.chain,
                    view_tables: c.tables,
                },
            )
            .unwrap();
            let records:Vec<_>=frame.records.iter().map(|r|{
                let desync=if let Some(record)=&r.decoded{match record.stop{EntityViewStop::Complete=>-1,EntityViewStop::UnsupportedComponent{index,..}|EntityViewStop::InvalidComponent{index}=>index as i64,_=>0}}else if r.archetype.is_some(){-1}else{0};
                serde_json::json!({"kind":match r.header.kind{RecordKind::End=>0,RecordKind::New=>1,RecordKind::Delete=>2,RecordKind::Delta=>3},"id":r.header.id.unwrap(),"ti":r.archetype.unwrap_or(0),"desync":desync,"mask":r.decoded.as_ref().and_then(|r|r.mask).unwrap_or(0),"components":r.decoded.as_ref().map_or(0,|r|r.attempts.len())})
            }).collect();
            assert_eq!(
                serde_json::to_value(records).unwrap(),
                c.records,
                "records {i}"
            );
            if let Some(expected) = c.diagnostics {
                assert_eq!(frame.diagnostics, expected, "diagnostics {i}");
            }
            assert_eq!(frame.inferred_count, c.inferred, "inferred {i}");
            assert_eq!(
                serde_json::json!(frame.end_bit),
                serde_json::json!(c.end),
                "end {i}"
            );
            assert_eq!(frame.hit_end(), c.hit, "hit {i}");
            let slots:BTreeMap<_,_>=world.slots.iter().map(|(slot,state)|(*slot,serde_json::json!({"ti":state.archetype,"id":state.full_id,"soft":state.soft,"view":state.view.unwrap_or(-1)}))).collect();
            assert_eq!(serde_json::to_value(slots).unwrap(), c.slots, "world {i}");
            let restored: InferenceFrame =
                serde_json::from_slice(&serde_json::to_vec(&frame).unwrap()).unwrap();
            assert_eq!(restored, frame);
        }
    }
}
