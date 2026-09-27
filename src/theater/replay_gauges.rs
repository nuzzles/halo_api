//! Shared native gauge scaling, thinning and frame-series publication.
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ReplayGaugePoint {
    pub t: i64,
    pub v: f32,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayGaugeSample {
    pub t: i64,
    pub v: u64,
}
pub const REPLAY_GAUGE_ZERO: u64 = 8388607;
pub fn replay_gauge_progress(q: u64) -> f32 {
    if q <= REPLAY_GAUGE_ZERO {
        0.
    } else if q >= REPLAY_GAUGE_ZERO + 83886 {
        1.
    } else {
        ((q - REPLAY_GAUGE_ZERO) as f64 / 83886.) as f32
    }
}
pub fn replay_gauge_gap_frames(interval_ms: i64) -> i64 {
    if interval_ms <= 0 {
        1
    } else {
        (1000 / interval_ms).max(1)
    }
}
pub fn push_replay_gauge_point(out: &mut Vec<ReplayGaugePoint>, p: ReplayGaugePoint) {
    if let Some(last) = out.last_mut().filter(|last| last.t >= p.t) {
        if last.t == p.t {
            *last = p;
        }
        return;
    }
    out.push(p);
}
/// First/last samples are retained. Native thinning compares signed increases,
/// so falling values do not force an emission before the maximum time gap.
pub fn append_replay_gauge_thinned(
    out: &mut Vec<ReplayGaugePoint>,
    samples: &[ReplayGaugeSample],
    t0: i64,
    t1: i64,
    gap: i64,
) -> usize {
    let mut i = samples.partition_point(|s| s.t < t0);
    let mut first = true;
    let mut last_t = 0i64;
    let mut last_m = 0i64;
    while i < samples.len() && samples[i].t <= t1 {
        let s = samples[i];
        let m = (f64::from(replay_gauge_progress(s.v)) * 1000.).round() as i64;
        let last = i + 1 >= samples.len() || samples[i + 1].t > t1;
        if first || last || m.wrapping_sub(last_m) >= 20 || s.t.wrapping_sub(last_t) >= gap {
            push_replay_gauge_point(
                out,
                ReplayGaugePoint {
                    t: s.t,
                    v: m as f32 / 1000.,
                },
            );
            first = false;
            last_t = s.t;
            last_m = m;
        }
        i += 1;
    }
    i
}
pub fn replay_gauge_series(
    samples: &[ReplayGaugeSample],
    windows: &[(i64, i64)],
    gap: i64,
) -> Vec<ReplayGaugePoint> {
    let mut wins = windows.to_vec();
    wins.sort_by_key(|w| w.0);
    let mut out = Vec::new();
    for (t0, t1) in wins {
        let next = append_replay_gauge_thinned(&mut out, samples, t0, t1, gap);
        if let Some(s) = samples.get(next).filter(|s| s.v <= REPLAY_GAUGE_ZERO) {
            push_replay_gauge_point(&mut out, ReplayGaugePoint { t: s.t, v: 0. });
        }
    }
    out
}
