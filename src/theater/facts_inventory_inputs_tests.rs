use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    num::NonZeroU64,
};

use super::*;
use std::io::Read;
#[derive(Deserialize)]
struct Case {
    logs: Vec<serde_json::Value>,
    raw: Vec<Input>,
    origin: u64,
    step: u64,
    lives: Vec<IdentityLife>,
    indices: BTreeMap<u64, i64>,
    offset: i64,
    deaths: Vec<IdentityDeath>,
    slots: BTreeSet<u32>,
    before: Vec<ReplayInventory>,
    reads: Vec<ReplayInventory>,
    coverage: ReplayInventoryCoverage,
    marked: usize,
}
#[test]
fn native_facts_inventory_publication_and_death_attribution() {
    let mut bytes = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/facts-inventory-publication-v41.json.zlib")[..],
    )
    .read_to_end(&mut bytes)
    .unwrap();
    let cases: Vec<Case> = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(cases.len(), 1024);
    for (i, c) in cases.into_iter().enumerate() {
        let input: Vec<_> = c
            .raw
            .iter()
            .map(|r| FactsKeyframeInventory {
                timestamp_us: r.timestamp_us,
                slot: r.slot,
                grenades_read: r.grenades_read,
                grenades: r.grenades,
                selected_grenade_rank: r.selected_grenade_rank,
                drawn_slot: r.drawn_slot,
                ammo_candidates: r.ammo_candidates,
                ammo_read: r.ammo_read,
                ammo: r.ammo.clone(),
                ..Default::default()
            })
            .collect();
        let (mut reads, dropped) =
            build_facts_replay_inventory(&input, c.origin, NonZeroU64::new(c.step).unwrap());
        let built = reads.len();
        reads.retain(|r| c.slots.contains(&r.slot));
        assert_eq!(reads, c.before, "projection {i}");
        assert_eq!(
            ReplayInventoryCoverage {
                decoded: c.raw.len(),
                dropped_before_origin: dropped,
                unpublished: built - reads.len(),
                published: reads.len()
            },
            c.coverage,
            "coverage {i}"
        );
        let identity = ReplayIdentityState::from_lives(c.lives, &c.indices);
        let marked = mark_replay_inventory_dead(
            &mut reads,
            &c.deaths,
            &identity,
            c.offset,
            IdentityClock {
                origin_us: c.origin,
                step_us: c.step,
                frame_count: 30000,
            },
        );
        assert_eq!(marked, c.marked, "marked {i}");
        assert_eq!(
            crate::theater::log_test_support::capture_logs(|| log_replay_inventory_empty(
                &reads, marked
            )),
            c.logs,
            "logs {i}"
        );
        assert_eq!(reads, c.reads, "attribution {i}");
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Input {
    #[serde(rename = "TimestampUS")]
    timestamp_us: u64,
    slot: u32,
    grenades_read: bool,
    grenades: [u32; 4],
    selected_grenade_rank: i64,
    drawn_slot: i64,
    ammo_candidates: i64,
    ammo_read: bool,
    ammo: [KeyframeSlotAmmo; 4],
}
