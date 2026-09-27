//! Native frame-view orchestration and explicit target harvesting.
use super::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct InferenceViewsOptions {
    pub inference: InferenceFrameOptions,
    pub view_classes: bool,
    pub view_count: i64,
    pub skip_lead_bits: i64,
    pub packet_preamble_bits: i64,
}
impl Default for InferenceViewsOptions {
    fn default() -> Self {
        Self {
            inference: InferenceFrameOptions::default(),
            view_classes: true,
            view_count: 3,
            skip_lead_bits: 2,
            packet_preamble_bits: 2,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InferenceViews {
    pub messages: Option<DecodedFrameView>,
    pub entities: Vec<InferenceFrame>,
    pub controls: Option<DecodedFrameView>,
    pub views_completed: usize,
    pub end_bit: i64,
    pub padded_bits: usize,
}
/// Native class mode always reads A/B/C, regardless of view_count. Generic mode
/// reads up to view_count entity views, committing each view's world mutations.
/// Stops at a failed view boundary and retains the exact final cursor.
pub fn decode_inference_views(
    data: &[u8],
    registry: &FilmRegistry,
    profile: &KillWalkProfile,
    world: &mut FilmWorld,
    options: InferenceViewsOptions,
) -> Option<InferenceViews> {
    decode_inference_views_contextual(data, registry, profile, world, options, None)
}
pub(crate) fn decode_inference_views_contextual(
    data: &[u8],
    registry: &FilmRegistry,
    profile: &KillWalkProfile,
    world: &mut FilmWorld,
    options: InferenceViewsOptions,
    context: Option<&NativeReaderContext>,
) -> Option<InferenceViews> {
    if !profile.encoding.valid() {
        return None;
    }
    let mut out = InferenceViews {
        messages: None,
        entities: vec![],
        controls: None,
        views_completed: 0,
        end_bit: options.skip_lead_bits,
        padded_bits: 0,
    };
    let mut capture_slot = 0;
    if options.view_classes {
        if options.skip_lead_bits == options.packet_preamble_bits
            && options.packet_preamble_bits >= 1
        {
            let messages = decode_message_view_signed(data, options.packet_preamble_bits - 1);
            out.end_bit = messages.end_bit;
            let complete = messages.stop == FrameViewStop::Complete;
            out.messages = Some(messages);
            if !complete {
                out.padded_bits = super::bits::padded_from_native(out.end_bit, data.len() * 8);
                if let Some(observer) = context.and_then(|c| c.observer.as_ref()) {
                    observer.publish_control_verdict(data, None);
                }
                return Some(out);
            }
            out.views_completed += 1;
        }
        world.current_view = 0;
        let view = super::inference_frame::decode_inference_frame_with_capture(
            data,
            out.end_bit,
            registry,
            profile,
            world,
            options.inference,
            context,
            &mut capture_slot,
        )?;
        out.end_bit = view.end_bit;
        let complete = view.hit_end();
        out.entities.push(view);
        if complete {
            out.views_completed += 1;
            let controls = components::decode_control_view_contextual(
                data,
                out.end_bit,
                profile.encoding.position.as_ref(),
                context,
            );
            out.end_bit = controls.end_bit;
            out.views_completed += usize::from(controls.stop == FrameViewStop::Complete);
            out.controls = Some(controls);
        }
    } else {
        for v in 0..options.view_count {
            if out.end_bit >= (data.len() as i64 * 8).wrapping_sub(3) {
                break;
            }
            world.current_view = v as i8;
            let start = out.end_bit;
            let view = super::inference_frame::decode_inference_frame_with_capture(
                data,
                start,
                registry,
                profile,
                world,
                options.inference,
                context,
                &mut capture_slot,
            )?;
            out.end_bit = view.end_bit;
            let complete = view.hit_end();
            out.entities.push(view);
            if !complete {
                break;
            }
            out.views_completed += 1;
            if out.end_bit == start {
                break;
            }
        }
    }
    if options.view_classes
        && let Some(observer) = context.and_then(|c| c.observer.as_ref())
    {
        observer.publish_control_verdict(data, out.controls.as_ref());
    }
    out.padded_bits = super::bits::padded_from_native(out.end_bit, data.len() * 8);
    Some(out)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HarvestConfirmation {
    NextBound,
    ChainWalk,
}

#[derive(Clone, Copy)]
enum DeltaReadMode {
    Trial,
    HardTrial,
    Observed,
    RawObserved,
    Reread,
}

fn clean_delta_at(
    data: &[u8],
    start: impl TryInto<i64>,
    registry: &FilmRegistry,
    profile: &KillWalkProfile,
    world: &FilmWorld,
    diagnostics: &mut FilmReadDiagnostics,
    mode: DeltaReadMode,
) -> Option<EntityRecord> {
    clean_delta_contextual(
        data,
        start,
        registry,
        profile,
        world,
        diagnostics,
        (mode, None),
    )
}
fn clean_delta_contextual(
    data: &[u8],
    start: impl TryInto<i64>,
    registry: &FilmRegistry,
    profile: &KillWalkProfile,
    world: &FilmWorld,
    diagnostics: &mut FilmReadDiagnostics,
    policy: (DeltaReadMode, Option<&NativeReaderContext>),
) -> Option<EntityRecord> {
    let (mode, context) = policy;
    let start = start.try_into().ok()?;
    if (data.len() as i64 * 8).wrapping_sub(start) < 24 {
        return None;
    }
    let header_bit = start.wrapping_add(if profile.encoding.extra_fields { 32 } else { 0 });
    let header = super::records::decode_frame_header_signed(data, header_bit, &profile.encoding)?;
    if header.kind != RecordKind::Delta {
        return None;
    }
    if matches!(mode, DeltaReadMode::HardTrial)
        && !header
            .id
            .is_some_and(|id| world.hard_bound(id & 0x3fff_ffff))
    {
        return None;
    }
    let movement_slot = (context.is_some()
        || matches!(
            mode,
            DeltaReadMode::Observed | DeltaReadMode::RawObserved | DeltaReadMode::Reread
        ))
    .then(|| header.id.unwrap_or(0) & 0x3fff_ffff);
    let mut record = super::inference_frame::read_bound_record_contextual(
        data,
        header_bit,
        &header,
        registry,
        profile,
        world,
        super::components::RecordCaptureSlots {
            context: context.cloned(),
            movement: movement_slot,
            position: movement_slot,
        },
    )?;
    if matches!(mode, DeltaReadMode::RawObserved) {
        // Native raw scanning replaces the caller's position hook with a local
        // first-position collector; other hook closures remain shared.
        record.diagnostics.suppress_positions();
    }
    diagnostics.merge(&record.diagnostics);
    (matches!(mode, DeltaReadMode::Reread) || record.stop == EntityViewStop::Complete)
        .then_some(record)
}
pub(crate) fn next_bound_clean(
    data: &[u8],
    pos: i64,
    registry: &FilmRegistry,
    profile: &KillWalkProfile,
    world: &FilmWorld,
    diagnostics: &mut FilmReadDiagnostics,
    context: Option<&NativeReaderContext>,
) -> bool {
    let remaining = (data.len() as i64 * 8).wrapping_sub(pos);
    if remaining < 24 {
        if !(0..=15).contains(&remaining) {
            return false;
        }
        let mut reader = NativeFilmBits::new(data);
        reader.set_position(pos);
        return (0..remaining).all(|_| !reader.read_bit());
    }
    clean_delta_contextual(
        data,
        pos,
        registry,
        profile,
        world,
        diagnostics,
        (DeltaReadMode::HardTrial, context),
    )
    .is_some_and(|r| {
        r.header
            .id
            .is_some_and(|id| world.hard_bound(id & 0x3fff_ffff))
            && r.end_bit <= (data.len() * 8) as i64
            && !r.components.is_empty()
    })
}
/// Exhaustive native target harvest, with overlapping accepted records collapsed.
/// This is a heuristic bit scan, not a sequential record-chain decode. NextBound
/// also accepts a short all-zero tail (0..15 bits), exactly as the native probe;
/// ChainWalk uses the stronger recursive continuation check. Returned fields
/// retain their actual bit offsets and do not mutate world state during trials.
pub fn scan_frame_targets(
    data: &[u8],
    registry: &FilmRegistry,
    profile: &KillWalkProfile,
    world: &FilmWorld,
    targets: &std::collections::BTreeSet<u32>,
    mode: HarvestConfirmation,
) -> Vec<EntityRecord> {
    scan_frame_targets_observed(data, registry, profile, world, targets, mode)
        .result
        .unwrap_or_default()
}
/// Native harvest observer state, including rejected trials and accepted rereads.
pub fn scan_frame_targets_observed(
    data: &[u8],
    registry: &FilmRegistry,
    profile: &KillWalkProfile,
    world: &FilmWorld,
    targets: &std::collections::BTreeSet<u32>,
    mode: HarvestConfirmation,
) -> FilmReadAttempt<Vec<EntityRecord>> {
    scan_frame_targets_contextual(data, registry, profile, world, targets, mode, None)
}
pub(crate) fn scan_frame_targets_contextual(
    data: &[u8],
    registry: &FilmRegistry,
    profile: &KillWalkProfile,
    world: &FilmWorld,
    targets: &std::collections::BTreeSet<u32>,
    mode: HarvestConfirmation,
    context: Option<&NativeReaderContext>,
) -> FilmReadAttempt<Vec<EntityRecord>> {
    let mut diagnostics = FilmReadDiagnostics::default();
    if !profile.encoding.valid() {
        return FilmReadAttempt {
            result: None,
            diagnostics,
        };
    }
    let mut confirm =
        super::chain_inference::harvest_confirmer(data, registry, profile, world, context);
    let mut records = Vec::new();
    let mut bit = 0i64;
    while bit < (data.len() as i64 * 8).wrapping_sub(24) {
        let scope = context
            .and_then(|c| c.observer.as_ref())
            .map(NativeFilmObserver::neutralize_position_capture);
        let candidate = clean_delta_contextual(
            data,
            bit,
            registry,
            profile,
            world,
            &mut diagnostics,
            (DeltaReadMode::Trial, context),
        );
        let confirmed = candidate.as_ref().is_some_and(|record| {
            record
                .header
                .id
                .is_some_and(|id| targets.contains(&(id & 0x3fff_ffff)))
                && !record.components.is_empty()
                && match mode {
                    HarvestConfirmation::NextBound => next_bound_clean(
                        data,
                        record.end_bit,
                        registry,
                        profile,
                        world,
                        &mut diagnostics,
                        context,
                    ),
                    HarvestConfirmation::ChainWalk => {
                        let (confirmed, observed) = confirm(record.end_bit);
                        diagnostics.merge(&observed);
                        confirmed
                    }
                }
        });
        drop(scope);
        if confirmed {
            let record = candidate.unwrap();
            let reread = clean_delta_contextual(
                data,
                bit,
                registry,
                profile,
                world,
                &mut diagnostics,
                (DeltaReadMode::Reread, context),
            )
            .unwrap_or(record);
            bit = reread.end_bit;
            records.push(reread);
            continue;
        }
        bit = bit.wrapping_add(1);
    }
    FilmReadAttempt {
        result: Some(records),
        diagnostics,
    }
}

/// Position capture kind shared with the native component observer.
pub type ResyncPositionKind = NativePositionKind;
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ResyncPosition {
    pub kind: ResyncPositionKind,
    pub vector: [f32; 3],
    pub bit: i64,
}
/// First native i0 observation without accumulation. Delta variants remain
/// relative. Map and quantum must match the context used to decode the record.
pub fn resync_position(
    record: &EntityRecord,
    map: &FilmMapBounds,
    quantum: f32,
) -> Option<ResyncPosition> {
    capture_record_positions(
        record,
        record.header.id.unwrap_or(0) & 0x3fff_ffff,
        map,
        quantum,
        None,
    )
    .first()
    .map(|sample| ResyncPosition {
        kind: sample.kind,
        vector: sample.vector,
        bit: sample.bit,
    })
}

/// Find the first clean, substantive target delta at or after `from`. This native
/// heuristic does not confirm the successor. The acceptance callback receives the
/// candidate's complete fields; use `resync_position` for native position filtering.
/// Rejected trials never mutate world state or appear in the returned records.
pub fn scan_for_target_delta(
    data: &[u8],
    from: impl TryInto<i64>,
    registry: &FilmRegistry,
    profile: &KillWalkProfile,
    world: &FilmWorld,
    targets: &std::collections::BTreeSet<u32>,
    accept: impl FnMut(&EntityRecord) -> bool,
) -> Option<(i64, EntityRecord)> {
    scan_for_target_delta_observed(data, from, registry, profile, world, targets, accept).result
}
/// All reads made in the temporary native capture observer. Use
/// `RawResyncDiagnostics::absorb_scan` to apply native caller-map ownership.
pub fn scan_for_target_delta_observed(
    data: &[u8],
    from: impl TryInto<i64>,
    registry: &FilmRegistry,
    profile: &KillWalkProfile,
    world: &FilmWorld,
    targets: &std::collections::BTreeSet<u32>,
    accept: impl FnMut(&EntityRecord) -> bool,
) -> FilmReadAttempt<(i64, EntityRecord)> {
    scan_for_target_delta_with_movement(
        data,
        (from, true),
        registry,
        profile,
        world,
        targets,
        accept,
    )
}
fn scan_for_target_delta_with_movement(
    data: &[u8],
    scan: (impl TryInto<i64>, bool),
    registry: &FilmRegistry,
    profile: &KillWalkProfile,
    world: &FilmWorld,
    targets: &std::collections::BTreeSet<u32>,
    mut accept: impl FnMut(&EntityRecord) -> bool,
) -> FilmReadAttempt<(i64, EntityRecord)> {
    let (from, capture_movement) = scan;
    let mode = if capture_movement {
        DeltaReadMode::RawObserved
    } else {
        DeltaReadMode::Trial
    };
    let mut diagnostics = FilmReadDiagnostics::default();
    let result = (|| {
        let from = from.try_into().ok()?;
        if !profile.encoding.valid() {
            return None;
        }
        for bit in from..(data.len() as i64 * 8).wrapping_sub(24) {
            if let Some(record) =
                clean_delta_at(data, bit, registry, profile, world, &mut diagnostics, mode)
                && record
                    .header
                    .id
                    .is_some_and(|id| targets.contains(&(id & 0x3fff_ffff)))
                && !record.components.is_empty()
                && accept(&record)
            {
                return Some((bit, record));
            }
        }
        None
    })();
    FilmReadAttempt {
        result,
        diagnostics,
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResyncFrame {
    #[serde(default)]
    pub observations: RawResyncDiagnostics,
    pub records: Vec<EntityRecord>,
    /// Candidate start bits reached by scanning after a failed sequential read.
    pub resync_bits: Vec<i64>,
    pub end_bit: i64,
    pub stop: InferenceFrameStop,
}
/// Port of native DecodeFrameResync. Starts at bit zero, follows sequential records,
/// and scans from one bit after each failed record start. NEW and DELETE commit
/// bindings; successful resync deltas do not infer bindings. The callback filters
/// only scanned candidates. This explicit heuristic is for offline extraction.
pub fn decode_frame_resync(
    data: &[u8],
    registry: &FilmRegistry,
    profile: &KillWalkProfile,
    world: &mut FilmWorld,
    targets: &std::collections::BTreeSet<u32>,
    accept: impl FnMut(&EntityRecord) -> bool,
) -> Option<ResyncFrame> {
    decode_frame_resync_observed(
        data,
        registry,
        profile,
        world,
        targets,
        RawResyncDiagnostics::default(),
        accept,
    )
}
/// Raw frame resync with explicit initial histogram ownership and all trial counts.
pub fn decode_frame_resync_observed(
    data: &[u8],
    registry: &FilmRegistry,
    profile: &KillWalkProfile,
    world: &mut FilmWorld,
    targets: &std::collections::BTreeSet<u32>,
    observations: RawResyncDiagnostics,
    mut accept: impl FnMut(&EntityRecord) -> bool,
) -> Option<ResyncFrame> {
    if !profile.encoding.valid() {
        return None;
    }
    let mut out = ResyncFrame {
        observations,
        records: vec![],
        resync_bits: vec![],
        end_bit: 0,
        stop: InferenceFrameStop::PayloadBoundary,
    };
    let mut capture_slot = 0;
    for _ in 0..4096 {
        if out.end_bit >= (data.len() * 8) as i64 {
            return Some(out);
        }
        let start = out.end_bit;
        let header_bit = start.wrapping_add(if profile.encoding.extra_fields { 32 } else { 0 });
        let header =
            super::records::decode_frame_header_signed(data, header_bit, &profile.encoding)?;
        out.end_bit = header.end_bit;
        if header.kind == RecordKind::End {
            out.stop = InferenceFrameStop::End(header);
            return Some(out);
        }
        if header.kind == RecordKind::Delta {
            capture_slot = header.id? & 0x3fff_ffff;
        }
        let record = super::inference_frame::read_bound_record_with_movement(
            data,
            header_bit,
            &header,
            registry,
            profile,
            world,
            Some(capture_slot),
        );
        if let Some(record) = record.as_ref() {
            out.observations.absorb_direct(&record.diagnostics);
            out.end_bit = record.end_bit;
        }
        if let Some(record) = record
            .as_ref()
            .filter(|r| r.stop == EntityViewStop::Complete)
        {
            match header.kind {
                RecordKind::New => world.bind_full(header.id?, record.archetype?),
                RecordKind::Delete => world.unbind(header.id? & 0x3fff_ffff),
                _ => {}
            }
            out.records.push(record.clone());
            continue;
        }
        let scanned = scan_for_target_delta_with_movement(
            data,
            (start.wrapping_add(1), false),
            registry,
            profile,
            world,
            targets,
            &mut accept,
        );
        out.observations.absorb_scan(&scanned.diagnostics);
        let Some((bit, recovered)) = scanned.result.filter(|(bit, _)| *bit >= 0) else {
            out.stop =
                InferenceFrameStop::Failure(record.map_or(EntityViewStop::Truncated, |r| r.stop));
            return Some(out);
        };
        let mut reread_diagnostics = FilmReadDiagnostics::default();
        let recovered = clean_delta_at(
            data,
            bit,
            registry,
            profile,
            world,
            &mut reread_diagnostics,
            DeltaReadMode::Observed,
        )
        .unwrap_or(recovered);
        out.observations.absorb_direct(&reread_diagnostics);
        out.end_bit = recovered.end_bit;
        out.resync_bits.push(bit);
        out.records.push(recovered);
        // Native resync replaces the sequential reader, resetting its capture slot.
        capture_slot = 0;
    }
    out.stop = InferenceFrameStop::RecordLimit;
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::BTreeMap, io::Read};
    #[test]
    fn native_frame_resync() {
        compare_raw_resync(include_bytes!("fixtures/frame-resync-v41.json.zlib"));
    }
    #[test]
    fn native_raw_resync_diagnostics() {
        compare_raw_resync(include_bytes!(
            "fixtures/raw-resync-diagnostics-v41.json.zlib"
        ));
    }
    #[test]
    fn native_mobility_resync() {
        compare_raw_resync(include_bytes!("fixtures/mobility-resync-v41.json.zlib"));
    }
    #[test]
    fn native_movement_hook_resync() {
        compare_raw_resync(include_bytes!(
            "fixtures/movement-hook-resync-v41.json.zlib"
        ));
    }
    #[test]
    fn native_position_hook_resync() {
        compare_raw_resync(include_bytes!(
            "fixtures/position-hook-resync-v41.json.zlib"
        ));
    }
    #[test]
    fn native_component_hook_resync() {
        compare_raw_resync(include_bytes!(
            "fixtures/component-hook-resync-v41.json.zlib"
        ));
    }
    #[test]
    fn native_ability_hook_resync() {
        compare_raw_resync(include_bytes!("fixtures/ability-hook-resync-v41.json.zlib"));
    }
    #[test]
    fn native_unit_reference_hook_resync() {
        compare_raw_resync(include_bytes!(
            "fixtures/unit-reference-hook-resync-v41.json.zlib"
        ));
    }
    #[test]
    fn native_object_hook_resync() {
        compare_raw_resync(include_bytes!("fixtures/object-hook-resync-v41.json.zlib"));
    }
    #[test]
    fn native_default_hook_frames() {
        check_default_hook_frames(include_bytes!(
            "fixtures/default-hook-frames-d61443e-v41.json.zlib"
        ));
    }
    #[test]
    fn native_new_record_profile_recovery() {
        check_default_hook_frames(include_bytes!(
            "fixtures/new-record-profile-recovery-v41.json.zlib"
        ));
    }
    fn check_default_hook_frames(fixture: &[u8]) {
        #[derive(Deserialize)]
        struct Run {
            mode: u8,
            observations: Vec<FilmComponentObservation>,
            records: serde_json::Value,
        }
        #[derive(Deserialize)]
        struct Case {
            #[serde(default)]
            new_record: NewRecordEncoding,
            hex: String,
            encoding: PositionEncoding,
            mpp: [usize; 2],
            extra: bool,
            runs: Vec<Run>,
        }
        let mut json = String::new();
        flate2::read::ZlibDecoder::new(fixture)
            .read_to_string(&mut json)
            .unwrap();
        let cases: Vec<Case> = serde_json::from_str(&json).unwrap();
        assert_eq!(cases.len(), 512);
        let mut registry = FilmRegistry {
            archetypes: (0..44)
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
        registry.archetypes[0].components = vec!["biped-emp-timer-component".into()];
        registry.archetypes[0].levels = vec![0];
        for (i, c) in cases.into_iter().enumerate() {
            let data: Vec<u8> = c
                .hex
                .as_bytes()
                .as_chunks::<2>()
                .0
                .iter()
                .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap())
                .collect();
            let profile = KillWalkProfile {
                encoding: FrameEncoding {
                    keyframe_layout: Default::default(),
                    keyframe_simulation_complete: None,
                    native_id_low_bits: None,
                    component_widths: Default::default(),
                    new_record: c.new_record,
                    position_capture: None,
                    ids: RecordIdLayout {
                        low_bits: 11,
                        base: 0,
                    },
                    mpp_widths: c.mpp,
                    position: Some(c.encoding),
                    extra_fields: c.extra,
                    corruption_check: false,
                },
                simulation_complete: false,
            };
            for run in c.runs {
                let mut world = FilmWorld::default();
                world.bind_full(50, 0);
                let before = world.clone();
                let targets = std::collections::BTreeSet::from([50]);
                let (records, observations) = if run.mode < 2 {
                    let observed = scan_frame_targets_observed(
                        &data,
                        &registry,
                        &profile,
                        &world,
                        &targets,
                        if run.mode == 0 {
                            HarvestConfirmation::NextBound
                        } else {
                            HarvestConfirmation::ChainWalk
                        },
                    );
                    assert_eq!(world, before);
                    (
                        observed.result.unwrap(),
                        observed.diagnostics.component_observations,
                    )
                } else {
                    let frame = decode_frame_resync(
                        &data,
                        &registry,
                        &profile,
                        &mut world,
                        &targets,
                        |_| true,
                    )
                    .unwrap();
                    assert!(
                        frame
                            .observations
                            .detached_scans
                            .component_observations
                            .is_empty()
                    );
                    (
                        frame.records,
                        frame.observations.caller_visible.component_observations,
                    )
                };
                assert_eq!(
                    observations, run.observations,
                    "default frame observations {i}/{}",
                    run.mode
                );
                let actual: Vec<_> = records.iter().map(|r| serde_json::json!({"kind":match r.header.kind {RecordKind::End=>0,RecordKind::New=>1,RecordKind::Delete=>2,RecordKind::Delta=>3},"id":r.header.id.unwrap(),"ti":r.archetype.unwrap_or(0),"mask":r.mask.unwrap_or(0),"components":r.attempts.len()})).collect();
                assert_eq!(
                    serde_json::to_value(actual).unwrap(),
                    run.records,
                    "default frame records {i}/{}",
                    run.mode
                );
                let restored: Vec<FilmComponentObservation> =
                    serde_json::from_slice(&serde_json::to_vec(&observations).unwrap()).unwrap();
                assert_eq!(restored, observations);
            }
        }
    }
    #[test]
    fn native_managed_hook_resync() {
        compare_raw_resync(include_bytes!("fixtures/managed-hook-resync-v41.json.zlib"));
    }
    #[test]
    fn native_engine_hook_resync() {
        compare_raw_resync(include_bytes!("fixtures/engine-hook-resync-v41.json.zlib"));
    }
    #[test]
    fn native_player_hook_resync() {
        compare_raw_resync(include_bytes!("fixtures/player-hook-resync-v41.json.zlib"));
    }
    #[test]
    fn native_equipment_hook_resync() {
        compare_raw_resync(include_bytes!(
            "fixtures/equipment-hook-resync-v41.json.zlib"
        ));
    }
    #[test]
    fn native_probe_hook_resync() {
        compare_raw_resync(include_bytes!("fixtures/probe-hook-resync-v41.json.zlib"));
    }
    fn compare_raw_resync(fixture: &[u8]) {
        #[derive(Deserialize)]
        struct Case {
            capture: Option<PositionCaptureEncoding>,
            name: Option<String>,
            scan_observations: Option<Vec<crate::theater::FilmComponentObservation>>,
            frame_observations: Option<Vec<crate::theater::FilmComponentObservation>>,
            #[serde(default)]
            initialized: bool,
            scan_visible: Option<BTreeMap<i32, u64>>,
            scan_all: Option<BTreeMap<i32, u64>>,
            frame_visible: Option<BTreeMap<i32, u64>>,
            frame_all: Option<BTreeMap<i32, u64>>,
            scan_offsets: Option<Vec<usize>>,
            frame_offsets: Option<Vec<usize>>,
            scan_actions: Option<Vec<[bool; 2]>>,
            frame_actions: Option<Vec<[bool; 2]>>,
            hex: String,
            position: PositionEncoding,
            min: [f32; 3],
            max: [f32; 3],
            region: u32,
            quantum: f32,
            extra: bool,
            records: serde_json::Value,
            slots: serde_json::Value,
            from: usize,
            landing: i64,
            scan_calls: serde_json::Value,
            calls: serde_json::Value,
        }
        let mut json = String::new();
        flate2::read::ZlibDecoder::new(fixture)
            .read_to_string(&mut json)
            .unwrap();
        let cases: Vec<Case> = serde_json::from_str(&json).unwrap();
        assert_eq!(cases.len(), 512);
        for (i, c) in cases.into_iter().enumerate() {
            let data: Vec<u8> = (0..c.hex.len())
                .step_by(2)
                .map(|n| u8::from_str_radix(&c.hex[n..n + 2], 16).unwrap())
                .collect();
            let registry = FilmRegistry {
                archetypes: vec![FilmArchetype {
                    index: 0,
                    components: vec![
                        c.name.unwrap_or_else(|| {
                            if c.scan_actions.is_some() {
                                "biped-mobility-action-component"
                            } else {
                                "object-position-dynamic-precision-component"
                            }
                            .into()
                        }),
                        "biped-emp-timer-component".into(),
                        "not-ported".into(),
                    ],
                    levels: vec![0; 3],
                }],
                major_version: 41,
                format_version: 27,
                end_byte: 0,
                truncated: false,
            };
            let map = FilmMapBounds {
                module: String::new(),
                min: c.min,
                max: c.max,
                region: c.region,
                axis_widths: c.position.world_axis_bits.unwrap(),
                region_index_bits: c.position.index_bits,
            };
            let profile = KillWalkProfile {
                encoding: FrameEncoding {
                    keyframe_layout: Default::default(),
                    keyframe_simulation_complete: None,
                    native_id_low_bits: None,
                    component_widths: Default::default(),
                    new_record: Default::default(),
                    position_capture: c.capture,
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
            let before = world.clone();
            let mut calls = vec![];
            let mut accept = |r: &EntityRecord| {
                let pos = resync_position(r, &map, c.quantum);
                let has = pos.is_some();
                calls.push(serde_json::json!({"slot":r.header.id.unwrap()&0x3fff_ffff,"has":has,"pos":pos.map(|p|p.vector.map(f32::to_bits))}));
                match i % 4 {
                    0 => true,
                    1 => has,
                    2 => !has,
                    _ => pos.is_some_and(|p| p.vector[0] > 0.),
                }
            };
            let scan = scan_for_target_delta_observed(
                &data,
                c.from,
                &registry,
                &profile,
                &world,
                &[50, 51].into(),
                &mut accept,
            );
            if let Some(expected) = c.scan_all {
                assert_eq!(scan.diagnostics.absolute_indices, expected, "all scan {i}");
            }
            let mut scan_observation = RawResyncDiagnostics {
                absolute_indices_initialized: c.initialized,
                ..Default::default()
            };
            scan_observation.absorb_scan(&scan.diagnostics);
            if let Some(expected) = &c.scan_observations {
                assert_eq!(
                    &scan.diagnostics.component_observations, expected,
                    "scan observations {i}"
                );
                assert_eq!(
                    &scan_observation.caller_visible.component_observations, expected,
                    "visible scan observations {i}"
                );
                assert!(
                    scan_observation
                        .detached_scans
                        .component_observations
                        .is_empty()
                );
            }
            if let Some(expected) = &c.scan_actions {
                assert_eq!(
                    &scan.diagnostics.mobility_actions, expected,
                    "scan actions {i}"
                );
                assert_eq!(
                    &scan_observation.caller_visible.mobility_actions, expected,
                    "visible scan actions {i}"
                );
                assert!(scan_observation.detached_scans.mobility_actions.is_empty());
            }
            if let Some(expected) = &c.scan_offsets {
                for diagnostics in [&scan.diagnostics, &scan_observation.caller_visible] {
                    assert_eq!(
                        diagnostics.mobility_offsets.as_deref().unwrap_or_default(),
                        expected,
                        "scan order {i}"
                    );
                    assert!(
                        diagnostics.ordered_publications().is_some(),
                        "scan order unavailable {i}"
                    );
                }
                assert!(scan_observation.detached_scans.mobility_offsets.is_none());
            }
            if let Some(expected) = c.scan_visible {
                assert_eq!(
                    scan_observation.caller_visible.absolute_indices, expected,
                    "visible scan {i}"
                );
            }
            let landing = scan.result.map_or(-1, |(b, _)| b);
            assert_eq!(landing, c.landing, "landing {i}");
            assert_eq!(
                serde_json::to_value(&calls).unwrap(),
                c.scan_calls,
                "scan calls {i}"
            );
            assert_eq!(world, before);
            calls.clear();
            let frame=decode_frame_resync_observed(&data,&registry,&profile,&mut world,&[50,51].into(),RawResyncDiagnostics { absolute_indices_initialized:c.initialized, ..Default::default() },|r|{
                let pos=resync_position(r,&map,c.quantum);let has=pos.is_some();
                calls.push(serde_json::json!({"slot":r.header.id.unwrap()&0x3fff_ffff,"has":has,"pos":pos.map(|p|p.vector.map(f32::to_bits))}));
                match i%4 {0=>true,1=>has,2=>!has,_=>pos.is_some_and(|p|p.vector[0]>0.)}
            }).unwrap();
            if let Some(expected) = c.frame_visible {
                assert_eq!(
                    frame.observations.caller_visible.absolute_indices, expected,
                    "visible frame {i}"
                );
            }
            if let Some(expected) = &c.frame_actions {
                assert_eq!(
                    &frame.observations.caller_visible.mobility_actions, expected,
                    "frame actions {i}"
                );
                assert!(
                    frame
                        .observations
                        .detached_scans
                        .mobility_actions
                        .is_empty()
                );
            }
            if let Some(expected) = &c.frame_observations {
                assert_eq!(
                    &frame.observations.caller_visible.component_observations, expected,
                    "frame observations {i}"
                );
                assert!(
                    frame
                        .observations
                        .detached_scans
                        .component_observations
                        .is_empty()
                );
            }
            if let Some(expected) = &c.frame_offsets {
                let diagnostics = &frame.observations.caller_visible;
                assert_eq!(
                    diagnostics.mobility_offsets.as_deref().unwrap_or_default(),
                    expected,
                    "frame order {i}"
                );
                assert!(
                    diagnostics.ordered_publications().is_some(),
                    "frame order unavailable {i}"
                );
                assert!(frame.observations.detached_scans.mobility_offsets.is_none());
            }
            if let Some(expected) = c.frame_all {
                let mut all = frame.observations.caller_visible.clone();
                all.merge(&frame.observations.detached_scans);
                assert_eq!(all.absolute_indices, expected, "all frame {i}");
            }
            assert_eq!(serde_json::to_value(&calls).unwrap(), c.calls, "calls {i}");
            let records:Vec<_>=frame.records.iter().map(|r|serde_json::json!({"kind":match r.header.kind{RecordKind::End=>0,RecordKind::New=>1,RecordKind::Delete=>2,RecordKind::Delta=>3},"id":r.header.id.unwrap(),"ti":r.archetype.unwrap_or(0),"mask":r.mask.unwrap_or(0),"components":r.attempts.len()})).collect();
            assert_eq!(
                serde_json::to_value(records).unwrap(),
                c.records,
                "records {i}"
            );
            let slots:BTreeMap<_,_>=world.slots.iter().map(|(slot,state)|(*slot,serde_json::json!({"ti":state.archetype,"id":state.full_id,"soft":state.soft,"view":state.view.unwrap_or(-1)}))).collect();
            assert_eq!(serde_json::to_value(slots).unwrap(), c.slots, "world {i}");
            let restored: ResyncFrame =
                serde_json::from_slice(&serde_json::to_vec(&frame).unwrap()).unwrap();
            assert_eq!(restored, frame);
        }
    }
    #[test]
    fn native_frame_target_harvest() {
        compare_harvest(include_bytes!("fixtures/frame-targets-v41.json.zlib"));
    }
    #[test]
    fn native_harvest_diagnostics() {
        compare_harvest(include_bytes!("fixtures/harvest-diagnostics-v41.json.zlib"));
    }
    #[test]
    fn native_mobility_harvest() {
        compare_harvest(include_bytes!("fixtures/mobility-harvest-v41.json.zlib"));
    }
    #[test]
    fn native_movement_hook_harvest() {
        compare_harvest(include_bytes!(
            "fixtures/movement-hook-harvest-v41.json.zlib"
        ));
    }
    #[test]
    fn native_position_hook_harvest() {
        compare_harvest(include_bytes!(
            "fixtures/position-hook-harvest-v41.json.zlib"
        ));
    }
    #[test]
    fn native_component_hook_harvest() {
        compare_harvest(include_bytes!(
            "fixtures/component-hook-harvest-v41.json.zlib"
        ));
    }
    #[test]
    fn native_ability_hook_harvest() {
        compare_harvest(include_bytes!(
            "fixtures/ability-hook-harvest-v41.json.zlib"
        ));
    }
    #[test]
    fn native_unit_reference_hook_harvest() {
        compare_harvest(include_bytes!(
            "fixtures/unit-reference-hook-harvest-v41.json.zlib"
        ));
    }
    #[test]
    fn native_object_hook_harvest() {
        compare_harvest(include_bytes!("fixtures/object-hook-harvest-v41.json.zlib"));
    }
    #[test]
    fn native_managed_hook_harvest() {
        compare_harvest(include_bytes!(
            "fixtures/managed-hook-harvest-v41.json.zlib"
        ));
    }
    #[test]
    fn native_engine_hook_harvest() {
        compare_harvest(include_bytes!("fixtures/engine-hook-harvest-v41.json.zlib"));
    }
    #[test]
    fn native_player_hook_harvest() {
        compare_harvest(include_bytes!("fixtures/player-hook-harvest-v41.json.zlib"));
    }
    #[test]
    fn native_equipment_hook_harvest() {
        compare_harvest(include_bytes!(
            "fixtures/equipment-hook-harvest-v41.json.zlib"
        ));
    }
    #[test]
    fn native_probe_hook_harvest() {
        compare_harvest(include_bytes!("fixtures/probe-hook-harvest-v41.json.zlib"));
    }
    fn compare_harvest(fixture: &[u8]) {
        #[derive(Deserialize)]
        struct Case {
            capture: Option<PositionCaptureEncoding>,
            name: Option<String>,
            position: Option<PositionEncoding>,
            diagnostics: Option<FilmReadDiagnostics>,
            hex: String,
            extra: bool,
            chain: bool,
            records: serde_json::Value,
        }
        let mut json = String::new();
        flate2::read::ZlibDecoder::new(fixture)
            .read_to_string(&mut json)
            .unwrap();
        let cases: Vec<Case> = serde_json::from_str(&json).unwrap();
        assert_eq!(cases.len(), 512);
        for (i, c) in cases.into_iter().enumerate() {
            let data: Vec<u8> = (0..c.hex.len())
                .step_by(2)
                .map(|n| u8::from_str_radix(&c.hex[n..n + 2], 16).unwrap())
                .collect();
            let registry = FilmRegistry {
                archetypes: vec![FilmArchetype {
                    index: 0,
                    levels: vec![0],
                    components: vec![c.name.unwrap_or_else(|| "biped-emp-timer-component".into())],
                }],
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
                    position_capture: c.capture,
                    ids: RecordIdLayout {
                        low_bits: 11,
                        base: 0,
                    },
                    mpp_widths: [9, 5],
                    position: c.position,
                    extra_fields: c.extra,
                    corruption_check: false,
                },
                simulation_complete: false,
            };
            let mut world = FilmWorld::default();
            world.bind_full(50, 0);
            world.bind_soft(51, 0);
            let before = world.clone();
            let observed = scan_frame_targets_observed(
                &data,
                &registry,
                &profile,
                &world,
                &[50, 51].into(),
                if c.chain {
                    HarvestConfirmation::ChainWalk
                } else {
                    HarvestConfirmation::NextBound
                },
            );
            if let Some(expected) = c.diagnostics {
                assert_eq!(observed.diagnostics, expected, "diagnostics {i}");
            }
            let restored: FilmReadAttempt<Vec<EntityRecord>> =
                serde_json::from_slice(&serde_json::to_vec(&observed).unwrap()).unwrap();
            assert_eq!(restored, observed);
            let records = observed.result.unwrap();
            let result:Vec<_>=records.iter().map(|r|serde_json::json!({"id":r.header.id.unwrap(),"ti":r.archetype.unwrap(),"mask":r.mask.unwrap(),"components":r.components.len(),"end":r.end_bit})).collect();
            assert_eq!(
                serde_json::to_value(result).unwrap(),
                c.records,
                "records {i}"
            );
            assert_eq!(world, before);
            for record in records {
                for field in record.fields {
                    assert_eq!(
                        serde_json::json!(
                            super::super::bits::Bits(&data).read(field.bit, field.width)
                        ),
                        serde_json::json!(Some(field.raw))
                    );
                }
            }
        }
    }
    #[test]
    fn native_inference_views() {
        #[derive(Deserialize)]
        struct Case {
            hex: String,
            names: Vec<Vec<String>>,
            chain: bool,
            tables: bool,
            extra: bool,
            position: PositionEncoding,
            records: serde_json::Value,
            classes: bool,
            skip: i64,
            views: i64,
            completed: usize,
            end: usize,
            slots: serde_json::Value,
        }
        let mut json = String::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/inference-views-v41.json.zlib")[..],
        )
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
            let frame = decode_inference_views(
                &data,
                &registry,
                &profile,
                &mut world,
                InferenceViewsOptions {
                    inference: InferenceFrameOptions {
                        chain_inference: c.chain,
                        view_tables: c.tables,
                    },
                    view_classes: c.classes,
                    view_count: c.views,
                    skip_lead_bits: c.skip,
                    packet_preamble_bits: 2,
                },
            )
            .unwrap();
            let records:Vec<_>=frame.entities.iter().flat_map(|v|v.records.iter()).map(|r|{
                let desync=if let Some(record)=&r.decoded{match record.stop{EntityViewStop::Complete=>-1,EntityViewStop::UnsupportedComponent{index,..}|EntityViewStop::InvalidComponent{index}=>index as i64,_=>0}}else if r.archetype.is_some(){-1}else{0};
                serde_json::json!({"kind":match r.header.kind{RecordKind::End=>0,RecordKind::New=>1,RecordKind::Delete=>2,RecordKind::Delta=>3},"id":r.header.id.unwrap(),"ti":r.archetype.unwrap_or(0),"desync":desync,"mask":r.decoded.as_ref().and_then(|r|r.mask).unwrap_or(0),"components":r.decoded.as_ref().map_or(0,|r|r.attempts.len())})
            }).collect();
            assert_eq!(
                serde_json::to_value(records).unwrap(),
                c.records,
                "records {i}"
            );
            assert_eq!(frame.views_completed, c.completed, "views {i}");
            assert_eq!(
                serde_json::json!(frame.end_bit),
                serde_json::json!(c.end),
                "end {i}"
            );

            let slots:BTreeMap<_,_>=world.slots.iter().map(|(slot,state)|(*slot,serde_json::json!({"ti":state.archetype,"id":state.full_id,"soft":state.soft,"view":state.view.unwrap_or(-1)}))).collect();
            assert_eq!(serde_json::to_value(slots).unwrap(), c.slots, "world {i}");
            let restored: InferenceViews =
                serde_json::from_slice(&serde_json::to_vec(&frame).unwrap()).unwrap();
            assert_eq!(restored, frame);
        }
    }
}
