//! Native document-stage bomb statistics from already assembled evidence.
use super::*;

pub struct ReplayBombDocumentStatsInput<'a> {
    /// Family-wide carry gate, independent of the narrower arming scan gate.
    pub carry_scanned: bool,
    pub score_read: bool,
    pub objectives: &'a [StatborgIdentifiedEvent],
    pub bridge_established: bool,
    pub carry: &'a ReplayHeldObjectCarry,
    pub kills_read: bool,
    pub kills: &'a [ReplayKillReference],
    pub kills_dropped: i64,
    pub film_clock_origin_us: u64,
    pub death_offset_ms: i64,
}

/// Attach after arming publication. Its coverage, including an absent envelope,
/// is the authority for whether armings were read; an empty vector is not enough.
/// Disabled carry recognition is a no-op, preserving an existing document.
pub fn attach_replay_bomb_stats_to_document(
    doc: &mut ReplayDocument,
    input: ReplayBombDocumentStatsInput<'_>,
) {
    if !input.carry_scanned {
        return;
    }
    let armings_read = doc
        .coverage
        .as_ref()
        .and_then(|c| c.bomb_armings.as_ref())
        .is_some_and(|c| c.scanned && !c.suppressed);
    let output = build_replay_bomb_stats(&ReplayBombStatsInput {
        detonations_read: input.score_read,
        objectives: input.objectives.to_vec(),
        carry_read: input.bridge_established,
        carry: input.carry.clone(),
        kills_read: input.kills_read,
        kills: input.kills.to_vec(),
        armings_read,
        armings: doc.content.bomb_armings.clone(),
        film_to_match_offset_ms: ((input.film_clock_origin_us as i64) / 1000)
            .wrapping_sub(input.death_offset_ms),
    });
    doc.content.bomb_stats = Some(output.stats);
    doc.content.bomb_events = output.events;
    let c = &doc.content.bomb_stats.as_ref().unwrap().coverage;
    tracing::info!(
        match_id = doc.content.match_id,
        joueurs = c.players,
        explosions = c.detonations,
        armements = c.armings,
        attribues = c.armings_attributed,
        parLacher = c.armings_by_drop,
        parRepli = c.armings_by_active_carry,
        sansPorteur = c.armings_no_carrier,
        sansPont = c.armings_no_bridge,
        ambigus = c.armings_ambiguous,
        periodes = c.periods,
        periodesSansPont = c.periods_no_bridge,
        periodesOuvertes = c.periods_open,
        kills = c.kills,
        killsEcartes = input.kills_dropped,
        killsSurPorteur = c.kills_on_carrier,
        lu_explosions = c.detonations_read,
        lu_portage = c.carry_read,
        lu_armements = c.armings_read,
        lu_kills = c.kills_read,
        "rejeu : statistiques d'objectif de l'Assaut"
    );
}

/// Publish armings after objective actions. Return the complete reducer result so
/// the composer can accumulate start-zero fallbacks without rescanning anything.
/// This gate is independent of carry recognition and statistics publication.
pub fn attach_replay_bomb_armings_to_document(
    doc: &mut ReplayDocument,
    reads: &[NavpointRadialRead],
    scanned: bool,
    clock: ReplayScoreClock,
) -> Option<ReplayBombArmings> {
    if !scanned {
        return None;
    }
    let mut detonations: Vec<_> = doc
        .content
        .objectives
        .iter()
        .filter(|a| a.stat == "bomb_detonations")
        .map(|a| a.time_ms)
        .collect();
    detonations.sort_unstable();
    let out = build_replay_bomb_armings(reads, &detonations, clock);
    doc.content.bomb_armings = out.armings.clone();
    if let Some(c) = doc.coverage.as_mut() {
        c.bomb_armings = Some(out.coverage.clone());
    }
    let c = &out.coverage;
    let v = &out.verdict;
    if c.suppressed {
        tracing::warn!(
            match_id = doc.content.match_id,
            explosions = c.detonations,
            couvertes = c.detonations_covered,
            armements = c.armed,
            segments = c.rises,
            mecheIncoherente = v.inconsistent,
            meche_ms = v.fuse_ms,
            cv = v.cv,
            "rejeu : armement RETENU A LA SOURCE — le film contredit la lecture (explosion sans armement, ou meches qui se contredisent)"
        );
    } else {
        tracing::info!(
            match_id = doc.content.match_id,
            lectures = c.reads,
            segments = c.rises,
            sousLePlein = c.below_full,
            armements = c.armed,
            paireFondue = c.pair_merged,
            publies = c.published,
            horsFenetre = c.out_of_window,
            explosions = c.detonations,
            couvertes = c.detonations_covered,
            meche_ms = v.fuse_ms,
            mecheMesuree = v.measured,
            cv = v.cv,
            "rejeu : armement de la bombe"
        );
    }
    Some(out)
}

pub struct FactsReplayBombCarryDocumentInput<'a> {
    pub scanned: bool,
    pub changes: &'a [FactsWeaponChange],
    pub deaths: &'a [FactsDeath],
    pub state: &'a ReplayIdentityState,
    pub clock: ReplayMatchClock,
    pub deduced_tracks: &'a std::collections::BTreeSet<usize>,
}

/// Publish using the document's current tracks and return the untrimmed raw
/// timeline for statistics. Missing coverage does not prevent publication.
pub fn attach_facts_replay_bomb_carries_to_document(
    doc: &mut ReplayDocument,
    input: FactsReplayBombCarryDocumentInput<'_>,
) -> ReplayHeldObjectCarry {
    if !input.scanned {
        return ReplayHeldObjectCarry::default();
    }
    let events = replay_facts_bomb_held_events(input.changes, input.clock.death_offset_ms);
    if !input.state.bridge_established() {
        tracing::warn!(
            match_id = doc.content.match_id,
            transitions = events.len(),
            "rejeu : portage de la bombe sans pont slot->xuid — aucune periode publiable"
        );
    }
    let result = super::replay_bomb_carries::assemble_bomb_carry_values(
        events,
        input.state,
        input.clock,
        &ReplayCarrierPresence::from_tracks(
            doc.content.tracks.as_deref().unwrap_or_default(),
            input.deduced_tracks,
        ),
        input.deaths.iter().map(|d| (d.xuid, d.time_ms)),
    );
    doc.content.bomb_carries = result.publication.carries;
    if let Some(c) = doc.coverage.as_mut() {
        c.bomb_carries = result.publication.coverage.clone();
    }
    if let Some(c) = result.publication.coverage {
        tracing::info!(
            transitions = c.events,
            periodes = c.periods,
            portages = c.carries,
            fermes = c.closed,
            ouverts = c.open,
            parMort = c.by_death,
            sansPont = c.no_bridge,
            horsFenetre = c.out_of_window,
            porteurAbsent = c.carrier_absent,
            "rejeu : portage de la bombe d'Assaut"
        );
    }
    result.raw
}
