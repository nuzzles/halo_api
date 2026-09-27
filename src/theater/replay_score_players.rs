//! Player score publication through the native single- and multiple-round joins.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayPlayerScore {
    pub xuid: String,
    pub score: ReplayScoreSeries,
    pub kills: ReplayScoreSeries,
    pub deaths: ReplayScoreSeries,
    pub assists: ReplayScoreSeries,
}
impl ReplayPlayerScore {
    fn empty(&self) -> bool {
        [&self.score, &self.kills, &self.deaths, &self.assists]
            .iter()
            .all(|s| s.rounds.is_empty() && s.total.is_empty())
    }
}
fn counters() -> [(StatborgCounterKey, bool); 4] {
    [
        (
            StatborgCounterKey {
                component: 1,
                side: StatborgSide::B,
            },
            false,
        ),
        (
            StatborgCounterKey {
                component: 2,
                side: StatborgSide::A,
            },
            true,
        ),
        (
            StatborgCounterKey {
                component: 2,
                side: StatborgSide::B,
            },
            true,
        ),
        (
            StatborgCounterKey {
                component: 3,
                side: StatborgSide::A,
            },
            true,
        ),
    ]
}
fn player(
    xuid: String,
    [score, kills, deaths, assists]: [ReplayScoreSeries; 4],
) -> ReplayPlayerScore {
    ReplayPlayerScore {
        xuid,
        score,
        kills,
        deaths,
        assists,
    }
}
/// Single-round identities are resolved by complete match K/D/A totals. In a
/// multiple-round match, identities are reconstructed per round from deaths and
/// completed only by the native elimination/residue rules.
pub fn build_replay_player_scores(
    records: &[StatborgRecord],
    lines: &[StatborgPlayerLine],
    deaths: &[StatborgDeathInstant],
    clock: ReplayScoreClock,
) -> Vec<ReplayPlayerScore> {
    build_replay_player_scores_observed(records, lines, deaths, clock, None)
}

/// Retain native chronology and identity-budget warnings for this player-score pass.
pub fn build_replay_player_scores_with_diagnostics(
    records: &[StatborgRecord],
    lines: &[StatborgPlayerLine],
    deaths: &[StatborgDeathInstant],
    clock: ReplayScoreClock,
) -> (Vec<ReplayPlayerScore>, Vec<StatborgDiagnostic>) {
    let mut diagnostics = Vec::new();
    let out =
        build_replay_player_scores_observed(records, lines, deaths, clock, Some(&mut diagnostics));
    (out, diagnostics)
}

pub(super) fn build_replay_player_scores_observed(
    records: &[StatborgRecord],
    lines: &[StatborgPlayerLine],
    deaths: &[StatborgDeathInstant],
    clock: ReplayScoreClock,
    mut diagnostics: Option<&mut Vec<StatborgDiagnostic>>,
) -> Vec<ReplayPlayerScore> {
    // The native caller computes flat identity even for a multi-round match.
    let flat = if let Some(sink) = diagnostics.as_deref_mut() {
        let (identity, warnings) = statborg_slot_identity_with_diagnostics(records, lines);
        sink.extend(warnings);
        identity.by_slot
    } else {
        statborg_slot_identity(records, lines).by_slot
    };
    build_replay_player_scores_with_flat_observed(records, &flat, lines, deaths, clock, diagnostics)
}

pub(super) fn build_replay_player_scores_with_flat_observed(
    records: &[StatborgRecord],
    flat: &BTreeMap<i64, String>,
    lines: &[StatborgPlayerLine],
    deaths: &[StatborgDeathInstant],
    clock: ReplayScoreClock,
    diagnostics: Option<&mut Vec<StatborgDiagnostic>>,
) -> Vec<ReplayPlayerScore> {
    if resolve_statborg_rounds(records).real.len() > 1 {
        let identity = resolve_statborg_round_identity(records, deaths)
            .completed_by_elimination(records, lines)
            .completed_by_round_residue(records, lines);
        build_replay_player_scores_by_round_observed(records, &identity, clock, diagnostics)
    } else {
        build_replay_player_scores_flat_observed(records, flat, clock, diagnostics)
    }
}
pub fn build_replay_player_scores_flat(
    records: &[StatborgRecord],
    identity: &BTreeMap<i64, String>,
    clock: ReplayScoreClock,
) -> Vec<ReplayPlayerScore> {
    build_replay_player_scores_flat_observed(records, identity, clock, None)
}

/// Retain native chronology warnings before frame clipping or empty-curve filtering.
pub fn build_replay_player_scores_flat_with_diagnostics(
    records: &[StatborgRecord],
    identity: &BTreeMap<i64, String>,
    clock: ReplayScoreClock,
) -> (Vec<ReplayPlayerScore>, Vec<StatborgDiagnostic>) {
    let mut diagnostics = Vec::new();
    let out =
        build_replay_player_scores_flat_observed(records, identity, clock, Some(&mut diagnostics));
    (out, diagnostics)
}

