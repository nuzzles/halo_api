//! Ordered publications from replay/build_score.go, player_teams.go and objectives.go.
use super::*;
use serde_json::json;
macro_rules! record {
    ($out:expr, $level:literal, $message:expr $(, $key:literal => $value:expr)* $(,)?) => {
        $out.push(StatborgDiagnostic {
            level: $level.into(), message: $message.into(),
            attributes: vec![$(($key.into(), json!($value))),*],
        });
    };
}
pub(super) fn team(out: &mut Vec<StatborgDiagnostic>, id: &str, c: &ReplayTeamCoverage) {
    record!(out, "INFO", "rejeu : equipes lues dans le film",
        "match_id" => id, "lue" => c.read, "refus" => c.refusal, "records" => c.records,
        "rejetes" => c.rejected, "divergences" => c.divergences, "film" => c.film,
        "sansEquipe" => c.no_team, "nonLus" => c.unread, "accord" => c.accord,
        "contradiction" => c.contradiction, "silence" => c.silence, "vies" => c.tracks,
        "viesAvecEquipe" => c.tracks_named, "viesSlotAmbigu" => c.tracks_slot_ambiguous);
    if c.tracks_slot_ambiguous > 0 {
        record!(out, "WARN", "rejeu : le pont slot -> index s'ABSTIENT sur des vies dont le slot a porte deux joueurs nommes — leur equipe reste inconnue plutot qu'empruntee au premier occupant",
            "match_id" => id, "vies" => c.tracks_slot_ambiguous);
    }
    if c.contradiction > 0 {
        record!(out, "WARN", "rejeu : la base CONTREDIT le film sur l'equipe de joueurs — le film fait foi, l'ecart est compte",
            "match_id" => id, "contradictions" => c.contradiction, "accords" => c.accord);
    }
    if !c.read {
        record!(out, "WARN", "rejeu : equipes NON LUES dans le film — aucune vie ne portera d'equipe",
            "match_id" => id, "refus" => c.refusal);
    }
}
pub(super) fn origin(out: &mut Vec<StatborgDiagnostic>, id: &str, origin: Option<i64>) {
    if origin.is_none() {
        record!(out, "WARN", "rejeu : aucune origine etablie — calques dates sur l'horloge du film NON recales", "match_id" => id);
    }
}
pub(super) fn objectives(
    out: &mut Vec<StatborgDiagnostic>,
    id: &str,
    c: &ReplayLayerCoverage,
    missing: usize,
    published: usize,
) {
    if missing > 0 {
        record!(out, "WARN", "rejeu : actions d'objectif dont l'auteur n'a aucune trajectoire publiee",
            "match_id" => id, "actions" => missing, "publiees" => published);
    }
    if c.available == 0 {
        return;
    }
    for (cause, n) in [
        ("slotIntrouvable", c.no_slot),
        ("slotAmbigu", c.ambiguous),
        ("horsFenetre", c.out_of_window),
        ("sansTrajectoirePubliee", c.unpublished),
    ] {
        if (n as f64) / (c.available as f64) < 0.10 {
            continue;
        }
        record!(out, "WARN", "rejeu : rejets au-dessus du seuil", "calque" => "objectifs",
            "cause" => cause, "rejetes" => n, "disponibles" => c.available, "rattaches" => c.attached);
    }
}
pub(super) fn score(
    out: &mut Vec<StatborgDiagnostic>,
    id: &str,
    cov: Option<&ReplayScoreCoverage>,
    tl: Option<&ReplayScoreTimeline>,
    records: Option<&[StatborgRecord]>,
) {
    let Some(c) = cov else {
        return;
    };
    let (teams, players) = tl.map_or((0, 0), |t| (t.teams.len(), t.players.len()));
    if c.team_identity == "unresolved" {
        record!(out, "WARN", "rejeu : identite des camps NON RESOLUE — courbes de score publiees sans equipe",
            "match_id" => id, "equipes" => teams, "joueurs" => players, "manches" => c.rounds);
    }
    if c.truncated {
        record!(out, "WARN", "rejeu : lecture des enregistrements TRONQUEE — courbes de score incompletes",
            "match_id" => id, "points" => c.points);
    }
    record!(out, "INFO", "rejeu : courbe de score", "match_id" => id, "identiteEquipes" => c.team_identity,
        "manches" => c.rounds, "modePorte" => c.mode_supported, "equipes" => teams,
        "joueurs" => players, "points" => c.points);
    let Some(records) = records.filter(|r| !r.is_empty() && c.rounds >= 2) else {
        return;
    };
    let bounds = resolve_statborg_round_bounds(records);
    for s in &bounds.kept {
        record!(out, "WARN", "rejeu : bloc de manche GARDE hors de la fenetre consensuelle — le slot et le consensus ne s'accordent pas sur les bornes de cette manche",
            "match_id" => id, "slot" => s.slot, "manche" => s.round, "debut_ms" => s.from_ms,
            "fin_ms" => s.to_ms, "ecart_ms" => s.gap_ms, "enregistrements" => s.records);
    }
    if !bounds.posed() {
        record!(out, "WARN", "rejeu : AUCUNE borne de manche posee sur un film a plusieurs manches — le numero de manche ne suit pas l'horloge, les compteurs restent ceux d'avant",
            "match_id" => id, "manches" => c.rounds, "enregistrements" => records.len());
        return;
    }
    let n = bounds.outliers(records);
    if n > STATBORG_OUTLIERS_NOMINAL_MAX {
        record!(out, "WARN", "rejeu : enregistrements hors de la fenetre de leur manche declaree AU-DELA DU NOMINAL — l'etiquetage de manche de ce film est a regarder",
            "match_id" => id, "ecartes" => n, "nominal_max" => STATBORG_OUTLIERS_NOMINAL_MAX,
            "enregistrements" => records.len(), "manches" => c.rounds);
    } else {
        record!(out, "INFO", "rejeu : enregistrements hors de la fenetre de leur manche declaree, ecartes",
            "match_id" => id, "ecartes" => n, "enregistrements" => records.len(),
            "manches" => c.rounds, "blocs_gardes" => bounds.kept.len());
    }
}
