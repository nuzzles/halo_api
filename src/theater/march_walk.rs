//! Native eight-view death/occupancy walk and event-packet localization.
use super::bits::{Bits, Cursor};
use super::*;

fn trial_delta(
    data: &[u8],
    start: usize,
    registry: &FilmRegistry,
    encoding: &FrameEncoding,
    world: &FilmWorld,
    simulation_complete: bool,
) -> Option<EntityRecord> {
    if data.len().saturating_mul(8).saturating_sub(start) < 24 {
        return None;
    }
    let header = super::records::decode_frame_header_cursor(
        Cursor::new_padded(data, start + if encoding.extra_fields { 32 } else { 0 }),
        encoding,
    )?;
    if header.kind != RecordKind::Delta {
        return None;
    }
    let id = header.id?;
    // Both locator passes only admit slot 123. Reject other headers before
    // component decoding: speculative TLV bodies can be large even when this
    // candidate cannot possibly be selected.
    if id & 0x3fff_ffff != 123 {
        return None;
    }
    let mut bindings = EntityBindings::default();
    if let Some(ti) = world.archetype(id & 0x3fff_ffff) {
        bindings.bind(id, ti);
    }
    let record = super::components::decode_entity_record_at_with_simulation_policy(
        data,
        start,
        registry,
        encoding,
        &bindings,
        simulation_complete,
    )?;
    (record.stop == EntityViewStop::Complete).then_some(record)
}

/// Locate the native slot-123 signature. Strict search selects its first match
/// before checking generation; a rejected first match falls back to free-width
/// search, rather than continuing the strict search. Trial reads never bind.
pub fn locate_march_event_records(
    data: &[u8],
    registry: &FilmRegistry,
    encoding: &FrameEncoding,
    world: &FilmWorld,
    generation_strict: bool,
) -> Option<usize> {
    locate_march_event_records_with_simulation_policy(
        data,
        registry,
        encoding,
        world,
        generation_strict,
        true,
    )
}
/// Native kill-source localization carries the same simulation gate as its walk.
pub fn locate_march_event_records_with_simulation_policy(
    data: &[u8],
    registry: &FilmRegistry,
    encoding: &FrameEncoding,
    world: &FilmWorld,
    generation_strict: bool,
    simulation_complete: bool,
) -> Option<usize> {
    if !encoding.valid() {
        return None;
    }
    let bits = Bits(data);
    let acceptable =
        |r: &EntityRecord| world.generation_matches(r.header.id.unwrap(), generation_strict);
    for s in 2..bits.len().saturating_sub(35) {
        if bits.read(s - 1, 1) != Some(0) {
            continue;
        }
        if let Some(r) = trial_delta(data, s, registry, encoding, world, simulation_complete)
            .filter(|r| {
                r.header.id.unwrap() & 0x3fff_ffff == 123
                    && r.end_bit == (s + 35) as i64
                    && r.components.len() == 1
            })
        {
            if acceptable(&r) {
                return Some(s);
            }
            break;
        }
    }
    for s in 2..bits.len().saturating_sub(16) {
        if bits.read(s - 1, 1) != Some(0) {
            continue;
        }
        if trial_delta(data, s, registry, encoding, world, simulation_complete)
            .is_some_and(|r| r.header.id.unwrap() & 0x3fff_ffff == 123 && acceptable(&r))
        {
            return Some(s);
        }
    }
    None
}

/// Native locator result and ordered callbacks from all trials, including rejected
/// offsets. Trial observations are evidence of reads, not accepted entity records.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MarchLocatorObservations {
    pub start: Option<usize>,
    pub diagnostics: FilmReadDiagnostics,
}

