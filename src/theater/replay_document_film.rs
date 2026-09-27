//! LegacyFilm-to-document assembly. Captured end-to-end parity is still under validation.
use super::*;
use crate::clients::hi::models::FilmChunkData;
use std::collections::BTreeMap;

/// Mode gates are supplied by the match owner, as in the native replay options.
#[derive(Debug, Clone, Copy, Default)]
pub struct ReplayDocumentModes {
    pub flag: bool,
    pub vip: bool,
    pub skull: bool,
    pub bomb: bool,
}
#[derive(Default)]
pub struct FilmReplayDocumentOptions<'a> {
    pub title_slug: &'a str,
    pub map_id: &'a str,
    pub variant: &'a str,
    pub players: FilmReplayPlayerOptions<'a>,
    pub score: Option<FilmReplayScoreOptions<'a>>,
    pub initial_seats: usize,
    pub modes: ReplayDocumentModes,
    pub labels: Option<&'a ReplayLabelCatalog>,
    pub geometry: Vec<ReplayMapObject>,
    pub structure: Vec<ReplaySurface>,
    pub neutral_deaths: Vec<ReplayNeutralDeath>,
    pub equipment_kills: &'a [ReplayEquipmentKill],
    pub kills_read: bool,
    pub match_kills: &'a [ReplayKillReference],
    pub deaths_paths: Option<ReplayDeathsPathsCoverage>,
    pub pickups: FilmReplayPickupOptions<'a>,
    pub zone_teams: BTreeMap<String, i64>,
    /// Upstream decoder counters are merged with counters from document publication.
    pub fallbacks: Vec<ReplayFallbackHit>,
}

/// Statborg observation scope within document assembly, in execution order.
/// This excludes grammar/scanner observations and outer application logging.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilmReplayStatborgStage {
    RetainedSource,
    PlayerIdentityCompletion,
    GeneralObjectives,
    Score,
    FlagAttachment,
    VipAttachment,
}

/// A completed observation stage; an empty list means no warnings, not no execution.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FilmReplayStatborgStageReport {
    pub stage: FilmReplayStatborgStage,
    pub diagnostics: Vec<StatborgDiagnostic>,
}

/// Ordered statborg observations only. Source warnings were retained during LegacyFilm
/// decoding; subsequent stages run during this call. Stages not reached are absent.
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FilmReplayStatborgReport {
    pub stages: Vec<FilmReplayStatborgStageReport>,
}
impl FilmReplayStatborgReport {
    fn push(&mut self, stage: FilmReplayStatborgStage, diagnostics: Vec<StatborgDiagnostic>) {
        self.stages
            .push(FilmReplayStatborgStageReport { stage, diagnostics });
    }
}

/// Build the native replay document with a separate, explicitly scoped statborg
/// report. The report survives errors and does not alter the document JSON schema.
#[allow(dead_code)]
pub(crate) fn build_film_replay_document_with_statborg_report(
    film: &LegacyFilm,
    chunks: &[FilmChunkData],
    options: FilmReplayDocumentOptions<'_>,
) -> (
    Result<ReplayDocument, DecodeError>,
    FilmReplayStatborgReport,
) {
    let mut report = FilmReplayStatborgReport::default();
    let result = build_film_replay_document_observed(film, chunks, options, &mut report);
    (result, report)
}

/// Assemble all LegacyFilm-facing layers on the player publication's shared clock.
/// Original chunks are used only for identity evidence, as by the player builder.
/// The caller supplies match evidence and title labels explicitly; no network I/O occurs.
#[allow(dead_code)]
pub(crate) fn build_film_replay_document(
    film: &LegacyFilm,
    chunks: &[FilmChunkData],
    options: FilmReplayDocumentOptions<'_>,
) -> Result<ReplayDocument, DecodeError> {
    build_film_replay_document_with_statborg_report(film, chunks, options).0
}

