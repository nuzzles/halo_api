//! Native replay viewing bounds with percentile-based outlier rejection.
use super::{ReplayPoint, ReplayTrack};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ReplayBounds {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
    #[serde(skip_serializing_if = "zero")]
    pub min_z: f32,
    #[serde(skip_serializing_if = "zero")]
    pub max_z: f32,
}
fn zero(v: &f32) -> bool {
    *v == 0.0
}
fn native_min(a: f32, b: f32) -> f32 {
    if a < b { a } else { b }
}
fn native_max(a: f32, b: f32) -> f32 {
    if a > b { a } else { b }
}
fn raw_bounds(tracks: &[ReplayTrack], reject: impl Fn(&ReplayPoint) -> bool) -> ReplayBounds {
    let mut b: Option<ReplayBounds> = None;
    for p in tracks
        .iter()
        .flat_map(ReplayTrack::points)
        .filter(|p| !reject(p))
    {
        if let Some(b) = &mut b {
            b.min_x = native_min(b.min_x, p.x);
            b.max_x = native_max(b.max_x, p.x);
            b.min_y = native_min(b.min_y, p.y);
            b.max_y = native_max(b.max_y, p.y);
            b.min_z = native_min(b.min_z, p.z);
            b.max_z = native_max(b.max_z, p.z);
        } else {
            b = Some(ReplayBounds {
                min_x: p.x,
                max_x: p.x,
                min_y: p.y,
                max_y: p.y,
                min_z: p.z,
                max_z: p.z,
            });
        }
    }
    b.unwrap_or(ReplayBounds {
        min_x: 1.0,
        max_x: -1.0,
        ..Default::default()
    })
}
/// Return bounds and the number of excluded points. Fewer than 200 points use
/// raw bounds. An empty cloud retains the native inverted X extent sentinel.
pub fn replay_bounds(tracks: &[ReplayTrack]) -> (ReplayBounds, usize) {
    let mut axes: [Vec<f32>; 3] = Default::default();
    for p in tracks.iter().flat_map(ReplayTrack::points) {
        for (axis, v) in axes.iter_mut().zip([p.x, p.y, p.z]) {
            axis.push(v);
        }
    }
    let n = axes[0].len();
    if n < 200 {
        return (raw_bounds(tracks, |_| false), 0);
    }
    let guards = axes.map(|mut values| {
        super::native_sort::sort_by(&mut values, |a, b| {
            a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
        });
        let low = values[n / 100];
        let high = values[99 * n / 100];
        let mut spread = high - low;
        if spread < 0.5 {
            spread = 0.5;
        }
        let margin = 12.0 * spread;
        [low - margin, high + margin]
    });
    let rejects = |p: &ReplayPoint| {
        [p.x, p.y, p.z]
            .into_iter()
            .zip(guards)
            .any(|(v, [lo, hi])| v < lo || v > hi)
    };
    let b = raw_bounds(tracks, rejects);
    if b.min_x > b.max_x {
        return (raw_bounds(tracks, |_| false), 0);
    }
    let rejected = tracks
        .iter()
        .flat_map(ReplayTrack::points)
        .filter(|p| rejects(p))
        .count();
    (b, rejected)
}
impl super::ReplayTrackPublication {
    pub fn bounds(&self) -> (ReplayBounds, usize) {
        replay_bounds(&self.tracks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Case {
        tracks: Vec<ReplayTrack>,
        bounds: ReplayBounds,
        rejected: usize,
    }
    #[test]
    fn native_bounds_and_outlier_counts() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/replay-bounds-v41.json.zlib")[..])
            .read_to_end(&mut raw)
            .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&raw).unwrap();
        for (i, c) in cases.into_iter().enumerate() {
            assert_eq!(
                replay_bounds(&c.tracks),
                (c.bounds, c.rejected),
                "bounds {i}"
            );
        }
    }
}