pub(super) fn build_replay_player_scores_flat_observed(
    records: &[StatborgRecord],
    identity: &BTreeMap<i64, String>,
    clock: ReplayScoreClock,
    mut diagnostics: Option<&mut Vec<StatborgDiagnostic>>,
) -> Vec<ReplayPlayerScore> {
    if identity.is_empty() {
        return Vec::new();
    }
    let sources = counters().map(|(key, unitary)| {
        (
            statborg_series_by_round(records, key, false, false, unitary),
            super::statborg_residue::series_total_impl(
                records,
                key,
                false,
                false,
                unitary,
                diagnostics.as_deref_mut(),
            ),
        )
    });
    let mut out = Vec::new();
    for (slot, xuid) in identity {
        let series = std::array::from_fn(|i| ReplayScoreSeries {
            rounds: sources[i]
                .0
                .get(slot)
                .map_or_else(Vec::new, |r| replay_score_rounds(r, clock)),
            total: sources[i]
                .1
                .get(slot)
                .map_or_else(Vec::new, |p| replay_score_ticks(p, clock)),
        });
        let p = player(xuid.clone(), series);
        if !p.empty() {
            out.push(p);
        }
    }
    out.sort_by(|a, b| a.xuid.cmp(&b.xuid));
    out
}
type PlayerRounds = BTreeMap<String, BTreeMap<i64, Vec<StatborgScorePoint>>>;
pub fn build_replay_player_scores_by_round(
    records: &[StatborgRecord],
    identity: &StatborgRoundIdentity,
    clock: ReplayScoreClock,
) -> Vec<ReplayPlayerScore> {
    build_replay_player_scores_by_round_observed(records, identity, clock, None)
}

/// Retain native chronology warnings before frame clipping or empty-curve filtering.
pub fn build_replay_player_scores_by_round_with_diagnostics(
    records: &[StatborgRecord],
    identity: &StatborgRoundIdentity,
    clock: ReplayScoreClock,
) -> (Vec<ReplayPlayerScore>, Vec<StatborgDiagnostic>) {
    let mut diagnostics = Vec::new();
    let out = build_replay_player_scores_by_round_observed(
        records,
        identity,
        clock,
        Some(&mut diagnostics),
    );
    (out, diagnostics)
}

pub(super) fn build_replay_player_scores_by_round_observed(
    records: &[StatborgRecord],
    identity: &StatborgRoundIdentity,
    clock: ReplayScoreClock,
    mut diagnostics: Option<&mut Vec<StatborgDiagnostic>>,
) -> Vec<ReplayPlayerScore> {
    let sources: [PlayerRounds; 4] = counters().map(|(key, unitary)| {
        let mut out = PlayerRounds::new();
        for (slot, rounds) in statborg_series_by_round(records, key, false, false, unitary) {
            for (round, points) in rounds {
                if let Some(xuid) = identity
                    .publication
                    .by_round
                    .get(&round)
                    .and_then(|r| r.get(&slot))
                    .filter(|x| !x.is_empty())
                {
                    out.entry(xuid.clone()).or_default().insert(round, points);
                }
            }
        }
        out
    });
    let xuids: BTreeSet<_> = sources.iter().flat_map(|s| s.keys()).collect();
    xuids
        .into_iter()
        .filter_map(|xuid| {
            let p = player(
                xuid.clone(),
                std::array::from_fn(|i| {
                    sources[i]
                        .get(xuid)
                        .map_or_else(ReplayScoreSeries::default, |r| {
                            super::replay_score_series::score_series_of_rounds_impl(
                                r,
                                clock,
                                diagnostics.as_deref_mut(),
                            )
                        })
                }),
            );
            (!p.empty()).then_some(p)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn native_player_score_publication() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/replay-score-players-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        assert_eq!(rows.as_array().unwrap().len(), 1536);
        let mut warning_counts = [0; 2];
        for (i, r) in rows.as_array().unwrap().iter().enumerate() {
            let records: Vec<StatborgRecord> =
                serde_json::from_value(r["records"].clone()).unwrap();
            let lines: Vec<StatborgPlayerLine> =
                serde_json::from_value(r["lines"].clone()).unwrap();
            let deaths: Vec<StatborgDeathInstant> =
                serde_json::from_value(r["deaths"].clone()).unwrap();
            let flat: BTreeMap<i64, String> = serde_json::from_value(r["flat"].clone()).unwrap();
            let clock: ReplayScoreClock = serde_json::from_value(r["clock"].clone()).unwrap();
            let direct = build_replay_player_scores_flat_with_diagnostics(&records, &flat, clock);
            let automatic =
                build_replay_player_scores_with_diagnostics(&records, &lines, &deaths, clock);
            for (j, (actual, key)) in [(&direct.1, "direct_logs"), (&automatic.1, "automatic_logs")]
                .into_iter()
                .enumerate()
            {
                let expected: Vec<StatborgDiagnostic> =
                    serde_json::from_value(r[key].clone()).unwrap();
                assert_eq!(*actual, expected, "{key} {i}");
                warning_counts[j] += actual.len();
            }
            assert_eq!(
                direct.0,
                build_replay_player_scores_flat(&records, &flat, clock)
            );
            assert_eq!(
                automatic.0,
                build_replay_player_scores(&records, &lines, &deaths, clock)
            );
            assert_eq!(
                build_replay_player_scores_flat(&records, &flat, clock),
                serde_json::from_value::<Vec<ReplayPlayerScore>>(r["direct"].clone()).unwrap(),
                "flat {i}"
            );
            assert_eq!(
                build_replay_player_scores(&records, &lines, &deaths, clock),
                serde_json::from_value::<Vec<ReplayPlayerScore>>(r["automatic"].clone()).unwrap(),
                "automatic {i}"
            );
        }
        assert!(warning_counts.iter().all(|n| *n > 0));
        println!("player score warnings: {warning_counts:?} direct/automatic");
    }
}
