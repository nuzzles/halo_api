//! Native shot attachment to undownsampled biped positions and published tracks.
use super::{FilmFireEvent, ReplayPlayerPosition, ReplayTrack, native_sort};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    num::NonZeroU64,
};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayLayerCoverage {
    pub available: i64,
    pub attached: i64,
    pub no_slot: i64,
    pub ambiguous: i64,
    pub out_of_window: i64,
    pub unpublished: i64,
    #[serde(default, skip_serializing_if = "zero")]
    pub refused_by_roster: i64,
}
fn zero(n: &i64) -> bool {
    *n == 0
}
fn zero_f(n: &f32) -> bool {
    *n == 0.0
}
impl ReplayLayerCoverage {
    pub fn verdict(&self) -> &'static str {
        if self.available == 0 {
            "aucune donnée"
        } else if !self.balanced() {
            "non publiable : fuite dans le comptage"
        } else if (self.attached as f64 / self.available as f64) < 0.66 {
            "partiel : moins des deux tiers rattachés"
        } else {
            "nominal"
        }
    }

    pub fn balanced(&self) -> bool {
        self.attached
            .wrapping_add(self.no_slot)
            .wrapping_add(self.ambiguous)
            .wrapping_add(self.out_of_window)
            .wrapping_add(self.unpublished)
            .wrapping_add(self.refused_by_roster)
            == self.available
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayShot {
    pub t: i64,
    pub slot: u32,
    pub x: f32,
    pub y: f32,
    #[serde(default, skip_serializing_if = "zero_f")]
    pub h: f32,
    #[serde(default, rename = "w", skip_serializing_if = "String::is_empty")]
    pub weapon: String,
    #[serde(default, rename = "v", skip_serializing_if = "Option::is_none")]
    pub vehicle: Option<u32>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReplayShotOrphanReason {
    NoSlot,
    OutOfWindow,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayOrphanShot {
    pub event: FilmFireEvent,
    pub reason: ReplayShotOrphanReason,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReplayShotPublication {
    pub shots: Vec<ReplayShot>,
    pub orphans: Vec<ReplayOrphanShot>,
    pub coverage: ReplayLayerCoverage,
}
impl ReplayShotPublication {
    /// The native publication gate checks slot membership, not the track's time
    /// interval. Removed shots are counted but are not vehicle-recovery orphans.
    pub fn retain_published_tracks(&mut self, tracks: &[ReplayTrack]) {
        let slots: BTreeSet<_> = tracks.iter().map(|t| t.slot).collect();
        let before = self.shots.len();
        self.shots.retain(|s| slots.contains(&s.slot));
        self.coverage.unpublished = self
            .coverage
            .unpublished
            .wrapping_add((before - self.shots.len()) as i64);
        self.coverage.attached = self.shots.len() as i64;
    }
}
pub(super) fn nearest(
    points: &[ReplayPlayerPosition],
    time: u64,
) -> (Option<&ReplayPlayerPosition>, u64) {
    let i = points.partition_point(|p| p.timestamp_us < time);
    let mut best = None;
    let mut distance = u64::MAX;
    for p in i
        .checked_sub(1)
        .and_then(|j| points.get(j))
        .into_iter()
        .chain(points.get(i))
    {
        let d = p.timestamp_us.abs_diff(time);
        if d < distance {
            best = Some(p);
            distance = d;
        }
    }
    (best, distance)
}
pub(super) enum AttachmentFailure {
    NoSlot,
    Ambiguous,
}
pub(super) struct PositionIndex(BTreeMap<u32, Vec<ReplayPlayerPosition>>);
impl PositionIndex {
    pub(super) fn new(positions: &[ReplayPlayerPosition]) -> Self {
        let mut tracks = BTreeMap::<u32, Vec<ReplayPlayerPosition>>::new();
        for p in positions {
            tracks.entry(p.slot).or_default().push(p.clone());
        }
        for points in tracks.values_mut() {
            native_sort::sort_by(points, |a, b| a.timestamp_us.cmp(&b.timestamp_us));
        }
        Self(tracks)
    }
    pub(super) fn owned_at(
        &self,
        owners: &BTreeMap<u32, i64>,
        index: i64,
        time: u64,
    ) -> Result<&ReplayPlayerPosition, AttachmentFailure> {
        let mut found = None;
        for (&slot, &owner) in owners {
            if owner != index {
                continue;
            }
            if let Some(points) = self.0.get(&slot) {
                let (point, distance) = nearest(points, time);
                if distance <= 120_000 {
                    if found.is_some() {
                        return Err(AttachmentFailure::Ambiguous);
                    }
                    found = point;
                }
            }
        }
        found.ok_or(AttachmentFailure::NoSlot)
    }
}
/// Attach only when exactly one owned slot has a position within 120ms. Missing
/// world coordinates fail after slot selection, so they can still cause ambiguity.
/// Early missing-input failure produces no recoverable orphans, matching LevelUp.
pub fn build_replay_shots(
    positions: &[ReplayPlayerPosition],
    events: &[FilmFireEvent],
    origin_us: u64,
    step_us: NonZeroU64,
    owners: &BTreeMap<u32, i64>,
) -> ReplayShotPublication {
    let inputs: Vec<_> = events
        .iter()
        .map(|e| ShotEventValue {
            timestamp_us: e.timestamp_us,
            film_index: i64::from(e.film_index),
            weapon_id: e.weapon_id,
            heading: e.aim_heading_degrees(),
        })
        .collect();
    let result = assemble_shots(positions, &inputs, origin_us, step_us, owners);
    ReplayShotPublication {
        shots: result.shots,
        coverage: result.coverage,
        orphans: result
            .orphans
            .into_iter()
            .map(|(index, reason)| ReplayOrphanShot {
                event: events[index].clone(),
                reason,
            })
            .collect(),
    }
}

pub(super) struct ShotEventValue {
    pub timestamp_us: u64,
    pub film_index: i64,
    pub weapon_id: u64,
    pub heading: Option<f64>,
}
#[derive(Default)]
pub(super) struct ShotAssembly {
    pub shots: Vec<ReplayShot>,
    pub coverage: ReplayLayerCoverage,
    pub orphans: Vec<(usize, ReplayShotOrphanReason)>,
}
pub(super) fn assemble_shots(
    positions: &[ReplayPlayerPosition],
    events: &[ShotEventValue],
    origin_us: u64,
    step_us: NonZeroU64,
    owners: &BTreeMap<u32, i64>,
) -> ShotAssembly {
    let mut out = ShotAssembly::default();
    out.coverage.available = events.len() as i64;
    if positions.is_empty() || events.is_empty() || owners.is_empty() {
        out.coverage.no_slot = events.len() as i64;
        return out;
    }
    let tracks = PositionIndex::new(positions);
    for (index, e) in events.iter().enumerate() {
        let p = match tracks.owned_at(owners, e.film_index, e.timestamp_us) {
            Ok(p) => p,
            Err(AttachmentFailure::Ambiguous) => {
                out.coverage.ambiguous = out.coverage.ambiguous.wrapping_add(1);
                continue;
            }
            Err(AttachmentFailure::NoSlot) => {
                out.coverage.no_slot = out.coverage.no_slot.wrapping_add(1);
                out.orphans.push((index, ReplayShotOrphanReason::NoSlot));
                continue;
            }
        };
        if !p.has_world {
            out.coverage.out_of_window = out.coverage.out_of_window.wrapping_add(1);
            out.orphans
                .push((index, ReplayShotOrphanReason::OutOfWindow));
            continue;
        }
        let round = |v: f32, scale: f64| ((f64::from(v) * scale).round() / scale) as f32;
        let heading = e
            .heading
            .map(|h| {
                let r = round(h as f32, 10.0);
                if r <= 0.0 { 360.0 } else { r }
            })
            .unwrap_or(0.0);
        out.shots.push(ReplayShot {
            t: (e.timestamp_us.wrapping_sub(origin_us) / step_us.get()) as i64,
            slot: p.slot,
            x: round(p.x, 100.0),
            y: round(p.y, 100.0),
            h: heading,
            weapon: if e.weapon_id == 0 {
                String::new()
            } else {
                format!("0x{:016X}", e.weapon_id)
            },
            vehicle: None,
        });
        out.coverage.attached = out.coverage.attached.wrapping_add(1);
    }
    native_sort::sort_by(&mut out.shots, |a, b| a.t.cmp(&b.t));
    out
}

/// Attach the film's shot events using the already-built player layer. None means
/// the required position stream or nonzero replay clock is unavailable.
#[allow(dead_code)]
pub(crate) fn build_film_replay_shots(
    film: &super::LegacyFilm,
    players: &super::FilmReplayPlayers,
) -> Option<ReplayShotPublication> {
    let step = NonZeroU64::new(players.clock.step_us)?;
    let mut positions: Vec<_> = film
        .biped_positions
        .as_ref()?
        .accepted()
        .map(|p| {
            let [x, y, z] = p.record.world;
            ReplayPlayerPosition {
                slot: p.record.slot,
                timestamp_us: p.source.timestamp_us,
                x,
                y,
                z,
                has_world: true,
            }
        })
        .collect();
    positions.sort_by_key(|p| p.timestamp_us);
    let mut out = build_replay_shots(
        &positions,
        &film.fire_events,
        players.clock.origin_us,
        step,
        players.registry.owners.state.indices_by_slot(),
    );
    out.retain_published_tracks(&players.players.publication.tracks);
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Published {
        slot: u32,
    }
    #[derive(Deserialize)]
    struct Case {
        positions: Vec<ReplayPlayerPosition>,
        events: Vec<FilmFireEvent>,
        owners: BTreeMap<u32, i64>,
        origin: u64,
        step: u64,
        published: Vec<Published>,
        raw: ReplayShotPublication,
        output: ReplayShotPublication,
    }
    #[test]
    fn native_shot_attachment_and_publication() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/replay-shots-v41.json.zlib")[..])
            .read_to_end(&mut raw)
            .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&raw).unwrap();
        for (i, c) in cases.into_iter().enumerate() {
            let mut out = build_replay_shots(
                &c.positions,
                &c.events,
                c.origin,
                NonZeroU64::new(c.step).unwrap(),
                &c.owners,
            );
            assert_eq!(out, c.raw, "raw {i}");
            assert!(out.coverage.balanced(), "raw balance {i}");
            let published: Vec<_> = c
                .published
                .iter()
                .map(|p| ReplayTrack {
                    slot: p.slot,
                    start_frame: 100000,
                    end_frame: 100001,
                    ..Default::default()
                })
                .collect();
            out.retain_published_tracks(&published);
            assert_eq!(out, c.output, "publication {i}");
            assert!(out.coverage.balanced(), "publication balance {i}");
        }
    }
}
