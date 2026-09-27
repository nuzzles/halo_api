//! Placement and spawn section, followed by the two world-object scan channels.
use super::facts_events::facts_slice_count;
use super::*;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactsLifeKey {
    #[serde(rename = "Slot")]
    pub slot: u32,
    #[serde(rename = "Gen")]
    pub generation: u32,
}
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FactsPlacement {
    pub start_us: u64,
    pub end_us: u64,
    pub life: FactsLifeKey,
    pub position: [f32; 3],
    pub global_id: u32,
    pub points: i64,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactsSpawnEvent {
    #[serde(rename = "TimestampUS")]
    pub timestamp_us: u64,
    #[serde(rename = "Chunk")]
    pub chunk: i64,
    #[serde(rename = "PacketIndex")]
    pub packet_index: i64,
    #[serde(rename = "Spawned")]
    pub spawned: FactsLifeKey,
    #[serde(rename = "SpawnedValid")]
    pub spawned_valid: bool,
    #[serde(rename = "Source")]
    pub source: FactsLifeKey,
    #[serde(rename = "SourceValid")]
    pub source_valid: bool,
    #[serde(rename = "Ref2Present")]
    pub ref2_present: bool,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactsSpawnStats {
    #[serde(rename = "Chunks")]
    pub chunks: i64,
    #[serde(rename = "Packets")]
    pub packets: i64,
    #[serde(rename = "Lists")]
    pub lists: i64,
    #[serde(rename = "Events")]
    pub events: i64,
    #[serde(rename = "WithSpawned")]
    pub with_spawned: i64,
    #[serde(rename = "WithSource")]
    pub with_source: i64,
    #[serde(rename = "Ref2")]
    pub ref2: i64,
}
pub fn encode_facts_spawns(
    w: &mut NativeFactsWriter,
    events: &[FactsSpawnEvent],
    s: &FactsSpawnStats,
) {
    w.unsigned(events.len() as u64);
    let mut last = 0u64;
    for e in events {
        w.unsigned(e.timestamp_us.wrapping_sub(last));
        last = e.timestamp_us;
        w.signed(e.chunk);
        w.signed(e.packet_index);
        w.unsigned(u64::from(e.spawned.slot));
        w.unsigned(u64::from(e.spawned.generation));
        w.boolean(e.spawned_valid);
        w.unsigned(u64::from(e.source.slot));
        w.unsigned(u64::from(e.source.generation));
        w.boolean(e.source_valid);
        w.boolean(e.ref2_present);
    }
    for n in [
        s.chunks,
        s.packets,
        s.lists,
        s.events,
        s.with_spawned,
        s.with_source,
        s.ref2,
    ] {
        w.unsigned(n as u64);
    }
}
pub fn decode_facts_spawns(
    r: &mut NativeFactsReader<'_>,
) -> (Vec<FactsSpawnEvent>, FactsSpawnStats) {
    let mut events = Vec::new();
    let mut last = 0u64;
    for _ in 0..facts_slice_count(r) {
        if r.error().is_some() {
            break;
        }
        last = last.wrapping_add(r.unsigned());
        events.push(FactsSpawnEvent {
            timestamp_us: last,
            chunk: r.signed(),
            packet_index: r.signed(),
            spawned: FactsLifeKey {
                slot: r.unsigned() as u32,
                generation: r.unsigned() as u32,
            },
            spawned_valid: r.boolean(),
            source: FactsLifeKey {
                slot: r.unsigned() as u32,
                generation: r.unsigned() as u32,
            },
            source_valid: r.boolean(),
            ref2_present: r.boolean(),
        });
    }
    (
        events,
        FactsSpawnStats {
            chunks: r.unsigned() as i64,
            packets: r.unsigned() as i64,
            lists: r.unsigned() as i64,
            events: r.unsigned() as i64,
            with_spawned: r.unsigned() as i64,
            with_source: r.unsigned() as i64,
            ref2: r.unsigned() as i64,
        },
    )
}
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FactsWorldSection {
    pub placements: Vec<FactsPlacement>,
    pub placement_stats: FactsPlacementStats,
    pub spawn_events: Vec<FactsSpawnEvent>,
    pub spawn_stats: FactsSpawnStats,
    pub weapons: FactsWorldObjectScan,
    pub powerups: FactsWorldObjectScan,
}
pub fn encode_facts_world_section(w: &mut NativeFactsWriter, g: &FactsWorldSection) {
    w.unsigned(g.placements.len() as u64);
    let mut last = 0u64;
    for p in &g.placements {
        w.unsigned(p.start_us.wrapping_sub(last));
        last = p.start_us;
        w.unsigned(p.end_us);
        w.unsigned(u64::from(p.life.slot));
        w.unsigned(u64::from(p.life.generation));
        for f in p.position {
            w.float32(f);
        }
        w.unsigned(u64::from(p.global_id));
        w.unsigned(p.points as u64);
    }
    encode_facts_placement_stats(w, &g.placement_stats);
    encode_facts_spawns(w, &g.spawn_events, &g.spawn_stats);
    encode_facts_world_scan(w, &g.weapons);
    encode_facts_world_scan(w, &g.powerups);
}
pub fn decode_facts_world_section(r: &mut NativeFactsReader<'_>) -> FactsWorldSection {
    let mut placements = Vec::new();
    let mut last = 0u64;
    for _ in 0..facts_slice_count(r) {
        if r.error().is_some() {
            break;
        }
        last = last.wrapping_add(r.unsigned());
        placements.push(FactsPlacement {
            start_us: last,
            end_us: r.unsigned(),
            life: FactsLifeKey {
                slot: r.unsigned() as u32,
                generation: r.unsigned() as u32,
            },
            position: std::array::from_fn(|_| r.float32()),
            global_id: r.unsigned() as u32,
            points: r.unsigned() as i64,
        });
    }
    let placement_stats = decode_facts_placement_stats(r);
    let (spawn_events, spawn_stats) = decode_facts_spawns(r);
    FactsWorldSection {
        placements,
        placement_stats,
        spawn_events,
        spawn_stats,
        weapons: decode_facts_world_scan(r),
        powerups: decode_facts_world_scan(r),
    }
}
