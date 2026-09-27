//! Complete score timeline publication, with optional input and explicit coverage.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayScoreTimeline {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub teams: Vec<ReplayTeamScore>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub players: Vec<ReplayPlayerScore>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_score: Option<i64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hold_ticks: Vec<ReplayTeamHold>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hold_ticks_per_point: Option<i64>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayScoreCoverage {
    pub team_identity: String,
    pub rounds: usize,
    pub mode_supported: bool,
    pub truncated: bool,
    pub oracle: String,
    pub points: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rounds_written: Vec<i64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rounds_contradicted: Vec<i64>,
    #[serde(default, skip_serializing_if = "score_zero")]
    pub rounds_contradicted_records: usize,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub rounds_decreed: bool,
}
fn score_zero(n: &usize) -> bool {
    *n == 0
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayScoreOutput {
    pub timeline: Option<ReplayScoreTimeline>,
    pub coverage: Option<ReplayScoreCoverage>,
    pub round_zero_fallbacks: usize,
}
#[derive(Default)]
pub struct ReplayScoreInput<'a> {
    pub teams: ReplayTeamScoreInput<'a>,
    pub hold_ticks_per_point: i64,
    pub truncated: bool,
}
fn count(s: &ReplayScoreSeries) -> usize {
    s.total.len() + s.rounds.iter().map(|r| r.points.len()).sum::<usize>()
}
/// Missing input omits coverage. Supplied empty input still publishes the
/// reference's round decision and coverage, even when it yields no timeline.
pub fn build_replay_score(
    input: Option<ReplayScoreInput<'_>>,
    deaths: &[StatborgDeathInstant],
    clock: ReplayScoreClock,
) -> ReplayScoreOutput {
    build_replay_score_observed(input, deaths, clock, None)
}

/// Score-pass warnings in native computation order. Source-scan diagnostics are
/// separate; this API consumes already-decoded records and never rescans a film.
pub fn build_replay_score_with_diagnostics(
    input: Option<ReplayScoreInput<'_>>,
    deaths: &[StatborgDeathInstant],
    clock: ReplayScoreClock,
) -> (ReplayScoreOutput, Vec<StatborgDiagnostic>) {
    let mut diagnostics = Vec::new();
    let out = build_replay_score_observed(input, deaths, clock, Some(&mut diagnostics));
    (out, diagnostics)
}

fn build_replay_score_observed(
    input: Option<ReplayScoreInput<'_>>,
    deaths: &[StatborgDeathInstant],
    clock: ReplayScoreClock,
    mut diagnostics: Option<&mut Vec<StatborgDiagnostic>>,
) -> ReplayScoreOutput {
    let Some(input) = input else {
        return ReplayScoreOutput::default();
    };
    let records = input.teams.records;
    let prepared = super::replay_score_teams::prepare_team_scores(
        records,
        input.teams.lines,
        diagnostics.as_deref_mut(),
    );
    let team = super::replay_score_teams::publish_team_scores(&input.teams, clock, &prepared);
    let players = super::replay_score_players::build_replay_player_scores_with_flat_observed(
        records,
        &prepared.flat,
        input.teams.lines,
        deaths,
        clock,
        diagnostics.as_deref_mut(),
    );
    let hold_ticks = if input.hold_ticks_per_point > 0 {
        super::replay_score_hold::build_replay_hold_ticks_observed(
            records,
            &prepared.flat,
            &input.teams.team_by_xuid,
            clock,
            diagnostics,
        )
    } else {
        Vec::new()
    };
    let points = team.teams.iter().map(|t| count(&t.series)).sum::<usize>()
        + players
            .iter()
            .map(|p| count(&p.score) + count(&p.kills) + count(&p.deaths) + count(&p.assists))
            .sum::<usize>();
    let hold_ticks_per_point = (input.hold_ticks_per_point > 0 && !hold_ticks.is_empty())
        .then_some(input.hold_ticks_per_point);
    let decision = resolve_statborg_rounds(records);
    let coverage = ReplayScoreCoverage {
        team_identity: team.identity,
        rounds: decision.real.len(),
        mode_supported: team.mode_supported,
        truncated: input.truncated,
        oracle: "displayed".into(),
        points,
        rounds_written: if decision.written.len() > 1 {
            decision.written
        } else {
            Vec::new()
        },
        rounds_contradicted: decision.contradicted,
        rounds_contradicted_records: decision.contradicted_records,
        rounds_decreed: decision.decreed,
    };
    let timeline = (!team.teams.is_empty() || !players.is_empty()).then_some(ReplayScoreTimeline {
        teams: team.teams,
        players,
        target_score: team.target_score,
        hold_ticks,
        hold_ticks_per_point,
    });
    ReplayScoreOutput {
        timeline,
        coverage: Some(coverage),
        round_zero_fallbacks: usize::from(decision.decreed),
    }
}
#[derive(Default)]
pub struct FilmReplayScoreOptions<'a> {
    pub lines: &'a [StatborgPlayerLine],
    pub team_by_xuid: BTreeMap<String, i64>,
    pub team_scores: Option<[i64; 2]>,
    pub target_score: i64,
    pub hold_ticks_per_point: i64,
}
#[allow(dead_code)]
pub(crate) fn build_film_replay_score(
    film: &LegacyFilm,
    players: &FilmReplayPlayers,
    options: FilmReplayScoreOptions<'_>,
) -> ReplayScoreOutput {
    build_film_replay_score_observed(film, players, options, None)
}

