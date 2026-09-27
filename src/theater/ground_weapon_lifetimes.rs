//! Reference ground-weapon lifetime bounds used by replay assembly.
use super::WorldObjectTrack;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GroundWeaponPickupBounds {
    #[serde(rename = "LowUS")]
    pub low_us: u64,
    #[serde(rename = "HighUS")]
    pub high_us: u64,
    pub never_picked: bool,
    #[serde(rename = "NoLaterKF")]
    pub no_later_kf: bool,
    #[serde(rename = "SeenKF")]
    pub seen_kf: usize,
}
/// Restrict sorted sightings to this reuse of a slot/generation, [birth, next birth).
pub fn ground_weapon_seen_within(seen: &[u64], birth: u64, life_end: u64) -> &[u64] {
    let lo = seen.partition_point(|t| *t < birth);
    let hi = seen.partition_point(|t| *t < life_end).max(lo);
    &seen[lo..hi]
}
/// Bound disappearance using sorted keyframe times and already-restricted sightings.
/// A surviving last sighting or lack of later keyframes does not establish a pickup.
pub fn ground_weapon_pickup_bounds(
    birth: u64,
    life_end: u64,
    film_end: u64,
    keyframes: &[u64],
    seen: &[u64],
) -> GroundWeaponPickupBounds {
    let mut b = GroundWeaponPickupBounds {
        low_us: seen.last().copied().unwrap_or(birth),
        high_us: film_end,
        never_picked: false,
        no_later_kf: false,
        seen_kf: seen.len(),
    };
    if !seen.is_empty() && seen.last() == keyframes.last() {
        b.never_picked = true;
        return b;
    }
    if let Some(t) = keyframes.get(keyframes.partition_point(|t| *t <= b.low_us)) {
        b.high_us = *t;
    } else {
        b.no_later_kf = true;
    }
    b.high_us = b.high_us.min(life_end).max(b.low_us);
    b
}
/// Pick the nearest mobile lifetime starting before reuse and no more than the
/// reference's tolerance before birth. Equal time gaps retain input order.
pub fn ground_weapon_life_track(
    tracks: &[WorldObjectTrack],
    birth: u64,
    life_end: u64,
) -> Option<&WorldObjectTrack> {
    select_life(
        tracks
            .iter()
            .map(|t| (t, t.pts.first().map(|p| p.timestamp_us))),
        birth,
        life_end,
    )
}
pub(super) fn select_life<T>(
    tracks: impl Iterator<Item = (T, Option<u64>)>,
    birth: u64,
    life_end: u64,
) -> Option<T> {
    let mut best = None;
    let mut gap = u64::MAX;
    for (track, start) in tracks {
        let Some(start) = start else { continue };
        if start >= life_end || start.wrapping_add(200_000) < birth {
            continue;
        }
        let g = start.abs_diff(birth);
        if g < gap {
            best = Some(track);
            gap = g;
        }
    }
    best
}
/// Only the endpoints consumed by native ground-object lifetime matching.
#[derive(Clone, Copy)]
pub(super) struct GroundMotion {
    pub start: u64,
    pub last: [f32; 3],
}
pub(super) fn source_ground_motion(track: &WorldObjectTrack) -> Option<GroundMotion> {
    let first = track.pts.first()?;
    let last = track.pts.last()?;
    Some(GroundMotion {
        start: first.timestamp_us,
        last: [last.x, last.y, last.z],
    })
}
pub(super) fn facts_ground_motion(track: &super::FactsProjectileTrack) -> Option<GroundMotion> {
    let points = track.points.as_deref().unwrap_or_default();
    Some(GroundMotion {
        start: points.first()?.timestamp_us,
        last: points.last()?.position,
    })
}

