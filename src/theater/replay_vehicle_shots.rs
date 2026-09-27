//! Second shot attachment pass using published vehicle occupancy.
use super::*;
use std::{
    collections::{BTreeMap, BTreeSet},
    num::NonZeroU64,
};

/// Native clamped document frame; the signed conversion precedes clamping.
pub(super) fn frame(time: u64, origin: u64, step: NonZeroU64, frames: i64) -> i64 {
    if time < origin {
        return 0;
    }
    let t = ((time - origin) / step.get()) as i64;
    if t < 0 {
        0
    } else if t >= frames {
        frames.wrapping_sub(1)
    } else {
        t
    }
}

/// Hold endpoints and interpolate in f32, matching the vehicle displayed by the client.
pub fn replay_vehicle_position_at(track: &ReplayVehicleTrack, frame: i64) -> Option<[f32; 2]> {
    let samples = &track.samples;
    if samples.is_empty() {
        return track.spawn.as_ref().map(|s| [s.x, s.y]);
    }
    // Go sort.Search traversal also specifies behavior for malformed sample order.
    let (mut lo, mut hi) = (0, samples.len());
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        if samples[mid].t >= frame {
            hi = mid;
        } else {
            lo = mid + 1;
        }
    }
    if lo == 0 {
        return Some([samples[0].x, samples[0].y]);
    }
    if lo == samples.len() {
        let s = &samples[lo - 1];
        return Some([s.x, s.y]);
    }
    let (a, b) = (&samples[lo - 1], &samples[lo]);
    if a.t == b.t {
        return Some([b.x, b.y]);
    }
    let f = frame.wrapping_sub(a.t) as f32 / b.t.wrapping_sub(a.t) as f32;
    // The pinned native compiler fuses multiplication and addition in float32.
    Some([(b.x - a.x).mul_add(f, a.x), (b.y - a.y).mul_add(f, a.y)])
}