fn build_film_replay_document_observed(
    film: &LegacyFilm,
    chunks: &[FilmChunkData],
    options: FilmReplayDocumentOptions<'_>,
    report: &mut FilmReplayStatborgReport,
) -> Result<ReplayDocument, DecodeError> {
    report.push(
        FilmReplayStatborgStage::RetainedSource,
        film.statborg_diagnostics.clone(),
    );
    let FilmReplayDocumentOptions {
        title_slug,
        map_id,
        variant,
        players: player_options,
        score: score_options,
        initial_seats,
        modes,
        labels,
        geometry,
        structure,
        mut neutral_deaths,
        equipment_kills,
        kills_read,
        match_kills,
        deaths_paths,
        pickups: pickup_options,
        zone_teams,
        mut fallbacks,
    } = options;
    let interval = if player_options.frame_interval_ms > 0 {
        player_options.frame_interval_ms
    } else {
        100
    };
    let mut doc = ReplayDocument {
        content: ReplayDocumentContent {
            schema_version: REPLAY_SCHEMA_VERSION,
            match_id: film.match_id.clone().unwrap_or_default(),
            title_slug: title_slug.into(),
            frame_interval_ms: interval,
            geometry_bounds: replay_geometry_bounds(&geometry),
            structure_bounds: replay_surface_bounds(&structure),
            geometry,
            structure,
            ..Default::default()
        },
        coverage: None,
    };
    let Some(players) = build_film_replay_players(film, chunks, player_options)? else {
        doc.publish_layers(ReplayLayerInputs::default());
        return Ok(doc);
    };
    report.push(
        FilmReplayStatborgStage::PlayerIdentityCompletion,
        players.statborg_diagnostics.clone(),
    );
    players.registry.log(
        film.match_id.as_deref().unwrap_or_default(),
        &players.players.identity.coverage,
    );
    let decoded_kills =
        (film.kill_sources.is_some() || film.kill_sources_error.is_some()).then(|| {
            build_replay_decoded_kill_inputs(
                film.kill_sources.as_ref(),
                players
                    .evidence
                    .death_error
                    .is_none()
                    .then_some(players.evidence.deaths.as_slice()),
            )
        });
    let (equipment_kills, match_kills, kills_read, deaths_paths) =
        if let Some(input) = &decoded_kills {
            neutral_deaths = input.neutral_deaths.clone();
            (
                input.equipment.as_slice(),
                input.pairs.as_slice(),
                input.read,
                input.paths.clone(),
            )
        } else {
            (equipment_kills, match_kills, kills_read, deaths_paths)
        };
    let empty_labels = ReplayLabelCatalog::default();
    let labels = labels.unwrap_or(&empty_labels);
    let equipment_catalog = ReplayEquipmentCatalog {
        families: labels.equipment_families.clone(),
        weapon_keys: labels.keys.clone(),
        objective_labels: labels.objective_objects.clone(),
        objective_families: labels.objective_families.clone(),
        objective_objects: labels.objective_objects.keys().copied().collect(),
        spawned_piece_ids: replay_equipment_catalog().spawned_piece_ids,
    };
    let tracks = &players.players.publication.tracks;
    doc.content.frame_count = players.clock.frame_count;
    doc.content.frame_interval_ms = players.frame_interval_ms;
    doc.content.duration_ms = players.duration_ms;
    doc.content.origin_ms = players.origin_ms;
    doc.content.bounds = players.players.bounds.clone();
    doc.content.tracks = Some(tracks.clone());
    doc.content.identity =
        (!players.players.identity.is_empty()).then(|| players.players.identity.clone());
    doc.content.roster = players.players.roster.clone();

    let combat = build_film_replay_combat(film, &players)
        .ok_or(DecodeError::Missing("replay combat clock or positions"))?;
    let ((identified, unnamed, refused), diagnostics) =
        identify_film_replay_objective_events_with_diagnostics(
            film,
            &players,
            variant,
            initial_seats,
        );
    report.push(FilmReplayStatborgStage::GeneralObjectives, diagnostics);
    let mut objectives = build_replay_objective_actions(
        &identified,
        unnamed as i64,
        refused as i64,
        ReplayScoreClock {
            origin_ms: players.origin_ms.unwrap_or(0),
            interval_ms: players.frame_interval_ms,
            frames: players.clock.frame_count,
        },
    );
    objectives.without_track = count_objective_actions_without_track(
        &objectives.actions,
        &players.players.publication.tracks,
        &players.registry.owners.state.naming_bridge(),
    );
    let score_read = score_options.is_some();
    let score = score_options.map(|o| {
        let (score, diagnostics) = build_film_replay_score_with_diagnostics(film, &players, o);
        report.push(FilmReplayStatborgStage::Score, diagnostics);
        score
    });
    let mut equipment = build_film_replay_equipment_episodes(film, &players);
    attach_film_replay_equipment_kills(&mut equipment, &players, equipment_kills, kills_read);
    let stances = build_film_replay_stances(film, &players);
    let mut coverage = ReplayCoverage::new(
        combat.shots.coverage,
        combat
            .grenades
            .as_ref()
            .map(|g| g.coverage.clone())
            .unwrap_or_default(),
        objectives.coverage,
        &players.registry,
        players.origin_ms.is_some(),
        score.as_ref().and_then(|s| s.coverage.clone()),
    );
    coverage.bridge = players.players.bridge.clone();
    coverage.tracks = Some(players.players.publication.coverage.clone());
    coverage.teams = Some(players.players.teams.clone());
    coverage.seats = Some(players.players.seats.clone());
    coverage.stances = Some(stances.coverage.into());
    coverage.equipment = Some(equipment.coverage);
    coverage.film_major_version = Some(i64::from(film.major_version));
    coverage.decoder = Some(build_film_replay_decoder_coverage(film));
    coverage.deaths_paths = deaths_paths;
    if let Some(verdict) = combat.vehicle_shot_verdict {
        coverage.verdict.insert("shots".into(), verdict);
    }
    doc.content.shots = combat.shots.shots;
    doc.content.loadouts = combat.loadouts;
    doc.content.objectives = objectives.actions;
    doc.content.equipment_episodes = equipment.episodes;
    doc.content.stances = stances.stances.into_iter().map(Into::into).collect();
    if let Some(score) = score {
        doc.content.score_timeline = score.timeline;
        fallbacks.push(ReplayFallbackHit {
            name: "repli_manche_zero_decretee".into(),
            hits: score.round_zero_fallbacks as i64,
        });
    }
    if let Some(projectiles) = combat.projectiles {
        if projectiles.coverage.tracks > 0 {
            coverage.projectiles = Some(projectiles.coverage);
        }
        doc.content.projectiles = projectiles.projectiles;
    }
    if let Some(grenades) = combat.grenades {
        doc.content.grenades = grenades.grenades;
    }
    doc.coverage = Some(coverage);
    doc.detect_kickoff();
    let coverage = doc.coverage.as_mut().unwrap();

    if let Some(grapple) = build_film_replay_grapple(film, &players) {
        doc.content.grapple_lines = grapple.lines;
        coverage.grapple = Some(grapple.coverage);
    }
    let placements =
        build_film_replay_equipment_placements_with_catalog(film, &players, &equipment_catalog);
    let pickups = build_film_replay_pickups_with_catalog(
        film,
        &players,
        &placements,
        pickup_options,
        &equipment_catalog,
    );
    let changes = build_film_replay_weapon_changes(film, &players);
    doc.content.weapon_changes = changes.changes;
    coverage.weapon_changes = Some(changes.coverage);
    let ground =
        build_film_replay_ground_with_catalog(film, &players, &pickups.pickups, &equipment_catalog);
    doc.content.pickups = pickups.pickups;
    coverage.pickups = Some(pickups.coverage);
    doc.content.equipment_placements = placements.placements;
    coverage.placements = Some(placements.coverage);
    doc.content.weapon_pads = ground.pads;
    doc.content.pad_pickups = ground.pickups;
    coverage.ground_weapons = Some(ground.coverage);
    coverage.pad_dating = Some(ground.dating);
    doc.content.ground_weapons = ground.items.weapons;
    coverage.ground_weapon_items = Some(ground.items.coverage);
    coverage.vehicles = Some(ReplayVehicleCoverage::default());
    let vehicles_scanned = combat.vehicles.as_ref().is_some_and(|v| v.coverage.scanned);
    let vehicle_rides = combat
        .vehicles
        .as_ref()
        .map(|v| v.rides.clone())
        .unwrap_or_default();
    if let Some(vehicles) = combat.vehicles {
        doc.content.vehicles = vehicles.tracks;
        doc.content.vehicle_cycles = vehicles.cycles;
        coverage.vehicles = Some(vehicles.coverage);
        fallbacks.extend(
            vehicles
                .fallbacks
                .into_iter()
                .map(|(name, hits)| ReplayFallbackHit {
                    name: name.into(),
                    hits: hits as i64,
                }),
        );
    }
    if let Some(vehicles) = &coverage.vehicles {
        vehicles.log();
    }
    super::replay_vehicle_tracks::log_replay_vehicle_heading_source(
        film.native_vehicles
            .as_ref()
            .and_then(|v| v.positions.as_ref())
            .into_iter()
            .flat_map(|p| p.accepted())
            .map(|p| &p.record.companions),
    );
    vehicle_rides.log();
    let selected = replay_match_objectives(map_id, variant);
    let (flags, diagnostics) = build_film_replay_flags_with_catalog_and_diagnostics(
        film,
        &players,
        FilmReplayFlagInput {
            scanned: modes.flag,
            spawns: &selected.flag_spawns,
        },
        &equipment_catalog,
    );
    report.push(FilmReplayStatborgStage::FlagAttachment, diagnostics);
    doc.content.flag_return_zone = labels.flag_return_zone.for_carries(&flags.carries);
    doc.content.flag_carries = flags.carries;
    coverage.flag_carries = flags.coverage;
    let (vip, diagnostics) =
        build_film_replay_vip_crown_with_diagnostics(film, &players, modes.vip);
    report.push(FilmReplayStatborgStage::VipAttachment, diagnostics);
    doc.content.vip_crown = vip.periods;
    coverage.vip_crown = vip.coverage;
    let skull = build_film_replay_skull_carries(film, &players, modes.skull);
    doc.content.skull_carries = skull.carries;
    coverage.skull_carries = skull.coverage;
    let bomb = build_film_replay_bomb(
        film,
        &players,
        FilmReplayBombInput {
            recognized: modes.bomb,
            score_read,
            identified_objectives: &identified,
            published_objectives: &doc.content.objectives,
            kills_read,
            kills: match_kills,
        },
    );
    doc.content.bomb_carries = bomb.carries.publication.carries;
    coverage.bomb_carries = bomb.carries.publication.coverage;
    if let Some(armings) = bomb.armings {
        doc.content.bomb_armings = armings.armings;
        coverage.bomb_armings = Some(armings.coverage);
        fallbacks.push(ReplayFallbackHit {
            name: "repli_armement_bombe_debut_a_zero".into(),
            hits: armings.start_zero_fallbacks as i64,
        });
    }
    if let Some(stats) = bomb.stats {
        doc.content.bomb_stats = Some(stats.stats);
        doc.content.bomb_events = stats.events;
    }
    let objects =
        build_film_replay_objective_objects_with_catalog(film, &players, &equipment_catalog);
    doc.content.objective_objects = objects.lives;
    coverage.objective_objects = Some(objects.coverage);
    let zones = build_film_replay_zones(
        film,
        &players,
        &doc.content.objectives,
        FilmReplayZoneInput {
            zones: &selected.held_zones,
            roles: &selected.held_roles,
            teams: &zone_teams,
            hill: selected.hill,
        },
    );
    doc.content.zone_states = zones.states;
    coverage.zones = zones.coverage;

    doc.content.weapon_labels = build_replay_weapon_labels(
        &doc.content.loadouts,
        &doc.content.shots,
        &doc.content.weapon_pads,
        labels,
    );
    doc.content.kill_effects = labels.effects.clone();
    retain_replay_neutral_deaths(
        &mut neutral_deaths,
        tracks,
        &players.registry.owners.state.naming_bridge(),
    );
    doc.content.neutral_deaths = neutral_deaths;
    if let Some(inventory) = build_film_replay_inventory(film, &players) {
        doc.content.inventory = inventory.reads;
        coverage.inventory = inventory.coverage;
    }
    doc.content.grenade_reads = combat.grenade_reads.reads;
    coverage.grenade_reads = combat.grenade_reads.coverage;
    if !doc.content.inventory.is_empty()
        || !doc.content.grenades.is_empty()
        || !doc.content.grenade_reads.is_empty()
    {
        doc.content.grenade_labels = labels.grenades.clone();
    }
    let abilities = build_film_replay_abilities_with_catalog(
        film,
        &players,
        &ReplayAbilityCatalog {
            palettes: labels.abilities.clone(),
            impulse_families: labels.ability_impulse_families.iter().cloned().collect(),
            charge_families: labels.ability_charge_families.iter().cloned().collect(),
        },
    )
    .ok_or(DecodeError::Missing("replay ability clock"))?;
    abilities.abilities.coverage.log();
    doc.content.abilities = abilities.abilities.reads;
    doc.content.ability_labels = abilities.abilities.labels;
    coverage.abilities = Some(abilities.abilities.coverage);
    doc.content.ability_impulses = abilities.impulses;
    coverage.ability_impulses = abilities.impulse_coverage;
    doc.content.ability_charges = abilities.charges;
    coverage.ability_charges = abilities.charge_coverage;
    let changes = build_film_replay_equipment_changes(film, &players);
    changes.coverage.log();
    doc.content.equipment_changes = changes.changes;
    coverage.equipment_changes = Some(changes.coverage);
    let translocations = build_film_replay_translocations(film, &players);
    translocations.coverage.log();
    doc.content.translocations = translocations.jumps;
    coverage.translocations = Some(translocations.coverage);
    super::replay_abilities::log_replay_ability_palette(
        abilities.abilities.palette.as_ref(),
        doc.content.abilities.len(),
        doc.content.ability_labels.len(),
    );
    super::replay_abilities_film::log_replay_ability_scans(
        coverage.ability_impulses.as_ref(),
        film.ability_states.as_ref().map_or(0, |s| s.impulses.len()),
        coverage.ability_charges.as_ref(),
        film.ability_charges.as_ref().map_or(0, |s| s.records.len()),
    );
    fallbacks.push(player_replication_gap_fallback(
        &players.registry.owners,
        &players.players.publication,
    ));
    for (name, hits) in [
        (
            "repli_plafond_grenade_par_defaut",
            usize::from(
                film.keyframe_inventory
                    .as_ref()
                    .is_some_and(|i| i.default_grenade_max),
            ),
        ),
        (
            "repli_identite_piste_meilleur_recouvrement",
            players.players.overlap_fallbacks,
        ),
        (
            "repli_piece_engendree_sans_evenement",
            placements.manifest_fallbacks,
        ),
        (
            "repli_siege_du_remplacant_par_appariement_ordinal",
            players.players.seats.apparies,
        ),
        (
            "repli_piste_drapeau_sans_pont_ecartee",
            flags.tracks_without_bridge,
        ),
        (
            "repli_position_lacher_prend_la_prise",
            flags.drops_using_pickup,
        ),
        (
            "repli_zone_camp_de_capture_deduit_de_l_issue",
            zones.inferred_capture_teams,
        ),
        (
            "repli_colline_votes_periode_entiere",
            zones.hill_fallbacks.whole_period_votes,
        ),
        (
            "repli_colline_dernier_intervalle_ouvert",
            zones.hill_fallbacks.open_owner_tail,
        ),
    ] {
        fallbacks.push(ReplayFallbackHit {
            name: name.into(),
            hits: hits as i64,
        });
    }
    if let Some(precision) = &film.scan_precision {
        fallbacks.extend(precision.fallbacks.iter().map(|f| ReplayFallbackHit {
            name: f.name.clone(),
            hits: f.hits,
        }));
    }
    doc.set_fallbacks(fallbacks);
    doc.publish_layers(ReplayLayerInputs {
        map_quant: film.profile.as_ref().is_some_and(|p| p.map.is_some()),
        inventory: film.keyframe_inventory.is_some() && film.keyframe_inventory_error.is_none(),
        ability_impulses_scanned: film
            .ability_states
            .as_ref()
            .is_some_and(|s| s.impulse_stats.scanned),
        ability_charges_scanned: film
            .ability_charges
            .as_ref()
            .is_some_and(|s| s.stats.scanned),
        score: score_read,
        flag_scanned: modes.flag,
        vip_scanned: modes.vip,
        skull_scanned: modes.skull,
        bomb_scanned: modes.bomb,
        bomb_carry_scanned: modes.bomb,
        zone_scanned: !selected.held_zones.is_empty(),
        vehicles_scanned,
    });
    Ok(doc)
}

