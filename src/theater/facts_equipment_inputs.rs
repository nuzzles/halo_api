//! Cache equipment inputs retain original values; publication follows native
//! normalization of every non-spawned, non-spent kind into taken.
use super::*;
use std::collections::BTreeSet;

pub fn build_facts_replay_equipment_changes(
    raw: &[FactsEquipmentChange],
    stats: &FactsEquipmentChangeStats,
    origin: u64,
    step: u64,
    published_slots: &BTreeSet<u32>,
) -> ReplayEquipmentChanges {
    let coverage = ReplayEquipmentChangeCoverage {
        decoded: raw.len(),
        lives: stats.lives,
        missed_estimate: stats.missed_estimate,
        counter_jumps: stats.counter_jumps,
        lives_first_off_spec: stats.lives_first_off_spec,
        repeats: stats.repeats,
        recovered: stats.recovered,
        ..Default::default()
    };
    let (selected, coverage) = super::replay_equipment_changes::select_equipment_changes(
        raw.iter()
            .map(|c| (c.timestamp_us, c.slot, c.kind.as_slice())),
        coverage,
        origin,
        step,
        published_slots,
    );
    ReplayEquipmentChanges {
        changes: selected
            .into_iter()
            .map(|(i, t)| {
                let c = &raw[i];
                ReplayEquipmentChange {
                    t,
                    slot: c.slot,
                    kind: if c.kind == b"spent" {
                        EquipmentChangeKind::Spent
                    } else {
                        EquipmentChangeKind::Taken
                    },
                    r: c.rank,
                    from: c.previous,
                    recovered: c.recovered,
                    gap: c.gap,
                }
            })
            .collect(),
        coverage,
    }
}
