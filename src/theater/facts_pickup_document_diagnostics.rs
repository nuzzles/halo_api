//! Native ordered log projections for replay/build_calques.go's pickup/pad pass.
use super::*;
use serde_json::json;
macro_rules! record {
    ($out:expr,$level:literal,$message:literal $(,$key:literal=>$value:expr)* $(,)?) => {
        $out.push(StatborgDiagnostic { level:$level.into(),message:$message.into(),attributes:vec![$(($key.into(),json!($value))),*] });
    };
}
pub(super) fn weapons(out: &mut Vec<StatborgDiagnostic>, c: &ReplayWeaponChangeCoverage) {
    record!(out,"INFO","rejeu : prises et lachers d arme","decodes"=>c.decoded,"publies"=>c.published,"prises"=>c.taken,"lachers"=>c.dropped,"echanges"=>c.swapped,"reannonces"=>c.restated,"avantOrigine"=>c.before_origin);
}
pub(super) fn pickups(out: &mut Vec<StatborgDiagnostic>, c: &ReplayPickupCoverage) {
    record!(out,"INFO","rejeu : ramassages natifs","decodes"=>c.decoded,"publies"=>c.published,"nommes"=>c.named,"armes"=>c.weapons,"objets"=>c.items,
        "origineSocle"=>c.origin_spawner,"origineSol"=>c.origin_ground,"origineInconnue"=>c.origin_unknown,"etatPoints"=>c.spawn_points_state,"pointsCatalogue"=>c.map_catalog_points,
        "socleParNature"=>if c.spawner_by_point_kind.is_empty(){serde_json::Value::Null}else{json!(c.spawner_by_point_kind)},"famillesInconnues"=>c.unknown_families,"avantOrigine"=>c.before_origin,"listesMultiples"=>c.multi_event,"refuses"=>c.refused);
}
pub(super) fn pads(out: &mut Vec<StatborgDiagnostic>, c: &ReplayGroundPadCoverage) {
    record!(out,"INFO","rejeu : socles d'arme au sol","balaye"=>c.scanned,"ancres"=>c.anchors,"acceptees"=>c.accepted,"retenues"=>c.kept,"ecartees"=>c.rejected,"objetsDObjectif"=>c.objectives,
        "lachees"=>c.dropped,"apparues"=>c.spawned,"auRepos"=>c.at_rest,"grappes"=>c.clusters,"socles"=>c.pads,"occupations"=>c.occupancies,"datees"=>c.dated,"sansPassage"=>c.unknown,"jamaisVidees"=>c.never,"cyclesEtablis"=>c.cycles);
    record!(out,"INFO","rejeu : socles de power-up","balaye"=>c.powerup_scanned,"acceptees"=>c.powerup_accepted,"retenues"=>c.powerup_kept,"socles"=>c.powerup_pads);
    if c.kept == 0 && c.accepted > 0 {
        record!(out,"WARN","rejeu : identite ti=42 non resolue sur AUCUNE creation — largeurs MPP ?","acceptees"=>c.accepted,"retenues"=>c.kept,"ancres"=>c.anchors);
    }
}
pub(super) fn dating(out: &mut Vec<StatborgDiagnostic>, c: &PadDatingStats) {
    record!(out,"INFO","rejeu : datation des occupations de socle","occupations"=>c.occupations,"datees"=>c.dated,"nommees"=>c.named,"ambigues"=>c.ambiguous,"nonCouvertes"=>c.uncovered);
}