// Both native trigger sites share the same per-build counter: identity life
// refinement and the unnamed-track fallback in bornesDesVies.
fn player_replication_gap_fallback(
    owners: &IdentityOwnerOutput,
    tracks: &ReplayTrackPublication,
) -> ReplayFallbackHit {
    ReplayFallbackHit {
        name: "repli_vie_coupee_au_trou_de_replication".into(),
        hits: (owners.replication_gap_fallbacks as i64)
            .wrapping_add(tracks.replication_gap_fallbacks as i64),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clients::hi::models::FilmChunk;
    use std::{fs, io::Read, path::Path};
    #[test]
    fn native_identity_fallbacks_reach_document_coverage() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/identity-owners-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        let mut positive = 0;
        let mut total = 0;
        for row in rows {
            // These are independently generated native owner results, including
            // the live native fallback counter after identity refinement.
            let owners: IdentityOwnerOutput =
                serde_json::from_value(row["output"].clone()).unwrap();
            let expected = row["output"]["replication_gap_fallbacks"].as_u64().unwrap() as usize;
            let publication = ReplayTrackPublication {
                tracks: vec![],
                coverage: Default::default(),
                replication_gap_fallbacks: 0,
            };
            let mut coverage = ReplayCoverage::default();
            coverage.set_fallbacks([player_replication_gap_fallback(&owners, &publication)]);
            let expected_report = if expected == 0 {
                vec![]
            } else {
                vec![ReplayFallbackHit {
                    name: "repli_vie_coupee_au_trou_de_replication".into(),
                    hits: expected as i64,
                }]
            };
            assert_eq!(coverage.fallbacks, expected_report);
            positive += usize::from(expected > 0);
            total += expected;
        }
        assert_eq!((positive, total), (823, 4164));
    }
    #[test]
    fn statborg_report_retains_source_on_early_exit() {
        let mut data = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/bootstrap-v41.zlib")[..])
            .read_to_end(&mut data)
            .unwrap();
        let mut chunks = vec![FilmChunkData {
            metadata: FilmChunk {
                index: 0,
                chunk_type: 1,
                start_time_offset_ms: 0,
                duration_ms: 0,
                size: data.len() as i64,
                file_relative_path: String::new(),
            },
            data,
        }];
        let mut payload = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/captured-keyframe-v41.zlib")[..])
            .read_to_end(&mut payload)
            .unwrap();
        let mut packet = vec![2, 0, 0, 0];
        packet.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        packet.extend_from_slice(&1000u64.to_le_bytes());
        packet.extend(payload);
        chunks.push(FilmChunkData {
            metadata: FilmChunk {
                index: 1,
                chunk_type: 2,
                start_time_offset_ms: 0,
                duration_ms: 1,
                size: packet.len() as i64,
                file_relative_path: String::new(),
            },
            data: packet,
        });
        let mut film = LegacyFilm::try_from_chunks(&chunks, DecodeOptions::v41()).unwrap();
        film.statborg_diagnostics.push(StatborgDiagnostic {
            level: "WARN".into(),
            message: "retained source sentinel".into(),
            attributes: vec![],
        });
        for major in [41, 75] {
            film.major_version = major;
            let (result, report) = build_film_replay_document_with_statborg_report(
                &film,
                &chunks,
                FilmReplayDocumentOptions::default(),
            );
            assert!(result.is_err());
            assert_eq!(report.stages.len(), 1);
            assert_eq!(
                report.stages[0].stage,
                FilmReplayStatborgStage::RetainedSource
            );
            assert_eq!(report.stages[0].diagnostics, film.statborg_diagnostics);
            let round_trip: FilmReplayStatborgReport =
                serde_json::from_value(serde_json::to_value(&report).unwrap()).unwrap();
            assert_eq!(round_trip, report);
        }
        film.major_version = 41;
        film.biped_positions = Some(BipedPositionStream {
            record_masks: vec![],
            slot_band: None,
            options: Default::default(),
            candidates: vec![],
        });
        let (result, report) = build_film_replay_document_with_statborg_report(
            &film,
            &chunks,
            FilmReplayDocumentOptions::default(),
        );
        assert!(result.is_ok());
        assert_eq!(
            report.stages.len(),
            1,
            "no player timeline means no downstream stages"
        );
    }

    #[test]
    #[ignore = "requires six downloaded films; compares complete native documents"]
    fn local_complete_film_documents() {
        compare_complete_film_documents(
            include_bytes!("fixtures/full-document-v41.json.zlib"),
            false,
        );
    }
    #[test]
    #[ignore = "complete embedded v41 CTF film; takes several minutes"]
    fn complete_ctf_document() {
        compare_complete_film_documents(
            include_bytes!("fixtures/ctf-document-v41.json.zlib"),
            false,
        );
    }

    #[test]
    #[ignore = "complete embedded v41 CTF film with decoded kills; takes several minutes"]
    fn complete_ctf_decoded_kill_document() {
        compare_complete_film_documents(
            include_bytes!("fixtures/ctf-decoded-kill-document-v41.json.zlib"),
            true,
        );
    }

    #[test]
    #[ignore = "complete embedded v41 CTF film with catalog flag geometry"]
    fn complete_ctf_geometry_document() {
        compare_complete_film_documents(
            include_bytes!("fixtures/ctf-geometry-document-v41.json.zlib"),
            true,
        );
    }

    #[test]
    #[ignore = "requires six downloaded films; integrated kill decoder and positive Oddball mode"]
    fn local_decoded_kill_documents() {
        compare_complete_film_documents(
            include_bytes!("fixtures/decoded-kill-document-v41.json.zlib"),
            true,
        );
    }
    #[test]
    #[ignore = "requires captured Bazaar film; native zero-axis fallback document"]
    fn local_zero_axis_document() {
        compare_complete_film_documents(
            include_bytes!("fixtures/zero-axis-document-v41.json.zlib"),
            false,
        );
    }
    #[test]
    #[ignore = "requires captured Bazaar film; native missing-identification document"]
    fn local_missing_identity_document() {
        compare_complete_film_documents(
            include_bytes!("fixtures/missing-identity-document-v41.json.zlib"),
            false,
        );
    }
    #[test]
    #[ignore = "requires captured Bazaar film; native unknown-format documents"]
    fn local_unknown_format_documents() {
        compare_complete_film_documents(
            include_bytes!("fixtures/unknown-format-document-v41.json.zlib"),
            false,
        );
    }
    #[test]
    #[ignore = "requires the hour-long raid and full-document oracle; generate with --include-raid"]
    fn local_complete_raid_document() {
        let path = std::env::var_os("HALO_FULL_RAID_DOCUMENT_ORACLE")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("src/theater/fixtures/full-raid-document-v41.json.zlib")
            });
        let bytes = fs::read(path).expect("generate full raid oracle with --include-raid");
        compare_complete_film_documents(&bytes, false);
    }
    fn compare_complete_film_documents(fixture: &[u8], integrated: bool) {
        let mut bytes = Vec::new();
        flate2::read::ZlibDecoder::new(fixture)
            .read_to_end(&mut bytes)
            .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&bytes).unwrap();
        let mut failures = Vec::new();
        for row in rows {
            let folder = row["folder"].as_str().unwrap();
            let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("experiments/films")
                .join(folder);
            let captured = row.get("captured_chunks").and_then(|v| v.as_array());
            let meta: serde_json::Value = if let Some(captured) = captured {
                serde_json::json!({"chunks":captured.iter().map(|c|c["metadata"].clone()).collect::<Vec<_>>()})
            } else {
                serde_json::from_slice(&fs::read(dir.join("film.json")).unwrap()).unwrap()
            };
            let mut chunks: Vec<_> = meta["chunks"]
                .as_array()
                .unwrap()
                .iter()
                .map(|c| FilmChunkData {
                    metadata: FilmChunk {
                        index: c["index"].as_i64().unwrap() as i32,
                        chunk_type: c["chunk_type"].as_i64().unwrap() as i32,
                        start_time_offset_ms: c["start_time_offset_ms"].as_i64().unwrap(),
                        duration_ms: c["duration_ms"].as_i64().unwrap(),
                        size: 0,
                        file_relative_path: String::new(),
                    },
                    data: if let Some(captured) = captured {
                        let source = captured
                            .iter()
                            .find(|s| s["metadata"]["index"] == c["index"])
                            .unwrap();
                        let hex = source["hex"].as_str().unwrap();
                        (0..hex.len())
                            .step_by(2)
                            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                            .collect()
                    } else {
                        fs::read(dir.join(c["file"].as_str().unwrap())).unwrap()
                    },
                })
                .collect();
            if row["missing_identity"] == true {
                let chunk = chunks
                    .iter_mut()
                    .find(|c| c.metadata.chunk_type == 1)
                    .unwrap();
                let registry = parse_registry(&chunk.data).unwrap();
                let identity = decode_film_identity(&chunk.data, &registry)
                    .unwrap()
                    .unwrap();
                chunk.data[identity.build_offset..identity.build_offset + 3].fill(0);
            }
            if let Some(format) = row["format_override"].as_u64() {
                let chunk = chunks
                    .iter_mut()
                    .find(|c| c.metadata.chunk_type == 1)
                    .unwrap();
                chunk.data[4..8].copy_from_slice(&(format as u32).to_le_bytes());
            }
            let decode_options = DecodeOptions {
                match_id: Some(folder.into()),
                ..DecodeOptions::v41()
            };
            let film = if integrated {
                LegacyFilm::try_from_chunks_with_map_and_kill_sources(
                    &chunks,
                    decode_options,
                    row["map"].as_str().unwrap(),
                    None,
                    RecordIdLayout {
                        low_bits: 13,
                        base: 0,
                    },
                    &KillDecodeOptions::default(),
                )
            } else if let Some(bounds) = row.get("map_bounds") {
                LegacyFilm::try_from_chunks_with_map_bounds(
                    &chunks,
                    decode_options,
                    row["map"].as_str().unwrap(),
                    serde_json::from_value(bounds.clone()).unwrap(),
                    RecordIdLayout {
                        low_bits: 13,
                        base: 0,
                    },
                )
            } else {
                LegacyFilm::try_from_chunks_with_map(
                    &chunks,
                    decode_options,
                    row["map"].as_str().unwrap(),
                    RecordIdLayout {
                        low_bits: 13,
                        base: 0,
                    },
                )
            }
            .unwrap();
            if let Some(name) = row.get("inputs_fixture").and_then(|v| v.as_str()) {
                let path = Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("src/theater/fixtures")
                    .join(name);
                let inputs = super::super::film_input_retention_tests::read_selected_inputs(&path);
                super::super::film_input_retention_tests::assert_native_scan_inputs(
                    &film,
                    &inputs,
                    folder,
                    row["flag"].as_bool().unwrap_or(false),
                );
            }
            if let Some(name) = row.get("objective_inputs_fixture").and_then(|v| v.as_str()) {
                let path = Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("src/theater/fixtures")
                    .join(name);
                let reader = std::io::BufReader::new(flate2::read::ZlibDecoder::new(
                    fs::File::open(path).unwrap(),
                ));
                let inputs: serde_json::Value = serde_json::from_reader(reader).unwrap();
                let input = inputs
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|r| r["folder"] == folder)
                    .unwrap();
                super::super::film_input_retention_tests::assert_objective_variants(&film, input);
            }
            if let Some(name) = row.get("vehicle_inputs_fixture").and_then(|v| v.as_str()) {
                let path = Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("src/theater/fixtures")
                    .join(name);
                let reader = std::io::BufReader::new(flate2::read::ZlibDecoder::new(
                    fs::File::open(path).unwrap(),
                ));
                let inputs: serde_json::Value = serde_json::from_reader(reader).unwrap();
                assert_eq!(inputs[0]["folder"], folder);
                super::super::vehicle_input_retention_tests::assert_native_vehicle_inputs(
                    &film,
                    inputs[0]["Vehicles"].clone(),
                );
            }
            if integrated {
                assert_eq!(
                    film.kill_sources.is_some(),
                    row["kills_read"].as_bool().unwrap(),
                    "{folder}"
                );
                assert_ne!(
                    film.kill_sources.is_some(),
                    film.kill_sources_error.is_some(),
                    "decode attempt is retained"
                );
            }
            if let Some(expected) = row.get("march") {
                native_march::tests::assert_native_march_facts(
                    film.native_march_facts.as_ref().unwrap(),
                    expected,
                    folder,
                );
            }
            if let Some(expected) = row.get("kill_result").filter(|v| !v.is_null()) {
                super::super::kill_decode::tests::assert_native_result(
                    film.kill_sources.as_ref().unwrap(),
                    expected,
                    row["publishable"].as_bool().unwrap(),
                    folder,
                );
            }
            let ability = replay_ability_catalog();
            let equipment = replay_equipment_catalog();
            let labels = ReplayLabelCatalog {
                abilities: ability.palettes,
                ability_impulse_families: ability.impulse_families.into_iter().collect(),
                ability_charge_families: ability.charge_families.into_iter().collect(),
                equipment_families: equipment.families,
                keys: equipment.weapon_keys,
                objective_objects: equipment.objective_labels,
                objective_families: equipment.objective_families,
                ..Default::default()
            };
            let lines: Vec<StatborgPlayerLine> = row
                .get("lines")
                .filter(|v| !v.is_null())
                .map(|v| serde_json::from_value(v.clone()).unwrap())
                .unwrap_or_default();
            let (doc, report) = build_film_replay_document_with_statborg_report(
                &film,
                &chunks,
                FilmReplayDocumentOptions {
                    title_slug: "halo-infinite",
                    map_id: row["map_id"].as_str().unwrap_or(""),
                    modes: ReplayDocumentModes {
                        flag: row["flag"].as_bool().unwrap_or(false),
                        skull: row["skull"].as_bool().unwrap_or(false),
                        ..Default::default()
                    },
                    players: FilmReplayPlayerOptions {
                        statborg_lines: &lines,
                        ..Default::default()
                    },
                    labels: Some(&labels),
                    score: Some(FilmReplayScoreOptions {
                        lines: &lines,
                        team_by_xuid: BTreeMap::new(),
                        team_scores: None,
                        target_score: 0,
                        hold_ticks_per_point: 0,
                    }),
                    ..Default::default()
                },
            );
            let doc = doc.unwrap();
            // The native Aquarius golden has no timeline: the builder returns
            // its empty document before downstream statborg stages. Derive this
            // expectation from the independent oracle, not the Rust result.
            let expected_stages = if row["document"]["frameCount"] == 0 {
                vec![FilmReplayStatborgStage::RetainedSource]
            } else {
                vec![
                    FilmReplayStatborgStage::RetainedSource,
                    FilmReplayStatborgStage::PlayerIdentityCompletion,
                    FilmReplayStatborgStage::GeneralObjectives,
                    FilmReplayStatborgStage::Score,
                    FilmReplayStatborgStage::FlagAttachment,
                    FilmReplayStatborgStage::VipAttachment,
                ]
            };
            assert_eq!(
                report.stages.iter().map(|s| s.stage).collect::<Vec<_>>(),
                expected_stages,
                "{folder}"
            );
            assert_eq!(report.stages[0].diagnostics, film.statborg_diagnostics);
            let expected: ReplayDocument = serde_json::from_value(row["document"].clone()).unwrap();
            let actual = serde_json::to_value(doc).unwrap();
            if let Some(facts) = row.get("independent_match_facts") {
                // Team scores are withheld from both builders: they are an independent
                // aggregate check, not an input used to generate these captures.
                let captures: i64 = facts["teamScores"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_i64().unwrap())
                    .sum();
                assert!(captures > 0);
                if row.get("map_catalog_sha256").is_some() {
                    assert_eq!(actual["coverage"]["flagCarries"]["spawns"], 2);
                    assert_eq!(actual["coverage"]["flagCarries"]["teamBirths"], 4);
                    let carries = actual["flagCarries"].as_array().unwrap();
                    assert_eq!(carries.len(), 2);
                    assert!(carries.iter().all(|c| c["team"].as_i64().unwrap() >= 0));
                    assert!(carries.iter().any(|c| {
                        c["spans"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .any(|s| s["state"] == "home")
                    }));
                }
                if integrated {
                    let kills: i64 = facts["players"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|p| p["kills"].as_i64().unwrap())
                        .sum();
                    let paths = &actual["coverage"]["deathsPaths"];
                    assert_eq!(
                        paths["walk"]["published"].as_i64().unwrap()
                            + paths["directScan"]["published"].as_i64().unwrap(),
                        kills
                    );
                    assert_eq!(
                        film.kill_sources.as_ref().unwrap().attribution.kills.len() as i64,
                        kills
                    );
                }
                assert_eq!(actual["coverage"]["flagCarries"]["flagFilm"], true);
                assert_eq!(actual["coverage"]["flagCarries"]["captures"], captures);
                assert!(
                    actual["coverage"]["flagCarries"]["carries"]
                        .as_u64()
                        .unwrap()
                        > 0
                );
                assert_eq!(chunks.len(), 8);
                assert_eq!(film.major_version, 41);
            }
            let expected = serde_json::to_value(expected).unwrap();
            let mut mismatches = Vec::new();
            for key in actual
                .as_object()
                .unwrap()
                .keys()
                .chain(expected.as_object().unwrap().keys())
            {
                if actual.get(key) != expected.get(key) && !mismatches.contains(key) {
                    mismatches.push(key.clone());
                }
            }
            if !mismatches.is_empty() {
                fs::write(
                    format!(
                        "/private/tmp/halo-full-document-{}-{}-actual.json",
                        if integrated {
                            "decoded-kills"
                        } else {
                            "baseline"
                        },
                        folder.replace('/', "-")
                    ),
                    serde_json::to_vec_pretty(&actual).unwrap(),
                )
                .unwrap();
                fs::write(
                    format!(
                        "/private/tmp/halo-full-document-{}-{}-expected.json",
                        if integrated {
                            "decoded-kills"
                        } else {
                            "baseline"
                        },
                        folder.replace('/', "-")
                    ),
                    serde_json::to_vec_pretty(&expected).unwrap(),
                )
                .unwrap();
                failures.push(format!("{folder}: different top-level fields {mismatches:?}; per-film comparison JSON in /private/tmp/halo-full-document-*"));
            } else {
                println!("complete document matches {folder}");
            }
        }
        assert!(failures.is_empty(), "{}", failures.join("\n"));
    }
}
