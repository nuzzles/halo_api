//! Live-objective document pass over cached inputs, in native assembly order.
//! This is one assembly stage, not the complete BuildFromFacts entry point.
use super::facts_live_objective_logs::*;
use super::*;
use std::collections::{BTreeMap, BTreeSet};

pub struct FactsReplayLiveObjectiveOptions<'a> {
    pub flag: FilmReplayFlagInput<'a>,
    pub flag_records: &'a [StatborgRecord],
    pub flag_bursts: &'a [i64],
    pub flag_identity: &'a StatborgRoundIdentity,
    pub flag_return_zone: ReplayFlagReturnZone,
    pub vip_scanned: bool,
    pub vip_records: &'a [StatborgRecord],
    pub skull_scanned: bool,
    pub skull_records: &'a [StatborgRecord],
    pub skull_identity: &'a StatborgRoundIdentity,
    pub bomb_carry_scanned: bool,
    pub bomb_arming_scanned: bool,
    pub score_read: bool,
    pub identified_objectives: &'a [StatborgIdentifiedEvent],
    pub kills_read: bool,
    pub kills: &'a [ReplayKillReference],
    pub kills_dropped: i64,
    pub zone: FilmReplayZoneInput<'a>,
    pub catalog: &'a ReplayEquipmentCatalog,
}

pub struct FactsReplayLiveObjectiveContext<'a> {
    pub state: &'a ReplayIdentityState,
    pub clock: ReplayMatchClock,
    pub score_clock: ReplayScoreClock,
    pub flag_teams: &'a BTreeMap<String, i64>,
    pub deduced_tracks: &'a BTreeSet<usize>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FactsReplayLiveObjectiveReport {
    pub flag_diagnostics: Vec<StatborgDiagnostic>,
    pub vip_diagnostics: Vec<StatborgDiagnostic>,
    pub fallbacks: Vec<ReplayFallbackHit>,
    pub bomb_carry: ReplayHeldObjectCarry,
}

