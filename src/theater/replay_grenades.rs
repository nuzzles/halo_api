//! Native grenade locations: projectile birth first, then the author's biped.
use super::{
    FilmGrenadeThrow, ReplayLayerCoverage, ReplayPlayerPosition, ReplayTrack, WorldObjectTrack,
    native_sort, replay_shots::PositionIndex,
};
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
    num::NonZeroU64,
};
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayGrenade {
    pub t: i64,
    pub slot: u32,
    #[serde(rename = "i")]
    pub film_index: i64,
    pub x: f32,
    pub y: f32,
    pub rank: usize,
    #[serde(rename = "s")]
    pub source: String,
    #[serde(default, rename = "proj", skip_serializing_if = "Option::is_none")]
    pub projectile: Option<usize>,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReplayGrenadePublication {
    pub grenades: Vec<ReplayGrenade>,
    pub coverage: ReplayLayerCoverage,
}
impl ReplayGrenadePublication {
    /// Projectile-located throws remain publishable even without a player track.
    pub fn retain_published_tracks(&mut self, tracks: &[ReplayTrack]) {
        let slots: BTreeSet<_> = tracks.iter().map(|t| t.slot).collect();
        let before = self.grenades.len();
        self.grenades
            .retain(|g| g.source == "projectile" || slots.contains(&g.slot));
        self.coverage.unpublished = self
            .coverage
            .unpublished
            .wrapping_add((before - self.grenades.len()) as i64);
        self.coverage.attached = self.grenades.len() as i64;
    }
}
/// Each throw independently considers all projectile births within +/-200ms.
/// An available author chooses the nearest planar birth within 4m; without an
/// author, exactly one candidate is required. Births are not consumed by matches.
pub fn build_replay_grenades(
    positions: &[ReplayPlayerPosition],
    throws: &[FilmGrenadeThrow],
    origin_us: u64,
    step_us: NonZeroU64,
    owners: &BTreeMap<u32, i64>,
    projectiles: &[WorldObjectTrack],
    published_by_raw: &BTreeMap<usize, usize>,
) -> ReplayGrenadePublication {
    let births = projectiles
        .iter()
        .enumerate()
        .filter_map(|(i, t)| t.pts.first().map(|p| (i, p)))
        .collect();
    build_grenade_values(
        positions,
        throws
            .iter()
            .map(|g| (g.timestamp_us, i64::from(g.film_index), g.rank())),
        origin_us,
        step_us,
        owners,
        births,
        published_by_raw,
    )
}
/// Cached throw identity remains a signed native integer through publication.
pub fn build_facts_replay_grenades(
    positions: &[ReplayPlayerPosition],
    throws: &[super::FactsGrenadeThrow],
    origin_us: u64,
    step_us: NonZeroU64,
    owners: &BTreeMap<u32, i64>,
    projectiles: &[super::FactsProjectileTrack],
    published_by_raw: &BTreeMap<usize, usize>,
) -> ReplayGrenadePublication {
    let births = projectiles
        .iter()
        .enumerate()
        .filter_map(|(i, t)| t.points.as_deref().and_then(|p| p.first()).map(|p| (i, p)))
        .collect();
    build_grenade_values(
        positions,
        throws.iter().map(|g| {
            (
                g.timestamp_us,
                g.film_index,
                super::grenade_rank_of(g.type_id),
            )
        }),
        origin_us,
        step_us,
        owners,
        births,
        published_by_raw,
    )
}
fn build_grenade_values<P: super::replay_projectiles::ProjectilePoint>(
    positions: &[ReplayPlayerPosition],
    throws: impl ExactSizeIterator<Item = (u64, i64, Option<usize>)>,
    origin_us: u64,
    step_us: NonZeroU64,
    owners: &BTreeMap<u32, i64>,
    mut births: Vec<(usize, &P)>,
    published_by_raw: &BTreeMap<usize, usize>,
) -> ReplayGrenadePublication {
    let mut out = ReplayGrenadePublication::default();
    out.coverage.available = throws.len() as i64;
    native_sort::sort_by(&mut births, |a, b| {
        if a.1.timestamp_us() != b.1.timestamp_us() {
            return a.1.timestamp_us().cmp(&b.1.timestamp_us());
        }
        for (av, bv) in a.1.position().into_iter().zip(b.1.position()) {
            if av != bv {
                return av.partial_cmp(&bv).unwrap_or(Ordering::Equal);
            }
        }
        a.0.cmp(&b.0)
    });
    let index = PositionIndex::new(positions);
    for (timestamp_us, film_index, rank) in throws {
        let Some(rank) = rank else {
            out.coverage.no_slot = out.coverage.no_slot.wrapping_add(1);
            continue;
        };
        let position = index.owned_at(owners, film_index, timestamp_us).ok();
        let slot = position.map_or(0, |p| p.slot);
        let author = position.filter(|p| p.has_world);
        let low = timestamp_us.saturating_sub(200_000);
        let high = timestamp_us.wrapping_add(200_000);
        let start = births.partition_point(|b| b.1.timestamp_us() < low);
        let candidates: Vec<_> = births[start..]
            .iter()
            .take_while(|b| b.1.timestamp_us() <= high)
            .collect();
        let chosen = if let Some(author) = author {
            let mut best = None;
            let mut best_distance = f64::MAX;
            for c in &candidates {
                let d = super::replay_plan_distance(
                    [c.1.position()[0], c.1.position()[1]],
                    [author.x, author.y],
                );
                if d < best_distance {
                    best = Some(*c);
                    best_distance = d;
                }
            }
            if best_distance > 4.0 { None } else { best }
        } else if candidates.len() == 1 {
            Some(candidates[0])
        } else {
            None
        };
        let (x, y, source, projectile) = if let Some(&(raw, p)) = chosen {
            (
                p.position()[0],
                p.position()[1],
                "projectile",
                published_by_raw.get(&raw).copied(),
            )
        } else if let Some(p) = author {
            (p.x, p.y, "biped", None)
        } else {
            out.coverage.no_slot = out.coverage.no_slot.wrapping_add(1);
            continue;
        };
        let round = |v: f32| ((f64::from(v) * 100.0).round() / 100.0) as f32;
        out.grenades.push(ReplayGrenade {
            t: (timestamp_us.wrapping_sub(origin_us) / step_us.get()) as i64,
            slot,
            film_index,
            x: round(x),
            y: round(y),
            rank,
            source: source.into(),
            projectile,
        });
        out.coverage.attached = out.coverage.attached.wrapping_add(1);
    }
    native_sort::sort_by(&mut out.grenades, |a, b| {
        if a.t != b.t {
            return a.t.cmp(&b.t);
        }
        if a.film_index != b.film_index {
            return a.film_index.cmp(&b.film_index);
        }
        if a.slot != b.slot {
            return a.slot.cmp(&b.slot);
        }
        if a.x != b.x {
            return a.x.partial_cmp(&b.x).unwrap_or(Ordering::Equal);
        }
        a.y.partial_cmp(&b.y).unwrap_or(Ordering::Equal)
    });
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Case {
        positions: Vec<ReplayPlayerPosition>,
        throws: Vec<FilmGrenadeThrow>,
        owners: BTreeMap<u32, i64>,
        projectiles: Vec<WorldObjectTrack>,
        origin: u64,
        step: u64,
        slots: Vec<u32>,
        projectile_output: super::super::ReplayProjectilePublication,
        raw: ReplayGrenadePublication,
        output: ReplayGrenadePublication,
    }
    #[test]
    fn native_grenade_sources_and_projectile_links() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/replay-grenades-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<Case> = serde_json::from_slice(&raw).unwrap();
        for (i, c) in rows.into_iter().enumerate() {
            let step = NonZeroU64::new(c.step).unwrap();
            let projectiles =
                super::super::build_replay_projectiles(&c.projectiles, c.origin, step);
            assert_eq!(projectiles, c.projectile_output, "projectiles {i}");
            let mut got = build_replay_grenades(
                &c.positions,
                &c.throws,
                c.origin,
                step,
                &c.owners,
                &c.projectiles,
                &projectiles.published_by_raw,
            );
            assert_eq!(got, c.raw, "raw {i}");
            assert!(got.coverage.balanced());
            let tracks: Vec<_> = c
                .slots
                .into_iter()
                .map(|slot| ReplayTrack {
                    slot,
                    ..Default::default()
                })
                .collect();
            got.retain_published_tracks(&tracks);
            assert_eq!(got, c.output, "publication {i}");
            assert!(got.coverage.balanced());
        }
    }
}
