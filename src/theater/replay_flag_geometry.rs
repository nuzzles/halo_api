//! Published carrier geometry and free-object drop corrections for flags.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub fn replay_published_track_xuid(
    track: &ReplayTrack,
    bridge: &BTreeMap<u32, u64>,
) -> Option<String> {
    if !track.xuid.is_empty() {
        Some(track.xuid.clone())
    } else {
        bridge
            .get(&track.slot)
            .filter(|&&x| x != 0)
            .map(u64::to_string)
    }
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReplayFlagCarrierTracks {
    pub tracks: BTreeMap<String, Vec<ReplayTrack>>,
    pub ambiguous_slots: Vec<u32>,
    pub without_bridge: usize,
}
pub fn replay_flag_carrier_tracks(
    tracks: &[ReplayTrack],
    bridge: &BTreeMap<u32, u64>,
    ambiguous: &BTreeMap<u32, bool>,
) -> ReplayFlagCarrierTracks {
    let mut named = BTreeMap::<u32, BTreeSet<&str>>::new();
    for t in tracks.iter().filter(|t| !t.xuid.is_empty()) {
        named.entry(t.slot).or_default().insert(&t.xuid);
    }
    let mut out = ReplayFlagCarrierTracks::default();
    let mut refused = BTreeSet::new();
    for t in tracks {
        if t.xuid.is_empty() {
            let contradiction = bridge.get(&t.slot).filter(|&&x| x != 0).is_some_and(|x| {
                named.get(&t.slot).is_some_and(|names| {
                    names.len() > 1
                        || (!names.is_empty() && !names.contains(x.to_string().as_str()))
                })
            });
            if ambiguous.get(&t.slot).copied().unwrap_or(false) || contradiction {
                refused.insert(t.slot);
                continue;
            }
        }
        if let Some(x) = replay_published_track_xuid(t, bridge) {
            out.tracks.entry(x).or_default().push(t.clone());
        } else {
            out.without_bridge += 1;
        }
    }
    out.ambiguous_slots = refused.into_iter().collect();
    out
}
/// First point wins ties, preserving track and point input order. Only a point
/// within one frame is admissible; interpolation is not part of this layer.
pub fn replay_flag_carrier_point(tracks: &[ReplayTrack], frame: i64) -> Option<&ReplayPoint> {
    let mut best = None;
    let mut distance = 0;
    for p in tracks.iter().flat_map(ReplayTrack::points) {
        let d = p.t.wrapping_sub(frame).wrapping_abs();
        if best.is_none() || d < distance {
            best = Some(p);
            distance = d;
        }
    }
    best.filter(|_| distance <= 1)
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayFlagPositionCoverage {
    pub ambiguous_slot: usize,
    pub out_of_window: usize,
    pub no_track: usize,
    pub drop_uses_pickup: usize,
    pub without_bridge: usize,
}
pub fn attach_replay_flag_positions(
    raw: Vec<ReplayFlagCarryRaw>,
    index: &ReplayFlagCarrierTracks,
    clock: ReplayMatchClock,
) -> (Vec<ReplayFlagCarryRaw>, ReplayFlagPositionCoverage) {
    let mut cov = ReplayFlagPositionCoverage {
        ambiguous_slot: index.ambiguous_slots.len(),
        without_bridge: index.without_bridge,
        ..Default::default()
    };
    let mut out = Vec::new();
    for mut r in raw {
        let f0 = clock.frame_of_match_ms(r.t0);
        if f0 < 0 || f0 >= clock.frames {
            cov.out_of_window += 1;
            continue;
        }
        let tracks = index.tracks.get(&r.xuid).map_or(&[][..], Vec::as_slice);
        let Some(p) = replay_flag_carrier_point(tracks, f0) else {
            cov.no_track += 1;
            continue;
        };
        r.x0 = p.x;
        r.y0 = p.y;
        r.x1 = p.x;
        r.y1 = p.y;
        let f = clock.frame_of_match_ms(r.t1);
        let f = if f < 0 {
            0
        } else if f >= clock.frames {
            clock.frames.wrapping_sub(1)
        } else {
            f
        };
        if let Some(p) = replay_flag_carrier_point(tracks, f) {
            r.x1 = p.x;
            r.y1 = p.y;
        } else {
            cov.drop_uses_pickup += 1;
        }
        out.push(r);
    }
    (out, cov)
}
pub(super) fn flag_sq_distance(ax: f32, ay: f32, bx: f32, by: f32) -> f64 {
    let dx = f64::from(ax - bx);
    let dy = f64::from(ay - by);
    dx * dx + dy * dy
}
/// Signed, floor-rounded frame conversion. Like the native replay caller this
/// requires a nonzero step; timestamps before origin produce negative frames.
pub(super) fn flag_frame_of(ts: u64, clock: ReplayMatchClock) -> i64 {
    if ts >= clock.origin_us {
        ((ts - clock.origin_us) / clock.step_us) as i64
    } else {
        (((clock.origin_us - ts)
            .wrapping_add(clock.step_us)
            .wrapping_sub(1)
            / clock.step_us) as i64)
            .wrapping_neg()
    }
}
pub fn replay_flag_free_drop_inside(
    r: &ReplayFlagCarryRaw,
    tracks: &[ReplayTrack],
    lives: &[FreeObjectiveLife],
    spawns: &[ReplayFlagSpawn],
    clock: ReplayMatchClock,
) -> Option<i64> {
    for l in lives {
        let f = flag_frame_of(l.t0_us, clock);
        let at = clock.match_ms_of_frame(f);
        if at <= r.t0 || at >= r.t1 {
            continue;
        }
        let (x, y) = l.pts.first().map_or((0., 0.), |p| (p.x, p.y));
        if replay_flag_spawn_at(spawns, x, y).is_some() {
            continue;
        }
        let Some(p) = replay_flag_carrier_point(tracks, f) else {
            continue;
        };
        if flag_sq_distance(p.x, p.y, x, y) > 2.25 {
            continue;
        }
        return Some(at);
    }
    None
}
pub fn close_replay_flags_by_free_lives(
    raw: &mut [ReplayFlagCarryRaw],
    index: &ReplayFlagCarrierTracks,
    lives: &[FreeObjectiveLife],
    spawns: &[ReplayFlagSpawn],
    clock: ReplayMatchClock,
) {
    for r in raw {
        let tracks = index.tracks.get(&r.xuid).map_or(&[][..], Vec::as_slice);
        if let Some(at) = replay_flag_free_drop_inside(r, tracks, lives, spawns, clock) {
            r.close_at(at, ReplayFlagCloser::Object);
        }
    }
}
pub fn replay_flag_free_at_drop<'a>(
    r: &ReplayFlagCarryRaw,
    lives: &'a [FreeObjectiveLife],
    spawns: &[ReplayFlagSpawn],
    clock: ReplayMatchClock,
) -> Option<&'a FreeObjectiveLife> {
    let mut best = None;
    let mut best_gap = i64::MAX;
    for l in lives {
        let at = clock.match_ms_of_frame(flag_frame_of(l.t0_us, clock));
        let gap = at.wrapping_sub(r.t1).wrapping_abs();
        if gap > 1000 || gap >= best_gap {
            continue;
        }
        let (x, y) = l.pts.first().map_or((0., 0.), |p| (p.x, p.y));
        if replay_flag_spawn_at(spawns, x, y).is_some() || flag_sq_distance(r.x1, r.y1, x, y) > 2.25
        {
            continue;
        }
        best = Some(l);
        best_gap = gap;
    }
    best
}
pub fn reposition_replay_flag_drops(
    raw: &mut [ReplayFlagCarryRaw],
    lives: &[FreeObjectiveLife],
    spawns: &[ReplayFlagSpawn],
    clock: ReplayMatchClock,
) -> usize {
    let mut moved = 0;
    for r in raw.iter_mut().filter(|r| r.closed && !r.ends_home()) {
        if let Some(l) = replay_flag_free_at_drop(r, lives, spawns, clock) {
            let (x, y) = l.pts.last().map_or((0., 0.), |p| (p.x, p.y));
            if x != r.x1 || y != r.y1 {
                r.x1 = x;
                r.y1 = y;
                moved += 1;
            }
        }
    }
    moved
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn native_flag_geometry_and_object_drops() {
        let mut bytes = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/replay-flag-geometry-v41.json.zlib")[..],
        )
        .read_to_end(&mut bytes)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&bytes).unwrap();
        for (i, row) in rows.iter().enumerate() {
            let get = |s: &str| row[s].clone();
            let tracks: Vec<ReplayTrack> = serde_json::from_value(get("tracks")).unwrap();
            let bridge: BTreeMap<u32, u64> = serde_json::from_value(get("bridge")).unwrap();
            let ambiguous: BTreeMap<u32, bool> = serde_json::from_value(get("ambiguous")).unwrap();
            let index = replay_flag_carrier_tracks(&tracks, &bridge, &ambiguous);
            assert_eq!(
                index,
                serde_json::from_value(get("index")).unwrap(),
                "index {i}"
            );
            for p in row["probes"].as_array().unwrap() {
                let tracks = index
                    .tracks
                    .get(p["xuid"].as_str().unwrap())
                    .map_or(&[][..], Vec::as_slice);
                let expected: Option<ReplayPoint> =
                    serde_json::from_value(p["point"].clone()).unwrap();
                assert_eq!(
                    replay_flag_carrier_point(tracks, p["frame"].as_i64().unwrap()),
                    expected.as_ref(),
                    "point {i}"
                );
            }
            let clock: ReplayMatchClock = serde_json::from_value(get("clock")).unwrap();
            let spawns: Vec<ReplayFlagSpawn> = serde_json::from_value(get("spawns")).unwrap();
            let lives: Vec<FreeObjectiveLife> = serde_json::from_value(get("lives")).unwrap();
            let mut raw: Vec<ReplayFlagCarryRaw> = serde_json::from_value(get("raw")).unwrap();
            for (j, r) in raw.iter().enumerate() {
                let tracks = index.tracks.get(&r.xuid).map_or(&[][..], Vec::as_slice);
                assert_eq!(
                    replay_flag_free_drop_inside(r, tracks, &lives, &spawns, clock),
                    row["drops"][j].as_i64(),
                    "drop {i}/{j}"
                );
                assert_eq!(
                    replay_flag_free_at_drop(r, &lives, &spawns, clock).map(|l| u64::from(l.id)),
                    row["matches"][j].as_u64(),
                    "match {i}/{j}"
                );
            }
            close_replay_flags_by_free_lives(&mut raw, &index, &lives, &spawns, clock);
            assert_eq!(
                raw,
                serde_json::from_value::<Vec<ReplayFlagCarryRaw>>(get("closed")).unwrap(),
                "closed {i}"
            );
            let (mut raw, cov) = attach_replay_flag_positions(raw, &index, clock);
            assert_eq!(
                raw,
                serde_json::from_value::<Vec<ReplayFlagCarryRaw>>(get("attached")).unwrap(),
                "attached {i}"
            );
            assert_eq!(
                cov,
                serde_json::from_value(get("position_cov")).unwrap(),
                "coverage {i}"
            );
            let mut assigned: Vec<ReplayFlagCarryRaw> =
                serde_json::from_value(get("assignment_input")).unwrap();
            let assignment_spawns: Vec<ReplayFlagSpawn> =
                serde_json::from_value(get("assignment_spawns")).unwrap();
            let teams: BTreeMap<String, i64> =
                serde_json::from_value(get("assignment_teams")).unwrap();
            let return_times: Vec<i64> = serde_json::from_value(get("return_times")).unwrap();
            let homes = replay_flag_object_homecomings(&lives, &assignment_spawns, clock);
            let assignment_cov = assign_replay_flags(
                &mut assigned,
                &assignment_spawns,
                &teams,
                &return_times,
                &homes,
            );
            assert_eq!(
                assigned,
                serde_json::from_value::<Vec<ReplayFlagCarryRaw>>(get("assigned")).unwrap(),
                "assigned {i}"
            );
            assert_eq!(
                assignment_cov,
                serde_json::from_value(get("assignment_cov")).unwrap(),
                "assignment coverage {i}"
            );
            let marks: CarrierMarkScan = serde_json::from_value(get("marks")).unwrap();
            mark_replay_flag_carries(&mut assigned, &marks, &bridge, clock.death_offset_ms);
            assert_eq!(
                assigned,
                serde_json::from_value::<Vec<ReplayFlagCarryRaw>>(get("marked")).unwrap(),
                "marked {i}"
            );
            assert_eq!(
                tally_replay_flag_carries(&assigned),
                serde_json::from_value(get("tally")).unwrap(),
                "tally {i}"
            );
            let returns: Vec<ReplayFlagHomecoming> =
                serde_json::from_value(get("return_rows")).unwrap();
            assert_eq!(
                assemble_replay_flag_lives(&assigned, &assignment_spawns, &returns, &homes, clock),
                serde_json::from_value(get("flag_lives")).unwrap(),
                "flag lives {i}"
            );
            let full_events: Vec<StatborgNamedEvent> =
                serde_json::from_value(get("full_events")).unwrap();
            let by_slot: BTreeMap<i64, String> =
                serde_json::from_value(get("full_identity")).unwrap();
            let identity = StatborgRoundIdentity {
                publication: IdentityStatborgPublication {
                    by_round: BTreeMap::from([(0, by_slot)]),
                    ..Default::default()
                },
                starts: Vec::new(),
            };
            for (free, prefix) in [(&lives[..], "full"), (&[][..], "empty_free")] {
                let full = build_replay_flags(
                    ReplayFlagScan {
                        scanned: row["full_scanned"].as_bool().unwrap(),
                        signals: serde_json::from_value(get("full_signals")).unwrap(),
                        events: &full_events,
                        identity: &identity,
                        teams: &teams,
                        marks: &marks,
                        spawns: &assignment_spawns,
                        free,
                    },
                    ReplayFlagContext {
                        clock,
                        tracks: &tracks,
                        deaths: &[],
                        bridge: &bridge,
                        ambiguous_slots: &ambiguous,
                    },
                );
                assert_eq!(
                    full.carries,
                    serde_json::from_value::<Vec<ReplayFlagCarry>>(get(&format!(
                        "{prefix}_carries"
                    )))
                    .unwrap(),
                    "full carries {i}"
                );
                assert_eq!(
                    full.coverage,
                    serde_json::from_value(get(&format!("{prefix}_cov"))).unwrap(),
                    "full coverage {i}"
                );
                assert_eq!(
                    full.tracks_without_bridge,
                    row[&format!("{prefix}_no_bridge")].as_u64().unwrap() as usize,
                    "full no bridge {i}"
                );
                assert_eq!(
                    full.drops_using_pickup,
                    row[&format!("{prefix}_drop_pickup")].as_u64().unwrap() as usize,
                    "full pickup fallback {i}"
                );
                assert!(
                    full.coverage
                        .as_ref()
                        .is_none_or(ReplayFlagCoverage::balanced)
                );
            }
            let moved = reposition_replay_flag_drops(&mut raw, &lives, &spawns, clock);
            assert_eq!(moved, row["moved"].as_u64().unwrap() as usize, "moved {i}");
            assert_eq!(
                raw,
                serde_json::from_value::<Vec<ReplayFlagCarryRaw>>(get("repositioned")).unwrap(),
                "repositioned {i}"
            );
        }
    }
}
