//! Native projectile publication: first point per frame, bounded planar steps.
use super::{WorldObjectTrack, native_sort};
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, collections::BTreeMap, num::NonZeroU64};
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayProjectile {
    pub t0: i64,
    pub p: Vec<[f32; 3]>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub rest: bool,
}
fn is_false(b: &bool) -> bool {
    !*b
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayProjectileCoverage {
    pub tracks: usize,
    pub published: usize,
    pub truncated: usize,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReplayProjectilePublication {
    pub projectiles: Vec<ReplayProjectile>,
    pub published_by_raw: BTreeMap<usize, usize>,
    pub coverage: ReplayProjectileCoverage,
}
/// Keep input point order and suppress consecutive same-frame samples. A planar
/// jump over 10m between rounded points cuts the flight permanently. That cut is
/// counted even if too few grid points remain to publish the trajectory.
pub fn build_replay_projectiles(
    tracks: &[WorldObjectTrack],
    origin_us: u64,
    step_us: NonZeroU64,
) -> ReplayProjectilePublication {
    publish_projectile_points(tracks.iter().map(|t| t.pts.as_slice()), origin_us, step_us)
}
/// Publish cached trajectories without casting their native chunk coordinates.
pub fn build_facts_replay_projectiles(
    tracks: &[super::FactsProjectileTrack],
    origin_us: u64,
    step_us: NonZeroU64,
) -> ReplayProjectilePublication {
    publish_projectile_points(
        tracks
            .iter()
            .map(|t| t.points.as_deref().unwrap_or_default()),
        origin_us,
        step_us,
    )
}
pub(super) trait ProjectilePoint {
    fn timestamp_us(&self) -> u64;
    fn position(&self) -> [f32; 3];
    fn at_rest(&self) -> bool;
}
impl ProjectilePoint for super::WorldObjectSample {
    fn timestamp_us(&self) -> u64 {
        self.timestamp_us
    }
    fn position(&self) -> [f32; 3] {
        [self.x, self.y, self.z]
    }
    fn at_rest(&self) -> bool {
        self.at_rest
    }
}
impl ProjectilePoint for super::FactsProjectileSample {
    fn timestamp_us(&self) -> u64 {
        self.timestamp_us
    }
    fn position(&self) -> [f32; 3] {
        self.position
    }
    fn at_rest(&self) -> bool {
        self.at_rest
    }
}
fn publish_projectile_points<'a, P: ProjectilePoint + 'a>(
    tracks: impl ExactSizeIterator<Item = &'a [P]>,
    origin_us: u64,
    step_us: NonZeroU64,
) -> ReplayProjectilePublication {
    let mut out = ReplayProjectilePublication::default();
    out.coverage.tracks = tracks.len();
    let mut kept = Vec::<(usize, ReplayProjectile)>::new();
    for (raw, points_in) in tracks.enumerate() {
        if points_in.len() < 3 || points_in[0].timestamp_us() < origin_us {
            continue;
        }
        let t0 = ((points_in[0].timestamp_us() - origin_us) / step_us.get()) as i64;
        let mut points = Vec::<[f32; 3]>::new();
        let mut last = -1;
        let mut cut = false;
        for p in points_in {
            if p.timestamp_us() < origin_us {
                continue;
            }
            let frame = ((p.timestamp_us() - origin_us) / step_us.get()) as i64;
            if frame == last {
                continue;
            }
            let round = |v: f32| ((f64::from(v) * 100.0).round() / 100.0) as f32;
            let (x, y) = (round(p.position()[0]), round(p.position()[1]));
            if let Some(previous) = points.last()
                && super::replay_plan_distance([x, y], [previous[1], previous[2]]) > 10.0
            {
                cut = true;
                break;
            }
            last = frame;
            points.push([frame.wrapping_sub(t0) as f32, x, y]);
        }
        if cut {
            out.coverage.truncated += 1;
        }
        if points.len() < 2 {
            continue;
        }
        kept.push((
            raw,
            ReplayProjectile {
                t0,
                p: points,
                rest: !cut && points_in.last().unwrap().at_rest(),
            },
        ));
    }
    native_sort::sort_by(&mut kept, |a, b| {
        let (a_raw, a) = (a.0, &a.1);
        let (b_raw, b) = (b.0, &b.1);
        if a.t0 != b.t0 {
            return a.t0.cmp(&b.t0);
        }
        if a.p[0][1] != b.p[0][1] {
            return a.p[0][1].partial_cmp(&b.p[0][1]).unwrap_or(Ordering::Equal);
        }
        if a.p[0][2] != b.p[0][2] {
            return a.p[0][2].partial_cmp(&b.p[0][2]).unwrap_or(Ordering::Equal);
        }
        a.p.len().cmp(&b.p.len()).then(a_raw.cmp(&b_raw))
    });
    for (index, (raw, p)) in kept.into_iter().enumerate() {
        out.published_by_raw.insert(raw, index);
        out.projectiles.push(p);
    }
    out.coverage.published = out.projectiles.len();
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Case {
        tracks: Vec<WorldObjectTrack>,
        origin: u64,
        step: u64,
        output: ReplayProjectilePublication,
    }
    #[test]
    fn native_projectile_grid_cuts_and_links() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/replay-projectiles-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&raw).unwrap();
        for (i, c) in cases.into_iter().enumerate() {
            assert_eq!(
                build_replay_projectiles(&c.tracks, c.origin, NonZeroU64::new(c.step).unwrap()),
                c.output,
                "case {i}"
            );
        }
    }
}
