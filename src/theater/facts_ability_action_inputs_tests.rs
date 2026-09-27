use super::*;
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Read,
};
#[derive(Deserialize)]
struct Rank {
    slot: u32,
    time: u64,
    rank: i64,
}
#[derive(Deserialize)]
struct Charge {
    slot: u32,
    time: u64,
    charges: i64,
    emplacement: i64,
    low: i64,
}
#[derive(Deserialize)]
struct Life {
    slot: u32,
    from: i64,
    to: i64,
}
#[derive(Deserialize)]
struct Probe {
    slot: u32,
    at: u64,
    rank: Option<i64>,
}
#[derive(Deserialize)]
struct Case {
    lives: Vec<Life>,
    ranks: Vec<Rank>,
    reads: Vec<Charge>,
    impulses: Option<Vec<FactsAbilityImpulse>>,
    impulse_stats: FactsAbilityImpulseStats,
    palette: Option<BTreeMap<i64, String>>,
    measured: BTreeSet<String>,
    published: BTreeSet<u32>,
    absent: bool,
    origin: u64,
    step: u64,
    output: ReplayAbilityChargePublication,
    impulse_output: ReplayAbilityImpulsePublication,
    folded: Vec<ReplayAbilityImpulseEpisode>,
    probes: Vec<Probe>,
}
#[test]
fn native_facts_ability_action_domains_and_lifetimes() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/facts-ability-actions-v41.json.zlib")[..],
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Case> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 1024);
    let mut wide_ranks = 0;
    let mut wide_charges = 0;
    let mut impulse_count = 0;
    for (i, row) in rows.into_iter().enumerate() {
        let ranks: Vec<_> = row
            .ranks
            .into_iter()
            .map(|r| FactsAbilityRank {
                timestamp_us: r.time,
                slot: r.slot,
                rank: r.rank,
            })
            .collect();
        let lives: Vec<_> = row
            .lives
            .into_iter()
            .map(|l| IdentityLife {
                slot: l.slot,
                from: l.from,
                to: l.to,
                ..Default::default()
            })
            .collect();
        let reads: Vec<_> = row
            .reads
            .into_iter()
            .map(|r| FactsAbilityCharge {
                timestamp_us: r.time,
                slot: r.slot,
                charges: r.charges,
                emplacement: r.emplacement,
                low: r.low,
            })
            .collect();
        let impulses = row.impulses.unwrap_or_default();
        let context = || FactsReplayAbilityContext {
            ranks: &ranks,
            lives: &lives,
            palette: row.palette.as_ref(),
            measured_families: &row.measured,
            published_slots: &row.published,
            origin_us: row.origin,
            step_us: row.step,
        };
        let got = build_facts_replay_ability_charges(
            &reads,
            &FactsAbilityChargeStats {
                absent: row.absent,
                ..Default::default()
            },
            context(),
        );
        wide_charges += got
            .charges
            .iter()
            .filter(|r| r.charges < 0 || r.charges > 255)
            .count();
        assert_eq!(got, row.output, "charges {i}");
        assert_eq!(
            fold_facts_replay_ability_impulses(&impulses),
            row.folded,
            "fold {i}"
        );
        let got = build_facts_replay_ability_impulses(&impulses, &row.impulse_stats, context());
        impulse_count += got.impulses.len();
        assert_eq!(got, row.impulse_output, "impulses {i}");
        let index = ReplayAbilityRankIndex::from_facts(&ranks, &lives);
        for probe in row.probes {
            let got = index.rank_in_life(probe.slot, probe.at);
            wide_ranks += usize::from(got.is_some_and(|rank| !(0..=255).contains(&rank)));
            assert_eq!(got, probe.rank, "rank lookup {i}");
        }
    }
    assert!(wide_ranks > 0 && wide_charges > 0 && impulse_count > 0);
}
