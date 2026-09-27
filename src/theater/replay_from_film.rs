//! Player-layer assembly from a map-calibrated LegacyFilm and its original chunks.
use super::*;
use crate::clients::hi::models::FilmChunkData;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Default)]
pub struct FilmReplayPlayerOptions<'a> {
    pub frame_interval_ms: i64,
    pub min_points: i64,
    pub roster_xuids: &'a [u64],
    pub participants: &'a [IdentityParticipant],
    pub successions: &'a [IdentitySuccession],
    pub scoreboard_teams: BTreeMap<String, i64>,
    /// Optional match totals used only by the native statborg identity completions.
    pub statborg_lines: &'a [StatborgPlayerLine],
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilmReplayPlayers {
    pub clock: IdentityClock,
    pub frame_interval_ms: i64,
    pub duration_ms: i64,
    pub origin_ms: Option<i64>,
    pub evidence: ReplayIdentityEvidence,
    pub registry: IdentityRegistryOutput,
    pub statborg_identity: StatborgRoundIdentity,
    /// Warnings from the scoreboard identity-completion pass; source and score
    /// timeline diagnostics are retained by their respective outputs separately.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub statborg_diagnostics: Vec<StatborgDiagnostic>,
    pub players: ReplayPlayers,
}
/// Build the native player layer using the same chunks that produced `film`.
/// Map-calibrated biped positions and team readings must already be present.
/// External match evidence is optional; names, teams and scope come from the film.
/// This is the player layer, not yet the complete multi-layer replay document.
#[allow(dead_code)]
pub(crate) fn build_film_replay_players(
    film: &LegacyFilm,
    chunks: &[FilmChunkData],
    options: FilmReplayPlayerOptions<'_>,
) -> Result<Option<FilmReplayPlayers>, DecodeError> {
    if film.major_version != 41 {
        return Err(DecodeError::UnsupportedVersion(film.major_version));
    }
    let stream = film
        .biped_positions
        .as_ref()
        .ok_or(DecodeError::Missing("map-calibrated biped positions"))?;
    let samples: Vec<_> = stream
        .accepted()
        .map(|p| {
            let [x, y, z] = p.record.world;
            ReplayPositionSample::from_companions(
                ReplayPlayerPosition {
                    slot: p.record.slot,
                    timestamp_us: p.source.timestamp_us,
                    x,
                    y,
                    z,
                    has_world: true,
                },
                &p.record.companions,
            )
        })
        .collect();
    let Some(timeline) = prepare_replay_timeline(&samples, options.frame_interval_ms) else {
        return Ok(None);
    };
    let teams = film
        .player_teams
        .as_ref()
        .ok_or(DecodeError::Missing("film player-team scan context"))?;
    let evidence = super::replay_evidence::scan_replay_identity_evidence_retained(
        chunks,
        film.major_version,
        options.roster_xuids,
        film.native_clock_origin.as_ref(),
        film.native_identity_inputs.as_ref(),
    )?;
    let creations: Vec<_> = film
        .biped_creations
        .records
        .iter()
        .map(IdentityCreationRecord::from)
        .collect();
    let bots: Vec<_> = film
        .bot_metadata
        .bots
        .iter()
        .map(IdentityBot::from)
        .collect();
    // Native closures consume the legacy FilmIndex field, not ShooterIndex5.
    let fire: Vec<_> = film
        .fire_events
        .iter()
        .map(|e| IdentityFireReference {
            film_index: i64::from(e.film_index),
            timestamp_us: e.timestamp_us,
        })
        .collect();
    let film_table = film
        .player_table
        .as_ref()
        .map(ReplayFilmPlayerTable::from_decoded)
        .unwrap_or_else(|| ReplayFilmPlayerTable {
            refusal: if film.identity.is_none() {
                "sans_section"
            } else {
                "table_introuvable"
            }
            .into(),
            ..Default::default()
        });
    let records = &film.statborg.records;
    // Scope uses raw replication stays, before identity/death refinement of lives.
    let scope = ReplayScopeLookup::new(
        &film.zoom_events,
        &build_identity_life_spans(
            &timeline
                .samples
                .iter()
                .map(|s| s.position.clone())
                .collect::<Vec<_>>(),
        ),
        REPLAY_ZOOM_HOLD_US,
    );
    Ok(Some(assemble_replay_player_inputs(
        ReplayPlayerAssemblyInput {
            timeline,
            evidence,
            creations: &creations,
            bots: &bots,
            fire: &fire,
            film_table: &film_table,
            records,
            teams,
            scope: &scope,
        },
        options,
    )))
}

