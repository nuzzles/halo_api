//! Flag layer wiring from decoded LegacyFilm channels and published player evidence.
use super::*;
use std::collections::BTreeMap;
/// Match-owner inputs retained by the native assembler. Spawn positions and
/// neutrality come from the objective map catalog.
pub struct FilmReplayFlagInput<'a> {
    pub scanned: bool,
    pub spawns: &'a [ReplayFlagSpawn],
}
/// Resolve canonical flag spawns by map asset ID. Missing catalog entries retain
/// the native empty-spawn fallback and do not change the film-recognition gate.
#[allow(dead_code)]
pub(crate) fn build_film_replay_flags_for_map(
    film: &LegacyFilm,
    players: &FilmReplayPlayers,
    scanned: bool,
    map_id: &str,
) -> ReplayFlags {
    let spawns = replay_map_objectives_catalog()
        .lookup(map_id)
        .map(MapObjectivesEntry::flag_spawns)
        .unwrap_or_default();
    build_film_replay_flags(
        film,
        players,
        FilmReplayFlagInput {
            scanned,
            spawns: &spawns,
        },
    )
}

/// Title rule, published only when there are carries and all dimensions are positive.
#[derive(Debug, Clone, Copy, Default, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayFlagReturnZone {
    pub radius_m: f32,
    pub reset_seconds: f32,
    pub solo_seconds: f32,
}
impl ReplayFlagReturnZone {
    /// Pinned Halo Infinite replay_labels.toml rule; not inferred from film gauges.
    pub const HALO_INFINITE: Self = Self {
        radius_m: 1.3,
        reset_seconds: 30.,
        solo_seconds: 3.1,
    };
    pub fn for_carries(self, carries: &[ReplayFlagCarry]) -> Option<Self> {
        if carries.is_empty()
            || self.radius_m <= 0.
            || self.reset_seconds <= 0.
            || self.solo_seconds <= 0.
        {
            None
        } else {
            Some(self)
        }
    }
}
#[allow(dead_code)]
pub(crate) fn build_film_replay_flags(
    film: &LegacyFilm,
    players: &FilmReplayPlayers,
    input: FilmReplayFlagInput<'_>,
) -> ReplayFlags {
    build_film_replay_flags_with_catalog(film, players, input, &replay_equipment_catalog())
}

#[allow(dead_code)]
pub(crate) fn build_film_replay_flags_with_catalog(
    film: &LegacyFilm,
    players: &FilmReplayPlayers,
    input: FilmReplayFlagInput<'_>,
    catalog: &ReplayEquipmentCatalog,
) -> ReplayFlags {
    build_film_replay_flags_observed(film, players, input, catalog, None)
}

/// Retain both native named-event passes, including when Scanned is false.
#[allow(dead_code)]
pub(crate) fn build_film_replay_flags_with_diagnostics(
    film: &LegacyFilm,
    players: &FilmReplayPlayers,
    input: FilmReplayFlagInput<'_>,
) -> (ReplayFlags, Vec<StatborgDiagnostic>) {
    build_film_replay_flags_with_catalog_and_diagnostics(
        film,
        players,
        input,
        &replay_equipment_catalog(),
    )
}

#[allow(dead_code)]
pub(crate) fn build_film_replay_flags_with_catalog_and_diagnostics(
    film: &LegacyFilm,
    players: &FilmReplayPlayers,
    input: FilmReplayFlagInput<'_>,
    catalog: &ReplayEquipmentCatalog,
) -> (ReplayFlags, Vec<StatborgDiagnostic>) {
    let mut diagnostics = Vec::new();
    let out =
        build_film_replay_flags_observed(film, players, input, catalog, Some(&mut diagnostics));
    (out, diagnostics)
}

pub(super) fn flag_named_inputs(
    records: &[StatborgRecord],
    bursts: &[i64],
    mut diagnostics: Option<&mut Vec<StatborgDiagnostic>>,
) -> (ReplayFlagFilmSignals, Vec<StatborgNamedEvent>) {
    let first =
        super::statborg_named::named_events_with_sink(records, "flag", diagnostics.as_deref_mut());
    let signals = ReplayFlagFilmSignals::from_events(bursts, &first);
    drop(first);
    // attachFlagCarries calls flagFilmSignalsOf and then NamedEventsFrom again.
    let events = super::statborg_named::named_events_with_sink(records, "flag", diagnostics);
    (signals, events)
}

fn build_film_replay_flags_observed(
    film: &LegacyFilm,
    players: &FilmReplayPlayers,
    input: FilmReplayFlagInput<'_>,
    catalog: &ReplayEquipmentCatalog,
    diagnostics: Option<&mut Vec<StatborgDiagnostic>>,
) -> ReplayFlags {
    let (signals, events) =
        flag_named_inputs(&film.statborg.records, &film.capture_bursts_ms, diagnostics);
    let recognized = input.scanned && signals.is_flag_film();
    let clock = ReplayMatchClock {
        origin_us: players.clock.origin_us,
        step_us: players.clock.step_us,
        frames: players.clock.frame_count,
        death_offset_ms: players.registry.owners.clock.offset_ms,
    };
    let mut free = Vec::new();
    if recognized {
        let complete = film
            .ground_weapon_creations
            .as_ref()
            .zip(film.ground_object_tracks.get(&42));
        free = free_objective_lives(
            complete.is_some(),
            complete.map_or(&[], |(c, _)| c.records.as_slice()),
            complete.map_or(&[], |(_, t)| t.tracks.as_slice()),
            &catalog.objective_labels,
        );
    }
    let marks = film.carrier_marks.as_ref().cloned().unwrap_or_default();
    let state = &players.registry.owners.state;
    let bridge = state.naming_bridge();
    let ambiguous: BTreeMap<_, _> = state.ambiguous_slots().iter().map(|&s| (s, true)).collect();
    let mut result = build_replay_flags(
        ReplayFlagScan {
            scanned: input.scanned,
            signals,
            events: &events,
            identity: &players.statborg_identity,
            teams: &players.players.flag_carrier_teams,
            marks: &marks,
            spawns: input.spawns,
            free: &free,
        },
        ReplayFlagContext {
            clock,
            tracks: &players.players.publication.tracks,
            deaths: &players.evidence.deaths,
            bridge: &bridge,
            ambiguous_slots: &ambiguous,
        },
    );
    if recognized {
        if let Some(c) = result.coverage.as_mut() {
            c.object_lives = free.len();
        }
        attach_replay_flag_return_gauges(
            &mut result.carries,
            replay_managed_property_reads(film),
            // Native film_scan marks the recognized consumer as scanned even
            // when the shared scanner fails and returns no readings.
            true,
            clock,
            result.coverage.as_mut(),
        );
    }
    result
}