/// Position projection used by the reference replay matching rules. Inputs must
/// be sorted by timestamp; unavailable world positions remain explicit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ReplayPlayerPosition {
    pub slot: u32,
    #[serde(rename = "TimestampUS")]
    pub timestamp_us: u64,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub has_world: bool,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GroundWeaponPickupHit {
    pub slot: u32,
    #[serde(rename = "TUS")]
    pub timestamp_us: u64,
    #[serde(rename = "DistM")]
    pub distance_m: f64,
    pub found: bool,
}
/// First passage strictly within 1.5m. At the same timestamp, choose the nearest
/// player, then the lowest slot. The distance is a matching diagnostic.
pub fn ground_weapon_nearest_pass(
    pos: [f32; 3],
    low_us: u64,
    high_us: u64,
    samples: &[ReplayPlayerPosition],
) -> GroundWeaponPickupHit {
    let mut out = GroundWeaponPickupHit::default();
    let start = samples.partition_point(|p| p.timestamp_us < low_us);
    for p in &samples[start..] {
        if p.timestamp_us > high_us {
            break;
        }
        if !p.has_world {
            continue;
        }
        let dx = (pos[0] - p.x) as f64;
        let dy = (pos[1] - p.y) as f64;
        let dz = (pos[2] - p.z) as f64;
        let d = (dx * dx + dy * dy + dz * dz).sqrt();
        if d >= 1.5 {
            continue;
        }
        if !out.found
            || p.timestamp_us < out.timestamp_us
            || (p.timestamp_us == out.timestamp_us
                && (d < out.distance_m || (d == out.distance_m && p.slot < out.slot)))
        {
            out = GroundWeaponPickupHit {
                slot: p.slot,
                timestamp_us: p.timestamp_us,
                distance_m: d,
                found: true,
            };
        }
        if out.found && p.timestamp_us > out.timestamp_us {
            break;
        }
    }
    out
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroundWeaponPickupResolution {
    pub position: [f32; 3],
    pub moved: bool,
    pub bounds: GroundWeaponPickupBounds,
    pub picker: GroundWeaponPickupHit,
    pub status: GroundWeaponPickupStatus,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GroundWeaponPickupStatus {
    Never,
    Unknown,
    Dated,
}
pub struct GroundWeaponResolveInputs<'a> {
    pub keyframe_times: &'a [u64],
    pub seen: &'a [u64],
    /// Tracks already restricted to the creation's slot/generation.
    pub tracks: &'a [WorldObjectTrack],
    pub positions: &'a [ReplayPlayerPosition],
}
/// Match movement and keyframe evidence before looking for a nearby player.
/// A dated result is the reference's inferred passage time, not a wire pickup event.
pub fn resolve_ground_weapon_pickup(
    creation: &super::EquipmentCreation,
    life_end: u64,
    film_end: u64,
    input: GroundWeaponResolveInputs<'_>,
) -> GroundWeaponPickupResolution {
    resolve_ground_values(
        creation.timestamp_us,
        [creation.x, creation.y, creation.z],
        life_end,
        film_end,
        input.tracks.iter().filter_map(source_ground_motion),
        GroundEvidence {
            keyframe_times: input.keyframe_times,
            seen: input.seen,
            positions: input.positions,
        },
    )
}
pub struct FactsGroundWeaponResolveInputs<'a> {
    pub keyframe_times: &'a [u64],
    pub seen: &'a [u64],
    /// Tracks already restricted to the creation's slot/generation.
    pub tracks: &'a [super::FactsProjectileTrack],
    pub positions: &'a [ReplayPlayerPosition],
}
pub fn resolve_facts_ground_weapon_pickup(
    creation: &super::FactsEquipmentCreation,
    life_end: u64,
    film_end: u64,
    input: FactsGroundWeaponResolveInputs<'_>,
) -> GroundWeaponPickupResolution {
    resolve_ground_values(
        creation.timestamp_us,
        creation.position,
        life_end,
        film_end,
        input.tracks.iter().filter_map(facts_ground_motion),
        GroundEvidence {
            keyframe_times: input.keyframe_times,
            seen: input.seen,
            positions: input.positions,
        },
    )
}
pub(super) struct GroundEvidence<'a> {
    pub keyframe_times: &'a [u64],
    pub seen: &'a [u64],
    pub positions: &'a [ReplayPlayerPosition],
}
pub(super) fn resolve_ground_values(
    timestamp: u64,
    initial_position: [f32; 3],
    life_end: u64,
    film_end: u64,
    tracks: impl Iterator<Item = GroundMotion>,
    input: GroundEvidence<'_>,
) -> GroundWeaponPickupResolution {
    let life = select_life(tracks.map(|t| (t, Some(t.start))), timestamp, life_end);
    let position = life.map_or(initial_position, |t| t.last);
    let bounds = ground_weapon_pickup_bounds(
        timestamp,
        life_end,
        film_end,
        input.keyframe_times,
        ground_weapon_seen_within(input.seen, timestamp, life_end),
    );
    let mut out = GroundWeaponPickupResolution {
        position,
        moved: life.is_some(),
        bounds,
        picker: Default::default(),
        status: GroundWeaponPickupStatus::Never,
    };
    if out.bounds.never_picked || out.bounds.no_later_kf {
        return out;
    }
    out.picker = ground_weapon_nearest_pass(
        position,
        out.bounds.low_us,
        out.bounds.high_us,
        input.positions,
    );
    out.status = if out.picker.found {
        GroundWeaponPickupStatus::Dated
    } else {
        GroundWeaponPickupStatus::Unknown
    };
    out
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayPlayerLife {
    pub from: u64,
    pub to: u64,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
/// Split sorted player observations after gaps greater than five seconds.
pub fn replay_player_lives(
    positions: &[ReplayPlayerPosition],
) -> std::collections::BTreeMap<u32, Vec<ReplayPlayerLife>> {
    let mut out = std::collections::BTreeMap::<u32, Vec<ReplayPlayerLife>>::new();
    for p in positions {
        if !p.has_world {
            continue;
        }
        let lives = out.entry(p.slot).or_default();
        if let Some(last) = lives
            .last_mut()
            .filter(|l| p.timestamp_us.wrapping_sub(l.to) <= 5_000_000)
        {
            last.to = p.timestamp_us;
            last.x = p.x;
            last.y = p.y;
            last.z = p.z;
        } else {
            lives.push(ReplayPlayerLife {
                from: p.timestamp_us,
                to: p.timestamp_us,
                x: p.x,
                y: p.y,
                z: p.z,
            });
        }
    }
    out
}
/// Reference inference for a dropped weapon: a player lifetime ends within
/// 200ms and strictly within 1.5m of creation. Lowest matching slot wins.
pub fn ground_weapon_dropper(
    lives: &std::collections::BTreeMap<u32, Vec<ReplayPlayerLife>>,
    pos: [f32; 3],
    at_us: u64,
) -> Option<u32> {
    for (&slot, spans) in lives {
        for life in spans {
            if at_us.abs_diff(life.to) > 200_000 {
                continue;
            }
            let dx = (pos[0] - life.x) as f64;
            let dy = (pos[1] - life.y) as f64;
            let dz = (pos[2] - life.z) as f64;
            if (dx * dx + dy * dy + dz * dz).sqrt() < 1.5 {
                return Some(slot);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn native_ground_lifetime_bounds_and_track_selection() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/ground-lifetimes-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        for (i, r) in rows.as_array().unwrap().iter().enumerate() {
            let birth = r["birth"].as_u64().unwrap();
            let end = r["end"].as_u64().unwrap();
            let film_end = r["film_end"].as_u64().unwrap();
            let seen: Vec<u64> = serde_json::from_value(r["seen"].clone()).unwrap();
            let times: Vec<u64> = serde_json::from_value(r["times"].clone()).unwrap();
            let within = ground_weapon_seen_within(&seen, birth, end);
            let expected_within: Vec<u64> = serde_json::from_value(r["within"].clone()).unwrap();
            assert_eq!(within, expected_within, "within {i}");
            let expected: GroundWeaponPickupBounds =
                serde_json::from_value(r["bounds"].clone()).unwrap();
            assert_eq!(
                ground_weapon_pickup_bounds(birth, end, film_end, &times, within),
                expected,
                "bounds {i}"
            );
            let tracks: Vec<WorldObjectTrack> =
                serde_json::from_value(r["tracks"].clone()).unwrap();
            let pos: [f32; 3] = serde_json::from_value(r["pos"].clone()).unwrap();
            let samples: Vec<ReplayPlayerPosition> =
                serde_json::from_value(r["positions"].clone()).unwrap();
            let lives = replay_player_lives(&samples);
            let expected_lives: std::collections::BTreeMap<u32, Vec<ReplayPlayerLife>> =
                serde_json::from_value(r["lives"].clone()).unwrap();
            assert_eq!(lives, expected_lives, "player lives {i}");
            let dropper = ground_weapon_dropper(&lives, pos, birth);
            assert_eq!(
                dropper.map_or(-1, i64::from),
                r["dropper"].as_i64().unwrap(),
                "dropper {i}"
            );
            assert_eq!(
                if dropper.is_some() {
                    "dropped"
                } else {
                    "spawned"
                },
                r["class"].as_str().unwrap()
            );
            let hit = ground_weapon_nearest_pass(pos, birth, end, &samples);
            let mut expected_hit: GroundWeaponPickupHit =
                serde_json::from_value(r["hit"].clone()).unwrap();
            expected_hit.distance_m = f64::from_bits(r["hit_bits"].as_u64().unwrap());
            assert_eq!(hit, expected_hit, "hit {i}");
            let creation = super::super::EquipmentCreation {
                timestamp_us: birth,
                x: pos[0],
                y: pos[1],
                z: pos[2],
                ..Default::default()
            };
            let resolved = resolve_ground_weapon_pickup(
                &creation,
                end,
                film_end,
                GroundWeaponResolveInputs {
                    keyframe_times: &times,
                    seen: &seen,
                    tracks: &tracks,
                    positions: &samples,
                },
            );
            let mut expected_resolution: GroundWeaponPickupResolution =
                serde_json::from_value(r["resolution"].clone()).unwrap();
            expected_resolution.picker.distance_m =
                f64::from_bits(r["picker_bits"].as_u64().unwrap());
            assert_eq!(resolved, expected_resolution, "resolution {i}");
            let got = ground_weapon_life_track(&tracks, birth, end);
            assert_eq!(got.is_some(), r["found"].as_bool().unwrap(), "found {i}");
            if let Some(got) = got {
                let expected: WorldObjectTrack =
                    serde_json::from_value(r["track"].clone()).unwrap();
                assert_eq!(*got, expected, "track {i}");
            }
        }
    }
}
