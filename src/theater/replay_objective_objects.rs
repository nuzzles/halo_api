//! Free objective-object lives and the reference's qualified ball publication.
use super::*;
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, collections::BTreeMap};
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct FreeObjectiveSample {
    #[serde(rename = "TUS")]
    pub timestamp_us: u64,
    pub x: f32,
    pub y: f32,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct FreeObjectiveLife {
    #[serde(rename = "ID")]
    pub id: u32,
    pub key: EquipmentLifeKey,
    #[serde(rename = "T0US")]
    pub t0_us: u64,
    #[serde(rename = "T1US")]
    pub t1_us: u64,
    pub pts: Vec<FreeObjectiveSample>,
}
fn cmpf(a: f32, b: f32) -> Ordering {
    a.partial_cmp(&b).unwrap_or(Ordering::Equal)
}
fn free_order(a: &FreeObjectiveLife, b: &FreeObjectiveLife) -> Ordering {
    let head = (a.t0_us, &a.key, a.t1_us, a.id, a.pts.len()).cmp(&(
        b.t0_us,
        &b.key,
        b.t1_us,
        b.id,
        b.pts.len(),
    ));
    if head != Ordering::Equal {
        return head;
    }
    for (a, b) in a.pts.iter().zip(&b.pts) {
        let c = a
            .timestamp_us
            .cmp(&b.timestamp_us)
            .then_with(|| cmpf(a.x, b.x))
            .then_with(|| cmpf(a.y, b.y));
        if c != Ordering::Equal {
            return c;
        }
    }
    Ordering::Equal
}
/// Match each creation to motion within the next reuse of its slot/generation.
/// The last declared ID for each key wins before sorting, as in the native map.
/// Retain raw flag lives even though the current publication gate admits balls only.
pub fn free_objective_lives(
    scanned: bool,
    creations: &[EquipmentCreation],
    tracks: &[WorldObjectTrack],
    labels: &BTreeMap<u32, ReplayLabel>,
) -> Vec<FreeObjectiveLife> {
    free_objective_values(
        scanned,
        creations.iter().map(|c| FreeObjectiveCreation {
            key: EquipmentLifeKey {
                slot: c.slot,
                generation: c.generation,
            },
            timestamp_us: c.timestamp_us,
            id: c.mpp_present[1].then_some(c.mpp_val[1] as u32),
            position: [c.x, c.y],
        }),
        tracks.iter().map(|t| {
            (
                EquipmentLifeKey {
                    slot: t.slot,
                    generation: t.generation,
                },
                t.pts
                    .iter()
                    .map(|p| FreeObjectiveSample {
                        timestamp_us: p.timestamp_us,
                        x: p.x,
                        y: p.y,
                    })
                    .collect(),
            )
        }),
        labels,
    )
}
/// Preserve cached generations, timestamps and low-u32 identity; consume positions
/// through value projections without manufacturing recording provenance.
pub fn free_facts_objective_lives(
    scanned: bool,
    creations: &[FactsEquipmentCreation],
    tracks: &[FactsProjectileTrack],
    labels: &BTreeMap<u32, ReplayLabel>,
) -> Vec<FreeObjectiveLife> {
    free_objective_values(
        scanned,
        creations.iter().map(|c| FreeObjectiveCreation {
            key: EquipmentLifeKey {
                slot: c.slot,
                generation: c.generation,
            },
            timestamp_us: c.timestamp_us,
            id: c.mpp_present[1].then_some(c.mpp_val[1] as u32),
            position: [c.position[0], c.position[1]],
        }),
        tracks.iter().map(|t| {
            (
                EquipmentLifeKey {
                    slot: t.slot,
                    generation: t.generation,
                },
                t.points
                    .as_deref()
                    .unwrap_or(&[])
                    .iter()
                    .map(|p| FreeObjectiveSample {
                        timestamp_us: p.timestamp_us,
                        x: p.position[0],
                        y: p.position[1],
                    })
                    .collect(),
            )
        }),
        labels,
    )
}
struct FreeObjectiveCreation {
    key: EquipmentLifeKey,
    timestamp_us: u64,
    id: Option<u32>,
    position: [f32; 2],
}
fn free_objective_values(
    scanned: bool,
    creations: impl Iterator<Item = FreeObjectiveCreation>,
    tracks: impl Iterator<Item = (EquipmentLifeKey, Vec<FreeObjectiveSample>)>,
    labels: &BTreeMap<u32, ReplayLabel>,
) -> Vec<FreeObjectiveLife> {
    if !scanned || labels.is_empty() {
        return Vec::new();
    }
    let mut groups = BTreeMap::<EquipmentLifeKey, Vec<FreeObjectiveCreation>>::new();
    let mut ids = BTreeMap::new();
    for c in creations {
        let Some(id) =
            c.id.filter(|id| labels.get(id).is_some_and(|l| *l != ReplayLabel::default()))
        else {
            continue;
        };
        ids.insert(c.key, id);
        groups.entry(c.key).or_default().push(c);
    }
    let mut by_key = BTreeMap::<EquipmentLifeKey, Vec<Vec<FreeObjectiveSample>>>::new();
    for (key, points) in tracks {
        by_key.entry(key).or_default().push(points);
    }
    let mut out = Vec::new();
    for (key, mut list) in groups {
        native_sort::sort_by(&mut list, |a, b| a.timestamp_us.cmp(&b.timestamp_us));
        for (i, c) in list.iter().enumerate() {
            let end = list.get(i + 1).map_or(u64::MAX, |c| c.timestamp_us);
            let mut pts = vec![FreeObjectiveSample {
                timestamp_us: c.timestamp_us,
                x: c.position[0],
                y: c.position[1],
            }];
            if let Some(track) = by_key.get(&key).and_then(|tracks| {
                super::ground_weapon_lifetimes::select_life(
                    tracks
                        .iter()
                        .map(|t| (t, t.first().map(|p| p.timestamp_us))),
                    c.timestamp_us,
                    end,
                )
            }) {
                for p in track {
                    if p.timestamp_us > pts.last().unwrap().timestamp_us {
                        pts.push(p.clone());
                    }
                }
            }
            out.push(FreeObjectiveLife {
                id: ids[&key],
                key,
                t0_us: c.timestamp_us,
                t1_us: pts.last().unwrap().timestamp_us,
                pts,
            });
        }
    }
    native_sort::sort_by(&mut out, free_order);
    out
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayObjectivePoint {
    pub t: i64,
    pub x: f32,
    pub y: f32,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayObjectiveLife {
    pub family: String,
    pub en: String,
    pub fr: String,
    pub t0: i64,
    pub t1: i64,
    pub pts: Vec<ReplayObjectivePoint>,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayObjectiveCoverage {
    pub scanned: bool,
    pub declared: usize,
    pub lives: usize,
    pub points: usize,
    pub motionless: usize,
    pub out_of_axis: usize,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReplayObjectiveObjects {
    pub lives: Vec<ReplayObjectiveLife>,
    pub coverage: ReplayObjectiveCoverage,
}
fn frame(time: u64, c: IdentityClock) -> i64 {
    if time >= c.origin_us {
        ((time - c.origin_us) / c.step_us) as i64
    } else {
        (((c.origin_us - time).wrapping_add(c.step_us).wrapping_sub(1) / c.step_us) as i64)
            .wrapping_neg()
    }
}
fn life_order(a: &ReplayObjectiveLife, b: &ReplayObjectiveLife) -> Ordering {
    let head = (a.t0, &a.family, a.t1, &a.en, &a.fr, a.pts.len()).cmp(&(
        b.t0,
        &b.family,
        b.t1,
        &b.en,
        &b.fr,
        b.pts.len(),
    ));
    if head != Ordering::Equal {
        return head;
    }
    for (a, b) in a.pts.iter().zip(&b.pts) {
        let c =
            a.t.cmp(&b.t)
                .then_with(|| cmpf(a.x, b.x))
                .then_with(|| cmpf(a.y, b.y));
        if c != Ordering::Equal {
            return c;
        }
    }
    Ordering::Equal
}
pub struct ReplayObjectiveInput<'a> {
    pub scanned: bool,
    pub creations: &'a [EquipmentCreation],
    pub tracks: &'a [WorldObjectTrack],
    pub labels: &'a BTreeMap<u32, ReplayLabel>,
    pub families: &'a BTreeMap<u32, String>,
}
pub fn build_replay_objective_objects(
    input: ReplayObjectiveInput<'_>,
    clock: IdentityClock,
) -> ReplayObjectiveObjects {
    publish_objective_objects(
        input.scanned,
        input.labels,
        input.families,
        clock,
        |labels| free_objective_lives(true, input.creations, input.tracks, labels),
    )
}
pub struct FactsReplayObjectiveInput<'a> {
    pub scanned: bool,
    pub creations: &'a [FactsEquipmentCreation],
    pub tracks: &'a [FactsProjectileTrack],
    pub labels: &'a BTreeMap<u32, ReplayLabel>,
    pub families: &'a BTreeMap<u32, String>,
}
pub fn build_facts_replay_objective_objects(
    input: FactsReplayObjectiveInput<'_>,
    clock: IdentityClock,
) -> ReplayObjectiveObjects {
    publish_objective_objects(
        input.scanned,
        input.labels,
        input.families,
        clock,
        |labels| free_facts_objective_lives(true, input.creations, input.tracks, labels),
    )
}
fn publish_objective_objects(
    scanned: bool,
    all_labels: &BTreeMap<u32, ReplayLabel>,
    families: &BTreeMap<u32, String>,
    clock: IdentityClock,
    free: impl FnOnce(&BTreeMap<u32, ReplayLabel>) -> Vec<FreeObjectiveLife>,
) -> ReplayObjectiveObjects {
    let labels: BTreeMap<_, _> = families
        .iter()
        .filter(|(_, f)| *f == "ball")
        .filter_map(|(id, _)| {
            all_labels
                .get(id)
                .filter(|l| **l != ReplayLabel::default())
                .map(|l| (*id, l.clone()))
        })
        .collect();
    let mut out = ReplayObjectiveObjects {
        coverage: ReplayObjectiveCoverage {
            scanned,
            declared: labels.len(),
            ..Default::default()
        },
        ..Default::default()
    };
    if !scanned || labels.is_empty() || clock.step_us == 0 {
        return out;
    }
    for life in free(&labels) {
        let t0 = frame(life.t0_us, clock);
        if life.pts.is_empty() || t0 < 0 || t0 >= clock.frame_count {
            out.coverage.out_of_axis += 1;
            continue;
        }
        let mut pts: Vec<ReplayObjectivePoint> = Vec::new();
        for p in life.pts {
            let t = frame(p.timestamp_us, clock);
            if t < 0 || t >= clock.frame_count {
                continue;
            }
            let point = ReplayObjectivePoint { t, x: p.x, y: p.y };
            if let Some(last) = pts.last_mut().filter(|p| p.t == t) {
                *last = point;
            } else {
                pts.push(point);
            }
        }
        if pts.is_empty() {
            out.coverage.out_of_axis += 1;
            continue;
        }
        out.coverage.lives += 1;
        out.coverage.points += pts.len();
        out.coverage.motionless += usize::from(pts.len() == 1);
        let l = &labels[&life.id];
        out.lives.push(ReplayObjectiveLife {
            family: families[&life.id].clone(),
            en: l.en.clone(),
            fr: l.fr.clone(),
            t0: pts[0].t,
            t1: pts.last().unwrap().t,
            pts,
        });
    }
    out.lives.sort_by(life_order);
    out
}
#[allow(dead_code)]
pub(crate) fn build_film_replay_objective_objects(
    film: &LegacyFilm,
    players: &FilmReplayPlayers,
) -> ReplayObjectiveObjects {
    build_film_replay_objective_objects_with_catalog(film, players, &replay_equipment_catalog())
}

#[allow(dead_code)]
pub(crate) fn build_film_replay_objective_objects_with_catalog(
    film: &LegacyFilm,
    players: &FilmReplayPlayers,
    catalog: &ReplayEquipmentCatalog,
) -> ReplayObjectiveObjects {
    let complete = film
        .ground_weapon_creations
        .as_ref()
        .zip(film.ground_object_tracks.get(&42));
    build_replay_objective_objects(
        ReplayObjectiveInput {
            scanned: complete.is_some(),
            creations: complete.map_or(&[], |(c, _)| c.records.as_slice()),
            tracks: complete.map_or(&[], |(_, t)| t.tracks.as_slice()),
            labels: &catalog.objective_labels,
            families: &catalog.objective_families,
        },
        players.clock,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn native_free_objective_lives_and_publication() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/ground-objects-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        for (i, r) in rows.as_array().unwrap().iter().enumerate() {
            let mut creation_rows = r["creations"].clone();
            for c in creation_rows.as_array_mut().unwrap() {
                if c["Mask"].is_null() {
                    c["Mask"] = serde_json::json!([]);
                }
            }
            let creations: Vec<EquipmentCreation> = serde_json::from_value(creation_rows).unwrap();
            let tracks: Vec<WorldObjectTrack> =
                serde_json::from_value(r["tracks"].clone()).unwrap();
            let labels: BTreeMap<u32, ReplayLabel> =
                serde_json::from_value(r["objective_labels"].clone()).unwrap();
            let families: BTreeMap<u32, String> =
                serde_json::from_value(r["objective_families"].clone()).unwrap();
            assert_eq!(
                free_objective_lives(true, &creations, &tracks, &labels),
                serde_json::from_value::<Vec<FreeObjectiveLife>>(r["free_lives"].clone()).unwrap(),
                "raw free lives {i}"
            );
            for (j, p) in r["objective_publications"]
                .as_array()
                .unwrap()
                .iter()
                .enumerate()
            {
                let input = ReplayObjectiveInput {
                    scanned: p["scanned"].as_bool().unwrap(),
                    creations: &creations,
                    tracks: &tracks,
                    labels: &labels,
                    families: &families,
                };
                assert_eq!(
                    build_replay_objective_objects(
                        input,
                        serde_json::from_value(p["clock"].clone()).unwrap()
                    ),
                    serde_json::from_value::<ReplayObjectiveObjects>(p["output"].clone()).unwrap(),
                    "objective publication {i}/{j}"
                );
            }
        }
    }
}