/// Recover the original orphans once, after vehicle and player publication.
/// Returns a replacement shot verdict only when at least one shot was added.
/// Coverage must belong to these orphans; original rejection counters are moved,
/// never duplicated. The orphan evidence itself remains available to callers.
#[allow(clippy::too_many_arguments)]
pub fn attach_replay_vehicle_shots(
    shots: &mut ReplayShotPublication,
    vehicles: &[ReplayVehicleTrack],
    owners: &BTreeMap<u32, i64>,
    published_slots: &BTreeSet<u32>,
    origin: u64,
    step: NonZeroU64,
    frames: i64,
    coverage: Option<&mut ReplayVehicleCoverage>,
) -> Option<&'static str> {
    let orphans: Vec<_> = shots
        .orphans
        .iter()
        .map(|o| {
            (
                super::replay_shots::ShotEventValue {
                    timestamp_us: o.event.timestamp_us,
                    film_index: i64::from(o.event.film_index),
                    weapon_id: o.event.weapon_id,
                    heading: o.event.aim_heading_degrees(),
                },
                o.reason,
            )
        })
        .collect();
    recover_vehicle_shots(
        &mut shots.shots,
        &mut shots.coverage,
        &orphans,
        vehicles,
        owners,
        published_slots,
        origin,
        step,
        frames,
        coverage,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn recover_vehicle_shots(
    shots: &mut Vec<ReplayShot>,
    shot_coverage: &mut ReplayLayerCoverage,
    orphans: &[(super::replay_shots::ShotEventValue, ReplayShotOrphanReason)],
    vehicles: &[ReplayVehicleTrack],
    owners: &BTreeMap<u32, i64>,
    published_slots: &BTreeSet<u32>,
    origin: u64,
    step: NonZeroU64,
    frames: i64,
    mut coverage: Option<&mut ReplayVehicleCoverage>,
) -> Option<&'static str> {
    if orphans.is_empty() || vehicles.is_empty() {
        return None;
    }
    let mut rides = BTreeMap::<u32, Vec<(usize, &ReplayVehicleRide)>>::new();
    for (i, v) in vehicles.iter().enumerate() {
        for r in &v.rides {
            rides.entry(r.slot).or_default().push((i, r));
        }
    }
    if rides.is_empty() {
        return None;
    }
    let mut added = Vec::new();
    for (event, reason) in orphans {
        let t = frame(event.timestamp_us, origin, step, frames);
        let mut candidates = Vec::new();
        for (&slot, &owner) in owners {
            if owner != event.film_index {
                continue;
            }
            if let Some(rs) = rides.get(&slot) {
                candidates.extend(rs.iter().copied().filter(|(_, r)| t >= r.t0 && t <= r.t1));
            }
        }
        candidates.sort_by_key(|(i, r)| (r.seat.unwrap_or(1 << 30), *i, r.slot));
        let Some(&(index, ride)) = candidates.first() else {
            if let Some(c) = coverage.as_deref_mut() {
                c.shots_no_ride += 1;
            }
            continue;
        };
        if candidates.iter().any(|(i, _)| *i != index) {
            if let Some(c) = coverage.as_deref_mut() {
                c.shots_ambiguous += 1;
            }
            continue;
        }
        let Some([x, y]) = replay_vehicle_position_at(&vehicles[index], t) else {
            if let Some(c) = coverage.as_deref_mut() {
                c.shots_unplaced += 1;
            }
            continue;
        };
        if let Some(c) = coverage.as_deref_mut() {
            c.shots += 1;
        }
        match reason {
            ReplayShotOrphanReason::NoSlot => {
                shot_coverage.no_slot = shot_coverage.no_slot.wrapping_sub(1)
            }
            ReplayShotOrphanReason::OutOfWindow => {
                shot_coverage.out_of_window = shot_coverage.out_of_window.wrapping_sub(1)
            }
        }
        if !published_slots.contains(&ride.slot) {
            shot_coverage.unpublished = shot_coverage.unpublished.wrapping_add(1);
            continue;
        }
        shot_coverage.attached = shot_coverage.attached.wrapping_add(1);
        if let Some(c) = coverage.as_deref_mut()
            && event.weapon_id != 0
            && event.weapon_id as u32 != 0x42C9679F
        {
            c.shots_vehicle_weapon += 1;
        }
        let round = |v: f32, scale: f64| ((f64::from(v) * scale).round() / scale) as f32;
        let h = event
            .heading
            .map(|h| {
                let h = round(h as f32, 10.0);
                if h <= 0.0 { 360.0 } else { h }
            })
            .unwrap_or(0.0);
        added.push(ReplayShot {
            t,
            slot: ride.slot,
            x: round(x, 100.0),
            y: round(y, 100.0),
            h,
            weapon: if event.weapon_id == 0 {
                String::new()
            } else {
                format!("0x{:016X}", event.weapon_id)
            },
            vehicle: Some(vehicles[index].slot),
        });
    }
    if added.is_empty() {
        return None;
    }
    shots.extend(added);
    shots.sort_by_key(|s| s.t);
    let c = shot_coverage;
    Some(if c.available == 0 {
        "aucune donnée"
    } else if !c.balanced() {
        "non publiable : fuite dans le comptage"
    } else if (c.attached as f64) / (c.available as f64) < 0.66 {
        "partiel : moins des deux tiers rattachés"
    } else {
        "nominal"
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Probe {
        track: usize,
        frame: i64,
        position: Option<[f32; 2]>,
    }
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Case {
        vehicles: Vec<ReplayVehicleTrack>,
        owners: BTreeMap<u32, i64>,
        published: BTreeSet<u32>,
        origin: u64,
        step: u64,
        frames: i64,
        input: ReplayShotPublication,
        output: ReplayShotPublication,
        vehicle_coverage: Option<ReplayVehicleCoverage>,
        verdict: Option<String>,
        probes: Vec<Probe>,
    }
    #[test]
    fn native_vehicle_shot_recovery() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/replay-vehicle-shots-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&raw).unwrap();
        for (i, mut c) in cases.into_iter().enumerate() {
            for p in c.probes {
                assert_eq!(
                    replay_vehicle_position_at(&c.vehicles[p.track], p.frame),
                    p.position,
                    "probe {i}/{}",
                    p.frame
                );
            }
            let mut coverage = c
                .vehicle_coverage
                .as_ref()
                .map(|_| ReplayVehicleCoverage::default());
            let verdict = attach_replay_vehicle_shots(
                &mut c.input,
                &c.vehicles,
                &c.owners,
                &c.published,
                c.origin,
                NonZeroU64::new(c.step).unwrap(),
                c.frames,
                coverage.as_mut(),
            );
            assert_eq!(c.input, c.output, "shots {i}");
            assert_eq!(coverage, c.vehicle_coverage, "coverage {i}");
            assert_eq!(verdict, c.verdict.as_deref(), "verdict {i}");
            assert!(c.input.coverage.balanced(), "balance {i}");
        }
    }
}
