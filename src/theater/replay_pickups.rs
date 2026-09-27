//! Native pickup publication on an explicit replay clock with timestamp-aware identity.
use super::{BipedPickupStream, GroundPadClock, ReplayPickup};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayPickupCoverage {
    pub decoded: usize,
    pub published: usize,
    pub named: usize,
    pub weapons: usize,
    pub items: usize,
    pub unknown_families: usize,
    pub before_origin: usize,
    pub multi_event: i64,
    pub refused: i64,
    pub origin_spawner: usize,
    pub origin_ground: usize,
    pub origin_unknown: usize,
    pub spawn_points_state: String,
    pub map_catalog_points: usize,
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
    pub spawner_by_point_kind: BTreeMap<String, usize>,
}
pub type PickupOriginCallback<'a> = dyn FnMut(u32, u64, i64) -> (String, String) + 'a;
/// Origin and catalog state are supplied together; an empty origin is abstention.
pub struct ReplayPickupOrigin<'a> {
    pub state: &'a str,
    pub catalog_points: usize,
    pub resolve: &'a mut PickupOriginCallback<'a>,
}
pub struct ReplayPickupInputs<'a> {
    pub equipment_families: &'a BTreeMap<u32, String>,
    pub weapon_keys: &'a BTreeMap<u32, String>,
    pub occupant: Option<&'a dyn Fn(u32, u64) -> u64>,
    pub origin: Option<ReplayPickupOrigin<'a>>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayPickupOutput {
    pub pickups: Vec<ReplayPickup>,
    pub coverage: ReplayPickupCoverage,
}
/// Frames after the grid end are retained; events before its origin are rejected.
/// No catalog is tried as a fallback for an unrelated pickup class.
pub fn build_replay_pickups(
    stream: &BipedPickupStream,
    clock: GroundPadClock,
    input: ReplayPickupInputs<'_>,
) -> ReplayPickupOutput {
    build_pickup_values(
        stream.records.iter().map(|p| PickupValue {
            timestamp_us: p.source.timestamp_us,
            slot: p.slot,
            catalog_id: p.catalog_id,
            class: p.class,
        }),
        stream.stats.multi_event as i64,
        (stream.stats.refused_no_ref as i64)
            .wrapping_add(stream.stats.refused_no_catalog as i64)
            .wrapping_add(stream.stats.refused_off_band as i64),
        clock,
        input,
    )
}
/// Native cache counters are signed and their refusal sum wraps at 64 bits.
pub fn build_facts_replay_pickups(
    records: &[super::FactsPickup],
    stats: &super::FactsPickupStats,
    clock: GroundPadClock,
    input: ReplayPickupInputs<'_>,
) -> ReplayPickupOutput {
    build_pickup_values(
        records.iter().map(|p| PickupValue {
            timestamp_us: p.timestamp_us,
            slot: p.slot,
            catalog_id: p.catalog_id,
            class: p.class,
        }),
        stats.multi_event,
        stats
            .refused_no_ref
            .wrapping_add(stats.refused_no_catalog)
            .wrapping_add(stats.refused_off_band),
        clock,
        input,
    )
}
struct PickupValue {
    timestamp_us: u64,
    slot: u32,
    catalog_id: u32,
    class: u8,
}
fn build_pickup_values(
    records: impl ExactSizeIterator<Item = PickupValue>,
    multi_event: i64,
    refused: i64,
    clock: GroundPadClock,
    mut input: ReplayPickupInputs<'_>,
) -> ReplayPickupOutput {
    let mut out = ReplayPickupOutput {
        pickups: Vec::new(),
        coverage: ReplayPickupCoverage {
            decoded: records.len(),
            multi_event,
            refused,
            spawn_points_state: input
                .origin
                .as_ref()
                .map_or("map_absent", |o| o.state)
                .into(),
            map_catalog_points: input.origin.as_ref().map_or(0, |o| o.catalog_points),
            ..Default::default()
        },
    };
    if clock.step_us == 0 {
        return out;
    }
    for p in records {
        let ts = p.timestamp_us;
        if ts < clock.origin_us {
            out.coverage.before_origin += 1;
            continue;
        }
        let kind = match p.class {
            0 | 1 => "weapon",
            2 => "grenade",
            3 => "equipment",
            _ => "item",
        };
        let family = match p.class {
            0 | 1 => input.weapon_keys.get(&p.catalog_id),
            2 | 3 => input.equipment_families.get(&p.catalog_id),
            _ => None,
        }
        .cloned()
        .unwrap_or_default();
        let mut e = ReplayPickup {
            t: ((ts - clock.origin_us) / clock.step_us) as i64,
            slot: p.slot,
            w: format!("{:08x}", p.catalog_id),
            family,
            kind: kind.into(),
            class: p.class,
            origin: String::new(),
            xuid: String::new(),
        };
        if let Some(occupant) = input.occupant {
            let x = occupant(p.slot, ts);
            if x != 0 {
                e.xuid = x.to_string();
                out.coverage.named += 1;
            }
        }
        if e.family.is_empty() {
            out.coverage.unknown_families += 1;
        }
        let mut point_kind = String::new();
        if kind != "weapon"
            && let Some(origin) = &mut input.origin
        {
            (e.origin, point_kind) = (origin.resolve)(p.slot, ts, e.t);
        }
        out.coverage.published += 1;
        if kind == "weapon" {
            out.coverage.weapons += 1;
        } else {
            out.coverage.items += 1;
            match e.origin.as_str() {
                "spawner" => {
                    out.coverage.origin_spawner += 1;
                    *out.coverage
                        .spawner_by_point_kind
                        .entry(point_kind)
                        .or_default() += 1;
                }
                "ground" => out.coverage.origin_ground += 1,
                _ => out.coverage.origin_unknown += 1,
            }
        }
        out.pickups.push(e);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn native_replay_pickup_construction() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/replay-pickups-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        for (i, r) in rows.as_array().unwrap().iter().enumerate() {
            let st = &r["stats"];
            let mut stream = BipedPickupStream::default();
            stream.stats.multi_event = st["MultiEvent"].as_u64().unwrap() as usize;
            stream.stats.refused_no_ref = st["RefusedNoRef"].as_u64().unwrap() as usize;
            stream.stats.refused_no_catalog = st["RefusedNoCatalog"].as_u64().unwrap() as usize;
            stream.stats.refused_off_band = st["RefusedOffBand"].as_u64().unwrap() as usize;
            for p in r["input"].as_array().unwrap() {
                stream.records.push(super::super::BipedPickup {
                    source: super::super::FilmPacket {
                        chunk_index: 0,
                        packet_type: 0,
                        byte_2: 0,
                        byte_3: 0,
                        payload_offset: 0,
                        payload_size: 0,
                        timestamp_us: p["TimestampUS"].as_u64().unwrap(),
                    },
                    slot: p["Slot"].as_u64().unwrap() as u32,
                    catalog_id: p["CatalogID"].as_u64().unwrap() as u32,
                    class: p["Class"].as_u64().unwrap() as u8,
                });
            }
            let clock = serde_json::from_value(r["clock"].clone()).unwrap();
            let equipment = serde_json::from_value(r["equipment"].clone()).unwrap();
            let weapons = serde_json::from_value(r["weapons"].clone()).unwrap();
            let occupant = |slot: u32, ts: u64| {
                if (slot as u64 + ts / 100000).is_multiple_of(3) {
                    0
                } else {
                    slot as u64 * 100000 + ts / 100000 + 1
                }
            };
            use super::super::{
                MapSpawnPoint, PickupOriginJudge, PickupOriginPlacement, ReplayPlayerPosition,
            };
            let positions: Vec<_> = stream
                .records
                .iter()
                .map(|p| ReplayPlayerPosition {
                    slot: p.slot,
                    timestamp_us: p.source.timestamp_us,
                    x: p.slot as f32,
                    y: 0.0,
                    z: 0.0,
                    has_world: true,
                })
                .collect();
            let judge = PickupOriginJudge::new(
                "established",
                vec![
                    MapSpawnPoint {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                        kind: "equipment".into(),
                    },
                    MapSpawnPoint {
                        x: 1.0,
                        y: 0.0,
                        z: 0.0,
                        kind: "grenade".into(),
                    },
                ],
                &positions,
                &[
                    PickupOriginPlacement {
                        t0: 0,
                        until_max: 8,
                        end: "seen".into(),
                        origin: "dropped".into(),
                        x: 2.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    PickupOriginPlacement {
                        t0: 2,
                        until_max: 0,
                        end: "open".into(),
                        origin: "dropped".into(),
                        x: 3.0,
                        y: 0.0,
                        z: 0.0,
                    },
                ],
            );
            let mut resolve = |slot, ts, frame| judge.resolve(slot, ts, frame);
            let actual = build_replay_pickups(
                &stream,
                clock,
                ReplayPickupInputs {
                    equipment_families: &equipment,
                    weapon_keys: &weapons,
                    occupant: if r["occupant"] == true {
                        Some(&occupant)
                    } else {
                        None
                    },
                    origin: if r["origin"] == true {
                        Some(ReplayPickupOrigin {
                            state: &judge.state,
                            catalog_points: judge.catalog_points(),
                            resolve: &mut resolve,
                        })
                    } else {
                        None
                    },
                },
            );
            let pickups: Vec<ReplayPickup> = serde_json::from_value(r["pickups"].clone()).unwrap();
            let coverage: ReplayPickupCoverage =
                serde_json::from_value(r["coverage"].clone()).unwrap();
            assert_eq!(actual.pickups, pickups, "pickups {i}");
            assert_eq!(actual.coverage, coverage, "coverage {i}");
        }
    }
}
