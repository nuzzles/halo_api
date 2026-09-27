//! LegacyFilm wiring for zone consumers of the shared managed-property scan.
use super::*;
use std::collections::BTreeMap;
pub struct FilmReplayZoneInput<'a> {
    /// Held zones in the same role/spatial order as static map publication.
    pub zones: &'a [ObjectiveZone],
    pub roles: &'a str,
    /// External roster, matching native ZoneInput; empty uses offline semantics.
    pub teams: &'a BTreeMap<String, i64>,
    pub hill: bool,
}
#[allow(dead_code)]
pub(crate) fn build_film_replay_zones(
    film: &LegacyFilm,
    players: &FilmReplayPlayers,
    actions: &[ReplayObjectiveAction],
    input: FilmReplayZoneInput<'_>,
) -> ReplayZones {
    let bridge = players.registry.owners.state.naming_bridge();
    build_replay_zones(
        ReplayZoneInput {
            // Native film_scan enables the consumer from its catalog, not scan success.
            scanned: !input.zones.is_empty(),
            reads: replay_managed_property_reads(film),
            zones: input.zones,
            roles: input.roles,
            teams: input.teams,
            hill: input.hill,
        },
        ReplayZoneContext {
            origin_us: players.clock.origin_us,
            step_us: players.clock.step_us,
            frames: players.clock.frame_count,
            interval_ms: players.frame_interval_ms,
            tracks: &players.players.publication.tracks,
            actions,
            bridge: &bridge,
        },
    )
}

/// Select the native held-zone catalog from a map asset ID and raw film variant.
#[allow(dead_code)]
pub(crate) fn build_film_replay_zones_for_map(
    film: &LegacyFilm,
    players: &FilmReplayPlayers,
    actions: &[ReplayObjectiveAction],
    teams: &BTreeMap<String, i64>,
    map_id: &str,
    variant: &str,
) -> ReplayZones {
    let selected = replay_match_objectives(map_id, variant);
    build_film_replay_zones(
        film,
        players,
        actions,
        FilmReplayZoneInput {
            zones: &selected.held_zones,
            roles: &selected.held_roles,
            teams,
            hill: selected.hill,
        },
    )
}

/// Native consumer view: failed scans publish no reads while LegacyFilm keeps their
/// partial observations and failure for inspection. Both ti13 consumers share it.
#[allow(dead_code)]
pub(crate) fn replay_managed_property_reads(film: &LegacyFilm) -> &[ManagedPropertyRead] {
    film.managed_properties
        .as_ref()
        .filter(|_| film.managed_properties_error.is_none())
        .map_or(&[], |scan| scan.reads.as_slice())
}
