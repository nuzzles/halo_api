//! Versioned objective catalog, selected only by map asset ID.
use super::*;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, io::Read, sync::OnceLock};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MapObjective {
    pub role: String,
    pub type_id: i32,
    pub pos: ObjectiveVec3,
    pub forward: ObjectiveVec3,
    pub team_index: i64,
    pub instance_id: i32,
    pub labels: Vec<String>,
    #[serde(rename = "unresolved_labels", skip_serializing_if = "Vec::is_empty")]
    pub unresolved: Vec<i32>,
    #[serde(rename = "object_index")]
    pub object_idx: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shape: Option<ObjectiveShape>,
}
impl MapObjective {
    pub fn is_ctf_neutral(&self) -> bool {
        self.labels.iter().any(|l| l == "ctf_neutral_include")
    }
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MapObjectivesEntry {
    pub map_id: String,
    pub version_id: String,
    pub public_name: String,
    pub mvar_file: String,
    pub module: String,
    pub level_id: i64,
    pub objects_n: usize,
    pub objectives: Vec<MapObjective>,
    pub carried_from_schema: i64,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MapObjectivesCatalog {
    pub schema_version: i64,
    pub title_slug: String,
    pub generated_at: String,
    pub maps: BTreeMap<String, MapObjectivesEntry>,
}
impl MapObjectivesCatalog {
    pub fn from_json(bytes: &[u8]) -> Result<Self, String> {
        let c: Self = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        if c.schema_version != 2 {
            return Err(format!(
                "objective catalog schema {}, expected 2",
                c.schema_version
            ));
        }
        Ok(c)
    }
    pub fn lookup(&self, map_id: &str) -> Option<&MapObjectivesEntry> {
        self.maps.get(map_id)
    }
}
pub fn replay_map_objectives_catalog() -> &'static MapObjectivesCatalog {
    static CATALOG: OnceLock<MapObjectivesCatalog> = OnceLock::new();
    CATALOG.get_or_init(|| {
        let mut bytes = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("reference/map-objectives-v75.json.zlib")[..],
        )
        .read_to_end(&mut bytes)
        .expect("embedded objective catalog decompression");
        MapObjectivesCatalog::from_json(&bytes).expect("embedded objective catalog schema")
    })
}
#[derive(Debug, Clone, PartialEq)]
pub struct ObjectiveZone {
    pub role: String,
    pub instance_id: i32,
    pub object_idx: i64,
    pub team_index: i64,
    pub spatial_rank: usize,
    pub center: ObjectiveVec3,
    pub volume: ObjectiveVolume,
    pub shape: ObjectiveShape,
}
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ObjectiveZoneSet {
    pub zones: Vec<ObjectiveZone>,
    pub pointless: usize,
    pub degenerate: usize,
    pub carried: bool,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PointObjective {
    pub role: String,
    #[serde(rename = "InstanceID")]
    pub instance_id: i32,
    pub object_idx: i64,
    pub team_index: i64,
    pub center: ObjectiveVec3,
    pub neutral: bool,
}
fn spatial(a: ObjectiveVec3, aid: i32, b: ObjectiveVec3, bid: i32) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    if a.x != b.x {
        return if a.x < b.x {
            Ordering::Less
        } else {
            Ordering::Greater
        };
    }
    if a.y != b.y {
        return if a.y < b.y {
            Ordering::Less
        } else {
            Ordering::Greater
        };
    }
    if a.z != b.z {
        return if a.z < b.z {
            Ordering::Less
        } else {
            Ordering::Greater
        };
    }
    aid.cmp(&bid)
}
impl MapObjectivesEntry {
    pub fn zones_of_role(&self, role: &str) -> ObjectiveZoneSet {
        let mut out = ObjectiveZoneSet {
            carried: self.carried_from_schema != 0,
            ..Default::default()
        };
        for o in self.objectives.iter().filter(|o| o.role == role) {
            match ObjectiveVolume::new(o.pos, o.shape.as_ref()) {
                Err(ObjectiveVolumeError::NoShape) => out.pointless += 1,
                Err(_) => out.degenerate += 1,
                Ok(volume) => out.zones.push(ObjectiveZone {
                    role: o.role.clone(),
                    instance_id: o.instance_id,
                    object_idx: o.object_idx,
                    team_index: o.team_index,
                    spatial_rank: 0,
                    center: o.pos,
                    volume,
                    shape: o.shape.clone().unwrap(),
                }),
            }
        }
        out.zones
            .sort_by(|a, b| spatial(a.center, a.instance_id, b.center, b.instance_id));
        for (i, z) in out.zones.iter_mut().enumerate() {
            z.spatial_rank = i;
        }
        out
    }
    pub fn points_of_role(&self, role: &str) -> Vec<PointObjective> {
        let mut out: Vec<_> = self
            .objectives
            .iter()
            .filter(|o| o.role == role && o.shape.is_none())
            .map(|o| PointObjective {
                role: o.role.clone(),
                instance_id: o.instance_id,
                object_idx: o.object_idx,
                team_index: o.team_index,
                center: o.pos,
                neutral: o.is_ctf_neutral(),
            })
            .collect();
        out.sort_by(|a, b| spatial(a.center, a.instance_id, b.center, b.instance_id));
        out
    }
    pub fn flag_spawns(&self) -> Vec<ReplayFlagSpawn> {
        self.points_of_role("flag_spawn")
            .into_iter()
            .map(|p| ReplayFlagSpawn {
                team: if p.neutral { -1 } else { p.team_index },
                neutral: p.neutral,
                x: p.center.x as f32,
                y: p.center.y as f32,
            })
            .collect()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_objective_catalog_geometry_and_projection() {
        let mut bytes = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/map-objectives-v41.json.zlib")[..],
        )
        .read_to_end(&mut bytes)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&bytes).unwrap();
        for (i, row) in rows.iter().enumerate() {
            let entry: MapObjectivesEntry = serde_json::from_value(row["entry"].clone()).unwrap();
            if entry.map_id != "synthetic" {
                assert_eq!(
                    replay_map_objectives_catalog().lookup(&entry.map_id),
                    Some(&entry),
                    "embedded entry {i}"
                );
            }
            for r in row["roles"].as_array().unwrap() {
                let role = r["role"].as_str().unwrap();
                let set = entry.zones_of_role(role);
                assert_eq!(
                    set.pointless,
                    r["pointless"].as_u64().unwrap() as usize,
                    "pointless {i}/{role}"
                );
                assert_eq!(
                    set.degenerate,
                    r["degenerate"].as_u64().unwrap() as usize,
                    "degenerate {i}/{role}"
                );
                assert_eq!(set.carried, r["carried"].as_bool().unwrap());
                assert_eq!(
                    entry.points_of_role(role),
                    serde_json::from_value::<Vec<PointObjective>>(r["points"].clone()).unwrap(),
                    "points {i}/{role}"
                );
                let expected = r["zones"].as_array().unwrap();
                assert_eq!(set.zones.len(), expected.len());
                for (z, e) in set.zones.iter().zip(expected) {
                    assert_eq!(z.object_idx, e["object_idx"].as_i64().unwrap());
                    assert_eq!(
                        z.instance_id,
                        i32::try_from(e["instance_id"].as_i64().unwrap()).unwrap()
                    );
                    assert_eq!(z.spatial_rank, e["rank"].as_u64().unwrap() as usize);
                    for p in e["probes"].as_array().unwrap() {
                        let point: ObjectiveVec3 =
                            serde_json::from_value(p["point"].clone()).unwrap();
                        assert_eq!(
                            z.volume.contains(point),
                            p["contains"].as_bool().unwrap(),
                            "contains {i}/{role}, volume {:?}, point {point:?}",
                            z.volume
                        );
                        let actual = z.volume.distance_to(point);
                        if let Some(d) = p["distance"].as_f64() {
                            assert!(
                                (actual - d).abs() <= 1e-12 * d.abs().max(1.),
                                "distance {i}/{role}: {actual} != {d}"
                            );
                        } else {
                            assert_eq!(actual, f64::INFINITY);
                        }
                    }
                }
            }
            for p in row["projections"].as_array().unwrap() {
                let specs: Vec<ObjectiveRoleSpec> =
                    serde_json::from_value(p["specs"].clone()).unwrap();
                assert_eq!(
                    build_replay_map_objectives(&entry, &specs),
                    serde_json::from_value(p["output"].clone()).unwrap(),
                    "projection {i}"
                );
            }
        }
        assert!(
            replay_map_objectives_catalog()
                .lookup("not-a-map-id")
                .is_none()
        );
        assert!(
            MapObjectivesCatalog::from_json(
                br#"{"schema_version":1,"title_slug":"x","generated_at":"","maps":{}}"#
            )
            .is_err()
        );
    }
}