/// Uses caller catalogs and mode options while readings/scanned flags come from
/// the cache. Statborg/Kills file sections are deliberately not applied here.
/// Returned fallbacks belong to assembly and must not enter scan-only capture.
/// Runtime diagnostics emit directly; returned records are retained for inspection
/// and must not be emitted a second time by the composer.
pub fn assemble_facts_replay_live_objectives(
    doc: &mut ReplayDocument,
    file: &NativeFilmFactsFile,
    options: FactsReplayLiveObjectiveOptions<'_>,
    ctx: FactsReplayLiveObjectiveContext<'_>,
) -> FactsReplayLiveObjectiveReport {
    let facts = &file.facts;
    let guards = &file.mode_guards;
    let deaths = &facts.queue.deaths;
    let bridge = ctx.state.naming_bridge();
    let ambiguous = ctx
        .state
        .ambiguous_slots()
        .iter()
        .map(|&s| (s, true))
        .collect();
    let weapon_scan = &facts.world.weapons;
    let (flags, flag_diagnostics) =
        super::facts_objective_inputs::build_facts_replay_flag_layer_observed(
            FactsReplayFlagLayerInput {
                scanned: options.flag.scanned,
                records: options.flag_records,
                bursts: options.flag_bursts,
                identity: options.flag_identity,
                teams: ctx.flag_teams,
                marks: &guards.flag_marks,
                spawns: options.flag.spawns,
                free: &[],
                gauge: guards.flag_gauge.as_deref().unwrap_or_default(),
                gauge_scanned: guards.flag_gauge_scanned,
            },
            ReplayFlagContext {
                clock: ctx.clock,
                tracks: doc.content.tracks.as_deref().unwrap_or_default(),
                deaths,
                bridge: &bridge,
                ambiguous_slots: &ambiguous,
            },
            || {
                free_facts_objective_lives(
                    weapon_scan.scanned,
                    &weapon_scan.creations,
                    &weapon_scan.tracks,
                    &options.catalog.objective_labels,
                )
            },
            |records| {
                records
                    .iter()
                    .for_each(super::replay_diagnostic_sink::emit_replay_diagnostic)
            },
        );
    let mut report = FactsReplayLiveObjectiveReport {
        flag_diagnostics,
        ..Default::default()
    };
    report.fallbacks.push(ReplayFallbackHit {
        name: "repli_piste_drapeau_sans_pont_ecartee".into(),
        hits: flags.tracks_without_bridge as i64,
    });
    report.fallbacks.push(ReplayFallbackHit {
        name: "repli_position_lacher_prend_la_prise".into(),
        hits: flags.drops_using_pickup as i64,
    });
    // Native only writes a return-zone rule when it is publishable. A disabled
    // rule does not clear an earlier document value.
    if let Some(zone) = options.flag_return_zone.for_carries(&flags.carries) {
        doc.content.flag_return_zone = Some(zone);
    }
    log_flags(flags.coverage.as_ref());
    doc.content.flag_carries = flags.carries;
    if let Some(c) = doc.coverage.as_mut() {
        c.flag_carries = flags.coverage;
    }
    if options.vip_scanned {
        let (vip, diagnostics) =
            build_facts_replay_vip_crown(options.vip_records, deaths, ctx.clock, true);
        diagnostics
            .iter()
            .for_each(super::replay_diagnostic_sink::emit_replay_diagnostic);
        report.vip_diagnostics = diagnostics;
        doc.content.vip_crown = vip.periods;
        if let Some(c) = doc.coverage.as_mut() {
            c.vip_crown = vip.coverage;
        }
    }
    if options.skull_scanned {
        let skull = build_facts_replay_skull_carries(
            FactsReplaySkullInput {
                scanned: true,
                records: options.skull_records,
                identity: options.skull_identity,
                deaths,
            },
            ctx.clock,
            &ReplayCarrierPresence::from_tracks(
                doc.content.tracks.as_deref().unwrap_or_default(),
                ctx.deduced_tracks,
            ),
        );
        log_skull(skull.coverage.as_ref());
        doc.content.skull_carries = skull.carries;
        if let Some(c) = doc.coverage.as_mut() {
            c.skull_carries = skull.coverage;
        }
    }
    report.bomb_carry = attach_facts_replay_bomb_carries_to_document(
        doc,
        FactsReplayBombCarryDocumentInput {
            scanned: options.bomb_carry_scanned,
            changes: &facts.weapon_changes,
            deaths,
            state: ctx.state,
            clock: ctx.clock,
            deduced_tracks: ctx.deduced_tracks,
        },
    );
    let objects = build_facts_replay_objective_objects(
        FactsReplayObjectiveInput {
            scanned: weapon_scan.scanned,
            creations: &weapon_scan.creations,
            tracks: &weapon_scan.tracks,
            labels: &options.catalog.objective_labels,
            families: &options.catalog.objective_families,
        },
        IdentityClock {
            origin_us: ctx.clock.origin_us,
            step_us: ctx.clock.step_us,
            frame_count: ctx.clock.frames,
        },
    );
    log_objects(Some(&objects.coverage));
    doc.content.objective_objects = objects.lives;
    if let Some(c) = doc.coverage.as_mut() {
        c.objective_objects = Some(objects.coverage);
    }
    if guards.zone_scanned
        && (options.zone.zones.is_empty() || doc.content.frame_count <= 0 || ctx.clock.step_us == 0)
    {
        tracing::warn!(match_id = %doc.content.match_id, zones = options.zone.zones.len(), frames = doc.content.frame_count,
            "rejeu : etat des zones sans catalogue de carte — aucun intervalle publie");
    }
    let zones = build_replay_zones(
        ReplayZoneInput {
            scanned: guards.zone_scanned,
            reads: guards.zone_reads.as_deref().unwrap_or_default(),
            zones: options.zone.zones,
            roles: options.zone.roles,
            teams: options.zone.teams,
            hill: options.zone.hill,
        },
        ReplayZoneContext {
            origin_us: ctx.clock.origin_us,
            step_us: ctx.clock.step_us,
            frames: doc.content.frame_count,
            interval_ms: doc.content.frame_interval_ms,
            tracks: doc.content.tracks.as_deref().unwrap_or_default(),
            actions: &doc.content.objectives,
            bridge: &bridge,
        },
    );
    for (name, hits) in [
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
        report.fallbacks.push(ReplayFallbackHit {
            name: name.into(),
            hits: hits as i64,
        });
    }
    log_zones(&doc.content.match_id, zones.coverage.as_ref());
    doc.content.zone_states = zones.states;
    if let Some(c) = doc.coverage.as_mut() {
        c.zones = zones.coverage;
    }
    if let Some(armings) = attach_replay_bomb_armings_to_document(
        doc,
        guards.bomb_reads.as_deref().unwrap_or_default(),
        options.bomb_arming_scanned,
        ctx.score_clock,
    ) {
        report.fallbacks.push(ReplayFallbackHit {
            name: "repli_armement_bombe_debut_a_zero".into(),
            hits: armings.start_zero_fallbacks as i64,
        });
    }
    attach_replay_bomb_stats_to_document(
        doc,
        ReplayBombDocumentStatsInput {
            carry_scanned: options.bomb_carry_scanned,
            score_read: options.score_read,
            objectives: options.identified_objectives,
            bridge_established: ctx.state.bridge_established(),
            carry: &report.bomb_carry,
            kills_read: options.kills_read,
            kills: options.kills,
            kills_dropped: options.kills_dropped,
            film_clock_origin_us: facts.header.film_clock_origin_us,
            death_offset_ms: ctx.clock.death_offset_ms,
        },
    );
    if let Some(equipment) = doc.coverage.as_ref().and_then(|c| c.equipment.as_ref()) {
        log_equipment(equipment);
    }
    report
}
