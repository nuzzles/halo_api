//! Native initial document observations, at registry/track/naming boundaries.
use super::*;

pub(super) fn table(match_id: &str, c: &IdentityFilmTableCoverage) {
    tracing::info!(
        match_id,
        lue = c.read,
        refus = c.refusal.json_text().as_str(),
        sieges = c.seats,
        direct = c.direct,
        repli = c.fallback,
        accord = c.accord,
        contradiction = c.contradiction,
        silence = c.silence,
        collisionsIndex = c.index_collisions,
        "rejeu : table du film composee au registre d'identite"
    );
    if c.index_collisions > 0 {
        tracing::warn!(
            match_id,
            collisions = c.index_collisions,
            liens_restants = c.direct.wrapping_add(c.fallback),
            "rejeu : des INDEX de joueur etaient revendiques par deux xuids dans la table composee — l'index est retire POUR LES DEUX, aucune identite n'est tiree au sort"
        );
    }
    if c.contradiction > 0 {
        tracing::warn!(
            match_id,
            contradictions = c.contradiction,
            accords = c.accord,
            "rejeu : la lecture des chunks CONTREDIT la table du film sur des index — la table du film fait foi, l'ecart est compte"
        );
    }
    if !c.read {
        tracing::warn!(
            match_id,
            refus = c.refusal.json_text().as_str(),
            liens = c.fallback,
            "rejeu : table du film NON EMPLOYEE — le lien index <-> xuid retombe entierement sur la lecture des chunks de replication"
        );
    }
}
pub(super) fn tracks(match_id: &str, c: &ReplayTrackCoverage) {
    if c.gaps > 0 {
        tracing::info!(
            match_id,
            lacunes = c.gaps,
            duree_ms = c.gap_ms,
            vies = c.published,
            seuil_ms = 5000,
            "rejeu : lacunes de replication DANS une vie"
        );
    }
    if c.refused_min_points > 0 {
        tracing::info!(
            match_id,
            vies = c.refused_min_points,
            points = c.refused_points,
            seuil = c.min_points,
            viesPubliees = c.published,
            "rejeu : vies refusees par le seuil de publication"
        );
    }
}
pub(super) fn bounds(match_id: &str, rejected: usize) {
    if rejected > 0 {
        tracing::info!(
            match_id,
            ecartes = rejected,
            seuil_etendues = 12,
            "rejeu : echantillons aberrants ecartes des bornes"
        );
    }
}
pub(super) fn naming(match_id: &str, tracks: &[ReplayTrack], rep: &IdentityRemainingReport) {
    if rep.total() > 0 {
        tracing::info!(
            match_id,
            traitees = rep.total(),
            parViePrecedente = rep.by_previous,
            parVieSuivante = rep.by_next,
            parPont = rep.by_bridge,
            residu = rep.remaining,
            frontieresIndecidables = rep.contested,
            "rejeu : nommage final des vies restantes"
        );
    }
    if rep.remaining == 0 {
        return;
    }
    let (slot, from, to) = tracks
        .iter()
        .find(|t| t.xuid.is_empty() && t.bot.is_empty())
        .map_or((-1, 0, 0), |t| {
            (i64::from(t.slot), t.start_frame, t.end_frame)
        });
    tracing::error!(
        match_id,
        vies = rep.remaining,
        premierSlot = slot,
        premiereFrameDebut = from,
        premiereFrameFin = to,
        "rejeu : des vies PUBLIEES restent sans identite — defaut de nommage du pont"
    );
}
