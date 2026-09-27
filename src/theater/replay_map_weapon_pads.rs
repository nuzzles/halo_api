//! Static map spawners, published only when confirmed by a pad observed in the film.
use super::{GroundWeaponPad, ObjectiveVec3, ReplayMapWeaponPadDTO, ReplayMapWeaponPads};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, io::Read, sync::OnceLock};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MapWeaponPadSpot {
    pub pos: ObjectiveVec3,
    pub type_id: String,
    pub family: String,
    pub objects: usize,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MapSpawnPointSpot {
    pub pos: ObjectiveVec3,
    pub type_id: String,
    pub kind: String,
    pub objects: usize,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MapWeaponPadsEntry {
    pub map_id: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub public_name: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub module: String,
    pub mvar_file: String,
    pub level_id: i32,
    pub objects_n: usize,
    pub pads: Vec<MapWeaponPadSpot>,
    /// None means the spawn-point catalog was not established; Some([]) means none exist.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spawn_points: Option<Vec<MapSpawnPointSpot>>,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MapWeaponPadsCatalog {
    pub schema_version: i64,
    pub title_slug: String,
    pub generated_at: String,
    pub maps: BTreeMap<String, MapWeaponPadsEntry>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub notes: BTreeMap<String, String>,
}
impl MapWeaponPadsCatalog {
    pub fn from_json(bytes: &[u8]) -> Result<Self, String> {
        let catalog: Self = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        if catalog.schema_version != 1 {
            return Err(format!(
                "weapon-pad catalog schema {}, expected 1",
                catalog.schema_version
            ));
        }
        Ok(catalog)
    }
    pub fn lookup(&self, map_id: &str) -> Option<&MapWeaponPadsEntry> {
        self.maps.get(map_id)
    }
    /// Versioned entries take precedence over runtime overlay entries.
    pub fn merge_overlay(&mut self, overlay: Self) {
        for (id, entry) in overlay.maps {
            self.maps.entry(id).or_insert(entry);
        }
    }
}
pub fn replay_map_weapon_pads_catalog() -> &'static MapWeaponPadsCatalog {
    static CATALOG: OnceLock<MapWeaponPadsCatalog> = OnceLock::new();
    CATALOG.get_or_init(|| {
        let mut bytes = Vec::new();
        flate2::read::ZlibDecoder::new(
            include_bytes!("reference/map-weapon-pads-v75.json.zlib").as_slice(),
        )
        .read_to_end(&mut bytes)
        .expect("embedded weapon-pad catalog decompression");
        MapWeaponPadsCatalog::from_json(&bytes).expect("embedded weapon-pad catalog schema")
    })
}

/// Greedy catalog-order confirmation, with the first equal-distance film pad winning.
/// Each film pad confirms at most one catalog spot; distance must be strictly below 1m.
pub fn build_replay_map_weapon_pads(
    entry: &MapWeaponPadsEntry,
    pads: &[GroundWeaponPad],
) -> Option<ReplayMapWeaponPads> {
    let mut taken = vec![false; pads.len()];
    let mut out = ReplayMapWeaponPads {
        pads: Vec::new(),
        catalog_n: entry.pads.len() as i64,
    };
    for spot in &entry.pads {
        let mut best = None;
        let mut best_distance = 1.0;
        for (i, pad) in pads.iter().enumerate() {
            if taken[i] {
                continue;
            }
            let (dx, dy, dz) = (
                spot.pos.x - pad.x as f64,
                spot.pos.y - pad.y as f64,
                spot.pos.z - pad.z as f64,
            );
            let distance = (dx * dx + dy * dy + dz * dz).sqrt();
            if distance < best_distance {
                best = Some(i);
                best_distance = distance;
            }
        }
        if let Some(i) = best {
            taken[i] = true;
            out.pads.push(ReplayMapWeaponPadDTO {
                x: spot.pos.x as f32,
                y: spot.pos.y as f32,
                z: spot.pos.z as f32,
                pad: i as i64,
                family: spot.family.clone(),
            });
        }
    }
    (!out.pads.is_empty()).then_some(out)
}
pub fn replay_map_weapon_pads(
    map_id: &str,
    pads: &[GroundWeaponPad],
) -> Option<ReplayMapWeaponPads> {
    build_replay_map_weapon_pads(replay_map_weapon_pads_catalog().lookup(map_id)?, pads)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Deserialize)]
    struct Case {
        entry: MapWeaponPadsEntry,
        pads: Vec<[f32; 3]>,
        output: Option<ReplayMapWeaponPads>,
    }
    #[test]
    fn native_static_pad_confirmation() {
        let mut bytes = Vec::new();
        flate2::read::ZlibDecoder::new(
            include_bytes!("fixtures/map-weapon-pads-v41.json.zlib").as_slice(),
        )
        .read_to_end(&mut bytes)
        .unwrap();
        let rows: Vec<Case> = serde_json::from_slice(&bytes).unwrap();
        for (i, row) in rows.into_iter().enumerate() {
            let pads: Vec<_> = row
                .pads
                .into_iter()
                .map(|[x, y, z]| GroundWeaponPad {
                    x,
                    y,
                    z,
                    weapon: String::new(),
                    spawns: vec![],
                    presence: vec![],
                    cycle: None,
                })
                .collect();
            assert_eq!(
                build_replay_map_weapon_pads(&row.entry, &pads),
                row.output,
                "case {i}"
            );
        }
    }
    #[test]
    fn static_pad_catalog_preserves_spawn_knowledge_and_overlay_precedence() {
        let catalog = replay_map_weapon_pads_catalog();
        assert_eq!(catalog.maps.len(), 76);
        assert!(catalog.maps.values().any(|e| e.spawn_points.is_none()));
        let empty: MapWeaponPadsEntry = serde_json::from_str(r#"{"spawn_points":[]}"#).unwrap();
        assert_eq!(empty.spawn_points, Some(vec![]));
        assert_eq!(
            serde_json::to_value(&empty).unwrap()["spawn_points"],
            serde_json::json!([])
        );
        assert!(
            serde_json::to_value(MapWeaponPadsEntry::default())
                .unwrap()
                .get("spawn_points")
                .is_none()
        );
        assert!(
            catalog
                .maps
                .values()
                .any(|e| e.spawn_points.as_ref().is_some_and(|p| !p.is_empty()))
        );
        assert!(MapWeaponPadsCatalog::from_json(br#"{"schema_version":2}"#).is_err());
        let mut base = catalog.clone();
        let (id, original) = catalog.maps.first_key_value().unwrap();
        let mut overlay = MapWeaponPadsCatalog::default();
        overlay
            .maps
            .insert(id.clone(), MapWeaponPadsEntry::default());
        overlay
            .maps
            .insert("new".into(), MapWeaponPadsEntry::default());
        base.merge_overlay(overlay);
        assert_eq!(base.lookup(id), Some(original));
        assert!(base.lookup("new").is_some());
        assert!(base.lookup("missing").is_none());
    }
}
