//! Native live-objective publication diagnostics, emitted at each attachment boundary.
use super::*;
pub(super) fn log_flags(cov: Option<&ReplayFlagCoverage>) {
    let Some(cov) = cov else {
        return;
    };
    if !cov.flag_film {
        tracing::debug!(
            bursts = cov.bursts,
            captures = cov.captures,
            vols = cov.steals,
            "rejeu : film non reconnu CTF — aucun drapeau publie"
        );
        return;
    }
    if cov.closed_overlaps > 0 {
        tracing::warn!(
            depassements = cov.closed_overlaps,
            depassementsTous = cov.overlaps,
            portages = cov.carries,
            "rejeu : plus de deux drapeaux portes a la fois ENTRE PORTAGES FERMES — la lecture se contredit"
        );
    }
    tracing::info!(
        prises = cov.openings,
        portages = cov.carries,
        fermes = cov.closed,
        ouverts = cov.open,
        viesLibres = cov.object_lives,
        fermesParLObjet = cov.closed_by_object,
        lachersRepositionnes = cov.drops_repositioned,
        sansPont = cov.no_bridge,
        sansPiste = cov.no_track,
        horsFenetre = cov.out_of_window,
        slotsAmbigus = cov.ambiguous_slot,
        marqueurConfirme = cov.marker_confirmed,
        marqueurObserve = cov.marker_observed,
        socles = cov.spawns,
        simultaneite = cov.overlaps,
        porteursTuesAmbigus = cov.ambiguous_carrier_kills,
        retoursAmbigus = cov.ambiguous_returns,
        rentreesParLObjet = cov.home_by_object,
        rentreesAmbigues = cov.ambiguous_homecomings,
        "rejeu : vie des drapeaux"
    );

    tracing::info!(
        balaye = cov.gauge_scanned,
        slots = cov.gauge_slots,
        lectures = cov.gauge_reads,
        apparies = cov.gauge_paired,
        lachersAvecJauge = cov.gauge_spans,
        points = cov.gauge_points,
        "rejeu : jauge de retour du drapeau"
    );
    if cov.gauge_scanned && cov.gauge_slots > 0 && cov.gauge_paired == 0 {
        tracing::warn!(
            slots = cov.gauge_slots,
            lectures = cov.gauge_reads,
            portages = cov.carries,
            "rejeu : jauge de retour LUE mais AUCUNE ne correle a un drapeau — les lachers publies et les series du film ne se recouvrent pas"
        );
    }
}
pub(super) fn log_skull(cov: Option<&ReplaySkullCoverage>) {
    let Some(cov) = cov else {
        return;
    };
    tracing::info!(
        prises = cov.grabs,
        trains = cov.trains,
        portages = cov.carries,
        fermes = cov.closed,
        ouverts = cov.open,
        sansPont = cov.no_bridge,
        horsFenetre = cov.out_of_window,
        porteurAbsent = cov.carrier_absent,
        "rejeu : portage du crane d'Oddball"
    );
}
pub(super) fn log_objects(cov: Option<&ReplayObjectiveCoverage>) {
    let Some(cov) = cov else {
        return;
    };
    tracing::info!(
        balaye = cov.scanned,
        declares = cov.declared,
        vies = cov.lives,
        points = cov.points,
        immobiles = cov.motionless,
        horsAxe = cov.out_of_axis,
        "rejeu : objets d objectif libres"
    );
}
pub(super) fn log_zones(match_id: &str, cov: Option<&ReplayZonesCoverage>) {
    let Some(cov) = cov else {
        return;
    };
    if cov.unpaired > 0 {
        tracing::warn!(match_id = %match_id, methode = %cov.method, nonApparies = cov.unpaired, apparies = cov.paired, captures = cov.captures, attribuees = cov.attributed, "rejeu : appariements ECARTES — zones absentes de l'etat publie");
    }
    if cov.owner_unpaired > 0 {
        tracing::warn!(match_id = %match_id, sansProprietaire = cov.owner_unpaired, apparies = cov.paired, seuilAccord = 2, "rejeu : zones SANS canal de proprietaire elu — zones absentes de l'etat publie");
    }
    if cov.unknown_owner > 0 {
        tracing::warn!(match_id = %match_id, valeurs = cov.unknown_owner, "rejeu : valeurs de proprietaire INCONNUES — intervalles non ouverts");
    }
    tracing::info!(match_id = %match_id, methode = %cov.method, catalogue = cov.catalog, slots = cov.slots, apparies = cov.paired, nonApparies = cov.unpaired, sansProprietaire = cov.owner_unpaired, intervalles = cov.spans, periodesColline = cov.hill_periods, proprietaireVerifie = cov.owner_checked, proprietaireConcordant = cov.owner_agreed, capturesAttribuees = cov.attributed, capturesSansPosition = cov.no_position, capturesDehors = cov.outside, capturesAmbigues = cov.ambiguous_zone, "rejeu : etat des zones");
}
pub(super) fn log_equipment(cov: &ReplayEquipmentCoverage) {
    tracing::info!(
        viesPubliees = cov.tracks_total,
        viesCamo = cov.camo_lives,
        episodesCamo = cov.camo_episodes,
        viesSurbouclier = cov.overshield_lives,
        episodesSurbouclier = cov.overshield_episodes,
        "rejeu : episodes d'equipement actif"
    );
}
