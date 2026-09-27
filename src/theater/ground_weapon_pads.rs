//! Native spatial recurrence and respawn-cycle measurements for ground objects.
use super::{GroundObjectAppearance, GroundPickupObject, GroundWeaponPickupStatus, native_sort};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GroundPadCluster {
    pub kind: String,
    pub family: String,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    #[serde(rename = "TS")]
    pub times_us: Vec<u64>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroundPadClustering {
    pub clusters: Vec<GroundPadCluster>,
    pub assignments: Vec<usize>,
}
/// Group by kind and canonical family, choosing the nearest centroid within 1m.
/// Assignments retain input indices even though clusters are returned in sorted order.
pub fn cluster_ground_pads(appearances: &[GroundObjectAppearance]) -> GroundPadClustering {
    let mut order: Vec<_> = appearances.iter().cloned().enumerate().collect();
    native_sort::sort_by(&mut order, |a, b| {
        super::ground_weapon_objects::appearance_order(&a.1, &b.1)
    });
    let mut clusters = Vec::<GroundPadCluster>::new();
    let mut assignments = vec![0; appearances.len()];
    for (src, a) in order {
        let mut best = None;
        let mut best_d = f64::INFINITY;
        for (i, c) in clusters.iter().enumerate() {
            if c.kind != a.kind || c.family != a.family {
                continue;
            }
            let dx = (c.x - a.x) as f64;
            let dy = (c.y - a.y) as f64;
            let dz = (c.z - a.z) as f64;
            let d = (dx * dx + dy * dy + dz * dz).sqrt();
            if d <= 1. && d < best_d {
                best = Some(i);
                best_d = d;
            }
        }
        if let Some(i) = best {
            let c = &mut clusters[i];
            let n = c.times_us.len() as f32;
            c.x = c.x.mul_add(n, a.x) / (n + 1.);
            c.y = c.y.mul_add(n, a.y) / (n + 1.);
            c.z = c.z.mul_add(n, a.z) / (n + 1.);
            c.times_us.push(a.timestamp_us);
            assignments[src] = i;
        } else {
            assignments[src] = clusters.len();
            clusters.push(GroundPadCluster {
                kind: a.kind,
                family: a.family,
                x: a.x,
                y: a.y,
                z: a.z,
                times_us: vec![a.timestamp_us],
            });
        }
    }
    for c in &mut clusters {
        c.times_us.sort_unstable();
    }
    let mut order: Vec<_> = clusters.into_iter().enumerate().collect();
    native_sort::sort_by(&mut order, |a, b| {
        let a = &a.1;
        let b = &b.1;
        a.kind
            .cmp(&b.kind)
            .then(a.family.cmp(&b.family))
            .then_with(|| a.x.partial_cmp(&b.x).unwrap())
            .then_with(|| a.y.partial_cmp(&b.y).unwrap())
            .then_with(|| a.z.partial_cmp(&b.z).unwrap())
    });
    let mut inverse = vec![0; order.len()];
    for (new, (old, _)) in order.iter().enumerate() {
        inverse[*old] = new;
    }
    for a in &mut assignments {
        *a = inverse[*a];
    }
    GroundPadClustering {
        clusters: order.into_iter().map(|(_, c)| c).collect(),
        assignments,
    }
}
/// Keep clusters with at least two appearances, retaining their original indices.
pub fn recurring_ground_pads(clusters: &[GroundPadCluster]) -> (Vec<GroundPadCluster>, Vec<usize>) {
    clusters
        .iter()
        .enumerate()
        .filter(|(_, c)| c.times_us.len() >= 2)
        .map(|(i, c)| (c.clone(), i))
        .unzip()
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GroundPadCycle {
    #[serde(rename = "MedianS")]
    pub median_s: f64,
    #[serde(rename = "P10S")]
    pub p10_s: f64,
    #[serde(rename = "P90S")]
    pub p90_s: f64,
    #[serde(rename = "SDS")]
    pub sd_s: f64,
    pub gaps: usize,
    pub established: bool,
}
/// At least two gaps, positive median, and population deviation <=20% of median.
pub fn ground_pad_cycle(gaps: &[f64]) -> GroundPadCycle {
    if gaps.is_empty() {
        return Default::default();
    }
    let mut sorted = gaps.to_vec();
    sorted.sort_by(f64::total_cmp);
    let q = |v: f64| sorted[(v * (sorted.len() - 1) as f64) as usize];
    let mean = gaps.iter().sum::<f64>() / gaps.len() as f64;
    let mut acc = 0.;
    for x in gaps {
        let d = x - mean;
        acc = d.mul_add(d, acc);
    }
    let sd_s = (acc / gaps.len() as f64).sqrt();
    let median_s = q(0.5);
    GroundPadCycle {
        median_s,
        p10_s: q(0.1),
        p90_s: q(0.9),
        sd_s,
        gaps: gaps.len(),
        established: gaps.len() >= 2 && median_s > 0. && sd_s <= 0.2 * median_s,
    }
}
/// Measure from an inferred pickup to the next appearance. Undated occupations
/// and pickups later than the next appearance are counted as missing evidence.
pub fn ground_pad_pickup_gaps(
    objects: &[GroundPickupObject],
    members: &[usize],
) -> (Vec<f64>, usize) {
    let mut order: Vec<_> = members
        .iter()
        .map(|i| (*i, objects[*i].appearance.timestamp_us))
        .collect();
    native_sort::sort_by(&mut order, |a, b| a.1.cmp(&b.1));
    let mut gaps = Vec::new();
    let mut missing = 0;
    for pair in order.windows(2) {
        let prev = &objects[pair[0].0];
        let next = &objects[pair[1].0];
        if prev.status != GroundWeaponPickupStatus::Dated
            || prev.picker.timestamp_us > next.appearance.timestamp_us
        {
            missing += 1;
            continue;
        }
        gaps.push((next.appearance.timestamp_us - prev.picker.timestamp_us) as f64 / 1e6);
    }
    (gaps, missing)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroundPadClock {
    pub origin_us: u64,
    pub step_us: u64,
    pub frames: i64,
}
impl GroundPadClock {
    fn frame(self, t: u64) -> i64 {
        let f = super::replay_equipment_episodes::equipment_episode_frame(
            t,
            self.origin_us,
            self.step_us,
        );
        if f < 0 {
            0
        } else if f >= self.frames {
            self.frames.wrapping_sub(1)
        } else {
            f
        }
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroundPadPresence {
    pub t0: i64,
    pub t_low: i64,
    pub t_high: i64,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroundPadPublishedCycle {
    pub median_s: f32,
    pub p10_s: f32,
    pub p90_s: f32,
    pub gaps: usize,
    pub missing: usize,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroundWeaponPad {
    pub x: f32,
    pub y: f32,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub z: f32,
    pub weapon: String,
    pub spawns: Vec<i64>,
    pub presence: Vec<GroundPadPresence>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cycle: Option<GroundPadPublishedCycle>,
}
fn is_zero(x: &f32) -> bool {
    *x == 0.
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroundPadPickup {
    pub pad: i64,
    pub t_low: i64,
    pub t_high: i64,
    pub xuid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub t: Option<i64>,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroundPadCounts {
    pub dropped: usize,
    pub spawned: usize,
    pub at_rest: usize,
    pub clusters: usize,
    pub pads: usize,
    pub occupancies: usize,
    pub dated: usize,
    pub unknown: usize,
    pub never: usize,
    pub cycles: usize,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroundPadLayer {
    pub pads: Vec<GroundWeaponPad>,
    pub pickups: Vec<GroundPadPickup>,
    pub counts: GroundPadCounts,
}
/// Publish recurrent, stationary appearances on an explicit replay frame grid.
/// Pickup identities and exact times remain empty until joined to native events.
pub fn build_ground_pad_layer(
    objects: &[GroundPickupObject],
    clock: GroundPadClock,
) -> Option<GroundPadLayer> {
    if clock.step_us == 0 {
        return None;
    }
    let mut out = GroundPadLayer {
        pads: Vec::new(),
        pickups: Vec::new(),
        counts: Default::default(),
    };
    let mut app = Vec::new();
    let mut src = Vec::new();
    for (i, o) in objects.iter().enumerate() {
        if o.appearance.class == "dropped" {
            out.counts.dropped += 1;
            continue;
        }
        out.counts.spawned += 1;
        if o.appearance.has_delta {
            continue;
        }
        out.counts.at_rest += 1;
        app.push(o.appearance.clone());
        src.push(i);
    }
    let clustering = cluster_ground_pads(&app);
    out.counts.clusters = clustering.clusters.len();
    let mut members = vec![Vec::new(); clustering.clusters.len()];
    for (i, a) in clustering.assignments.iter().enumerate() {
        members[*a].push(src[i]);
    }
    let (kept, indices) = recurring_ground_pads(&clustering.clusters);
    for (cluster, old_index) in kept.into_iter().zip(indices) {
        let mut ms: Vec<_> = members[old_index]
            .iter()
            .map(|i| (*i, objects[*i].appearance.timestamp_us))
            .collect();
        native_sort::sort_by(&mut ms, |a, b| a.1.cmp(&b.1));
        let first = &objects[ms[0].0];
        let weapon = if first.appearance.kind == "powerup" {
            first.appearance.family.clone()
        } else {
            format!("0x{:08X}", first.family_id)
        };
        let mut pad = GroundWeaponPad {
            x: cluster.x,
            y: cluster.y,
            z: cluster.z,
            weapon,
            spawns: Vec::new(),
            presence: Vec::new(),
            cycle: None,
        };
        out.counts.occupancies += ms.len();
        for (index, _) in ms {
            let o = &objects[index];
            let low = clock.frame(o.bounds.low_us);
            let high = clock.frame(o.bounds.high_us);
            let birth = clock.frame(o.appearance.timestamp_us);
            pad.spawns.push(birth);
            pad.presence.push(GroundPadPresence {
                t0: birth,
                t_low: low,
                t_high: high,
            });
            match o.status {
                GroundWeaponPickupStatus::Never => {
                    out.counts.never += 1;
                    continue;
                }
                GroundWeaponPickupStatus::Dated => out.counts.dated += 1,
                GroundWeaponPickupStatus::Unknown => out.counts.unknown += 1,
            }
            out.pickups.push(GroundPadPickup {
                pad: out.pads.len() as i64,
                t_low: low,
                t_high: high,
                xuid: None,
                t: None,
            });
        }
        let (gaps, missing) = ground_pad_pickup_gaps(objects, &members[old_index]);
        let c = ground_pad_cycle(&gaps);
        if c.established {
            let round = |x: f64| (((x as f32 as f64) * 100.).round() / 100.) as f32;
            pad.cycle = Some(GroundPadPublishedCycle {
                median_s: round(c.median_s),
                p10_s: round(c.p10_s),
                p90_s: round(c.p90_s),
                gaps: c.gaps,
                missing,
            });
            out.counts.cycles += 1;
        }
        out.pads.push(pad);
    }
    out.counts.pads = out.pads.len();
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn native_production_ground_pad_layers() {
        fn read(bytes: &[u8]) -> serde_json::Value {
            let mut raw = Vec::new();
            flate2::read::ZlibDecoder::new(bytes)
                .read_to_end(&mut raw)
                .unwrap();
            serde_json::from_slice(&raw).unwrap()
        }
        let objects = read(include_bytes!(
            "fixtures/ground-objects-corpus-v41.json.zlib"
        ));
        let layers = read(include_bytes!("fixtures/ground-pad-corpus-v41.json.zlib"));
        for row in layers.as_array().unwrap() {
            let source = objects
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["folder"] == row["folder"])
                .unwrap();
            let objects: Vec<GroundPickupObject> =
                serde_json::from_value(source["objects"].clone()).unwrap();
            let clock: GroundPadClock = serde_json::from_value(row["clock"].clone()).unwrap();
            let expected: GroundPadLayer = serde_json::from_value(row["layer"].clone()).unwrap();
            assert_eq!(
                build_ground_pad_layer(&objects, clock).unwrap(),
                expected,
                "{}",
                row["folder"]
            );
        }
    }
    #[test]
    fn native_ground_pad_clustering_and_cycles() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/ground-pads-v41.json.zlib")[..])
            .read_to_end(&mut raw)
            .unwrap();
        let rows: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        for (i, r) in rows.as_array().unwrap().iter().enumerate() {
            let app: Vec<GroundObjectAppearance> =
                serde_json::from_value(r["appearances"].clone()).unwrap();
            let got = cluster_ground_pads(&app);
            let clusters: Vec<GroundPadCluster> =
                serde_json::from_value(r["clusters"].clone()).unwrap();
            assert_eq!(got.clusters, clusters, "clusters {i}");
            let assign: Vec<usize> = serde_json::from_value(r["assignments"].clone()).unwrap();
            assert_eq!(got.assignments, assign, "assign {i}");
            let kept: Vec<GroundPadCluster> = serde_json::from_value(r["kept"].clone()).unwrap();
            let src: Vec<usize> = serde_json::from_value(r["source"].clone()).unwrap();
            assert_eq!(recurring_ground_pads(&clusters), (kept, src), "keep {i}");
            let objects: Vec<GroundPickupObject> =
                serde_json::from_value(r["objects"].clone()).unwrap();
            let layer = build_ground_pad_layer(
                &objects,
                GroundPadClock {
                    origin_us: 2_000_000,
                    step_us: 100_000,
                    frames: 40,
                },
            )
            .unwrap();
            let expected_layer: GroundPadLayer =
                serde_json::from_value(r["layer"].clone()).unwrap();
            assert_eq!(layer, expected_layer, "layer {i}");
            let members: Vec<usize> = serde_json::from_value(r["members"].clone()).unwrap();
            let gaps: Vec<f64> = serde_json::from_value(r["gaps"].clone()).unwrap();
            assert_eq!(
                ground_pad_pickup_gaps(&objects, &members),
                (gaps, r["missing"].as_u64().unwrap() as usize),
                "gaps {i}"
            );
            let cycle_gaps: Vec<f64> = serde_json::from_value(r["cycle_gaps"].clone()).unwrap();
            let actual = ground_pad_cycle(&cycle_gaps);
            let mut expected: GroundPadCycle = serde_json::from_value(r["cycle"].clone()).unwrap();
            let bits: Vec<u64> = serde_json::from_value(r["cycle_bits"].clone()).unwrap();
            expected.median_s = f64::from_bits(bits[0]);
            expected.p10_s = f64::from_bits(bits[1]);
            expected.p90_s = f64::from_bits(bits[2]);
            expected.sd_s = f64::from_bits(bits[3]);
            assert_eq!(actual, expected, "cycle {i}");
        }
    }
}
