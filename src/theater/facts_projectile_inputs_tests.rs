use super::*;
use serde::Deserialize;
use std::{collections::BTreeMap, io::Read, num::NonZeroU64};
#[derive(Deserialize)]
struct Point {
    #[serde(rename = "TimestampUS")]
    time: u64,
    #[serde(rename = "Chunk")]
    chunk: i64,
    #[serde(rename = "X")]
    x: f32,
    #[serde(rename = "Y")]
    y: f32,
    #[serde(rename = "Z")]
    z: f32,
    #[serde(rename = "AtRest")]
    rest: bool,
}
#[derive(Deserialize)]
struct Track {
    #[serde(rename = "Slot")]
    slot: u32,
    #[serde(rename = "Gen")]
    generation: u32,
    #[serde(rename = "Pts")]
    points: Option<Vec<Point>>,
}
impl From<Track> for FactsProjectileTrack {
    fn from(t: Track) -> Self {
        Self {
            slot: t.slot,
            generation: t.generation,
            points: t.points.map(|p| {
                p.into_iter()
                    .map(|p| FactsProjectileSample {
                        timestamp_us: p.time,
                        chunk: p.chunk,
                        position: [p.x, p.y, p.z],
                        at_rest: p.rest,
                    })
                    .collect()
            }),
        }
    }
}
#[derive(Deserialize)]
struct Throw {
    #[serde(rename = "TimestampUS")]
    time: u64,
    #[serde(rename = "FilmIndex")]
    index: i64,
    #[serde(rename = "TypeID")]
    type_id: u32,
}

mod projectiles {
    use super::*;
    #[derive(Deserialize)]
    struct Case {
        tracks: Vec<Track>,
        origin: u64,
        step: u64,
        output: ReplayProjectilePublication,
    }
    #[test]
    fn native_facts_projectile_grid_cuts_and_links() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/facts-projectiles-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(cases.len(), 1024);
        for (i, c) in cases.into_iter().enumerate() {
            let tracks: Vec<_> = c
                .tracks
                .into_iter()
                .map(FactsProjectileTrack::from)
                .collect();
            assert_eq!(
                build_facts_replay_projectiles(&tracks, c.origin, NonZeroU64::new(c.step).unwrap()),
                c.output,
                "case {i}"
            );
        }
    }
}
mod grenades {
    use super::*;
    #[derive(Deserialize)]
    struct Case {
        positions: Vec<ReplayPlayerPosition>,
        throws: Vec<Throw>,
        owners: BTreeMap<u32, i64>,
        projectiles: Vec<Track>,
        origin: u64,
        step: u64,
        slots: Vec<u32>,
        projectile_output: ReplayProjectilePublication,
        raw: ReplayGrenadePublication,
        output: ReplayGrenadePublication,
    }
    #[test]
    fn native_facts_grenade_sources_and_projectile_links() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/facts-grenades-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<Case> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(rows.len(), 1024);
        for (i, c) in rows.into_iter().enumerate() {
            let tracks: Vec<_> = c
                .projectiles
                .into_iter()
                .map(FactsProjectileTrack::from)
                .collect();
            let throws: Vec<_> = c
                .throws
                .into_iter()
                .map(|g| FactsGrenadeThrow {
                    timestamp_us: g.time,
                    film_index: g.index,
                    type_id: g.type_id,
                })
                .collect();
            let step = NonZeroU64::new(c.step).unwrap();
            let projectiles = build_facts_replay_projectiles(&tracks, c.origin, step);
            assert_eq!(projectiles, c.projectile_output, "projectiles {i}");
            let mut got = build_facts_replay_grenades(
                &c.positions,
                &throws,
                c.origin,
                step,
                &c.owners,
                &tracks,
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
