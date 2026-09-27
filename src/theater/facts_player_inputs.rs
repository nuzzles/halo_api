//! Checked bridge from exact cache DTOs to the existing player assembly API.
//! The source cache stays intact; unsupported legacy DTO domains are errors.
use super::*;
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum FactsProjectionError {
    #[error("facts field {field} contains bytes outside the replay string domain")]
    String { field: &'static str },
}
impl From<&FactsBipedPosition> for ReplayPositionSample {
    fn from(p: &FactsBipedPosition) -> Self {
        Self {
            position: ReplayPlayerPosition {
                slot: p.slot,
                timestamp_us: p.timestamp_us,
                x: p.world[0],
                y: p.world[1],
                z: p.world[2],
                has_world: p.has_world,
            },
            aim: p.has_yaw.then_some([p.yaw_raw, p.pitch_raw]),
            health: p.has_body.then_some(p.health),
            shield: p.has_shield.then_some(p.shield),
        }
    }
}
impl From<&FactsBipedCreation> for IdentityCreationRecord {
    fn from(c: &FactsBipedCreation) -> Self {
        Self {
            slot: c.slot,
            generation: c.generation,
            timestamp_us: c.timestamp_us,
            participant_index: c.participant_index,
            has_index: c.has_index,
        }
    }
}
impl TryFrom<&FactsDeath> for IdentityDeath {
    type Error = FactsProjectionError;
    fn try_from(d: &FactsDeath) -> Result<Self, Self::Error> {
        Ok(Self {
            xuid: d.xuid,
            gamertag: ReplayByteString(d.gamertag.clone()),
            time_ms: d.time_ms,
        })
    }
}
impl TryFrom<&FactsPlayerIndices> for PlayerIndexTable {
    type Error = FactsProjectionError;
    fn try_from(v: &FactsPlayerIndices) -> Result<Self, Self::Error> {
        Ok(Self {
            by_xuid: v.by_xuid.clone(),
            readings: v.readings,
            disagreements: v.disagreements,
        })
    }
}
impl TryFrom<&FactsPlayerTable> for ReplayFilmPlayerTable {
    type Error = FactsProjectionError;
    fn try_from(v: &FactsPlayerTable) -> Result<Self, Self::Error> {
        Ok(Self {
            seats: v
                .seats
                .as_deref()
                .unwrap_or_default()
                .iter()
                .map(|s| {
                    Ok(ReplayFilmPlayerSeat {
                        film_index: s.film_index,
                        xuid: s.xuid,
                        gamertag: ReplayByteString(s.gamertag.clone()),
                    })
                })
                .collect::<Result<_, Self::Error>>()?,
            build: ReplayByteString(v.build.clone()),
            occupied: v.occupied,
            vacant: v.vacant,
            interleaved_vacant: v.interleaved_vacant,
            refusal: ReplayByteString(v.refusal.clone()),
        })
    }
}
impl TryFrom<&FactsTeamScan> for PlayerTeamScanReport {
    type Error = FactsProjectionError;
    fn try_from(v: &FactsTeamScan) -> Result<Self, Self::Error> {
        Ok(Self {
            archetype_absent: v.archetype_absent,
            component_mismatch: v.component_mismatch,
            component: ReplayByteString(v.component.clone()),
            packets: v.packets,
            records: v.records,
            read: v.read,
            unreached: v.unreached,
            out_of_domain_index: v.out_of_domain_index,
            out_of_domain_value: v.out_of_domain_value,
            entities: v.entities,
            entity_divergences: v.entity_divergences,
            index_divergences: v.index_divergences,
            indices: v.indices,
            no_team: v.no_team,
        })
    }
}
impl From<&FactsStatRecord> for StatborgRecord {
    fn from(v: &FactsStatRecord) -> Self {
        Self {
            time_ms: v.time_ms,
            slot: v.slot,
            round: v.round,
            comps: v
                .comps
                .as_ref()
                .into_iter()
                .flat_map(|m| m.iter())
                .map(|(&k, v)| {
                    (
                        k,
                        StatborgValue {
                            a: v.a,
                            b: v.b,
                            c: v.c,
                            d: v.d,
                            has_c: v.has_c,
                            has_d: v.has_d,
                        },
                    )
                })
                .collect(),
        }
    }
}
/// Player publication from decoded cache facts. Bots and statborg are caller
/// inputs, as in native BuildFromFacts; the file's extra sections are not applied
/// implicitly. No LegacyFilm is constructed and no chunks are read.
///
/// Cache counters and raw names retain their native domains through publication.
pub fn build_facts_replay_players(
    facts: &NativeFilmFacts,
    bots: &[IdentityBot],
    statborg_records: &[StatborgRecord],
    options: FilmReplayPlayerOptions<'_>,
) -> Result<Option<FilmReplayPlayers>, FactsProjectionError> {
    build_facts_replay_players_with_identity(facts, bots, statborg_records, options, None, None)
}
pub(super) fn build_facts_replay_players_with_identity(
    facts: &NativeFilmFacts,
    bots: &[IdentityBot],
    statborg_records: &[StatborgRecord],
    options: FilmReplayPlayerOptions<'_>,
    supplied_identity: Option<&StatborgRoundIdentity>,
    match_id: Option<&str>,
) -> Result<Option<FilmReplayPlayers>, FactsProjectionError> {
    let samples: Vec<_> = facts
        .positions
        .iter()
        .map(ReplayPositionSample::from)
        .collect();
    let Some(timeline) = prepare_replay_timeline(&samples, options.frame_interval_ms) else {
        return Ok(None);
    };
    let creations: Vec<_> = facts
        .biped_creations
        .iter()
        .map(IdentityCreationRecord::from)
        .collect();
    let fire: Vec<_> = facts
        .events
        .fire
        .iter()
        .map(|e| IdentityFireReference {
            film_index: e.film_index,
            timestamp_us: e.timestamp_us,
        })
        .collect();
    let film_table = ReplayFilmPlayerTable::try_from(&facts.queue.film_table)?;
    let teams = FilmPlayerTeams {
        attempts: Vec::new(),
        by_index: facts.queue.player_teams.clone().unwrap_or_default(),
        report: PlayerTeamScanReport::try_from(&facts.queue.team_scan)?,
    };
    let evidence = ReplayIdentityEvidence {
        deaths: facts
            .queue
            .deaths
            .iter()
            .map(IdentityDeath::try_from)
            .collect::<Result<_, _>>()?,
        replication_indices: PlayerIndexTable::try_from(&facts.queue.player_indices)?,
        index_collisions: 0,
        film_clock_us: facts.header.film_clock_origin_us,
        death_error: None,
        index_error: None,
        clock_error: None,
    };
    let positions: Vec<_> = timeline
        .samples
        .iter()
        .map(|s| s.position.clone())
        .collect();
    let scope = ReplayScopeLookup::from_facts(
        &facts.zoom_events,
        &build_identity_life_spans(&positions),
        REPLAY_ZOOM_HOLD_US,
    );
    Ok(Some(
        super::replay_from_film::assemble_replay_player_inputs_with_identity(
            super::replay_from_film::ReplayPlayerAssemblyInput {
                timeline,
                evidence,
                creations: &creations,
                bots,
                fire: &fire,
                film_table: &film_table,
                records: statborg_records,
                teams: &teams,
                scope: &scope,
            },
            options,
            supplied_identity,
            match_id,
        ),
    ))
}
