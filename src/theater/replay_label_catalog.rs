//! Injected title labels and native weapon identifier publication.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReplayWeaponIconRef {
    pub url: String,
    pub tinted: bool,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReplayVehicleFamilyInfo {
    pub en: String,
    pub fr: String,
    pub kind: String,
    pub sprite: bool,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ReplayLabelCatalog {
    pub weapons: BTreeMap<u32, ReplayWeaponLabel>,
    pub grenades: Vec<ReplayLabel>,
    pub abilities: Vec<ReplayAbilityPalette>,
    pub ability_impulse_families: Vec<String>,
    pub ability_charge_families: Vec<String>,
    pub icons: BTreeMap<u32, ReplayWeaponIconRef>,
    pub keys: BTreeMap<u32, String>,
    pub tints: BTreeMap<String, String>,
    pub roles: BTreeMap<String, String>,
    pub effects: BTreeMap<String, String>,
    pub equipment_families: BTreeMap<u32, String>,
    pub objective_objects: BTreeMap<u32, ReplayLabel>,
    pub objective_families: BTreeMap<u32, String>,
    pub flag_return_zone: ReplayFlagReturnZone,
    pub vehicle_families: BTreeMap<String, ReplayVehicleFamilyInfo>,
}
impl ReplayLabelCatalog {
    pub fn new(
        keys: &BTreeMap<u32, String>,
        names: &BTreeMap<String, ReplayLabel>,
        effects: &BTreeMap<String, String>,
        grenades: Vec<ReplayLabel>,
        abilities: Vec<ReplayAbilityPalette>,
    ) -> Self {
        let weapons = keys
            .iter()
            .filter_map(|(&family, key)| {
                names.get(key).map(|name| {
                    (
                        family,
                        ReplayWeaponLabel {
                            en: name.en.clone(),
                            fr: name.fr.clone(),
                            fx: effects.get(key).cloned().unwrap_or_default(),
                            ..Default::default()
                        },
                    )
                })
            })
            .collect();
        Self {
            weapons,
            keys: keys.clone(),
            effects: effects.clone(),
            grenades,
            abilities,
            ..Default::default()
        }
    }
    /// Native emptiness concerns the three naming tables, regardless of other metadata.
    pub fn is_empty(&self) -> bool {
        self.weapons.is_empty() && self.grenades.is_empty() && self.abilities.is_empty()
    }
}

/// Match native fmt.Sscanf("%X") prefix consumption and original UTF-8 byte-length rule.
/// The native routine accepts trailing text and only strips a lowercase leading `0x`.
pub fn replay_weapon_family(id: &str) -> Option<u32> {
    let raw = id.strip_prefix("0x").unwrap_or(id);
    let raw = raw.trim_start_matches(|c: char| c != '\n' && c.is_whitespace());
    let digits: String = raw.chars().take_while(char::is_ascii_hexdigit).collect();
    if digits.is_empty() {
        return None;
    }
    let value = u64::from_str_radix(&digits, 16).ok()?;
    Some(if id.len() > 10 {
        (value >> 32) as u32
    } else {
        value as u32
    })
}

pub fn build_replay_weapon_labels(
    loadouts: &[ReplayLoadout],
    shots: &[ReplayShot],
    pads: &[GroundWeaponPad],
    catalog: &ReplayLabelCatalog,
) -> BTreeMap<String, ReplayWeaponLabel> {
    let mut out = BTreeMap::new();
    for id in loadouts
        .iter()
        .flat_map(|l| l.w.iter())
        .chain(shots.iter().map(|s| &s.weapon))
        .chain(pads.iter().map(|p| &p.weapon))
    {
        if id.is_empty() || out.contains_key(id) {
            continue;
        }
        let Some(family) = replay_weapon_family(id) else {
            continue;
        };
        let Some(mut label) = catalog.weapons.get(&family).cloned() else {
            continue;
        };
        if let Some(icon) = catalog.icons.get(&family) {
            label.img = icon.url.clone();
            label.tinted = icon.tinted;
        }
        out.insert(id.clone(), label);
    }
    out
}
impl ReplayDocument {
    /// Fill newly known labels while preserving labels already present in the artifact.
    pub fn complete_weapon_labels(&mut self, catalog: &ReplayLabelCatalog) -> usize {
        let labels = build_replay_weapon_labels(
            &self.content.loadouts,
            &self.content.shots,
            &self.content.weapon_pads,
            catalog,
        );
        let mut added = 0;
        for (id, label) in labels {
            if let std::collections::btree_map::Entry::Vacant(e) =
                self.content.weapon_labels.entry(id)
            {
                e.insert(label);
                added += 1;
            }
        }
        added
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    fn inflate(bytes: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        flate2::read::ZlibDecoder::new(bytes)
            .read_to_end(&mut out)
            .unwrap();
        out
    }
    #[test]
    fn native_weapon_family_parsing() {
        #[derive(Deserialize)]
        struct Case {
            id: String,
            family: Option<u32>,
        }
        let rows: Vec<Case> = serde_json::from_slice(&inflate(include_bytes!(
            "fixtures/weapon-family-v41.json.zlib"
        )))
        .unwrap();
        for row in rows {
            assert_eq!(replay_weapon_family(&row.id), row.family, "{:?}", row.id);
        }
    }
    #[test]
    fn native_weapon_label_publication() {
        #[derive(Deserialize)]
        struct Case {
            keys: BTreeMap<u32, String>,
            names: BTreeMap<String, ReplayLabel>,
            effects: BTreeMap<String, String>,
            icons: BTreeMap<u32, ReplayWeaponIconRef>,
            loadouts: Vec<String>,
            shots: Vec<String>,
            pads: Vec<String>,
            weapons: BTreeMap<u32, ReplayWeaponLabel>,
            output: Option<BTreeMap<String, ReplayWeaponLabel>>,
            existing: BTreeMap<String, ReplayWeaponLabel>,
            completed: BTreeMap<String, ReplayWeaponLabel>,
            added: usize,
        }
        let rows: Vec<Case> = serde_json::from_slice(&inflate(include_bytes!(
            "fixtures/weapon-labels-v41.json.zlib"
        )))
        .unwrap();
        for (i, row) in rows.into_iter().enumerate() {
            let mut cat =
                ReplayLabelCatalog::new(&row.keys, &row.names, &row.effects, vec![], vec![]);
            assert_eq!(cat.weapons, row.weapons, "catalog {i}");
            assert_eq!(cat.effects, row.effects);
            assert_eq!(cat.keys, row.keys);
            cat.icons = row.icons;
            let mut doc = ReplayDocument::default();
            doc.content.loadouts = vec![ReplayLoadout {
                t: 0,
                slot: 0,
                w: row.loadouts,
            }];
            doc.content.shots = row
                .shots
                .into_iter()
                .map(|weapon| ReplayShot {
                    t: 0,
                    slot: 0,
                    x: 0.,
                    y: 0.,
                    h: 0.,
                    weapon,
                    vehicle: None,
                })
                .collect();
            doc.content.weapon_pads = row
                .pads
                .into_iter()
                .map(|weapon| GroundWeaponPad {
                    x: 0.,
                    y: 0.,
                    z: 0.,
                    weapon,
                    spawns: vec![],
                    presence: vec![],
                    cycle: None,
                })
                .collect();
            assert_eq!(
                build_replay_weapon_labels(
                    &doc.content.loadouts,
                    &doc.content.shots,
                    &doc.content.weapon_pads,
                    &cat
                ),
                row.output.unwrap_or_default(),
                "publication {i}"
            );
            doc.content.weapon_labels = row.existing;
            assert_eq!(doc.complete_weapon_labels(&cat), row.added, "added {i}");
            assert_eq!(doc.content.weapon_labels, row.completed, "completed {i}");
        }
    }
}
