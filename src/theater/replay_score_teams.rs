//! Team score publication and evidence-based assignment of scoreboard teams.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayTeamScore {
    #[serde(default, rename = "teamId", skip_serializing_if = "Option::is_none")]
    pub team_id: Option<i64>,
    #[serde(flatten)]
    pub series: ReplayScoreSeries,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayTeamScores {
    pub teams: Vec<ReplayTeamScore>,
    pub identity: String,
    pub target_score: Option<i64>,
    pub mode_supported: bool,
}
#[derive(Default)]
pub struct ReplayTeamScoreInput<'a> {
    pub records: &'a [StatborgRecord],
    pub lines: &'a [StatborgPlayerLine],
    pub team_by_xuid: BTreeMap<String, i64>,
    pub team_scores: Option<[i64; 2]>,
    pub target_score: i64,
}
type Totals = BTreeMap<i64, Vec<StatborgScorePoint>>;
fn final_value(totals: &Totals, slot: i64) -> Option<i64> {
    totals.get(&slot)?.last().map(|p| p.value)
}
fn pair(slots: &[i64], totals: &Totals, values: [(i64, i64); 2]) -> Option<BTreeMap<i64, i64>> {
    if slots.len() != 2 || values[0].1 == values[1].1 {
        return None;
    }
    let a = final_value(totals, slots[0])?;
    let b = final_value(totals, slots[1])?;
    let ids = if (a, b) == (values[0].1, values[1].1) {
        [values[0].0, values[1].0]
    } else if (a, b) == (values[1].1, values[0].1) {
        [values[1].0, values[0].0]
    } else {
        return None;
    };
    Some(
        [(slots[0], ids[0]), (slots[1], ids[1])]
            .into_iter()
            .collect(),
    )
}
/// Prefer exact, distinct final scores; otherwise require two distinct sums of
/// identified players' kills. Unresolved teams still publish unnamed curves.
pub fn build_replay_team_scores(
    input: ReplayTeamScoreInput<'_>,
    clock: ReplayScoreClock,
) -> ReplayTeamScores {
    let prepared = prepare_team_scores(input.records, input.lines, None);
    publish_team_scores(&input, clock, &prepared)
}

pub(super) struct PreparedTeamScores {
    score: Totals,
    rounds: StatborgRoundSeries,
    team_frags: Totals,
    player_frags: Totals,
    pub flat: BTreeMap<i64, String>,
}

/// Native buildScoreTimeline computes these once, before any publication.
pub(super) fn prepare_team_scores(
    records: &[StatborgRecord],
    lines: &[StatborgPlayerLine],
    mut diagnostics: Option<&mut Vec<StatborgDiagnostic>>,
) -> PreparedTeamScores {
    let mode = StatborgCounterKey {
        component: 0,
        side: StatborgSide::A,
    };
    let kills = StatborgCounterKey {
        component: 2,
        side: StatborgSide::A,
    };
    let rounds = statborg_series_by_round(records, mode, true, true, false);
    let score = super::statborg_residue::series_total_impl(
        records,
        mode,
        true,
        true,
        false,
        diagnostics.as_deref_mut(),
    );
    let team_frags = super::statborg_residue::series_total_impl(
        records,
        kills,
        true,
        false,
        true,
        diagnostics.as_deref_mut(),
    );
    let player_frags = super::statborg_residue::series_total_impl(
        records,
        kills,
        false,
        false,
        true,
        diagnostics.as_deref_mut(),
    );
    let flat = if let Some(sink) = diagnostics {
        let (identity, warnings) = statborg_slot_identity_with_diagnostics(records, lines);
        sink.extend(warnings);
        identity.by_slot
    } else {
        statborg_slot_identity(records, lines).by_slot
    };
    PreparedTeamScores {
        score,
        rounds,
        team_frags,
        player_frags,
        flat,
    }
}

pub(super) fn publish_team_scores(
    input: &ReplayTeamScoreInput<'_>,
    clock: ReplayScoreClock,
    prepared: &PreparedTeamScores,
) -> ReplayTeamScores {
    let PreparedTeamScores {
        score,
        rounds,
        team_frags,
        player_frags,
        flat,
    } = prepared;
    let slots: Vec<_> = score
        .keys()
        .chain(team_frags.keys())
        .copied()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let mut identity = "unresolved";
    let mut ids = input
        .team_scores
        .and_then(|s| pair(&slots, score, [(0, s[0]), (1, s[1])]));
    if ids.is_some() {
        identity = "a";
    }
    if ids.is_none() && slots.len() == 2 {
        let mut sums = BTreeMap::<i64, i64>::new();
        for (slot, xuid) in flat {
            if let (Some(team), Some(v)) = (
                input.team_by_xuid.get(xuid),
                final_value(player_frags, *slot),
            ) {
                let sum = sums.entry(*team).or_default();
                *sum = sum.wrapping_add(v);
            }
        }
        if sums.len() == 2 {
            let v: Vec<_> = sums.into_iter().collect();
            ids = pair(&slots, team_frags, [v[0], v[1]]);
            if ids.is_some() {
                identity = "b";
            }
        }
    }
    let teams: Vec<_> = slots
        .iter()
        .filter_map(|slot| {
            let series = ReplayScoreSeries {
                rounds: rounds
                    .get(slot)
                    .map_or_else(Vec::new, |r| replay_score_rounds(r, clock)),
                total: score
                    .get(slot)
                    .map_or_else(Vec::new, |p| replay_score_ticks(p, clock)),
            };
            (!series.rounds.is_empty() || !series.total.is_empty()).then(|| ReplayTeamScore {
                team_id: ids.as_ref().and_then(|m| m.get(slot)).copied(),
                series,
            })
        })
        .collect();
    let target_score = (input.target_score > 0
        && !teams.is_empty()
        && slots
            .iter()
            .all(|s| final_value(score, *s).is_none_or(|v| v <= input.target_score)))
    .then_some(input.target_score);
    ReplayTeamScores {
        teams,
        identity: identity.into(),
        target_score,
        mode_supported: !score.is_empty(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn native_team_score_publication() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/replay-score-teams-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        for (i, r) in rows.as_array().unwrap().iter().enumerate() {
            let records: Vec<StatborgRecord> =
                serde_json::from_value(r["records"].clone()).unwrap();
            let lines: Vec<StatborgPlayerLine> =
                serde_json::from_value(r["lines"].clone()).unwrap();
            let input = ReplayTeamScoreInput {
                records: &records,
                lines: &lines,
                team_by_xuid: serde_json::from_value(r["team_by_xuid"].clone()).unwrap(),
                team_scores: serde_json::from_value(r["team_scores"].clone()).unwrap(),
                target_score: r["target_score"].as_i64().unwrap(),
            };
            let clock: ReplayScoreClock = serde_json::from_value(r["clock"].clone()).unwrap();
            assert_eq!(
                build_replay_team_scores(input, clock),
                serde_json::from_value::<ReplayTeamScores>(r["expected"].clone()).unwrap(),
                "team scores {i}"
            );
        }
    }
}
