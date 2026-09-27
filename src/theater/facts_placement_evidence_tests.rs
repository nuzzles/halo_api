use super::*;
use serde::Deserialize;
use std::io::Read;
#[derive(Deserialize)]
struct RawPlacement {
    #[serde(rename = "T0US")]
    start: u64,
    #[serde(rename = "T1US")]
    end: u64,
    #[serde(rename = "Life")]
    life: FactsLifeKey,
    #[serde(rename = "X")]
    x: f32,
    #[serde(rename = "Y")]
    y: f32,
    #[serde(rename = "Z")]
    z: f32,
    #[serde(rename = "GlobalID")]
    id: u32,
    #[serde(rename = "Points")]
    points: i64,
}
impl RawPlacement {
    fn facts(self) -> FactsPlacement {
        FactsPlacement {
            start_us: self.start,
            end_us: self.end,
            life: self.life,
            position: [self.x, self.y, self.z],
            global_id: self.id,
            points: self.points,
        }
    }
}
#[derive(Deserialize)]
struct Spawn {
    slot: u32,
    generation: u32,
    time: u64,
    valid: bool,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Change {
    slot: u32,
    time: u64,
    kind_hex: String,
}
#[derive(Deserialize)]
struct Probe {
    placement: RawPlacement,
    piece: bool,
    owner: Option<ReplayEquipmentOwner>,
    origin: ReplayEquipmentOrigin,
}
#[derive(Deserialize)]
struct Case {
    spawns: Vec<Spawn>,
    lives: Vec<IdentityLife>,
    changes: Vec<Change>,
    probes: Vec<Probe>,
    events: usize,
    raw: Vec<RawPlacement>,
    census: WorldObjectKeyframes,
    clock: IdentityClock,
    ends: Vec<ReplayEquipmentEnd>,
}
#[test]
fn native_facts_placement_origins_and_census_bounds() {
    let mut data = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/facts-placement-evidence-v41.json.zlib")[..],
    )
    .read_to_end(&mut data)
    .unwrap();
    let cases: Vec<Case> = serde_json::from_slice(&data).unwrap();
    assert_eq!(cases.len(), 1024);
    for (i, c) in cases.into_iter().enumerate() {
        let spawns: Vec<_> = c
            .spawns
            .into_iter()
            .map(|s| FactsSpawnEvent {
                timestamp_us: s.time,
                spawned: FactsLifeKey {
                    slot: s.slot,
                    generation: s.generation,
                },
                spawned_valid: s.valid,
                ..Default::default()
            })
            .collect();
        let changes: Vec<_> = c
            .changes
            .into_iter()
            .map(|c| FactsEquipmentChange {
                timestamp_us: c.time,
                slot: c.slot,
                kind: c
                    .kind_hex
                    .as_bytes()
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
                    .collect(),
                ..Default::default()
            })
            .collect();
        let index = ReplayEquipmentOriginIndex::from_facts(&spawns, &c.lives, &changes);
        assert_eq!(index.spawn_events, c.events, "events {i}");
        for (j, p) in c.probes.into_iter().enumerate() {
            assert_eq!(
                index.classify_facts(&p.placement.facts(), p.piece, p.owner.map(|o| o.slot)),
                p.origin,
                "origin {i}/{j}"
            );
        }
        if let Some(step) = std::num::NonZeroU64::new(c.clock.step_us) {
            let placements: Vec<_> = c.raw.into_iter().map(RawPlacement::facts).collect();
            assert_eq!(
                replay_facts_equipment_ends(
                    &placements,
                    &FactsWorldKeyframes::from(&c.census),
                    c.clock.origin_us,
                    step,
                    c.clock.frame_count
                ),
                c.ends,
                "ends {i}"
            );
        }
    }
}

#[derive(Deserialize)]
struct PlacementStats {
    scanned: bool,
    widths: (i64, i64),
    lives: i64,
    anchors: i64,
    confirmed: i64,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PublicationCase {
    #[serde(flatten)]
    evidence: Case,
    stats: PlacementStats,
    spawn_lists: i64,
    positions: Vec<ReplayPositionSample>,
    families: std::collections::BTreeMap<u32, String>,
    publication: ReplayEquipmentPlacements,
    #[serde(rename = "fallback_report")]
    fallback_report: serde_json::Value,
}
#[test]
fn native_facts_placement_publication_signed_stats() {
    let mut data = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/facts-placement-publication-v41.json.zlib")[..],
    )
    .read_to_end(&mut data)
    .unwrap();
    let cases: Vec<PublicationCase> = serde_json::from_slice(&data).unwrap();
    assert_eq!(cases.len(), 1024);
    for (i, c) in cases.into_iter().enumerate() {
        let e = c.evidence;
        let spawns: Vec<_> = e
            .spawns
            .into_iter()
            .map(|s| FactsSpawnEvent {
                timestamp_us: s.time,
                spawned: FactsLifeKey {
                    slot: s.slot,
                    generation: s.generation,
                },
                spawned_valid: s.valid,
                ..Default::default()
            })
            .collect();
        let changes: Vec<_> = e
            .changes
            .into_iter()
            .map(|c| FactsEquipmentChange {
                timestamp_us: c.time,
                slot: c.slot,
                kind: c
                    .kind_hex
                    .as_bytes()
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
                    .collect(),
                ..Default::default()
            })
            .collect();
        let positions: Vec<_> = c
            .positions
            .iter()
            .map(|p| FactsBipedPosition {
                timestamp_us: p.position.timestamp_us,
                slot: p.position.slot,
                has_world: p.position.has_world,
                world: [p.position.x, p.position.y, p.position.z],
                has_yaw: p.aim.is_some(),
                yaw_raw: p.aim.unwrap_or_default()[0],
                pitch_raw: p.aim.unwrap_or_default()[1],
                ..Default::default()
            })
            .collect();
        let stats = FactsPlacementStats {
            scanned: c.stats.scanned,
            calibration: FactsMppCalibration {
                widths: c.stats.widths,
                ..Default::default()
            },
            lives: c.stats.lives,
            anchors: c.stats.anchors,
            confirmed: c.stats.confirmed,
            ..Default::default()
        };
        let raw: Vec<_> = e.raw.into_iter().map(RawPlacement::facts).collect();
        let got = build_facts_replay_equipment_placements(
            FactsReplayEquipmentPlacementInput {
                raw: &raw,
                stats: &stats,
                positions: &positions,
                census: &FactsWorldKeyframes::from(&e.census),
                spawns: &spawns,
                spawn_lists: c.spawn_lists,
                lives: &e.lives,
                changes: &changes,
                families: &c.families,
                spawned_piece_ids: &[0x528fce46, 0x686b40c9].into(),
            },
            e.clock,
        );
        assert_eq!(got, c.publication, "publication {i}");
        let report = if got.manifest_fallbacks == 0 {
            serde_json::json!([])
        } else {
            serde_json::json!([{"Nom":"repli_piece_engendree_sans_evenement","Declenchements":got.manifest_fallbacks}])
        };
        assert_eq!(report, c.fallback_report, "fallback counter {i}");
    }
}
