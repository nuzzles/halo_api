//! Grapple fire/attachment pairing and measured arrival on published player lives.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayGrappleLine {
    pub slot: u32,
    pub t0: i64,
    pub t1: i64,
    pub ax: f32,
    pub ay: f32,
    #[serde(default, skip_serializing_if = "grapple_zero")]
    pub az: f32,
}
fn grapple_zero(v: &f32) -> bool {
    *v == 0.
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayGrappleCoverage {
    pub light_reads: usize,
    pub heavy_reads: usize,
    pub pulls: usize,
    pub pull_lives: usize,
    pub unpaired_fires: usize,
    pub broken_bodies: usize,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReplayGrapple {
    pub lines: Vec<ReplayGrappleLine>,
    pub coverage: ReplayGrappleCoverage,
}
fn grapple_frame(time: u64, origin: u64, step: u64) -> i64 {
    if time >= origin {
        ((time - origin) / step) as i64
    } else {
        (((origin - time).wrapping_add(step).wrapping_sub(1) / step) as i64).wrapping_neg()
    }
}
fn covering<'a>(tracks: &[&'a ReplayTrack], frame: i64) -> Option<&'a ReplayTrack> {
    tracks
        .iter()
        .copied()
        .find(|t| frame >= t.start_frame && frame <= t.end_frame)
}
fn nearest<'a>(tracks: &[&'a ReplayTrack], frame: i64) -> Option<&'a ReplayTrack> {
    tracks.iter().copied().min_by_key(|t| {
        if frame < t.start_frame {
            t.start_frame.wrapping_sub(frame)
        } else if frame > t.end_frame {
            frame.wrapping_sub(t.end_frame)
        } else {
            0
        }
    })
}
/// Match attachments to the preceding fire within 500ms. Attachment-frame life
/// takes precedence, then fire-frame life, then the nearest life in input order.
/// Arrival is the closest published point within 2.5s, with first-point tie wins.
pub fn build_replay_grapple(
    reads: &[GrappleRead],
    map: &FilmMapBounds,
    origin: u64,
    step: u64,
    tracks: &[ReplayTrack],
) -> ReplayGrapple {
    build_grapple_values(
        reads.iter().map(|r| GrappleValue {
            timestamp_us: r.source.timestamp_us,
            slot: r.slot,
            heavy: r.heavy,
            position_quantized: r.position_quantized,
        }),
        map,
        origin,
        step,
        tracks,
    )
}
/// Exact cache grapple readings use the same native pairing and arrival rules.
pub fn build_facts_replay_grapple(
    reads: &[FactsGrappleRead],
    map: &FilmMapBounds,
    origin: u64,
    step: u64,
    tracks: &[ReplayTrack],
) -> ReplayGrapple {
    build_grapple_values(
        reads.iter().map(|r| GrappleValue {
            timestamp_us: r.timestamp_us,
            slot: r.slot,
            heavy: r.heavy,
            position_quantized: r.position_quanta,
        }),
        map,
        origin,
        step,
        tracks,
    )
}
struct GrappleValue {
    timestamp_us: u64,
    slot: u32,
    heavy: bool,
    position_quantized: [u32; 3],
}
fn build_grapple_values(
    reads: impl ExactSizeIterator<Item = GrappleValue>,
    map: &FilmMapBounds,
    origin: u64,
    step: u64,
    tracks: &[ReplayTrack],
) -> ReplayGrapple {
    let mut out = ReplayGrapple::default();
    if reads.len() == 0 || step == 0 {
        return out;
    }
    let mut by_track = BTreeMap::<u32, Vec<&ReplayTrack>>::new();
    for t in tracks {
        by_track.entry(t.slot).or_default().push(t);
    }
    let mut by_slot = BTreeMap::<u32, Vec<GrappleValue>>::new();
    for r in reads {
        if r.heavy {
            out.coverage.heavy_reads += 1;
        } else {
            out.coverage.light_reads += 1;
        }
        by_slot.entry(r.slot).or_default().push(r);
    }
    let mut lives = BTreeSet::new();
    for (slot, mut list) in by_slot {
        list.sort_by_key(|r| r.timestamp_us);
        let tracks = by_track.get(&slot).map_or(&[][..], Vec::as_slice);
        let mut pending: Option<u64> = None;
        for r in list {
            if !r.heavy {
                if pending.is_some() {
                    out.coverage.unpaired_fires += 1;
                }
                pending = Some(r.timestamp_us);
                continue;
            }
            let mut start = r.timestamp_us;
            if let Some(fire) = pending.take() {
                if start - fire <= 500_000 {
                    start = fire;
                } else {
                    out.coverage.unpaired_fires += 1;
                }
            }
            let mut t0 = grapple_frame(start, origin, step);
            let attach = grapple_frame(r.timestamp_us, origin, step);
            let Some(track) = covering(tracks, attach)
                .or_else(|| covering(tracks, t0))
                .or_else(|| nearest(tracks, attach))
            else {
                continue;
            };
            let anchor: [f32; 3] = std::array::from_fn(|i| {
                let width = 1u64.checked_shl(map.axis_widths[i] as u32).unwrap_or(0);
                let scale = (f64::from(map.max[i]) - f64::from(map.min[i])) / (width as f64);
                (scale.mul_add(
                    f64::from(r.position_quantized[i]) + 0.5,
                    f64::from(map.min[i]),
                )) as f32
            });
            let end = attach.wrapping_add((2_500_000 / step) as i64);
            let mut t1 = attach;
            let mut best = -1.;
            for p in track.points() {
                if p.t < attach || p.t > end {
                    continue;
                }
                let dx = p.x - anchor[0];
                let dy = p.y - anchor[1];
                let dz = p.z - anchor[2];
                // Pinned Go/arm64 evaluates x*x, then fused y*y, then fused z*z.
                let d = dz.mul_add(dz, dy.mul_add(dy, dx * dx));
                if best < 0. || d < best {
                    best = d;
                    t1 = p.t;
                }
            }
            t0 = t0.max(track.start_frame);
            t1 = t1.min(track.end_frame);
            if t1 <= t0 {
                continue;
            }
            if let Some(t) = covering(tracks, t0) {
                lives.insert((slot, t.start_frame));
            }
            let round = |v: f32| ((f64::from(v) * 100.).round() / 100.) as f32;
            out.lines.push(ReplayGrappleLine {
                slot,
                t0,
                t1,
                ax: round(anchor[0]),
                ay: round(anchor[1]),
                az: round(anchor[2]),
            });
        }
        if pending.is_some() {
            out.coverage.unpaired_fires += 1;
        }
    }
    out.coverage.pulls = out.lines.len();
    out.coverage.pull_lives = lives.len();
    out.lines.sort_by_key(|l| (l.t0, l.slot));
    out
}
/// Coverage is absent without map geometry. Native publication leaves the broken
/// body counter at zero; scanner diagnostics remain available on LegacyFilm separately.
#[allow(dead_code)]
pub(crate) fn build_film_replay_grapple(
    film: &LegacyFilm,
    players: &FilmReplayPlayers,
) -> Option<ReplayGrapple> {
    let map = film
        .scan_precision
        .as_ref()
        .map(|p| &p.world_map)
        .or_else(|| film.profile.as_ref()?.map.as_ref())?;
    let reads = film
        .ability_states
        .as_ref()
        .map_or(&[][..], |s| s.grapple.as_slice());
    Some(build_replay_grapple(
        reads,
        map,
        players.clock.origin_us,
        players.clock.step_us,
        &players.players.publication.tracks,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[derive(Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct Raw {
        #[serde(rename = "TimestampUS")]
        time: u64,
        slot: u32,
        heavy: bool,
        #[serde(rename = "PosQ")]
        position: [u32; 3],
    }
    #[derive(Deserialize)]
    struct Case {
        raw: Vec<Raw>,
        origin: u64,
        step: u64,
        tracks: Vec<ReplayTrack>,
        map: FilmMapBounds,
        output: ReplayGrapple,
    }
    #[test]
    fn native_grapple_publication() {
        let mut data = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/replay-grapple-v41.json.zlib")[..],
        )
        .read_to_end(&mut data)
        .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&data).unwrap();
        for (i, c) in cases.into_iter().enumerate() {
            let raw: Vec<_> = c
                .raw
                .into_iter()
                .map(|r| GrappleRead {
                    packet_index: None,
                    source: FilmPacket {
                        chunk_index: 1,
                        packet_type: 0,
                        byte_2: 0,
                        byte_3: 0,
                        payload_offset: 0,
                        payload_size: 0,
                        timestamp_us: r.time,
                    },
                    slot: r.slot,
                    heavy: r.heavy,
                    position_quantized: r.position,
                })
                .collect();
            assert_eq!(
                build_replay_grapple(&raw, &c.map, c.origin, c.step, &c.tracks),
                c.output,
                "case {i}"
            );
        }
    }
}