/// Internal assembly boundary; no recording bytes or fabricated source references.
pub(super) struct ReplayPlayerAssemblyInput<'a> {
    pub timeline: ReplayTimeline,
    pub evidence: ReplayIdentityEvidence,
    pub creations: &'a [IdentityCreationRecord],
    pub bots: &'a [IdentityBot],
    pub fire: &'a [IdentityFireReference],
    pub film_table: &'a ReplayFilmPlayerTable,
    pub records: &'a [StatborgRecord],
    pub teams: &'a FilmPlayerTeams,
    pub scope: &'a ReplayScopeLookup,
}
pub(super) fn assemble_replay_player_inputs(
    input: ReplayPlayerAssemblyInput<'_>,
    options: FilmReplayPlayerOptions<'_>,
) -> FilmReplayPlayers {
    assemble_replay_player_inputs_with_identity(input, options, None, None)
}
pub(super) fn assemble_replay_player_inputs_with_identity(
    input: ReplayPlayerAssemblyInput<'_>,
    options: FilmReplayPlayerOptions<'_>,
    supplied_identity: Option<&StatborgRoundIdentity>,
    match_id: Option<&str>,
) -> FilmReplayPlayers {
    let ReplayPlayerAssemblyInput {
        timeline,
        evidence,
        creations,
        bots,
        fire,
        film_table,
        records,
        teams,
        scope,
    } = input;
    let positions: Vec<_> = timeline
        .samples
        .iter()
        .map(|s| s.position.clone())
        .collect();
    let instants: Vec<_> = evidence
        .deaths
        .iter()
        .map(|d| StatborgDeathInstant {
            xuid: d.xuid.to_string(),
            time_ms: d.time_ms,
        })
        .collect();
    let (statborg_identity, statborg_diagnostics) = if let Some(identity) = supplied_identity {
        (identity.clone(), Vec::new())
    } else {
        let (statborg_identity, statborg_diagnostics) =
            resolve_statborg_round_identity(records, &instants)
                .completed_by_lines_with_diagnostics(records, options.statborg_lines);
        let statborg_identity = statborg_identity
            .completed_by_elimination(records, options.statborg_lines)
            .completed_by_round_residue(records, options.statborg_lines);
        (statborg_identity, statborg_diagnostics)
    };
    let registry = super::identity_registry::build_identity_registry_observed(
        IdentityRegistryInput {
            positions: &positions,
            creations,
            deaths: &evidence.deaths,
            replication_indices: &evidence.replication_indices,
            film_table,
            bots,
            fire,
            roster_xuids: options.roster_xuids,
            participants: options.participants,
            statborg_records: records,
        },
        match_id,
    );
    let scoped = |slot, stamp| scope.at(slot, stamp);
    let players = super::replay_players::build_replay_players_observed(
        ReplayPlayersInput {
            samples: &timeline.samples,
            registry: &registry,
            identity: IdentitySectionInput {
                clock: timeline.clock,
                deaths: &evidence.deaths,
                bots,
                roster_xuids: options.roster_xuids,
                statborg: &statborg_identity.publication,
                statborg_records: records,
            },
            film_table,
            teams,
            scoreboard_teams: &options.scoreboard_teams,
            successions: options.successions,
            fire,
            min_points: options.min_points.max(1),
            scoped: Some(&scoped),
        },
        match_id,
    )
    .expect("timeline has a nonzero frame step");
    let origin_ms = resolve_replay_origin_ms(
        timeline.clock.origin_us,
        evidence.film_clock_us,
        registry.owners.clock.offset_ms,
        registry.owners.clock.matched,
    );
    FilmReplayPlayers {
        clock: timeline.clock,
        frame_interval_ms: timeline.frame_interval_ms,
        duration_ms: timeline.duration_ms,
        origin_ms,
        evidence,
        registry,
        statborg_identity,
        statborg_diagnostics,
        players,
    }
}
