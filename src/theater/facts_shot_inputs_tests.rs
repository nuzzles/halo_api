use super::*;
use serde::Deserialize;
use std::{collections::BTreeMap, io::Read, num::NonZeroU64};
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Event {
    #[serde(rename = "TimestampUS")]
    timestamp_us: u64,
    film_index: i64,
    #[serde(rename = "WeaponID")]
    weapon_id: u64,
    has_aim: bool,
    aim: [f32; 3],
}
impl Event {
    fn cache(&self) -> FactsFireEvent {
        FactsFireEvent {
            timestamp_us: self.timestamp_us,
            film_index: self.film_index,
            weapon_id: self.weapon_id,
            has_aim: self.has_aim,
            aim: self.aim,
        }
    }
}
#[derive(Deserialize)]
struct Orphan {
    event: Event,
    reason: ReplayShotOrphanReason,
}
#[derive(Deserialize)]
struct Publication {
    shots: Vec<ReplayShot>,
    orphans: Vec<Orphan>,
    coverage: ReplayLayerCoverage,
}
impl Publication {
    fn cache(self) -> FactsReplayShotPublication {
        FactsReplayShotPublication {
            shots: self.shots,
            coverage: self.coverage,
            orphans: self
                .orphans
                .into_iter()
                .map(|o| FactsReplayOrphanShot {
                    event: o.event.cache(),
                    reason: o.reason,
                })
                .collect(),
        }
    }
}
#[derive(Deserialize)]
struct Published {
    slot: u32,
}
#[derive(Deserialize)]
struct Row {
    positions: Vec<ReplayPlayerPosition>,
    events: Vec<Event>,
    owners: BTreeMap<u32, i64>,
    origin: u64,
    step: u64,
    published: Vec<Published>,
    raw: Publication,
    output: Publication,
}
#[test]
fn native_facts_shot_signed_identity() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/facts-shots-v41.json.zlib")[..])
        .read_to_end(&mut raw)
        .unwrap();
    let rows: Vec<Row> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 1024);
    for (i, row) in rows.into_iter().enumerate() {
        let events: Vec<_> = row.events.iter().map(Event::cache).collect();
        let mut got = build_facts_replay_shots(
            &row.positions,
            &events,
            row.origin,
            NonZeroU64::new(row.step).unwrap(),
            &row.owners,
        );
        assert_eq!(got, row.raw.cache(), "raw {i}");
        assert!(got.coverage.balanced());
        let published: Vec<_> = row
            .published
            .iter()
            .map(|p| ReplayTrack {
                slot: p.slot,
                start_frame: 100000,
                end_frame: 100001,
                ..Default::default()
            })
            .collect();
        got.retain_published_tracks(&published);
        assert_eq!(got, row.output.cache(), "published {i}");
        assert!(got.coverage.balanced());
    }
}

#[test]
fn native_facts_vehicle_shot_signed_identity() {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Case {
        vehicles: Vec<ReplayVehicleTrack>,
        owners: BTreeMap<u32, i64>,
        published: std::collections::BTreeSet<u32>,
        origin: u64,
        step: u64,
        frames: i64,
        input: Publication,
        output: Publication,
        vehicle_coverage: Option<ReplayVehicleCoverage>,
        verdict: Option<String>,
    }
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/facts-vehicle-shots-v41.json.zlib")[..],
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Case> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 1024);
    for (i, row) in rows.into_iter().enumerate() {
        let mut shots = row.input.cache();
        let mut doc = ReplayDocument {
            content: ReplayDocumentContent {
                shots: shots.shots.clone(),
                vehicles: row.vehicles.clone(),
                tracks: Some(
                    row.published
                        .iter()
                        .map(|&slot| ReplayTrack {
                            slot,
                            ..Default::default()
                        })
                        .collect(),
                ),
                ..Default::default()
            },
            coverage: Some(ReplayCoverage {
                shots: shots.coverage.clone(),
                vehicles: row
                    .vehicle_coverage
                    .as_ref()
                    .map(|_| ReplayVehicleCoverage::default()),
                ..Default::default()
            }),
        };
        attach_facts_replay_vehicle_shots_to_document(
            &mut doc,
            &shots.orphans,
            &row.owners,
            row.origin,
            NonZeroU64::new(row.step).unwrap(),
            row.frames,
        );

        let mut coverage = row
            .vehicle_coverage
            .as_ref()
            .map(|_| ReplayVehicleCoverage::default());
        let verdict = attach_facts_replay_vehicle_shots(
            &mut shots,
            &row.vehicles,
            &row.owners,
            &row.published,
            row.origin,
            NonZeroU64::new(row.step).unwrap(),
            row.frames,
            coverage.as_mut(),
        );
        let expected = row.output.cache();
        assert_eq!(doc.content.shots, expected.shots, "document shots {i}");
        let doc_coverage = doc.coverage.unwrap();
        assert_eq!(
            doc_coverage.shots, expected.coverage,
            "document shot coverage {i}"
        );
        assert_eq!(
            doc_coverage.vehicles, row.vehicle_coverage,
            "document vehicle coverage {i}"
        );
        assert_eq!(
            doc_coverage.verdict.get("shots"),
            row.verdict.as_ref(),
            "document verdict {i}"
        );
        assert_eq!(shots, expected, "vehicle shots {i}");
        assert_eq!(coverage, row.vehicle_coverage, "vehicle coverage {i}");
        assert_eq!(verdict, row.verdict.as_deref(), "vehicle verdict {i}");
        assert!(shots.coverage.balanced());
    }
}
