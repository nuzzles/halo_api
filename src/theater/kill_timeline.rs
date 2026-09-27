//! Keyframe binding and biped allocation-gap windows for kill-source walks.
use super::{
    DecodeError, FilmRegistry, FilmWorld, FilmWorldSnapshot, bits::Bits, fire_events, native_sort,
    parse_registry_chunk, recover_keyframe_anchors,
};
use crate::clients::hi::models::FilmChunkData;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KillKeyframeRecord {
    pub slot: u32,
    pub generation: u32,
    pub archetype: u32,
    pub bit: usize,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KillKeyframeEvent {
    pub timestamp_us: u64,
    pub records: Vec<KillKeyframeRecord>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KillGapWindow {
    pub slot: u32,
    pub from: u64,
    pub to: u64,
}
/// Merge guarded native keyframe recovery with a biped-only sweep. The sweep
/// keeps the first additional occurrence per slot, then sorts by bit position.
pub fn recover_kill_keyframe_records(data: &[u8]) -> Vec<KillKeyframeRecord> {
    let mut records: Vec<_> = recover_keyframe_anchors(data)
        .into_iter()
        .map(|a| KillKeyframeRecord {
            slot: a.id & 0x3fff_ffff,
            generation: a.id >> 30,
            archetype: a.archetype,
            bit: a.bit,
        })
        .collect();
    let mut seen: BTreeSet<_> = records.iter().map(|r| r.slot).collect();
    let bits = Bits(data);
    if bits.len() >= 64 {
        for bit in 0..=bits.len() - 64 {
            let id = bits.read(bit, 32).unwrap() as u32;
            if id == u32::MAX
                || id >> 30 == 0
                || id & 0x3fff_ffff >= 8192
                || bits.read(bit + 32, 32).unwrap() >= 50
            {
                continue;
            }
            let ti = bits.read(bit + 58, 6).unwrap() as u32;
            if ti == 35 && seen.insert(id & 0x3fff_ffff) {
                records.push(KillKeyframeRecord {
                    slot: id & 0x3fff_ffff,
                    generation: id >> 30,
                    archetype: ti,
                    bit,
                });
            }
        }
    }
    native_sort::sort_by(&mut records, |a, b| a.bit.cmp(&b.bit));
    records
}
/// Separate from the replay march timeline: native kill-source additionally
/// binds undeclared slots as generation-wildcard bipeds during allocation gaps.
#[derive(Debug, Clone)]
pub struct KillTimeline {
    pub registry: FilmRegistry,
    /// Native trailing-byte count, when constructed from source chunks.
    /// A caller-supplied registry alone does not establish this count.
    pub registry_truncated_bytes: Option<usize>,
    pub events: Vec<KillKeyframeEvent>,
    pub gaps: Vec<KillGapWindow>,
    pub world: FilmWorld,
    initial: FilmWorldSnapshot,
    cursor: usize,
    initial_alias: bool,
    open: BTreeMap<usize, Vec<u32>>,
    close: BTreeMap<usize, Vec<u32>>,
}
impl KillTimeline {
    pub fn new(registry: FilmRegistry, mut events: Vec<KillKeyframeEvent>) -> Self {
        native_sort::sort_by(&mut events, |a, b| a.timestamp_us.cmp(&b.timestamp_us));
        let mut world = FilmWorld::default();
        let mut seen = BTreeSet::new();
        for e in &events {
            for r in &e.records {
                if seen.insert(r.slot) {
                    world.bind_full((r.generation << 30) | r.slot, r.archetype);
                }
            }
        }
        let initial = world.snapshot();
        let mut out = Self {
            registry,
            registry_truncated_bytes: None,
            events,
            gaps: Vec::new(),
            world,
            initial,
            cursor: 0,
            initial_alias: false,
            open: BTreeMap::new(),
            close: BTreeMap::new(),
        };
        let (lo, hi) = out.biped_range();
        let mut declared = BTreeSet::new();
        let mut max_at = vec![0; out.events.len()];
        for (i, e) in out.events.iter().enumerate() {
            for r in &e.records {
                if r.archetype == 35 {
                    declared.insert(r.slot);
                    max_at[i] = max_at[i].max(r.slot);
                }
            }
        }
        if hi >= lo {
            for slot in lo as u32..=hi as u32 {
                if declared.contains(&slot) {
                    continue;
                }
                let last = max_at.iter().rposition(|&m| m > 0 && m < slot);
                let from = last.map_or(0, |i| out.events[i].timestamp_us);
                let next = last.map_or(0, |i| i + 1);
                let to = out.events.get(next).map_or(u64::MAX, |e| e.timestamp_us);
                out.gaps.push(KillGapWindow { slot, from, to });
                for (i, e) in out.events.iter().enumerate() {
                    if e.timestamp_us == from {
                        out.open.entry(i).or_default().push(slot);
                    }
                    if e.timestamp_us == to {
                        out.close.entry(i).or_default().push(slot);
                    }
                }
            }
        }
        out
    }
    pub fn from_chunks(chunks: &[FilmChunkData]) -> Result<Self, DecodeError> {
        let first = chunks.first().ok_or(super::KillSourceFilmError::NoChunk)?;
        let read =
            parse_registry_chunk(&first.data).map_err(super::KillSourceFilmError::Registry)?;
        let mut events = Vec::new();
        for c in chunks {
            for p in fire_events::native_chunk_packets(c) {
                if p.packet_type == 2 {
                    events.push(KillKeyframeEvent {
                        timestamp_us: p.timestamp_us,
                        records: recover_kill_keyframe_records(
                            &c.data[p.payload_offset..p.payload_offset + p.payload_size],
                        ),
                    });
                }
            }
        }
        let mut timeline = Self::new(read.registry, events);
        timeline.registry_truncated_bytes = Some(read.truncated_bytes);
        Ok(timeline)
    }
    pub fn biped_range(&self) -> (i64, i64) {
        let (mut lo, mut hi) = (1i64 << 30, -1);
        for e in &self.events {
            for r in &e.records {
                if r.archetype == 35 {
                    lo = lo.min(i64::from(r.slot));
                    hi = hi.max(i64::from(r.slot));
                }
            }
        }
        (lo, hi)
    }
    pub fn rewind(&mut self) {
        self.world.restore(self.initial.clone());
        self.cursor = 0;
        self.initial_alias = true;
    }
    /// Native packet rollback restores a freshly copied map, detaching the
    /// initial snapshot alias without changing the observable slot values.
    pub(super) fn detach_snapshot_alias(&mut self) {
        self.initial_alias = false;
    }
    pub fn advance_to(&mut self, timestamp_us: u64) -> &FilmWorld {
        while self.cursor < self.events.len()
            && self.events[self.cursor].timestamp_us <= timestamp_us
        {
            for r in &self.events[self.cursor].records {
                self.world
                    .bind_full((r.generation << 30) | r.slot, r.archetype);
            }
            if let Some(slots) = self.close.get(&self.cursor) {
                for &slot in slots {
                    self.world.unbind(slot);
                }
            }
            if let Some(slots) = self.open.get(&self.cursor) {
                for &slot in slots {
                    self.world.bind_wildcard(slot, 35);
                }
            }
            self.cursor += 1;
            if self.initial_alias {
                self.initial = self.world.snapshot();
            }
        }
        &self.world
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[derive(Deserialize, Debug, PartialEq)]
    struct Slot {
        slot: u32,
        archetype: u32,
        hard: bool,
        generation_mask: u32,
    }
    #[derive(Deserialize)]
    struct State {
        time: u64,
        slots: Vec<Slot>,
    }
    #[derive(Deserialize)]
    struct Row {
        input: Vec<KillKeyframeEvent>,
        low: i64,
        high: i64,
        gaps: Vec<KillGapWindow>,
        states: Vec<State>,
        data: String,
        recovered: Vec<KillKeyframeRecord>,
    }
    #[test]
    fn native_kill_registry_admission() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            include_bytes!("fixtures/kill-registry-v41.json.zlib").as_slice(),
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(rows.len(), 20);
        for (i, row) in rows.iter().enumerate() {
            let hex = row["hex"].as_str().unwrap();
            let chunk = crate::clients::hi::models::FilmChunkData {
                metadata: crate::clients::hi::models::FilmChunk {
                    index: 19,
                    chunk_type: 0,
                    start_time_offset_ms: 0,
                    duration_ms: 0,
                    size: 0,
                    file_relative_path: String::new(),
                },
                data: (0..hex.len())
                    .step_by(2)
                    .map(|j| u8::from_str_radix(&hex[j..j + 2], 16).unwrap())
                    .collect(),
            };
            let result = KillTimeline::from_chunks(&[chunk]);
            let error = row["error"].as_str().unwrap();
            if !error.is_empty() {
                let e = result.unwrap_err();
                assert_eq!(e.to_string(), error, "error {i}");
                assert!(matches!(
                    e,
                    crate::theater::DecodeError::KillSource(
                        crate::theater::KillSourceFilmError::Registry(
                            crate::theater::FilmRegistryReadError::StillCompressed
                        )
                    )
                ));
                continue;
            }
            let timeline = result.unwrap();
            assert_eq!(
                timeline.registry.truncated,
                row["truncated"].as_bool().unwrap(),
                "truncated {i}"
            );
            assert_eq!(
                timeline.registry_truncated_bytes,
                Some(row["truncated_bytes"].as_u64().unwrap() as usize),
                "tail {i}"
            );
            assert_eq!(
                timeline.events.len(),
                row["events"].as_u64().unwrap() as usize
            );
            let mut expected = row["archetypes"].clone();
            for a in expected.as_array_mut().unwrap() {
                for key in ["components", "levels"] {
                    if a[key].is_null() {
                        a[key] = serde_json::json!([]);
                    }
                }
            }
            assert_eq!(
                serde_json::to_value(&timeline.registry.archetypes).unwrap(),
                expected,
                "registry {i}"
            );
        }
        assert!(matches!(
            KillTimeline::from_chunks(&[]).unwrap_err(),
            crate::theater::DecodeError::KillSource(crate::theater::KillSourceFilmError::NoChunk)
        ));
    }
    #[test]
    fn native_kill_timeline_oracle() {
        let mut json = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/kill-timeline-v41.json.zlib")[..])
            .read_to_end(&mut json)
            .unwrap();
        let rows: Vec<Row> = serde_json::from_slice(&json).unwrap();
        assert_eq!(rows.len(), 256);
        let registry = FilmRegistry {
            major_version: 41,
            format_version: 0,
            archetypes: Vec::new(),
            end_byte: 0,
            truncated: false,
        };
        for (i, row) in rows.into_iter().enumerate() {
            let data: Vec<u8> = row
                .data
                .as_bytes()
                .as_chunks::<2>()
                .0
                .iter()
                .map(|c| u8::from_str_radix(std::str::from_utf8(c).unwrap(), 16).unwrap())
                .collect();
            assert_eq!(
                recover_kill_keyframe_records(&data),
                row.recovered,
                "anchors {i}"
            );
            let mut timeline = KillTimeline::new(registry.clone(), row.input);
            assert_eq!(timeline.biped_range(), (row.low, row.high));
            assert_eq!(timeline.gaps, row.gaps, "gaps {i}");
            for (j, state) in row.states.into_iter().enumerate() {
                if state.time == 0 {
                    timeline.rewind();
                }
                let w = timeline.advance_to(state.time);
                let actual: Vec<_> = (0..40)
                    .filter_map(|slot| {
                        w.archetype(slot).map(|archetype| Slot {
                            slot,
                            archetype,
                            hard: w.hard_bound(slot),
                            generation_mask: (0..4)
                                .filter(|&g| w.generation_matches((g << 30) | slot, true))
                                .fold(0, |m, g| m | (1 << g)),
                        })
                    })
                    .collect();
                assert_eq!(actual, state.slots, "state {i}/{j}");
            }
        }
    }
}