/// Preserve native strict/fallback locator publications. Unlike the offset-only
/// API, this traverses candidates for every bound slot and repeats the accepted
/// strict candidate's generation-check read, because those reads publish callbacks.
/// Movement snapshots are suppressed; positions and other observers stay live.
pub fn locate_march_event_records_observed(
    data: &[u8],
    registry: &FilmRegistry,
    encoding: &FrameEncoding,
    world: &FilmWorld,
    generation_strict: bool,
    simulation_complete: bool,
) -> MarchLocatorObservations {
    let mut out = MarchLocatorObservations {
        start: None,
        diagnostics: FilmReadDiagnostics::default(),
    };
    if !encoding.valid() {
        return out;
    }
    let bits = Bits(data);
    let mut trial = |start: usize| -> Option<EntityRecord> {
        if bits.len().saturating_sub(start) < 24 {
            return None;
        }
        let header = super::records::decode_frame_header_cursor(
            Cursor::new_padded(data, start + if encoding.extra_fields { 32 } else { 0 }),
            encoding,
        )?;
        if header.kind != RecordKind::Delta {
            return None;
        }
        let id = header.id?;
        let slot = id & 0x3fff_ffff;
        let mut bindings = EntityBindings::default();
        if let Some(ti) = world.archetype(slot) {
            bindings.bind(id, ti);
        }
        let record = super::components::decode_entity_record_with_capture_slots(
            data,
            start,
            registry,
            encoding,
            &bindings,
            simulation_complete,
            super::components::RecordCaptureSlots {
                context: None,
                movement: None,
                position: Some(slot),
            },
        )?;
        out.diagnostics.merge(&record.diagnostics);
        (record.stop == EntityViewStop::Complete).then_some(record)
    };
    for start in 2..bits.len().saturating_sub(35) {
        if bits.read(start - 1, 1) != Some(0) {
            continue;
        }
        if trial(start).is_some_and(|r| {
            r.header.id.unwrap() & 0x3fff_ffff == 123
                && r.end_bit == (start + 35) as i64
                && r.components.len() == 1
        }) {
            if trial(start)
                .is_some_and(|r| world.generation_matches(r.header.id.unwrap(), generation_strict))
            {
                out.start = Some(start);
                return out;
            }
            break;
        }
    }
    for start in 2..bits.len().saturating_sub(16) {
        if bits.read(start - 1, 1) != Some(0) {
            continue;
        }
        if trial(start).is_some_and(|r| {
            r.header.id.unwrap() & 0x3fff_ffff == 123
                && world.generation_matches(r.header.id.unwrap(), generation_strict)
        }) {
            out.start = Some(start);
            break;
        }
    }
    out
}

/// Walk up to eight entity views, preserving records before a failure. Bindings
/// created/deleted within this packet affect later records, but never escape it.
/// This is the native march policy, distinct from production message/entity/control
/// view admission. No view-ownership gate is applied here.
pub fn walk_march_records(
    data: &[u8],
    start: impl TryInto<i64>,
    registry: &FilmRegistry,
    encoding: &FrameEncoding,
    world: &FilmWorld,
    generation_strict: bool,
) -> Vec<EntityRecord> {
    walk_march_record_views(data, start, registry, encoding, world, generation_strict, 8)
}

