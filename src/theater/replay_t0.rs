//! Kickoff dating from contiguous movement bursts in the published tracks.
use super::{ReplayT0FilmCoverage, ReplayTrack, native_sort};
use std::collections::{BTreeSet, VecDeque};

/// Return the native kickoff time on the origin clock, or an explicit refusal.
/// Tracks are individual lives; named lives count once per player in the burst.
pub fn detect_replay_t0(
    tracks: &[ReplayTrack],
    frame_interval_ms: i64,
    origin_ms: i64,
) -> (Option<i64>, ReplayT0FilmCoverage) {
    let mut coverage = ReplayT0FilmCoverage::default();
    let refuse = |mut coverage: ReplayT0FilmCoverage, reason: &str| {
        coverage.reason = reason.into();
        (None, coverage)
    };
    if frame_interval_ms <= 0 {
        return refuse(coverage, "noFrameInterval");
    }
    let mut departures = Vec::new();
    for track in tracks {
        if track.points().len() < 2 {
            continue;
        }
        coverage.tracks += 1;
        let mut points = track.points().to_vec();
        native_sort::sort_by(&mut points, |a, b| a.t.cmp(&b.t));
        let mut window = VecDeque::<(i64, f64)>::new();
        let mut sum = 0.0;
        for pair in points.windows(2) {
            let (a, b) = (&pair[0], &pair[1]);
            // Native subtraction happens at float32 precision before promotion.
            let (dx, dy, dz) = ((a.x - b.x) as f64, (a.y - b.y) as f64, (a.z - b.z) as f64);
            let distance = (dx * dx + dy * dy + dz * dz).sqrt();
            if b.t.wrapping_sub(a.t).wrapping_mul(frame_interval_ms) > 1000 || distance > 5.0 {
                window.clear();
                sum = 0.0;
                continue;
            }
            window.push_back((a.t, distance));
            sum += distance;
            while window.len() > 1 && b.t.wrapping_sub(window[0].0) > 1000 / frame_interval_ms {
                sum -= window.pop_front().unwrap().1;
            }
            if sum > 0.5 {
                if b.t >= 0 {
                    departures.push((b.t, track.xuid.as_str()));
                }
                break;
            }
        }
    }
    native_sort::sort_by(&mut departures, |a, b| a.0.cmp(&b.0));
    coverage.moving = departures.len();
    let Some(&(first, _)) = departures.first() else {
        return refuse(coverage, "noMovement");
    };
    coverage.margin_ms = first.wrapping_mul(frame_interval_ms);
    let mut seen = BTreeSet::new();
    for (frame, xuid) in departures {
        if frame
            .wrapping_mul(frame_interval_ms)
            .wrapping_sub(coverage.margin_ms)
            > 1000
        {
            break;
        }
        if xuid.is_empty() || seen.insert(xuid) {
            coverage.burst += 1;
        }
    }
    if coverage.burst < 2 {
        return refuse(coverage, "burstTooSmall");
    }
    if coverage.margin_ms > 120000 {
        return refuse(coverage, "tooLate");
    }
    coverage.detected = true;
    (Some(origin_ms.wrapping_add(coverage.margin_ms)), coverage)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    #[derive(Deserialize)]
    struct Case {
        tracks: Option<Vec<ReplayTrack>>,
        step: i64,
        origin: i64,
        time: Option<i64>,
        coverage: ReplayT0FilmCoverage,
    }
    #[test]
    fn native_kickoff_detection() {
        use std::io::Read;
        let mut bytes = Vec::new();
        flate2::read::ZlibDecoder::new(include_bytes!("fixtures/t0-v41.json.zlib").as_slice())
            .read_to_end(&mut bytes)
            .unwrap();
        let rows: Vec<Case> = serde_json::from_slice(&bytes).unwrap();
        for (i, row) in rows.into_iter().enumerate() {
            let actual = detect_replay_t0(
                row.tracks.as_deref().unwrap_or_default(),
                row.step,
                row.origin,
            );
            assert_eq!(actual, (row.time, row.coverage), "case {i}");
        }
    }
}
