//! Combined native player publication from decoded positions and resolved identity.
use super::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    num::NonZeroU64,
};

pub struct ReplayPlayersInput<'a> {
    pub samples: &'a [ReplayPositionSample],
    pub registry: &'a IdentityRegistryOutput,
    pub identity: IdentitySectionInput<'a>,
    pub film_table: &'a ReplayFilmPlayerTable,
    pub teams: &'a FilmPlayerTeams,
    /// Comparison evidence only; never used to assign teams.
    pub scoreboard_teams: &'a BTreeMap<String, i64>,
    pub successions: &'a [IdentitySuccession],
    pub fire: &'a [IdentityFireReference],
    pub min_points: i64,
    pub scoped: Option<&'a dyn Fn(u32, u64) -> i64>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayPlayers {
    pub publication: ReplayTrackPublication,
    pub bounds: ReplayBounds,
    pub bounds_rejected: usize,
    pub identity: IdentitySection,
    pub roster: Vec<ReplayRosterEntry>,
    pub teams: ReplayTeamCoverage,
    pub seats: ReplaySeatCoverage,
    pub bridge: IdentityBridgeHealth,
    pub naming: IdentityRemainingReport,
    pub death_closed_tracks: BTreeSet<usize>,
    pub overlap_fallbacks: usize,
    pub flag_carrier_teams: BTreeMap<String, i64>,
}
/// Compose the native player stages in dependency order. The caller supplies
/// chronologically ordered samples and a registry resolved from the same film.
/// A zero-step clock cannot define a replay and returns no publication.
pub fn build_replay_players(input: ReplayPlayersInput<'_>) -> Option<ReplayPlayers> {
    build_replay_players_observed(input, None)
}
pub(super) fn build_replay_players_observed(
    input: ReplayPlayersInput<'_>,
    match_id: Option<&str>,
) -> Option<ReplayPlayers> {
    let clock = input.identity.clock;
    let step = NonZeroU64::new(clock.step_us)?;
    let registry = input.registry;
    let identity = build_identity_section(
        registry,
        IdentitySectionInput {
            clock,
            deaths: input.identity.deaths,
            bots: input.identity.bots,
            roster_xuids: input.identity.roster_xuids,
            statborg: input.identity.statborg,
            statborg_records: input.identity.statborg_records,
        },
    );
    let mut publication = publish_replay_tracks(
        input.samples,
        registry.owners.state.lives(),
        clock.origin_us,
        step,
        input.min_points,
        input.scoped,
    );
    if let Some(id) = match_id {
        super::replay_player_document_logs::tracks(id, &publication.coverage);
    }
    let (bounds, bounds_rejected) = publication.bounds();
    if let Some(id) = match_id {
        super::replay_player_document_logs::bounds(id, bounds_rejected);
    }
    let naming = publication.name_tracks(IdentityTrackNamingInput {
        state: &registry.owners.state,
        bots: input.identity.bots,
        successions: input.successions,
        fire: input.fire,
        origin_us: clock.origin_us,
        step_us: clock.step_us,
        death_offset_ms: registry.owners.clock.offset_ms,
        offset_matches: registry.owners.clock.matched,
    });
    if let Some(id) = match_id {
        super::replay_player_document_logs::naming(id, &publication.tracks, &naming.remaining);
    }
    let team_publication =
        ReplayTeamPublication::new(registry, input.teams, input.scoreboard_teams);
    let team_counts = team_publication.apply_to_tracks(&mut publication.tracks);
    let mut roster =
        team_publication.roster_from_registry(registry, input.identity.deaths, input.identity.bots);
    let seats = assign_replay_seats(
        &mut roster,
        &publication.tracks,
        input.film_table,
        clock.frame_count,
    );
    let teams = team_publication.coverage(team_counts, &roster);
    let mut bridge = registry.bridge_health();
    naming.remaining.apply_to_health(&mut bridge);
    Some(ReplayPlayers {
        publication,
        bounds,
        bounds_rejected,
        identity,
        roster,
        teams,
        seats,
        bridge,
        naming: naming.remaining,
        death_closed_tracks: naming.death_closed,
        overlap_fallbacks: naming.overlap_fallbacks,
        flag_carrier_teams: team_publication.flag_carrier_teams(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Case {
        layer: Option<ReplayPlayers>,
        samples: Vec<ReplayPositionSample>,
        teams: FilmPlayerTeams,
        control: BTreeMap<String, i64>,
        successions: Vec<IdentitySuccession>,
        film_table: ReplayFilmPlayerTable,
        roster_xuids: Vec<u64>,
        participants: Vec<IdentityParticipant>,
        bots: Vec<IdentityBot>,
        statborg: IdentityStatborgPublication,
        clock: IdentityClock,
        statborg_records: Vec<StatborgRecord>,
        positions: Vec<ReplayPlayerPosition>,
        creations: Vec<IdentityCreationRecord>,
        deaths: Vec<IdentityDeath>,
        fire: Vec<IdentityFireReference>,
        indices: PlayerIndexTable,
        output: IdentityRegistryOutput,
    }
    #[test]
    fn native_combined_player_publication() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/replay-players-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&raw).unwrap();
        for (i, c) in cases.into_iter().enumerate() {
            let registry = build_identity_registry(IdentityRegistryInput {
                positions: &c.positions,
                creations: &c.creations,
                deaths: &c.deaths,
                replication_indices: &c.indices,
                film_table: &c.film_table,
                bots: &c.bots,
                fire: &c.fire,
                roster_xuids: &c.roster_xuids,
                participants: &c.participants,
                statborg_records: &c.statborg_records,
            });
            assert_eq!(registry, c.output, "registry {i}");
            let actual = build_replay_players(ReplayPlayersInput {
                samples: &c.samples,
                registry: &registry,
                identity: IdentitySectionInput {
                    clock: c.clock,
                    deaths: &c.deaths,
                    bots: &c.bots,
                    roster_xuids: &c.roster_xuids,
                    statborg: &c.statborg,
                    statborg_records: &c.statborg_records,
                },
                film_table: &c.film_table,
                teams: &c.teams,
                scoreboard_teams: &c.control,
                successions: &c.successions,
                fire: &c.fire,
                min_points: 1,
                scoped: None,
            });
            assert_eq!(actual, c.layer, "player publication {i}");
        }
    }
}
