//! Final native layer provenance: records whether each production pass ran.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayLayerInputs {
    pub map_quant: bool,
    pub inventory: bool,
    pub ability_impulses_scanned: bool,
    pub ability_charges_scanned: bool,
    pub score: bool,
    pub flag_scanned: bool,
    pub vip_scanned: bool,
    pub skull_scanned: bool,
    pub bomb_scanned: bool,
    pub bomb_carry_scanned: bool,
    pub zone_scanned: bool,
    pub vehicles_scanned: bool,
}

pub fn replay_layer_revisions() -> BTreeMap<String, String> {
    let decoder = build_replay_decoder_coverage(None);
    [
        ("source", decoder.source_rev),
        ("profile", decoder.profile_rev),
        ("grammar", decoder.grammar_rev),
        ("killsource", decoder.facts_rev),
        (
            "publication",
            format!("publication-{REPLAY_SCHEMA_VERSION}"),
        ),
    ]
    .into_iter()
    .map(|(k, v)| (k.into(), v))
    .collect()
}
pub fn replay_revision_family(revision: &str) -> &str {
    revision.split_once('-').map_or("", |(prefix, _)| prefix)
}
const PUBLICATION: &[&str] = &["schemaVersion", "matchId", "titleSlug", "frameIntervalMs"];
const GRAMMAR: &[&str] = &[
    "frameCount",
    "durationMs",
    "bounds",
    "tracks",
    "originMs",
    "t0FilmMs",
    "shots",
    "loadouts",
    "projectiles",
    "grenades",
    "inventory",
    "grenadeReads",
    "abilities",
    "equipmentChanges",
    "translocations",
    "abilityImpulses",
    "abilityCharges",
    "weaponLabels",
    "abilityLabels",
    "grappleLines",
    "equipmentPlacements",
    "weaponChanges",
    "pickups",
    "weaponPads",
    "padPickups",
    "groundWeapons",
    "objectiveObjects",
    "vehicles",
    "vehicleCycles",
    "stances",
    "zoneStates",
];
const KILLSOURCE: &[&str] = &[
    "identity",
    "roster",
    "neutralDeaths",
    "equipmentEpisodes",
    "objectives",
    "scoreTimeline",
    "flagCarries",
    "flagReturnZone",
    "vipCrown",
    "skullCarries",
    "bombArmings",
    "bombCarries",
    "bombStats",
    "bombEvents",
];

/// Absent options mean a pass did not run, regardless of whether output arrays are empty.
pub fn replay_produced_layers(
    doc: &ReplayDocument,
    input: ReplayLayerInputs,
) -> BTreeMap<String, String> {
    let revisions = replay_layer_revisions();
    let mut out = BTreeMap::new();
    for (family, names) in [
        ("publication", PUBLICATION),
        ("grammar", GRAMMAR),
        ("killsource", KILLSOURCE),
    ] {
        if doc.content.frame_count <= 0 && family != "publication" {
            continue;
        }
        for &name in names {
            let enabled = match name {
                "t0FilmMs" => doc.content.origin_ms.is_some(),
                "grappleLines" => input.map_quant,
                "inventory" => input.inventory,
                "abilityImpulses" => input.ability_impulses_scanned,
                "abilityCharges" => input.ability_charges_scanned,
                "scoreTimeline" => input.score,
                "flagCarries" | "flagReturnZone" => input.flag_scanned,
                "vipCrown" => input.vip_scanned,
                "skullCarries" => input.skull_scanned,
                "bombArmings" => input.bomb_scanned,
                "bombCarries" | "bombStats" | "bombEvents" => input.bomb_carry_scanned,
                "zoneStates" => input.zone_scanned,
                "vehicles" => input.vehicles_scanned,
                _ => true,
            };
            if enabled {
                out.insert(name.into(), revisions[family].clone());
            }
        }
    }
    out
}
impl ReplayDocument {
    /// Run last, after layer output and fallback coverage are final.
    pub fn publish_layers(&mut self, input: ReplayLayerInputs) {
        self.content.layers = replay_produced_layers(self, input);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn native_layer_guards_exhaustive() {
        #[derive(Deserialize)]
        struct Case {
            mask: u32,
            frames: i64,
            layers: BTreeMap<String, String>,
        }
        let mut bytes = Vec::new();
        flate2::read::ZlibDecoder::new(include_bytes!("fixtures/layers-v41.json.zlib").as_slice())
            .read_to_end(&mut bytes)
            .unwrap();
        let rows: Vec<Case> = serde_json::from_slice(&bytes).unwrap();
        for row in rows {
            let m = row.mask;
            let input = ReplayLayerInputs {
                map_quant: m & 2 != 0,
                inventory: m & 4 != 0,
                ability_impulses_scanned: m & 8 != 0,
                ability_charges_scanned: m & 16 != 0,
                score: m & 32 != 0,
                flag_scanned: m & 64 != 0,
                vip_scanned: m & 128 != 0,
                skull_scanned: m & 256 != 0,
                bomb_scanned: m & 512 != 0,
                bomb_carry_scanned: m & 1024 != 0,
                zone_scanned: m & 2048 != 0,
                vehicles_scanned: m & 4096 != 0,
            };
            let mut doc = ReplayDocument::default();
            doc.content.frame_count = row.frames;
            doc.content.origin_ms = (m & 1 != 0).then_some(0);
            doc.publish_layers(input);
            assert_eq!(
                doc.content.layers, row.layers,
                "mask {m}, frames {}",
                row.frames
            );
        }
    }
}
