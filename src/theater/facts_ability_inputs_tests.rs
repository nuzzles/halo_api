use super::*;
use serde::Deserialize;
use std::{collections::BTreeSet, io::Read, num::NonZeroU64};

#[derive(Deserialize)]
struct Inventory {
    #[serde(rename = "TimestampUS")]
    timestamp_us: u64,
    #[serde(rename = "Slot")]
    slot: u32,
    #[serde(rename = "AbilityRank")]
    rank: i64,
}
#[derive(Deserialize)]
struct Case {
    ranks: Vec<FactsAbilityRank>,
    inventory: Vec<Inventory>,
    origin: u64,
    step: u64,
    slots: BTreeSet<u32>,
    palettes: Vec<ReplayAbilityPalette>,
    raw: Vec<ReplayAbilityRead>,
    output: ReplayAbilityPublication,
}
#[test]
fn native_facts_ability_signed_ranks_and_publication() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/facts-abilities-v41.json.zlib")[..])
        .read_to_end(&mut raw)
        .unwrap();
    let rows: Vec<Case> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 1024);
    let mut negative = 0;
    let mut wide = 0;
    let mut classified = 0;
    for (i, row) in rows.into_iter().enumerate() {
        let inventory: Vec<_> = row
            .inventory
            .into_iter()
            .map(|r| FactsKeyframeInventory {
                timestamp_us: r.timestamp_us,
                slot: r.slot,
                ability_rank: r.rank,
                ..Default::default()
            })
            .collect();
        let reads = build_facts_replay_ability_reads(
            &row.ranks,
            &inventory,
            row.origin,
            NonZeroU64::new(row.step).unwrap(),
        );
        negative += reads.iter().filter(|r| r.r < i32::MIN as i64).count();
        wide += reads.iter().filter(|r| r.r > i32::MAX as i64).count();
        assert_eq!(reads, row.raw, "raw {i}");
        let published = publish_replay_abilities(reads, &row.slots, &row.palettes);
        classified += usize::from(published.palette.is_some());
        assert_eq!(published, row.output, "publication {i}");
    }
    assert!(negative > 0 && wide > 0 && classified > 0);
}
