//! Typed native coverage envelope; population is performed by document assembly.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ReplayCoverage {
    #[serde(rename = "shots")]
    pub shots: ReplayLayerCoverage,
    #[serde(rename = "grenades")]
    pub grenades: ReplayLayerCoverage,
    #[serde(rename = "tracks", skip_serializing_if = "Option::is_none")]
    pub tracks: Option<ReplayTrackCoverage>,
    #[serde(rename = "teams", skip_serializing_if = "Option::is_none")]
    pub teams: Option<ReplayTeamCoverage>,
    #[serde(rename = "seats", skip_serializing_if = "Option::is_none")]
    pub seats: Option<ReplaySeatCoverage>,
    #[serde(rename = "projectiles", skip_serializing_if = "Option::is_none")]
    pub projectiles: Option<ReplayProjectileCoverage>,
    #[serde(rename = "objectives")]
    pub objectives: ReplayLayerCoverage,
    #[serde(rename = "equipment", skip_serializing_if = "Option::is_none")]
    pub equipment: Option<ReplayEquipmentCoverage>,
    #[serde(rename = "stances", skip_serializing_if = "Option::is_none")]
    pub stances: Option<ReplayStanceCoverage<ReplayByteString>>,
    #[serde(rename = "grapple", skip_serializing_if = "Option::is_none")]
    pub grapple: Option<ReplayGrappleCoverage>,
    #[serde(rename = "placements", skip_serializing_if = "Option::is_none")]
    pub placements: Option<ReplayEquipmentPlacementCoverage>,
    #[serde(rename = "groundWeapons", skip_serializing_if = "Option::is_none")]
    pub ground_weapons: Option<ReplayGroundPadCoverage>,
    #[serde(rename = "score", skip_serializing_if = "Option::is_none")]
    pub score: Option<ReplayScoreCoverage>,
    #[serde(rename = "flagCarries", skip_serializing_if = "Option::is_none")]
    pub flag_carries: Option<ReplayFlagCoverage>,
    #[serde(rename = "vipCrown", skip_serializing_if = "Option::is_none")]
    pub vip_crown: Option<ReplayVipCoverage>,
    #[serde(rename = "skullCarries", skip_serializing_if = "Option::is_none")]
    pub skull_carries: Option<ReplaySkullCoverage>,
    #[serde(rename = "bombCarries", skip_serializing_if = "Option::is_none")]
    pub bomb_carries: Option<ReplayBombCarryCoverage>,
    #[serde(rename = "bombArmings", skip_serializing_if = "Option::is_none")]
    pub bomb_armings: Option<ReplayBombArmingsCoverage>,
    #[serde(rename = "weaponChanges", skip_serializing_if = "Option::is_none")]
    pub weapon_changes: Option<ReplayWeaponChangeCoverage>,
    #[serde(rename = "pickups", skip_serializing_if = "Option::is_none")]
    pub pickups: Option<ReplayPickupCoverage>,
    #[serde(rename = "padDating", skip_serializing_if = "Option::is_none")]
    pub pad_dating: Option<PadDatingStats>,
    #[serde(rename = "equipmentChanges", skip_serializing_if = "Option::is_none")]
    pub equipment_changes: Option<ReplayEquipmentChangeCoverage>,
    #[serde(rename = "translocations", skip_serializing_if = "Option::is_none")]
    pub translocations: Option<ReplayTranslocationCoverage>,
    #[serde(rename = "abilityImpulses", skip_serializing_if = "Option::is_none")]
    pub ability_impulses: Option<ReplayAbilityImpulseCoverage>,
    #[serde(rename = "abilityCharges", skip_serializing_if = "Option::is_none")]
    pub ability_charges: Option<ReplayAbilityChargeCoverage>,
    #[serde(rename = "groundWeaponItems", skip_serializing_if = "Option::is_none")]
    pub ground_weapon_items: Option<ReplayGroundWeaponCoverage>,
    #[serde(rename = "vehicles", skip_serializing_if = "Option::is_none")]
    pub vehicles: Option<ReplayVehicleCoverage>,
    #[serde(rename = "objectiveObjects", skip_serializing_if = "Option::is_none")]
    pub objective_objects: Option<ReplayObjectiveCoverage>,
    #[serde(rename = "inventory", skip_serializing_if = "Option::is_none")]
    pub inventory: Option<ReplayInventoryCoverage>,
    #[serde(rename = "grenadeReads", skip_serializing_if = "Option::is_none")]
    pub grenade_reads: Option<ReplayGrenadeReadCoverage>,
    #[serde(rename = "abilities", skip_serializing_if = "Option::is_none")]
    pub abilities: Option<ReplayAbilityCoverage>,
    #[serde(rename = "zones", skip_serializing_if = "Option::is_none")]
    pub zones: Option<ReplayZonesCoverage>,
    #[serde(rename = "originResolved")]
    pub origin_resolved: bool,
    #[serde(rename = "t0Film", skip_serializing_if = "Option::is_none")]
    pub t0_film: Option<ReplayT0FilmCoverage>,
    #[serde(rename = "filmMajorVersion", skip_serializing_if = "Option::is_none")]
    pub film_major_version: Option<i64>,
    #[serde(rename = "verdict", skip_serializing_if = "BTreeMap::is_empty")]
    pub verdict: BTreeMap<String, String>,
    #[serde(rename = "bridge")]
    pub bridge: IdentityBridgeHealth,
    #[serde(rename = "fallbacks", skip_serializing_if = "Vec::is_empty")]
    pub fallbacks: Vec<ReplayFallbackHit>,
    #[serde(rename = "decoder", skip_serializing_if = "Option::is_none")]
    pub decoder: Option<ReplayDecoderCoverage>,
    #[serde(rename = "deathsPaths", skip_serializing_if = "Option::is_none")]
    pub deaths_paths: Option<ReplayDeathsPathsCoverage>,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ReplayDeathsPathsCoverage {
    #[serde(rename = "walk")]
    pub walk: ReplayDeathsPathTally,
    #[serde(rename = "directScan")]
    pub direct_scan: ReplayDeathsPathTally,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ReplayDeathsPathTally {
    #[serde(rename = "population")]
    pub population: usize,
    #[serde(rename = "matched")]
    pub matched: usize,
    #[serde(rename = "published")]
    pub published: usize,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ReplayDecoderCoverage {
    #[serde(rename = "sourceRev")]
    pub source_rev: String,
    #[serde(rename = "profileRev")]
    pub profile_rev: String,
    #[serde(rename = "grammarRev")]
    pub grammar_rev: String,
    #[serde(rename = "factsRev")]
    pub facts_rev: String,
    #[serde(rename = "build")]
    pub build: String,
    #[serde(rename = "registry", skip_serializing_if = "Option::is_none")]
    pub registry: Option<ReplayRegistryCoverage>,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ReplayRegistryCoverage {
    #[serde(rename = "fingerprint")]
    pub fingerprint: String,
    #[serde(rename = "status")]
    pub status: String,
    #[serde(rename = "blocks")]
    pub blocks: i64,
    #[serde(rename = "namedSlots")]
    pub named_slots: i64,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ReplayT0FilmCoverage {
    #[serde(rename = "detected")]
    pub detected: bool,
    #[serde(rename = "reason", skip_serializing_if = "String::is_empty")]
    pub reason: String,
    #[serde(rename = "tracks")]
    pub tracks: usize,
    #[serde(rename = "moving")]
    pub moving: usize,
    #[serde(rename = "burst")]
    pub burst: usize,
    #[serde(rename = "marginMs")]
    pub margin_ms: i64,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ReplayFallbackHit {
    #[serde(rename = "name")]
    pub name: ReplayByteString,
    #[serde(rename = "hits")]
    pub hits: i64,
}

impl ReplayCoverage {
    /// Construct the native base envelope before downstream layers add their counters.
    pub fn new(
        shots: ReplayLayerCoverage,
        grenades: ReplayLayerCoverage,
        objectives: ReplayLayerCoverage,
        registry: &IdentityRegistryOutput,
        origin_resolved: bool,
        score: Option<ReplayScoreCoverage>,
    ) -> Self {
        let bridge = registry.bridge_health();
        bridge.warn_if_death_offset_ambiguous();
        Self::from_bridge(shots, grenades, objectives, bridge, origin_resolved, score)
    }

    pub(super) fn from_bridge(
        shots: ReplayLayerCoverage,
        grenades: ReplayLayerCoverage,
        objectives: ReplayLayerCoverage,
        bridge: IdentityBridgeHealth,
        origin_resolved: bool,
        score: Option<ReplayScoreCoverage>,
    ) -> Self {
        let verdict = [
            ("shots", shots.verdict()),
            ("grenades", grenades.verdict()),
            ("objectives", objectives.verdict()),
            ("bridge", bridge.verdict()),
        ]
        .into_iter()
        .map(|(k, v)| (k.into(), v.into()))
        .collect();
        Self {
            shots,
            grenades,
            objectives,
            bridge,
            origin_resolved,
            score,
            verdict,
            ..Self::default()
        }
    }

    /// Replace the final report, combining repeated names and omitting zero counts.
    pub fn set_fallbacks(&mut self, hits: impl IntoIterator<Item = ReplayFallbackHit>) {
        let mut counts = BTreeMap::<ReplayByteString, i64>::new();
        for hit in hits {
            if hit.hits > 0 {
                let count = counts.entry(hit.name).or_default();
                *count = count.wrapping_add(hit.hits);
            }
        }
        self.fallbacks = counts
            .into_iter()
            .filter(|(_, hits)| *hits > 0)
            .map(|(name, hits)| ReplayFallbackHit { name, hits })
            .collect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn layer_publication_threshold_and_accounting_precedence() {
        let mut cov = ReplayLayerCoverage::default();
        assert_eq!(cov.verdict(), "aucune donnée");
        cov.available = 100;
        cov.attached = 65;
        cov.no_slot = 35;
        assert_eq!(cov.verdict(), "partiel : moins des deux tiers rattachés");
        cov.attached = 66;
        cov.no_slot = 34;
        assert_eq!(cov.verdict(), "nominal");
        cov.no_slot = 35;
        assert_eq!(cov.verdict(), "non publiable : fuite dans le comptage");
    }
    #[test]
    fn document_origin_gate_and_final_fallback_report() {
        let mut doc = ReplayDocument {
            coverage: Some(ReplayCoverage::default()),
            ..Default::default()
        };
        doc.detect_kickoff();
        assert!(doc.coverage.as_ref().unwrap().t0_film.is_none());
        doc.content.origin_ms = Some(0);
        doc.content.frame_interval_ms = 100;
        doc.detect_kickoff();
        assert_eq!(
            doc.coverage
                .as_ref()
                .unwrap()
                .t0_film
                .as_ref()
                .unwrap()
                .reason,
            "noMovement"
        );
        doc.set_fallbacks(
            [("z", 2), ("a", 0), ("b", 1), ("z", 3)].map(|(name, hits)| ReplayFallbackHit {
                name: name.into(),
                hits,
            }),
        );
        assert_eq!(
            doc.coverage.as_ref().unwrap().fallbacks,
            vec![
                ReplayFallbackHit {
                    name: "b".into(),
                    hits: 1
                },
                ReplayFallbackHit {
                    name: "z".into(),
                    hits: 5
                }
            ]
        );
        doc.set_fallbacks([]);
        assert!(doc.coverage.as_ref().unwrap().fallbacks.is_empty());
        doc.coverage = None;
        doc.set_fallbacks([ReplayFallbackHit {
            name: "z".into(),
            hits: 1,
        }]);
        assert!(doc.coverage.is_none());
    }
}
