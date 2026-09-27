//! Native life-based sampling and publication of biped replay tracks.
use super::{
    BipedCompanions, IdentityLife, ReplayPlayerPosition, ReplayTrackIdentity,
    build_identity_life_spans,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, num::NonZeroU64};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayPositionSample {
    pub position: ReplayPlayerPosition,
    pub aim: Option<[u32; 2]>,
    pub health: Option<f32>,
    pub shield: Option<f32>,
}
impl ReplayPositionSample {
    pub fn from_companions(position: ReplayPlayerPosition, companions: &BipedCompanions) -> Self {
        Self {
            position,
            aim: companions.aim,
            health: companions.body.as_ref().map(|b| b.health),
            shield: companions.shield.as_ref().map(|s| s.shield),
        }
    }
}
fn zero_f(v: &f32) -> bool {
    *v == 0.0
}
fn zero_i(v: &i64) -> bool {
    *v == 0
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ReplayPoint {
    pub t: i64,
    pub x: f32,
    pub y: f32,
    #[serde(skip_serializing_if = "zero_f")]
    pub z: f32,
    #[serde(skip_serializing_if = "zero_i")]
    pub g: i64,
    #[serde(skip_serializing_if = "zero_f")]
    pub h: f32,
    #[serde(skip_serializing_if = "zero_f")]
    pub p: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sh: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hp: Option<f32>,
    #[serde(skip_serializing_if = "zero_i")]
    pub s: i64,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ReplayTrack {
    pub slot: u32,
    pub team: i64,
    #[serde(skip_serializing_if = "super::ReplayByteString::is_empty")]
    pub name: super::ReplayByteString,
    #[serde(rename = "xuid", skip_serializing_if = "String::is_empty")]
    pub xuid: String,
    #[serde(skip_serializing_if = "super::ReplayByteString::is_empty")]
    pub bot: super::ReplayByteString,
    pub points: Option<Vec<ReplayPoint>>,
    #[serde(skip_serializing_if = "zero_i")]
    pub start_frame: i64,
    #[serde(skip_serializing_if = "zero_i")]
    pub end_frame: i64,
}
impl ReplayTrack {
    /// Borrow recorded points for reducers; native nil slices iterate as empty.
    /// The stored option still distinguishes null from an explicit empty array.
    pub fn points(&self) -> &[ReplayPoint] {
        self.points.as_deref().unwrap_or_default()
    }

    pub fn identity(&self) -> ReplayTrackIdentity {
        ReplayTrackIdentity {
            slot: self.slot,
            start_frame: self.start_frame,
            end_frame: self.end_frame,
            xuid: self.xuid.clone(),
            bot: self.bot.clone(),
        }
    }
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayTrackCoverage {
    pub published: usize,
    pub published_points: usize,
    pub refused_min_points: usize,
    pub refused_points: usize,
    pub min_points: i64,
    pub gaps: usize,
    pub gap_ms: i64,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayTrackPublication {
    pub tracks: Vec<ReplayTrack>,
    pub coverage: ReplayTrackCoverage,
    pub replication_gap_fallbacks: usize,
}
fn rounded(value: f32, scale: f64) -> f32 {
    ((value as f64 * scale).round() / scale) as f32
}
pub fn replay_point(
    sample: &ReplayPositionSample,
    frame: i64,
    gap_ms: i64,
    scoped: i64,
) -> ReplayPoint {
    let pos = &sample.position;
    let mut pt = ReplayPoint {
        t: frame,
        x: rounded(pos.x, 100.0),
        y: rounded(pos.y, 100.0),
        z: rounded(pos.z, 100.0),
        g: gap_ms,
        s: scoped,
        ..Default::default()
    };
    if let Some([yaw, pitch]) = sample.aim {
        let h = super::biped_aim::aim_heading_from_raw(yaw);
        pt.h = rounded(h, 10.0);
        if pt.h <= 0.0 {
            pt.h = 360.0;
        }
        pt.p = rounded(super::biped_aim::aim_pitch_from_raw(pitch), 10.0);
        if pt.p == 0.0 {
            pt.p = 0.0;
        }
    }
    pt.hp = sample.health.map(|v| rounded(v.clamp(0.0, 1.0), 1000.0));
    pt.sh = sample.shield.map(|v| rounded(v.clamp(0.0, 1.0), 1000.0));
    pt
}
/// Samples must be in native input order (normally chronological). A nonzero
/// frame step is required. `min_points` defaults to one in native production.
pub fn publish_replay_tracks(
    samples: &[ReplayPositionSample],
    lives: &[IdentityLife],
    origin: u64,
    step: NonZeroU64,
    min_points: i64,
    scoped: Option<&dyn Fn(u32, u64) -> i64>,
) -> ReplayTrackPublication {
    let step = step.get();
    let fallback;
    let mut fallback_count = 0;
    let lives = if lives.is_empty() {
        fallback = build_identity_life_spans(
            &samples
                .iter()
                .map(|p| p.position.clone())
                .collect::<Vec<_>>(),
        );
        let slots: std::collections::BTreeSet<_> = fallback.iter().map(|l| l.slot).collect();
        fallback_count = fallback.len() - slots.len();
        &fallback
    } else {
        lives
    };
    let mut bounds = BTreeMap::<u32, Vec<&IdentityLife>>::new();
    for l in lives {
        bounds.entry(l.slot).or_default().push(l);
    }
    for spans in bounds.values_mut() {
        spans.sort_by_key(|l| l.from);
    }
    struct Acc {
        done: Vec<Vec<ReplayPoint>>,
        points: Vec<ReplayPoint>,
        last_frame: i64,
        last_us: u64,
        life: i64,
        gap_ms: i64,
    }
    let mut accs = BTreeMap::<u32, Acc>::new();
    let mut order = Vec::new();
    let mut coverage = ReplayTrackCoverage {
        min_points,
        ..Default::default()
    };
    for sample in samples {
        let p = &sample.position;
        if !p.has_world {
            continue;
        }
        let frame = (p.timestamp_us.wrapping_sub(origin) / step) as i64;
        let a = accs.entry(p.slot).or_insert_with(|| {
            order.push(p.slot);
            Acc {
                done: vec![],
                points: vec![],
                last_frame: -1,
                last_us: 0,
                life: -1,
                gap_ms: 0,
            }
        });
        let life = bounds
            .get(&p.slot)
            .and_then(|spans| {
                spans
                    .iter()
                    .enumerate()
                    .skip(a.life.max(0) as usize)
                    .find(|(_, l)| p.timestamp_us as i64 <= l.to)
                    .map(|(i, _)| i as i64)
            })
            .unwrap_or(a.life);
        if !a.points.is_empty() && life != a.life {
            a.done.push(std::mem::take(&mut a.points));
            a.last_frame = -1;
        } else if !a.points.is_empty()
            && (p.timestamp_us as i64).wrapping_sub(a.last_us as i64) > 5_000_000
        {
            let ms = (p.timestamp_us as i64).wrapping_sub(a.last_us as i64) / 1000;
            coverage.gaps += 1;
            coverage.gap_ms = coverage.gap_ms.wrapping_add(ms);
            a.gap_ms = ms;
        }
        a.life = life;
        a.last_us = p.timestamp_us;
        if frame == a.last_frame {
            continue;
        }
        a.last_frame = frame;
        a.points.push(replay_point(
            sample,
            frame,
            a.gap_ms,
            scoped.map_or(0, |s| s(p.slot, p.timestamp_us)),
        ));
        a.gap_ms = 0;
    }
    let mut tracks = Vec::new();
    for slot in order {
        let a = accs.remove(&slot).unwrap();
        for points in a.done.into_iter().chain(std::iter::once(a.points)) {
            if (points.len() as i64) < min_points {
                coverage.refused_min_points += 1;
                coverage.refused_points += points.len();
                continue;
            }
            let start_frame = points
                .first()
                .expect("native track publication requires a nonempty life")
                .t;
            let end_frame = points.last().unwrap().t;
            coverage.published += 1;
            coverage.published_points += points.len();
            tracks.push(ReplayTrack {
                slot,
                team: -1,
                points: Some(points),
                start_frame,
                end_frame,
                ..Default::default()
            });
        }
    }
    ReplayTrackPublication {
        tracks,
        coverage,
        replication_gap_fallbacks: fallback_count,
    }
}

impl ReplayTrackPublication {
    /// Apply the ordered native naming pipeline without changing sampled points
    /// or the coverage of track publication.
    pub fn name_tracks(
        &mut self,
        input: super::IdentityTrackNamingInput<'_>,
    ) -> super::IdentityTrackNamingReport {
        let mut identities: Vec<_> = self.tracks.iter().map(ReplayTrack::identity).collect();
        let report = super::name_replay_identity_tracks(&mut identities, input);
        for (track, identity) in self.tracks.iter_mut().zip(identities) {
            track.xuid = identity.xuid;
            track.bot = identity.bot;
        }
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Case {
        samples: Vec<ReplayPositionSample>,
        lives: Vec<IdentityLife>,
        origin: u64,
        step: NonZeroU64,
        min_points: i64,
        output: ReplayTrackPublication,
    }
    #[test]
    fn native_track_publication() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/replay-tracks-v41.json.zlib")[..])
            .read_to_end(&mut raw)
            .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&raw).unwrap();
        for (i, c) in cases.into_iter().enumerate() {
            let scoped = |slot: u32, ts: u64| ((ts / 100000 + u64::from(slot)) % 4) as i64;
            assert_eq!(
                publish_replay_tracks(
                    &c.samples,
                    &c.lives,
                    c.origin,
                    c.step,
                    c.min_points,
                    Some(&scoped)
                ),
                c.output,
                "tracks {i}"
            );
        }
    }
}
