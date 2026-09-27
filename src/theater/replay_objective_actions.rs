//! Objective action publication and native family-scoped coverage.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
/// Native variant-name classifier. VIP is deliberately absent here: its
/// selection layer is enabled separately by the caller in the reference.
pub fn replay_objective_type(variant: &str) -> &'static str {
    // Go uses simple Unicode lowercase, not multi-character case folding.
    let name: String = variant
        .chars()
        .map(|c| c.to_lowercase().next().unwrap_or(c))
        .collect();
    if name.contains("ctf") || name.contains("flag") {
        "flag"
    } else if name.contains("stronghold")
        || name.contains("land grab")
        || name.contains("total control")
    {
        "zone"
    } else if name.contains("king of the hill") || name.contains("koth") {
        "hill"
    } else if name.contains("oddball") {
        "skull"
    } else if name.contains("assault") || name.contains("bomb") {
        "bomb"
    } else {
        ""
    }
}
pub fn is_objective_family_stat(stat: &str) -> bool {
    ["flag_", "zone_", "hill_", "skull_", "vip_", "bomb_"]
        .iter()
        .any(|prefix| stat.starts_with(prefix))
}
/// Native common name contract for named and player-identified stat events.
pub trait ObjectiveStatName {
    fn stat_name(&self) -> &str;
}
impl ObjectiveStatName for StatborgNamedEvent {
    fn stat_name(&self) -> &str {
        &self.stat
    }
}
impl ObjectiveStatName for StatborgIdentifiedEvent {
    fn stat_name(&self) -> &str {
        &self.event.stat
    }
}
/// Count objective-family events regardless of identity or playback visibility.
/// The native predicate is case-sensitive and requires the family underscore.
pub fn count_objective_family<E: ObjectiveStatName>(events: &[E]) -> usize {
    events
        .iter()
        .filter(|e| is_objective_family_stat(e.stat_name()))
        .count()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayObjectiveAction {
    pub t: i64,
    pub xuid: String,
    pub stat: String,
    pub time_ms: i64,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayObjectiveActions {
    pub actions: Vec<ReplayObjectiveAction>,
    pub coverage: ReplayLayerCoverage,
    pub without_track: usize,
}
/// Kill/assist counter events remain published but are excluded from objective
/// coverage. Missing position tracks are diagnosed without deleting read facts.
pub fn build_replay_objective_actions(
    events: &[StatborgIdentifiedEvent],
    unnamed: i64,
    refused: i64,
    clock: ReplayScoreClock,
) -> ReplayObjectiveActions {
    if refused > 0 {
        return ReplayObjectiveActions {
            coverage: ReplayLayerCoverage {
                available: refused,
                refused_by_roster: refused,
                ..Default::default()
            },
            ..Default::default()
        };
    }
    let objectives = count_objective_family(events) as i64;
    let mut cov = ReplayLayerCoverage {
        available: objectives.wrapping_add(unnamed),
        no_slot: unnamed,
        ..Default::default()
    };
    if clock.interval_ms <= 0 || clock.frames <= 0 {
        cov.out_of_window = objectives;
        return ReplayObjectiveActions {
            coverage: cov,
            ..Default::default()
        };
    }
    let mut actions = Vec::new();
    for e in events {
        let count = is_objective_family_stat(&e.event.stat);
        if e.xuid.is_empty() {
            if count {
                cov.no_slot = cov.no_slot.wrapping_add(1);
            }
            continue;
        }
        let Some(t) = clock.frame_of(e.event.time_ms) else {
            if count {
                cov.out_of_window = cov.out_of_window.wrapping_add(1);
            }
            continue;
        };
        actions.push(ReplayObjectiveAction {
            t,
            xuid: e.xuid.clone(),
            stat: e.event.stat.clone(),
            time_ms: e.event.time_ms,
        });
        if count {
            cov.attached = cov.attached.wrapping_add(1);
        }
    }
    actions.sort_by(|a, b| (a.time_ms, &a.xuid, &a.stat).cmp(&(b.time_ms, &b.xuid, &b.stat)));
    ReplayObjectiveActions {
        actions,
        coverage: cov,
        without_track: 0,
    }
}
pub fn count_objective_actions_without_track(
    actions: &[ReplayObjectiveAction],
    tracks: &[ReplayTrack],
    slot_xuids: &BTreeMap<u32, u64>,
) -> usize {
    let published: BTreeSet<_> = tracks
        .iter()
        .filter_map(|t| replay_published_track_xuid(t, slot_xuids))
        .collect();
    actions
        .iter()
        .filter(|a| !published.contains(&a.xuid))
        .count()
}
/// Initial seats exclude bots and players who joined an existing seat later.
/// The caller supplies that count from match metadata; missing metadata is zero,
/// matching native offline behavior rather than counting all observed lifetimes.
#[allow(dead_code)]
pub(crate) fn build_film_replay_objective_actions(
    film: &LegacyFilm,
    players: &FilmReplayPlayers,
    variant: &str,
    initial_seats: usize,
) -> ReplayObjectiveActions {
    build_film_replay_objective_actions_observed(film, players, variant, initial_seats, None)
}

/// Retain named-event warnings before roster, death-evidence or clock filtering.
#[allow(dead_code)]
pub(crate) fn build_film_replay_objective_actions_with_diagnostics(
    film: &LegacyFilm,
    players: &FilmReplayPlayers,
    variant: &str,
    initial_seats: usize,
) -> (ReplayObjectiveActions, Vec<StatborgDiagnostic>) {
    let mut diagnostics = Vec::new();
    let out = build_film_replay_objective_actions_observed(
        film,
        players,
        variant,
        initial_seats,
        Some(&mut diagnostics),
    );
    (out, diagnostics)
}

fn build_film_replay_objective_actions_observed(
    film: &LegacyFilm,
    players: &FilmReplayPlayers,
    variant: &str,
    initial_seats: usize,
    diagnostics: Option<&mut Vec<StatborgDiagnostic>>,
) -> ReplayObjectiveActions {
    let (events, unnamed, refused) = identify_film_replay_objective_events_observed(
        film,
        players,
        variant,
        initial_seats,
        diagnostics,
    );
    let mut out = build_replay_objective_actions(
        &events,
        unnamed as i64,
        refused as i64,
        ReplayScoreClock {
            origin_ms: players.origin_ms.unwrap_or(0),
            interval_ms: players.frame_interval_ms,
            frames: players.clock.frame_count,
        },
    );
    out.without_track = count_objective_actions_without_track(
        &out.actions,
        &players.players.publication.tracks,
        &players.registry.owners.state.naming_bridge(),
    );
    out
}

/// Preserve identified millisecond events before frame-window publication for stats.
#[allow(dead_code)]
pub(crate) fn identify_film_replay_objective_events(
    film: &LegacyFilm,
    players: &FilmReplayPlayers,
    variant: &str,
    initial_seats: usize,
) -> (Vec<StatborgIdentifiedEvent>, usize, usize) {
    identify_film_replay_objective_events_observed(film, players, variant, initial_seats, None)
}

/// Retain named-event warnings before roster, death-evidence or clock filtering.
#[allow(dead_code)]
pub(crate) fn identify_film_replay_objective_events_with_diagnostics(
    film: &LegacyFilm,
    players: &FilmReplayPlayers,
    variant: &str,
    initial_seats: usize,
) -> (
    (Vec<StatborgIdentifiedEvent>, usize, usize),
    Vec<StatborgDiagnostic>,
) {
    let mut diagnostics = Vec::new();
    let out = identify_film_replay_objective_events_observed(
        film,
        players,
        variant,
        initial_seats,
        Some(&mut diagnostics),
    );
    (out, diagnostics)
}

fn identify_film_replay_objective_events_observed(
    film: &LegacyFilm,
    players: &FilmReplayPlayers,
    variant: &str,
    initial_seats: usize,
    diagnostics: Option<&mut Vec<StatborgDiagnostic>>,
) -> (Vec<StatborgIdentifiedEvent>, usize, usize) {
    let named = super::statborg_named::named_events_with_sink(
        &film.statborg.records,
        replay_objective_type(variant),
        diagnostics,
    );
    let available = count_objective_family(&named);
    if initial_seats > 8 {
        (Vec::new(), 0, available)
    } else if players.evidence.death_error.is_some() {
        (Vec::new(), available, 0)
    } else {
        let (events, _) =
            identify_statborg_named_events_by_round(&named, &players.statborg_identity);
        let kept = count_objective_family(&events);
        (events, available - kept, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn native_objective_actions_and_family_coverage() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/replay-objective-actions-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let data: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        for (i, r) in data["rows"].as_array().unwrap().iter().enumerate() {
            let events: Vec<StatborgIdentifiedEvent> =
                serde_json::from_value(r["events"].clone()).unwrap();
            assert_eq!(
                count_objective_family(&events),
                r["family_count"],
                "identified count {i}"
            );
            let named: Vec<_> = events.iter().map(|e| e.event.clone()).collect();
            assert_eq!(
                count_objective_family(&named),
                r["named_family_count"],
                "named count {i}"
            );
            for (a, b) in named.iter().zip(&events) {
                assert_eq!(a.stat_name(), b.stat_name());
            }
            let clock: ReplayScoreClock = serde_json::from_value(r["clock"].clone()).unwrap();
            let tracks: Vec<ReplayTrack> = serde_json::from_value(r["tracks"].clone()).unwrap();
            let bridge: BTreeMap<u32, u64> = serde_json::from_value(r["bridge"].clone()).unwrap();
            let mut out = build_replay_objective_actions(
                &events,
                r["unnamed"].as_i64().unwrap(),
                r["refused"].as_i64().unwrap(),
                clock,
            );
            out.without_track =
                count_objective_actions_without_track(&out.actions, &tracks, &bridge);
            assert_eq!(
                out,
                serde_json::from_value::<ReplayObjectiveActions>(r["output"].clone()).unwrap(),
                "objective actions {i}"
            );
            assert!(out.coverage.balanced());
        }
        for p in data["families"].as_array().unwrap() {
            assert_eq!(
                is_objective_family_stat(p[0].as_str().unwrap()),
                p[1].as_bool().unwrap()
            );
        }
        for p in data["modes"].as_array().unwrap() {
            assert_eq!(
                replay_objective_type(p[0].as_str().unwrap()),
                p[1].as_str().unwrap()
            );
        }
    }
}
