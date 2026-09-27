//! Join native vehicle re-creations without erasing written destruction.
use super::{ReplayVehicleRide, ReplayVehicleTrack};

/// Clamp occupancy and aim to the display interval. The native aim fast path
/// inspects endpoints only; preserve that behavior even for malformed aim order.
pub fn clamp_replay_vehicle_rides(
    rides: Vec<ReplayVehicleRide>,
    t0: i64,
    t1_max: i64,
) -> Vec<ReplayVehicleRide> {
    rides
        .into_iter()
        .filter_map(|mut r| {
            if r.t1 < t0 || r.t0 > t1_max {
                return None;
            }
            r.t0 = r.t0.max(t0);
            r.t1 = r.t1.min(t1_max);
            if let (Some(first), Some(last)) = (r.aim.first(), r.aim.last())
                && !(first.t >= r.t0 && last.t <= r.t1)
            {
                r.aim.retain(|a| a.t >= r.t0 && a.t <= r.t1);
            }
            Some(r)
        })
        .collect()
}
fn first_position(t: &ReplayVehicleTrack) -> Option<[f32; 2]> {
    t.samples
        .first()
        .map(|s| [s.x, s.y])
        .or_else(|| t.spawn.as_ref().map(|s| [s.x, s.y]))
}
fn last_position(t: &ReplayVehicleTrack) -> Option<[f32; 2]> {
    t.samples
        .last()
        .map(|s| [s.x, s.y])
        .or_else(|| t.spawn.as_ref().map(|s| [s.x, s.y]))
}
fn is_relay(a: &ReplayVehicleTrack, b: &ReplayVehicleTrack) -> bool {
    if a.end == "destroyed"
        || a.chassis.is_empty()
        || a.chassis != b.chassis
        || b.t0 < a.t1
        || b.t0 > a.t1_max
    {
        return false;
    }
    match (last_position(a), first_position(b)) {
        (Some(a), Some(b)) => super::replay_plan_distance(a, b) <= 0.5,
        _ => false,
    }
}
/// Repeatedly merge the first native relay pair. Input ordering is retained; the
/// caller normally sorts by birth, slot, generation before this pass. A written
/// death forbids merging its life into a replacement at the same spawn point.
pub fn merge_replay_vehicle_relays(
    mut tracks: Vec<ReplayVehicleTrack>,
) -> (Vec<ReplayVehicleTrack>, usize) {
    let mut merged = 0;
    loop {
        let pair = (0..tracks.len()).find_map(|i| {
            (0..tracks.len())
                .find(|&j| i != j && is_relay(&tracks[i], &tracks[j]))
                .map(|j| (i, j))
        });
        let Some((i, j)) = pair else {
            return (tracks, merged);
        };
        let b = tracks[j].clone();
        let a = &mut tracks[i];
        if let Some(last) = a.samples.last() {
            let mut last_t = last.t;
            for s in b.samples {
                if s.t > last_t {
                    last_t = s.t;
                    a.samples.push(s);
                }
            }
        } else {
            a.samples = b.samples;
        }
        a.t1 = b.t1;
        a.t1_max = b.t1_max.max(b.t1);
        a.end = b.end;
        a.t_end = b.t_end;
        let mut rides = std::mem::take(&mut a.rides);
        rides.extend(b.rides);
        rides.sort_by_key(|r| (r.t0, r.slot));
        a.rides = clamp_replay_vehicle_rides(rides, a.t0, a.t1_max);
        if a.spawn.is_none() {
            a.spawn = b.spawn;
        }
        tracks.remove(j);
        merged += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Case {
        input: Vec<ReplayVehicleTrack>,
        output: Vec<ReplayVehicleTrack>,
        merged: usize,
    }
    #[test]
    fn native_vehicle_relay_merging() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/replay-vehicle-relays-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&raw).unwrap();
        for (i, c) in cases.into_iter().enumerate() {
            let (out, merged) = merge_replay_vehicle_relays(c.input);
            assert_eq!(out, c.output, "tracks {i}");
            assert_eq!(merged, c.merged, "merged {i}");
        }
    }
}
