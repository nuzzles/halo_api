//! Hill hold progress is the maximum player increment at each emission time.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayTeamHold {
    #[serde(default, rename = "teamId", skip_serializing_if = "Option::is_none")]
    pub team_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ticks: Vec<ReplayScoreTick>,
}
/// The caller must establish that this variant uses hold ticks. Pre-origin
/// increments contribute to the cumulative value, but emit no displayed point.
/// Separate emissions within one frame remain separate, matching the reference.
pub fn build_replay_hold_ticks(
    records: &[StatborgRecord],
    identity: &BTreeMap<i64, String>,
    team_by_xuid: &BTreeMap<String, i64>,
    clock: ReplayScoreClock,
) -> Vec<ReplayTeamHold> {
    build_replay_hold_ticks_observed(records, identity, team_by_xuid, clock, None)
}

pub(super) fn build_replay_hold_ticks_observed(
    records: &[StatborgRecord],
    identity: &BTreeMap<i64, String>,
    team_by_xuid: &BTreeMap<String, i64>,
    clock: ReplayScoreClock,
    diagnostics: Option<&mut Vec<StatborgDiagnostic>>,
) -> Vec<ReplayTeamHold> {
    let mut slots = BTreeMap::<i64, Vec<i64>>::new();
    for (slot, xuid) in identity {
        if let Some(team) = team_by_xuid.get(xuid) {
            slots.entry(*team).or_default().push(*slot);
        }
    }
    if slots.is_empty() {
        return Vec::new();
    }
    let series = super::statborg_residue::series_total_impl(
        records,
        StatborgCounterKey {
            component: 23,
            side: StatborgSide::A,
        },
        false,
        false,
        false,
        diagnostics,
    );
    let times: BTreeSet<_> = slots
        .values()
        .flatten()
        .filter_map(|s| series.get(s))
        .flatten()
        .map(|p| p.time_ms)
        .collect();
    let mut cursors = BTreeMap::<i64, (usize, i64)>::new();
    let mut cumul = BTreeMap::<i64, i64>::new();
    let mut points = BTreeMap::<i64, Vec<ReplayScoreTick>>::new();
    for time in times {
        for (team, slots) in &slots {
            let mut increment = 0;
            for slot in slots {
                let cursor = cursors.entry(*slot).or_default();
                let previous = cursor.1;
                if let Some(seq) = series.get(slot) {
                    while cursor.0 < seq.len() && seq[cursor.0].time_ms <= time {
                        cursor.1 = seq[cursor.0].value;
                        cursor.0 += 1;
                    }
                }
                if cursor.1 >= previous {
                    increment = increment.max(cursor.1.wrapping_sub(previous));
                }
            }
            if increment == 0 {
                continue;
            }
            let total = cumul.entry(*team).or_default();
            *total = total.wrapping_add(increment);
            if let Some(t) = clock.frame_of(time) {
                points
                    .entry(*team)
                    .or_default()
                    .push(ReplayScoreTick { t, v: *total });
            }
        }
    }
    points
        .into_iter()
        .map(|(team, ticks)| ReplayTeamHold {
            team_id: Some(team),
            ticks,
        })
        .collect()
}
