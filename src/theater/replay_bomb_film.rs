//! LegacyFilm-level bomb assembly retaining raw carry periods for statistics.
use super::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FilmReplayBomb {
    pub carries: FilmReplayBombCarries,
    pub armings: Option<ReplayBombArmings>,
    pub stats: Option<ReplayBombStats>,
}

/// Inputs resolved by the match owner. Kill pairs already use the match clock;
/// `kills_read` distinguishes an empty observation from unavailable evidence.
pub struct FilmReplayBombInput<'a> {
    pub recognized: bool,
    /// Whether the upstream score input exists, including an empty read section.
    pub score_read: bool,
    pub identified_objectives: &'a [StatborgIdentifiedEvent],
    pub published_objectives: &'a [ReplayObjectiveAction],
    pub kills_read: bool,
    pub kills: &'a [ReplayKillReference],
}

/// Assemble in reference order: carry, armings, then all five statistics.
/// Fuse validation uses only published detonations; stats consume all identified
/// objective events and raw millisecond carries, before frame-window clipping.
#[allow(dead_code)]
pub(crate) fn build_film_replay_bomb(
    film: &LegacyFilm,
    players: &FilmReplayPlayers,
    input: FilmReplayBombInput<'_>,
) -> FilmReplayBomb {
    if !input.recognized {
        return FilmReplayBomb::default();
    }
    let carries = build_film_replay_bomb_carries(film, players, true);
    let mut detonations: Vec<_> = input
        .published_objectives
        .iter()
        .filter(|a| a.stat == "bomb_detonations")
        .map(|a| a.time_ms)
        .collect();
    detonations.sort_unstable();
    let armings = build_film_replay_bomb_armings(film, players, true, &detonations);
    let stats = build_replay_bomb_stats(&ReplayBombStatsInput {
        detonations_read: input.score_read,
        objectives: input.identified_objectives.to_vec(),
        carry_read: players.registry.owners.state.bridge_established(),
        carry: carries.raw.clone(),
        kills_read: input.kills_read,
        kills: input.kills.to_vec(),
        armings_read: armings
            .as_ref()
            .is_some_and(|a| a.coverage.scanned && !a.coverage.suppressed),
        armings: armings
            .as_ref()
            .map_or_else(Vec::new, |a| a.armings.clone()),
        film_to_match_offset_ms: ((players.evidence.film_clock_us as i64) / 1000)
            .wrapping_sub(players.registry.owners.clock.offset_ms),
    });
    FilmReplayBomb {
        carries,
        armings,
        stats: Some(stats),
    }
}
