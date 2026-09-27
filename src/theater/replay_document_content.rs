//! Typed replay content. Coverage assembly and the complete LegacyFilm constructor remain separate work.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
fn content_zero(v: &i64) -> bool {
    *v == 0
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ReplayDocumentContent {
    #[serde(rename = "schemaVersion")]
    pub schema_version: i64,
    #[serde(rename = "matchId")]
    pub match_id: String,
    #[serde(rename = "titleSlug")]
    pub title_slug: String,
    #[serde(rename = "frameCount")]
    pub frame_count: i64,
    #[serde(rename = "bounds")]
    pub bounds: ReplayBounds,
    #[serde(rename = "tracks")]
    pub tracks: Option<Vec<ReplayTrack>>,
    #[serde(rename = "frameIntervalMs", skip_serializing_if = "content_zero")]
    pub frame_interval_ms: i64,
    #[serde(rename = "durationMs", skip_serializing_if = "content_zero")]
    pub duration_ms: i64,
    #[serde(rename = "originMs", skip_serializing_if = "Option::is_none")]
    pub origin_ms: Option<i64>,
    #[serde(rename = "t0FilmMs", skip_serializing_if = "Option::is_none")]
    pub t0_film_ms: Option<i64>,
    #[serde(rename = "geometry", skip_serializing_if = "Vec::is_empty")]
    pub geometry: Vec<ReplayMapObject>,
    #[serde(rename = "geometryBounds", skip_serializing_if = "Option::is_none")]
    pub geometry_bounds: Option<ReplayBounds>,
    #[serde(rename = "structure", skip_serializing_if = "Vec::is_empty")]
    pub structure: Vec<ReplaySurface>,
    #[serde(rename = "structureBounds", skip_serializing_if = "Option::is_none")]
    pub structure_bounds: Option<ReplayBounds>,
    #[serde(rename = "shots", skip_serializing_if = "Vec::is_empty")]
    pub shots: Vec<ReplayShot>,
    #[serde(rename = "loadouts", skip_serializing_if = "Vec::is_empty")]
    pub loadouts: Vec<ReplayLoadout>,
    #[serde(rename = "inventory", skip_serializing_if = "Vec::is_empty")]
    pub inventory: Vec<ReplayInventory>,
    #[serde(rename = "grenadeLabels", skip_serializing_if = "Vec::is_empty")]
    pub grenade_labels: Vec<ReplayLabel>,
    #[serde(rename = "abilities", skip_serializing_if = "Vec::is_empty")]
    pub abilities: Vec<ReplayAbilityRead>,
    #[serde(rename = "grenadeReads", skip_serializing_if = "Vec::is_empty")]
    pub grenade_reads: Vec<ReplayGrenadeRead>,
    #[serde(rename = "abilityLabels", skip_serializing_if = "BTreeMap::is_empty")]
    pub ability_labels: BTreeMap<String, ReplayLabel>,
    #[serde(rename = "equipmentEpisodes", skip_serializing_if = "Vec::is_empty")]
    pub equipment_episodes: Vec<ReplayEquipmentEpisode>,
    #[serde(rename = "stances", skip_serializing_if = "Vec::is_empty")]
    pub stances: Vec<ReplayStance<ReplayByteString>>,
    #[serde(rename = "grappleLines", skip_serializing_if = "Vec::is_empty")]
    pub grapple_lines: Vec<ReplayGrappleLine>,
    #[serde(rename = "equipmentPlacements", skip_serializing_if = "Vec::is_empty")]
    pub equipment_placements: Vec<ReplayEquipmentPlacement>,
    #[serde(rename = "weaponChanges", skip_serializing_if = "Vec::is_empty")]
    pub weapon_changes: Vec<ReplayWeaponChange>,
    #[serde(rename = "pickups", skip_serializing_if = "Vec::is_empty")]
    pub pickups: Vec<ReplayPickup>,
    #[serde(rename = "equipmentChanges", skip_serializing_if = "Vec::is_empty")]
    pub equipment_changes: Vec<ReplayEquipmentChange>,
    #[serde(rename = "translocations", skip_serializing_if = "Vec::is_empty")]
    pub translocations: Vec<ReplayTranslocation>,
    #[serde(rename = "abilityImpulses", skip_serializing_if = "Vec::is_empty")]
    pub ability_impulses: Vec<ReplayAbilityImpulse>,
    #[serde(rename = "abilityCharges", skip_serializing_if = "Vec::is_empty")]
    pub ability_charges: Vec<ReplayAbilityCharge>,
    #[serde(rename = "groundWeapons", skip_serializing_if = "Vec::is_empty")]
    pub ground_weapons: Vec<ReplayGroundWeapon>,
    #[serde(rename = "vehicles", skip_serializing_if = "Vec::is_empty")]
    pub vehicles: Vec<ReplayVehicleTrack>,
    #[serde(rename = "vehicleLabels", skip_serializing_if = "BTreeMap::is_empty")]
    pub vehicle_labels: BTreeMap<String, ReplayVehicleLabel>,
    #[serde(rename = "vehicleCycles", skip_serializing_if = "Vec::is_empty")]
    pub vehicle_cycles: Vec<ReplayVehicleCycle>,
    #[serde(rename = "weaponPads", skip_serializing_if = "Vec::is_empty")]
    pub weapon_pads: Vec<GroundWeaponPad>,
    #[serde(rename = "padPickups", skip_serializing_if = "Vec::is_empty")]
    pub pad_pickups: Vec<GroundPadPickup>,
    #[serde(rename = "grenades", skip_serializing_if = "Vec::is_empty")]
    pub grenades: Vec<ReplayGrenade>,
    #[serde(rename = "projectiles", skip_serializing_if = "Vec::is_empty")]
    pub projectiles: Vec<ReplayProjectile>,
    #[serde(rename = "weaponLabels", skip_serializing_if = "BTreeMap::is_empty")]
    pub weapon_labels: BTreeMap<String, ReplayWeaponLabel>,
    #[serde(rename = "killEffects", skip_serializing_if = "BTreeMap::is_empty")]
    pub kill_effects: BTreeMap<String, String>,
    #[serde(rename = "neutralDeaths", skip_serializing_if = "Vec::is_empty")]
    pub neutral_deaths: Vec<ReplayNeutralDeath>,
    #[serde(rename = "roster", skip_serializing_if = "Vec::is_empty")]
    pub roster: Vec<ReplayRosterEntry>,
    #[serde(rename = "mapObjectives", skip_serializing_if = "Option::is_none")]
    pub map_objectives: Option<ReplayMapObjectives>,
    #[serde(rename = "mapWeaponPads", skip_serializing_if = "Option::is_none")]
    pub map_weapon_pads: Option<ReplayMapWeaponPads>,
    #[serde(rename = "weaponTiers", skip_serializing_if = "Option::is_none")]
    pub weapon_tiers: Option<ReplayWeaponTiersInfo>,
    #[serde(rename = "objectives", skip_serializing_if = "Vec::is_empty")]
    pub objectives: Vec<ReplayObjectiveAction>,
    #[serde(rename = "scoreTimeline", skip_serializing_if = "Option::is_none")]
    pub score_timeline: Option<ReplayScoreTimeline>,
    #[serde(rename = "flagCarries", skip_serializing_if = "Vec::is_empty")]
    pub flag_carries: Vec<ReplayFlagCarry>,
    #[serde(rename = "flagReturnZone", skip_serializing_if = "Option::is_none")]
    pub flag_return_zone: Option<ReplayFlagReturnZone>,
    #[serde(rename = "objectiveObjects", skip_serializing_if = "Vec::is_empty")]
    pub objective_objects: Vec<ReplayObjectiveLife>,
    #[serde(rename = "zoneStates", skip_serializing_if = "Vec::is_empty")]
    pub zone_states: Vec<ReplayZoneState>,
    #[serde(rename = "vipCrown", skip_serializing_if = "Vec::is_empty")]
    pub vip_crown: Vec<ReplayVipPeriod>,
    #[serde(rename = "bombArmings", skip_serializing_if = "Vec::is_empty")]
    pub bomb_armings: Vec<ReplayBombArming>,
    #[serde(rename = "skullCarries", skip_serializing_if = "Vec::is_empty")]
    pub skull_carries: Vec<ReplaySkullCarry>,
    #[serde(rename = "bombCarries", skip_serializing_if = "Vec::is_empty")]
    pub bomb_carries: Vec<ReplayBombCarry>,
    #[serde(rename = "bombStats", skip_serializing_if = "Option::is_none")]
    pub bomb_stats: Option<ReplayBombMatchStats>,
    #[serde(rename = "bombEvents", skip_serializing_if = "Vec::is_empty")]
    pub bomb_events: Vec<ReplayBombEvent>,
    #[serde(rename = "identity", skip_serializing_if = "Option::is_none")]
    pub identity: Option<IdentitySection>,
    #[serde(rename = "layers", skip_serializing_if = "BTreeMap::is_empty")]
    pub layers: BTreeMap<String, String>,
}
#[cfg(test)]
mod tests {
    use super::*;
    // JSON numbers have one semantic type; Go emits 0 where serde emits 0.0.
    fn canonical_numbers(v: &mut serde_json::Value) {
        match v {
            serde_json::Value::Number(n) => {
                *n = serde_json::Number::from_f64(n.as_f64().unwrap()).unwrap();
            }
            serde_json::Value::Array(a) => {
                for v in a {
                    canonical_numbers(v);
                }
            }
            serde_json::Value::Object(o) => {
                for v in o.values_mut() {
                    canonical_numbers(v);
                }
            }
            _ => {}
        }
    }
    #[test]
    fn native_document_content_schema() {
        let rows: Vec<serde_json::Value> =
            serde_json::from_str(include_str!("fixtures/document-content-v41.json")).unwrap();
        for (i, mut row) in rows.into_iter().enumerate() {
            let doc: ReplayDocumentContent =
                serde_json::from_value(row.clone()).unwrap_or_else(|e| panic!("case {i}: {e}"));
            if doc.schema_version == 68 {
                assert_eq!(replay_geometry_bounds(&doc.geometry), doc.geometry_bounds);
                assert_eq!(replay_surface_bounds(&doc.structure), doc.structure_bounds);
            }
            let mut actual = serde_json::to_value(doc).unwrap();
            canonical_numbers(&mut actual);
            canonical_numbers(&mut row);
            assert_eq!(actual, row, "case {i}");
        }
    }
}
