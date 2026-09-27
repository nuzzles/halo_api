//! Cached replay document composition in the pinned native assembly order.
use super::replay_diagnostic_sink::emit_replay_diagnostic;
use super::*;
use std::{collections::BTreeSet, num::NonZeroU64};

/// Assemble a decoded facts file without source reads or freshness checks.
/// Caller Statborg/Kills inputs are explicit; the file's extra sections are not
/// applied. Recorded inputs override their native option counterparts by design.
///
/// Full parity validation is ongoing. The existing player DTO bridge still
/// reports explicit errors for unsupported raw-string and host-counter domains.
/// Decoded cache inventories are present even at length zero, matching the
/// native binary decoder; this API does not represent a pre-encoding nil slice.
pub fn build_facts_replay_document(
    match_id: &str,
    title_slug: &str,
    file: &NativeFilmFactsFile,
    options: FactsReplayDocumentOptions<'_>,
) -> Result<ReplayDocument, FactsProjectionError> {
    let local_counter = FallbackCounter::default();
    let counter = options.fallbacks.unwrap_or(&local_counter);
    let decoder_identity = restore_facts_replay_provenance(file, counter);
    let facts = &file.facts;
    let guards = &file.mode_guards;
    let interval = if options.frame_interval_ms > 0 {
        options.frame_interval_ms
    } else {
        100
    };
    let layers = ReplayLayerInputs {
        map_quant: options.map.is_some(),
        inventory: true,
        ability_impulses_scanned: facts.abilities.impulse_stats.scanned,
        ability_charges_scanned: facts.abilities.charge_stats.scanned,
        score: options.score.is_some(),
        flag_scanned: options.flag.scanned,
        vip_scanned: options.vip.scanned,
        skull_scanned: options.skull.scanned,
        bomb_scanned: options.bomb_arming_scanned,
        bomb_carry_scanned: options.bomb_carry_scanned,
        zone_scanned: guards.zone_scanned,
        vehicles_scanned: facts.vehicles.scanned,
    };
    let mut doc = ReplayDocument {
        content: ReplayDocumentContent {
            schema_version: REPLAY_SCHEMA_VERSION,
            match_id: match_id.into(),
            title_slug: title_slug.into(),
            frame_interval_ms: interval,
            geometry_bounds: replay_geometry_bounds(&options.geometry),
            structure_bounds: replay_surface_bounds(&options.structure),
            geometry: options.geometry,
            structure: options.structure,
            ..Default::default()
        },
        ..Default::default()
    };
    if facts.positions.is_empty() {
        doc.publish_layers(layers);
        return Ok(doc);
    }
    let step = NonZeroU64::new((interval as u64).wrapping_mul(1000))
        .expect("native nonempty timeline requires nonzero wrapped step");
    let min_points = options.min_points.max(1);
    let unresolved = StatborgRoundIdentity {
        publication: IdentityStatborgPublication::default(),
        starts: Vec::new(),
    };
    let records = options.score.as_ref().map_or(&[][..], |s| s.teams.records);
    let players = super::facts_player_inputs::build_facts_replay_players_with_identity(
        facts,
        options.bots,
        records,
        FilmReplayPlayerOptions {
            frame_interval_ms: interval,
            min_points,
            roster_xuids: options.roster_xuids,
            participants: options.participants,
            successions: options.successions,
            scoreboard_teams: options.scoreboard_teams.clone(),
            statborg_lines: &[],
        },
        Some(options.statborg_identity.unwrap_or(&unresolved)),
        Some(match_id),
    )?
    .expect("nonempty positions and nonzero step have a timeline");
    let clock = players.clock;
    let state = &players.registry.owners.state;
    let mut sorted = facts.positions.clone();
    sorted.sort_by_key(|p| p.timestamp_us);
    let positions: Vec<_> = sorted
        .iter()
        .map(|p| ReplayPositionSample::from(p).position)
        .collect();
    doc.content.frame_count = clock.frame_count;
    doc.content.duration_ms = players.duration_ms;
    doc.content.origin_ms = players.origin_ms;
    doc.content.bounds = players.players.bounds.clone();
    doc.content.tracks = Some(players.players.publication.tracks.clone());
    doc.content.identity =
        (!players.players.identity.is_empty()).then(|| players.players.identity.clone());
    doc.content.roster = players.players.roster.clone();
    for (name, hits) in [
        (
            "repli_vie_coupee_au_trou_de_replication",
            (players.registry.owners.replication_gap_fallbacks as i64)
                .wrapping_add(players.players.publication.replication_gap_fallbacks as i64),
        ),
        (
            "repli_identite_piste_meilleur_recouvrement",
            players.players.overlap_fallbacks as i64,
        ),
        (
            "repli_siege_du_remplacant_par_appariement_ordinal",
            players.players.seats.apparies as i64,
        ),
    ] {
        counter.trigger_n(name, hits);
    }
    players
        .registry
        .log(match_id, &players.players.identity.coverage);
    let combat = assemble_facts_replay_combat(
        &mut doc,
        &facts.events,
        FactsReplayCombatContext {
            positions: &positions,
            owners: state.indices_by_slot(),
            origin_us: clock.origin_us,
            step_us: step,
        },
    );
    let teams = FilmPlayerTeams {
        attempts: Vec::new(),
        by_index: facts.queue.player_teams.clone().unwrap_or_default(),
        report: PlayerTeamScanReport::try_from(&facts.queue.team_scan)?,
    };
    let team_publication =
        ReplayTeamPublication::new(&players.registry, &teams, &options.scoreboard_teams);
    let deaths: Vec<_> = players
        .evidence
        .deaths
        .iter()
        .map(|d| StatborgDeathInstant {
            xuid: d.xuid.to_string(),
            time_ms: d.time_ms,
        })
        .collect();
    let score = attach_replay_score_and_objectives_to_document(
        &mut doc,
        ReplayScoreDocumentInput {
            score: options.score,
            deaths: &deaths,
            objectives: options.objectives,
            objectives_unnamed: options.objectives_unnamed,
            objectives_refused: options.objectives_refused,
        },
        ReplayScoreDocumentContext {
            interval_ms: interval,
            teams: &team_publication,
            track_counts: [
                players.players.teams.tracks,
                players.players.teams.tracks_named,
                players.players.teams.tracks_slot_ambiguous,
            ],
            slot_xuids: &state.naming_bridge(),
        },
    );
    score.diagnostics.iter().for_each(emit_replay_diagnostic);
    counter.trigger_n(
        "repli_manche_zero_decretee",
        score.round_zero_fallbacks as i64,
    );
    let equipment = assemble_facts_replay_equipment(
        &mut doc,
        FactsReplayEquipmentInput {
            positions: &sorted,
            camo: &facts.delta_channels.camo_states,
            movement: &facts.movement.states,
            movement_stats: facts.movement.stats.clone(),
            kills: options.equipment_kills,
            kills_read: options.equipment_kills_read,
        },
        FactsReplayEquipmentContext {
            identity: state,
            clock,
            interval_ms: interval,
        },
    );
    equipment
        .diagnostics
        .iter()
        .for_each(emit_replay_diagnostic);
    assemble_replay_document_coverage(
        &mut doc,
        ReplayCoverageDocumentInput {
            shots: combat.shots,
            grenades: combat.grenades.clone(),
            objectives: score.objectives,
            score: score.score,
            projectiles: combat.projectiles,
            tracks: players.players.publication.coverage.clone(),
            teams: score.teams,
            stances: equipment.stances,
            seats: players.players.seats.clone(),
            film_major_version: facts.header.film_major_version,
            kills_read: equipment.kills_read,
            deaths_paths: options.deaths_paths,
        },
        ReplayCoverageDocumentContext {
            registry: &players.registry,
            naming: &players.players.naming,
            closed_by_death: &equipment.closed_by_death,
            decoder_identity: decoder_identity.as_ref(),
            interval_ms: interval,
        },
    )
    .iter()
    .for_each(emit_replay_diagnostic);
    let default_labels = ReplayLabelCatalog::default();
    let labels = options.labels.unwrap_or(&default_labels);
    let catalog = ReplayEquipmentCatalog {
        families: labels.equipment_families.clone(),
        weapon_keys: labels.keys.clone(),
        objective_labels: labels.objective_objects.clone(),
        objective_families: labels.objective_families.clone(),
        objective_objects: labels.objective_objects.keys().copied().collect(),
        spawned_piece_ids: replay_equipment_catalog().spawned_piece_ids,
    };
    let placements = assemble_facts_replay_grapple_and_placements(
        &mut doc,
        FactsReplayPlacementDocumentInput {
            grapple: &facts.delta_channels.grapple_reads,
            map: options.map,
            origin_us: clock.origin_us,
            step_us: clock.step_us,
            placements: FactsReplayEquipmentPlacementInput {
                raw: &facts.world.placements,
                stats: &facts.world.placement_stats,
                positions: &sorted,
                census: &facts.world.powerups.keyframes,
                spawns: &facts.world.spawn_events,
                spawn_lists: facts.world.spawn_stats.lists,
                lives: state.lives(),
                changes: &facts.equipment_changes,
                families: &catalog.families,
                spawned_piece_ids: &catalog.spawned_piece_ids,
            },
        },
    );
    placements
        .diagnostics
        .iter()
        .for_each(emit_replay_diagnostic);
    counter.trigger_n(
        "repli_piece_engendree_sans_evenement",
        placements.manifest_fallbacks as i64,
    );
    let pickups = assemble_facts_replay_pickups_and_pads(
        &mut doc,
        FactsReplayPickupDocumentInput {
            weapon_changes: &facts.weapon_changes,
            pickups: &facts.pickups,
            pickup_stats: &facts.pickup_stats,
            positions: &facts.positions,
            sorted_positions: &sorted,
            weapons: &facts.world.weapons,
            powerups: &facts.world.powerups,
            spawn_points: options.spawn_points,
            spawn_points_state: options.spawn_points_state,
            catalog: &catalog,
            identity: state,
            origin_us: clock.origin_us,
            step_us: clock.step_us,
        },
    );
    pickups.diagnostics.iter().for_each(emit_replay_diagnostic);
    for (name, hits) in assemble_facts_replay_ground_and_vehicles(
        &mut doc,
        FactsReplayGroundVehicleDocumentInput {
            weapon_objects: &pickups.weapon_objects,
            weapon_changes: &facts.weapon_changes,
            sorted_positions: &sorted,
            vehicles: &facts.vehicles,
            identity: state,
            shot_orphans: &combat.shot_orphans,
            origin_us: clock.origin_us,
            step_us: clock.step_us,
        },
    ) {
        counter.trigger_n(name, hits as i64);
    }
    let live = assemble_facts_replay_live_objectives(
        &mut doc,
        file,
        FactsReplayLiveObjectiveOptions {
            flag: FilmReplayFlagInput {
                scanned: options.flag.scanned,
                spawns: options.flag.spawns,
            },
            flag_records: options.flag.records,
            flag_bursts: options.flag.bursts,
            flag_identity: options.flag.identity.unwrap_or(&unresolved),
            flag_return_zone: labels.flag_return_zone,
            vip_scanned: options.vip.scanned,
            vip_records: options.vip.records,
            skull_scanned: options.skull.scanned,
            skull_records: options.skull.records,
            skull_identity: options.skull.identity.unwrap_or(&unresolved),
            bomb_carry_scanned: options.bomb_carry_scanned,
            bomb_arming_scanned: options.bomb_arming_scanned,
            score_read: layers.score,
            identified_objectives: options.objectives,
            kills_read: options.match_kills_read,
            kills: options.match_kills,
            kills_dropped: options.match_kills_dropped,
            zone: FilmReplayZoneInput {
                zones: options.zone.zones,
                roles: options.zone.roles,
                teams: &options.zone.teams,
                hill: options.zone.hill,
            },
            catalog: &catalog,
        },
        FactsReplayLiveObjectiveContext {
            state,
            clock: ReplayMatchClock {
                origin_us: clock.origin_us,
                step_us: clock.step_us,
                frames: clock.frame_count,
                death_offset_ms: players.registry.owners.clock.offset_ms,
            },
            score_clock: score.clock,
            flag_teams: &players.players.flag_carrier_teams,
            deduced_tracks: &players.players.naming.deduced,
        },
    );
    for hit in live.fallbacks {
        counter.trigger_n(hit.name, hit.hits);
    }
    assemble_facts_replay_labels_and_inventory(
        &mut doc,
        FactsReplayInventoryDocumentInput {
            labels,
            neutral_deaths: options.neutral_deaths,
            inventory: Some(&facts.inventory.inventory),
            deltas: &facts.inventory.deltas,
            ammo_refused: facts.header.inventory_delta_ammo_refused,
            deaths: &players.evidence.deaths,
            identity: state,
            death_offset_ms: players.registry.owners.clock.offset_ms,
            origin_us: clock.origin_us,
            step_us: step,
        },
    );
    let palette = assemble_facts_replay_abilities_and_translocations(
        &mut doc,
        FactsReplayAbilityDocumentInput {
            labels,
            ranks: &facts.delta_channels.ability_ranks,
            inventory: &facts.inventory.inventory,
            equipment_changes: &facts.equipment_changes,
            equipment_stats: &facts.equipment_change_stats,
            translocations: &facts.delta_channels.translocations,
            origin_us: clock.origin_us,
            step_us: step,
        },
    );
    assemble_facts_replay_ability_actions(
        &mut doc,
        FactsReplayAbilityActionDocumentInput {
            ranks: &facts.delta_channels.ability_ranks,
            lives: state.lives(),
            palette: palette.as_ref(),
            impulse_families: &labels
                .ability_impulse_families
                .iter()
                .cloned()
                .collect::<BTreeSet<_>>(),
            charge_families: &labels
                .ability_charge_families
                .iter()
                .cloned()
                .collect::<BTreeSet<_>>(),
            impulses: &facts.abilities.impulses,
            impulse_stats: &facts.abilities.impulse_stats,
            charges: &facts.abilities.charges,
            charge_stats: &facts.abilities.charge_stats,
            origin_us: clock.origin_us,
            step_us: clock.step_us,
        },
    );
    finalize_replay_document(&mut doc, &combat.grenades, Some(counter), layers);
    Ok(doc)
}