/// Native march with an explicit view limit; the default wrapper keeps eight.
pub fn walk_march_record_views(
    data: &[u8],
    start: impl TryInto<i64>,
    registry: &FilmRegistry,
    encoding: &FrameEncoding,
    world: &FilmWorld,
    generation_strict: bool,
    views: usize,
) -> Vec<EntityRecord> {
    walk_march_with_policy(
        data,
        start,
        registry,
        encoding,
        world,
        MarchWalkPolicy {
            views,
            generation_strict,
            simulation_complete: true,
        },
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MarchWalkPolicy {
    pub views: usize,
    pub generation_strict: bool,
    pub simulation_complete: bool,
}

/// Explicit native profile gate for simulation-state publication. Its body is
/// read even when incomplete, then the record stops as in the reference.
pub fn walk_march_with_policy(
    data: &[u8],
    start: impl TryInto<i64>,
    registry: &FilmRegistry,
    encoding: &FrameEncoding,
    world: &FilmWorld,
    policy: MarchWalkPolicy,
) -> Vec<EntityRecord> {
    if !encoding.valid() {
        return Vec::new();
    }
    let Ok(start) = start.try_into() else {
        return Vec::new();
    };
    let mut working = world.clone();
    let mut at = start;
    let mut records = Vec::new();
    for _ in 0..policy.views {
        if (data.len() as i64 * 8).wrapping_sub(at) < 8 {
            break;
        }
        if at == 0 {
            at = 2;
        }
        loop {
            let Some(header) = super::records::decode_frame_header_signed(
                data,
                at.wrapping_add(if encoding.extra_fields { 32 } else { 0 }),
                encoding,
            ) else {
                return records;
            };
            if header.kind == RecordKind::End {
                at = header.end_bit;
                break;
            }
            let id = header.id.unwrap();
            let mut bindings = EntityBindings::default();
            let mismatch = header.kind == RecordKind::Delta
                && !working.generation_matches(id, policy.generation_strict);
            if !mismatch && let Some(ti) = working.archetype(id & 0x3fff_ffff) {
                bindings.bind(id, ti);
            }
            let Some(mut r) = super::components::decode_entity_record_with_movement(
                data,
                at,
                registry,
                encoding,
                &bindings,
                policy.simulation_complete,
                Some(id & 0x3fff_ffff),
            ) else {
                return records;
            };
            // Non-strict native deltas read the baseline selector before discovering
            // an absent slot. Strict generation rejection stops at the header instead.
            if !mismatch && matches!(r.stop, EntityViewStop::MissingBinding { .. }) {
                let mut tail = NativeFilmBits::new(data);
                tail.set_position(r.end_bit);
                let bit = tail.position();
                let present = tail.read(1);
                r.fields.push(ComponentField {
                    name: "baseline.gate".into(),
                    bit,
                    width: 1,
                    raw: present,
                });
                if present != 0 {
                    let bit = tail.position();
                    let raw = tail.read(7);
                    r.fields.push(ComponentField {
                        name: "baseline".into(),
                        bit,
                        width: 7,
                        raw,
                    });
                }
                r.end_bit = tail.position();
                r.padded_bits =
                    super::bits::padded_from_native(r.end_bit, data.len().saturating_mul(8));
            }
            if mismatch {
                r.stop = EntityViewStop::GenerationMismatch {
                    id,
                    bound_id: working
                        .slots
                        .get(&(id & 0x3fff_ffff))
                        .map_or(0, |s| s.full_id),
                };
            }
            at = r.end_bit;
            let complete = r.stop == EntityViewStop::Complete;
            if complete {
                match r.header.kind {
                    RecordKind::New => working.bind_full(id, r.archetype.unwrap()),
                    RecordKind::Delete => working.unbind(id & 0x3fff_ffff),
                    _ => {}
                }
            }
            records.push(r);
            if !complete {
                return records;
            }
        }
    }
    records
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Record {
        id: u32,
        ti: u32,
        mask: u64,
        desync: i64,
        header: usize,
        kind: u8,
        end: usize,
    }
    #[derive(Deserialize)]
    struct Case {
        payload: Vec<u8>,
        start: usize,
        low: usize,
        extra: bool,
        strict: bool,
        located: i64,
        records: Vec<Record>,
    }
    #[test]
    fn native_march_views_and_locator() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/march-walk-v41.json.zlib")[..])
            .read_to_end(&mut raw)
            .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&raw).unwrap();
        let registry = FilmRegistry {
            archetypes: (0..4)
                .map(|index| FilmArchetype {
                    index,
                    components: if index == 3 {
                        vec!["high-frequency".into(), "not-ported".into()]
                    } else {
                        vec![]
                    },
                    levels: vec![1, 0],
                })
                .collect(),
            major_version: 41,
            format_version: 0,
            end_byte: 0,
            truncated: false,
        };
        for (i, c) in cases.into_iter().enumerate() {
            let encoding = FrameEncoding {
                keyframe_layout: Default::default(),
                keyframe_simulation_complete: None,
                native_id_low_bits: None,
                component_widths: Default::default(),
                new_record: Default::default(),
                position_capture: None,
                ids: RecordIdLayout {
                    low_bits: c.low,
                    base: 0,
                },
                mpp_widths: [9, 5],
                position: None,
                extra_fields: c.extra,
                corruption_check: false,
            };
            let mut world = FilmWorld::default();
            world.bind_full(1 << 30 | 123, 3);
            world.bind_full(1 << 30 | 124, 3);
            let old = world.clone();
            assert_eq!(
                locate_march_event_records(&c.payload, &registry, &encoding, &world, c.strict)
                    .map_or(-1, |n| n as i64),
                c.located,
                "locator {i}"
            );
            let records =
                walk_march_records(&c.payload, c.start, &registry, &encoding, &world, c.strict);
            assert_eq!(world, old);
            assert_eq!(records.len(), c.records.len(), "count {i}");
            for (j, (r, e)) in records.iter().zip(c.records).enumerate() {
                assert_eq!(r.header.id, Some(e.id), "id {i}/{j}");
                assert_eq!(r.archetype.unwrap_or(0), e.ti, "ti {i}/{j}");
                assert_eq!(r.mask.unwrap_or(0), e.mask, "mask {i}/{j}");
                assert_eq!(
                    r.header.start_bit - i64::from(c.extra) * 32,
                    e.header as i64,
                    "header {i}/{j}"
                );
                assert_eq!(
                    match r.header.kind {
                        RecordKind::End => 0,
                        RecordKind::New => 1,
                        RecordKind::Delete => 2,
                        RecordKind::Delta => 3,
                    },
                    e.kind
                );
                if e.kind != 2 {
                    assert_eq!(
                        serde_json::json!(r.end_bit),
                        serde_json::json!(e.end),
                        "end {i}/{j}"
                    );
                }
                let desync = match r.stop {
                    EntityViewStop::Complete => -1,
                    EntityViewStop::UnsupportedComponent { index, .. }
                    | EntityViewStop::InvalidComponent { index } => index as i64,
                    _ => 0,
                };
                assert_eq!(desync, e.desync, "desync {i}/{j}");
            }
        }
    }
}
