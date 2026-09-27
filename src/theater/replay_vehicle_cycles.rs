//! Respawn-cycle inference from published vehicle birth locations and dated deaths.
use super::{ReplayVehicleCoverage, ReplayVehicleTrack, ground_pad_cycle};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayVehicleCycle {
    pub x: f32,
    pub y: f32,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub family: String,
    pub median_s: f32,
    pub p10_s: f32,
    pub p90_s: f32,
    pub gaps: usize,
    pub missing: usize,
}
#[derive(Default)]
struct Cluster {
    x: f64,
    y: f64,
    n: usize,
    families: BTreeMap<String, usize>,
    lives: Vec<(i64, i64)>,
}
fn round2(v: f32) -> f32 {
    ((f64::from(v) * 100.).round() / 100.) as f32
}
/// Cluster births within two meters of the current centroid. Equal-distance
/// clusters choose the last one. Only positive destruction-to-birth intervals
/// qualify; two consistent gaps are required by the shared ground-pad rule.
pub fn build_replay_vehicle_cycles(
    tracks: &[ReplayVehicleTrack],
    step_us: u64,
    coverage: &mut ReplayVehicleCoverage,
) -> Vec<ReplayVehicleCycle> {
    if tracks.is_empty() || step_us == 0 {
        return Vec::new();
    }
    let mut order: Vec<_> = tracks.iter().filter(|t| t.spawn.is_some()).collect();
    order.sort_by_key(|t| t.t0);
    let mut clusters = Vec::<Cluster>::new();
    for track in order {
        let spawn = track.spawn.as_ref().unwrap();
        let mut best = None;
        let mut best_distance = 4.;
        for (i, cluster) in clusters.iter().enumerate() {
            let dx = f64::from(spawn.x - (cluster.x / cluster.n as f64) as f32);
            let dy = f64::from(spawn.y - (cluster.y / cluster.n as f64) as f32);
            let distance = dx.mul_add(dx, dy * dy);
            if distance <= best_distance {
                best = Some(i);
                best_distance = distance;
            }
        }
        let index = best.unwrap_or_else(|| {
            clusters.push(Cluster::default());
            clusters.len() - 1
        });
        let cluster = &mut clusters[index];
        cluster.x += f64::from(spawn.x);
        cluster.y += f64::from(spawn.y);
        cluster.n += 1;
        if !track.family.is_empty() {
            *cluster.families.entry(track.family.clone()).or_default() += 1;
        }
        let end = if track.end == "destroyed" {
            track.t_end.unwrap_or(-1)
        } else {
            -1
        };
        cluster.lives.push((track.t0, end));
    }
    coverage.cycle_locations = clusters.len() as i64;
    let sec_per_frame = step_us as f64 / 1e6;
    let mut out = Vec::new();
    for cluster in &mut clusters {
        cluster.lives.sort_by_key(|l| l.0);
        let mut gaps = Vec::new();
        let mut missing = 0;
        for pair in cluster.lives.windows(2) {
            let end = pair[0].1;
            if end < 0 || pair[1].0 <= end {
                missing += 1;
            } else {
                gaps.push(pair[1].0.wrapping_sub(end) as f64 * sec_per_frame);
            }
        }
        coverage.cycle_gaps += gaps.len() as i64;
        coverage.cycle_missing += missing as i64;
        let cycle = ground_pad_cycle(&gaps);
        if !cycle.established {
            continue;
        }
        let mut family = String::new();
        let mut count = 0;
        for (f, &n) in &cluster.families {
            if n > count {
                family = f.clone();
                count = n;
            }
        }
        out.push(ReplayVehicleCycle {
            x: round2((cluster.x / cluster.n as f64) as f32),
            y: round2((cluster.y / cluster.n as f64) as f32),
            family,
            median_s: round2(cycle.median_s as f32),
            p10_s: round2(cycle.p10_s as f32),
            p90_s: round2(cycle.p90_s as f32),
            gaps: cycle.gaps,
            missing,
        });
    }
    out.sort_by(|a, b| {
        a.x.partial_cmp(&b.x)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.y.partial_cmp(&b.y).unwrap_or(std::cmp::Ordering::Equal))
    });
    coverage.cycles = out.len() as i64;
    out
}