/// Retain score-pass diagnostics. LegacyFilm.statborg_diagnostics retains the preceding
/// source pass separately; neither pass is duplicated by this function.
#[allow(dead_code)]
pub(crate) fn build_film_replay_score_with_diagnostics(
    film: &LegacyFilm,
    players: &FilmReplayPlayers,
    options: FilmReplayScoreOptions<'_>,
) -> (ReplayScoreOutput, Vec<StatborgDiagnostic>) {
    let mut diagnostics = Vec::new();
    let out = build_film_replay_score_observed(film, players, options, Some(&mut diagnostics));
    (out, diagnostics)
}

fn build_film_replay_score_observed(
    film: &LegacyFilm,
    players: &FilmReplayPlayers,
    options: FilmReplayScoreOptions<'_>,
    diagnostics: Option<&mut Vec<StatborgDiagnostic>>,
) -> ReplayScoreOutput {
    let deaths: Vec<_> = players
        .evidence
        .deaths
        .iter()
        .map(|d| StatborgDeathInstant {
            xuid: d.xuid.to_string(),
            time_ms: d.time_ms,
        })
        .collect();
    build_replay_score_observed(
        Some(ReplayScoreInput {
            teams: ReplayTeamScoreInput {
                records: &film.statborg.records,
                lines: options.lines,
                team_by_xuid: options.team_by_xuid,
                team_scores: options.team_scores,
                target_score: options.target_score,
            },
            truncated: film.statborg.truncated,
            hold_ticks_per_point: options.hold_ticks_per_point,
        }),
        &deaths,
        ReplayScoreClock {
            origin_ms: players.origin_ms.unwrap_or(0),
            interval_ms: players.frame_interval_ms,
            frames: players.clock.frame_count,
        },
        diagnostics,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn native_complete_score_publication() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/replay-score-teams-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        assert_eq!(rows.as_array().unwrap().len(), 1536);
        let mut warning_count = 0;
        for (i, r) in rows.as_array().unwrap().iter().enumerate() {
            let records: Vec<StatborgRecord> =
                serde_json::from_value(r["full_records"].clone()).unwrap();
            let lines: Vec<StatborgPlayerLine> =
                serde_json::from_value(r["full_lines"].clone()).unwrap();
            let clock: ReplayScoreClock = serde_json::from_value(r["clock"].clone()).unwrap();
            let input = || {
                (!r["absent"].as_bool().unwrap()).then(|| ReplayScoreInput {
                    teams: ReplayTeamScoreInput {
                        records: &records,
                        lines: &lines,
                        team_by_xuid: serde_json::from_value(r["team_by_xuid"].clone()).unwrap(),
                        team_scores: serde_json::from_value(r["team_scores"].clone()).unwrap(),
                        target_score: r["target_score"].as_i64().unwrap(),
                    },
                    truncated: r["truncated"].as_bool().unwrap(),
                    hold_ticks_per_point: r["hold_ticks_per_point"].as_i64().unwrap(),
                })
            };
            let (actual, warnings) = build_replay_score_with_diagnostics(input(), &[], clock);
            assert_eq!(
                warnings,
                serde_json::from_value::<Vec<StatborgDiagnostic>>(r["warnings"].clone()).unwrap(),
                "warnings {i}"
            );
            warning_count += warnings.len();
            assert_eq!(
                actual,
                build_replay_score(input(), &[], clock),
                "unobserved {i}"
            );
            assert_eq!(
                actual,
                serde_json::from_value::<ReplayScoreOutput>(r["full"].clone()).unwrap(),
                "score {i}"
            );
        }
        assert!(warning_count > 0);
        println!("complete score warnings: {warning_count}");
    }
}
