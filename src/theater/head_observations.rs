//! Reference gameplay observations assembled from bounded event-head reads.
use super::{FilmHeadEvent, FilmPacket, HeadEventPayload};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BipedPickup {
    pub source: FilmPacket,
    pub slot: u32,
    pub catalog_id: u32,
    pub class: u8,
}
impl BipedPickup {
    pub fn is_weapon_class(&self) -> bool {
        self.class <= 1
    }
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BipedPickupStats {
    pub packets: usize,
    pub type_9: usize,
    pub type_8: usize,
    /// Native fallback counter; the 0xC4 packet gate admits only types 8/9.
    #[serde(default)]
    pub other_type: usize,
    pub published: usize,
    pub multi_event: usize,
    pub refused_no_ref: usize,
    pub refused_no_catalog: usize,
    pub refused_off_band: usize,
    pub unexpected_wide_ref: usize,
    /// Rust additionally distinguishes truncated data from an absent reference.
    pub truncated: usize,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BipedPickupStream {
    pub slot_band: Option<[u32; 2]>,
    pub stats: BipedPickupStats,
    pub records: Vec<BipedPickup>,
}
/// Same 0xC4 family gate, slot-band sentinel and publication order as Go.
/// Only complete reads publish; the original packet head retains truncation detail.
pub fn biped_pickups_from_heads(
    heads: &[FilmHeadEvent],
    band: Option<[u32; 2]>,
) -> BipedPickupStream {
    let mut out = BipedPickupStream {
        slot_band: band,
        ..Default::default()
    };
    for h in heads {
        let e = &h.event;
        if !e.config || !matches!(e.kind, 8 | 9) {
            continue;
        }
        let s = &mut out.stats;
        s.packets += 1;
        if e.kind == 8 {
            s.type_8 += 1;
            continue;
        }
        s.type_9 += 1;
        let Some(first) = e.references.first() else {
            s.truncated += 1;
            continue;
        };
        if first.value.is_none() {
            s.refused_no_ref += 1;
            continue;
        }
        if e.references.len() < 3 {
            s.truncated += 1;
            continue;
        }
        if e.references[1..].iter().any(|r| r.value.is_some()) {
            s.unexpected_wide_ref += 1;
        }
        let Some(HeadEventPayload::Pickup {
            catalog_id,
            more_events,
            ..
        }) = &e.payload
        else {
            s.truncated += 1;
            continue;
        };
        if catalog_id.is_none() {
            s.refused_no_catalog += 1;
            continue;
        }
        if *more_events == Some(true) {
            s.multi_event += 1;
        }
        let Some((slot, catalog_id, class)) = e.pickup() else {
            s.truncated += 1;
            continue;
        };
        if band.is_some_and(|[min, max]| slot < min || slot > max) {
            s.refused_off_band += 1;
            continue;
        }
        out.records.push(BipedPickup {
            source: h.source,
            slot,
            catalog_id,
            class,
        });
        s.published += 1;
    }
    out
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BipedZoomEvent {
    pub source: FilmPacket,
    pub slot: u32,
    pub level: u8,
}
impl BipedZoomEvent {
    pub fn scoped(&self) -> bool {
        self.level > 0
    }
}
/// First events of the 0xCA family, ordered by timestamp.
pub fn biped_zoom_from_heads(heads: &[FilmHeadEvent]) -> Vec<BipedZoomEvent> {
    let mut out: Vec<_> = heads
        .iter()
        .filter(|h| h.event.config && h.event.kind == 21)
        .filter_map(|h| {
            let (slot, level) = h.event.zoom()?;
            Some(BipedZoomEvent {
                source: h.source,
                slot,
                level,
            })
        })
        .collect();
    out.sort_by_key(|e| e.source.timestamp_us);
    out
}
/// Last observed level, held for at most hold_us. Input must be time-sorted.
/// A zero result means no positive scope assertion, not proof of an observed exit.
pub fn biped_zoom_state_at(events: &[BipedZoomEvent], slot: u32, time_us: u64, hold_us: u64) -> u8 {
    let end = events.partition_point(|e| e.source.timestamp_us <= time_us);
    events[..end]
        .iter()
        .rev()
        .find(|e| e.slot == slot)
        .filter(|e| time_us - e.source.timestamp_us <= hold_us)
        .map_or(0, |e| e.level)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
    use std::io::Read;
    fn packet(time: u64) -> FilmPacket {
        FilmPacket {
            chunk_index: 0,
            packet_type: 0,
            byte_2: 0,
            byte_3: 0,
            payload_offset: 0,
            payload_size: 16,
            timestamp_us: time,
        }
    }
    #[test]
    fn truncated_payloads_do_not_publish_zero_filled_observations() {
        for bytes in [[0xc4, 0xc0], [0xca, 0xc0], [0xf3, 0xc0]] {
            let event = super::super::decode_packet_head_event(&bytes).unwrap();
            let heads = [FilmHeadEvent {
                source: packet(0),
                event,
            }];
            let pickups = biped_pickups_from_heads(&heads, None);
            assert!(pickups.records.is_empty());
            if bytes[0] == 0xc4 {
                assert_eq!(pickups.stats.truncated, 1);
            }
            assert!(biped_zoom_from_heads(&heads).is_empty());
        }
    }
    #[test]
    fn publication_rejections_and_scope_hold_match_go() {
        let mut text = String::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/biped-scan-levelup-v41.json.zlib")[..],
        )
        .read_to_string(&mut text)
        .unwrap();
        let oracle: Value = serde_json::from_str(&text).unwrap();
        let rows = oracle["head_observations"].as_array().unwrap();
        assert_eq!(rows.len(), 2048);
        for row in rows {
            let hex = row["hex"].as_str().unwrap();
            let bytes: Vec<u8> = (0..hex.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                .collect();
            let heads: Vec<_> = super::super::decode_packet_head_event(&bytes)
                .map(|event| FilmHeadEvent {
                    source: packet(0),
                    event,
                })
                .into_iter()
                .collect();
            let out = biped_pickups_from_heads(&heads, None);
            let restored: BipedPickupStream =
                serde_json::from_value(serde_json::to_value(&out).unwrap()).unwrap();
            assert_eq!(restored, out);
            let s = &out.stats;
            assert_eq!(s.truncated, 0);
            assert_eq!(
                json!({"Packets":s.packets,"Type9":s.type_9,"Type8":s.type_8,"OtherType":s.other_type,"Published":s.published,"MultiEvent":s.multi_event,"RefusedNoRef":s.refused_no_ref,"RefusedNoCatalog":s.refused_no_catalog,"RefusedOffBand":s.refused_off_band,"UnexpectedWideRef":s.unexpected_wide_ref}),
                row["stats"]
            );
            let actual=out.records.first().map(|p|json!({"Slot":p.slot,"CatalogID":p.catalog_id,"Class":p.class,"Chunk":0,"TimestampUS":0})).unwrap_or(Value::Null);
            assert_eq!(actual, row["pickup"]);
            if let Some(p) = out.records.first() {
                let rejected = biped_pickups_from_heads(&heads, Some([p.slot + 1, p.slot + 1]));
                assert_eq!(rejected.stats.refused_off_band, 1);
                assert!(rejected.records.is_empty());
            }
            let zoom = biped_zoom_from_heads(&heads);
            let actual = zoom
                .first()
                .map(|z| json!({"Slot":z.slot,"Level":z.level,"TimestampUS":123456}))
                .unwrap_or(Value::Null);
            assert_eq!(actual, row["zoom"]);
        }
        let events: Vec<_> = oracle["zoom_input"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| BipedZoomEvent {
                source: packet(v["TimestampUS"].as_u64().unwrap()),
                slot: v["Slot"].as_u64().unwrap() as u32,
                level: v["Level"].as_u64().unwrap() as u8,
            })
            .collect();
        let queries = oracle["zoom_queries"].as_array().unwrap();
        assert_eq!(queries.len(), 3000);
        for q in queries {
            assert_eq!(
                json!(biped_zoom_state_at(
                    &events,
                    q["slot"].as_u64().unwrap() as u32,
                    q["time"].as_u64().unwrap(),
                    q["hold"].as_u64().unwrap()
                )),
                q["level"]
            );
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjectLife {
    pub slot: u32,
    pub generation: u8,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EquipmentSpawnEvent {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub packet_index: Option<usize>,
    pub packet: FilmPacket,
    pub source: Option<ObjectLife>,
    pub spawned: Option<ObjectLife>,
    pub reference_2_present: bool,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EquipmentSpawnStats {
    pub chunks: usize,
    pub packets: usize,
    pub lists: usize,
    pub events: usize,
    pub with_spawned: usize,
    pub with_source: usize,
    pub reference_2: usize,
    pub truncated: usize,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EquipmentSpawnStream {
    pub records: Vec<EquipmentSpawnEvent>,
    pub stats: EquipmentSpawnStats,
}
/// Retains both object lives without assuming a deployment or interpreting the source.
pub fn equipment_spawns_from_heads(
    chunks: &[crate::clients::hi::models::FilmChunkData],
    heads: &[FilmHeadEvent],
) -> Result<EquipmentSpawnStream, super::DecodeError> {
    let mut indexed = std::collections::BTreeMap::new();
    for h in heads {
        indexed
            .entry((h.source.chunk_index, h.source.payload_offset))
            .or_insert(h);
    }
    equipment_spawn_scan(chunks, |source, _| {
        indexed
            .get(&(source.chunk_index, source.payload_offset))
            .filter(|h| h.source == source)
            .map(|h| h.event.clone())
    })
}

/// Native source traversal and counters. Unlike the native raw reference helpers,
/// truncated references produce a counted refusal instead of an indexing panic.
/// The scanner only reads the first event in each packet's list.
pub fn scan_equipment_spawn_events(
    chunks: &[crate::clients::hi::models::FilmChunkData],
) -> Result<EquipmentSpawnStream, super::DecodeError> {
    equipment_spawn_scan(chunks, |_, data| super::decode_packet_head_event(data))
}

fn equipment_spawn_scan(
    chunks: &[crate::clients::hi::models::FilmChunkData],
    mut head: impl FnMut(FilmPacket, &[u8]) -> Option<super::DecodedHeadEvent>,
) -> Result<EquipmentSpawnStream, super::DecodeError> {
    let selected = super::fire_events::native_chunk_prefix(chunks)?;
    let mut out = EquipmentSpawnStream::default();
    for chunk in selected {
        out.stats.chunks += 1;
        for (packet_index, packet) in super::fire_events::native_chunk_packets(chunk)
            .into_iter()
            .enumerate()
        {
            if packet.packet_type != 0 || packet.payload_size == 0 {
                continue;
            }
            let data =
                &chunk.data[packet.payload_offset..packet.payload_offset + packet.payload_size];
            out.stats.packets += 1;
            // PacketHeadEventType reads the nine-bit header with a zero tail;
            // one physical byte is enough for the continuation/list denominator.
            out.stats.lists += usize::from(data[0] & 0x40 != 0);
            let Some(event) = head(packet, data).filter(|h| h.kind == 103) else {
                continue;
            };
            if event.references.len() != 3 {
                out.stats.truncated += 1;
                continue;
            }
            let refs = &event.references;
            let life = |i: usize| {
                refs[i].value.as_ref().map(|r| ObjectLife {
                    slot: r.index + 512,
                    generation: r.generation,
                })
            };
            let record = EquipmentSpawnEvent {
                packet_index: Some(packet_index),
                packet,
                source: life(0),
                spawned: life(1),
                reference_2_present: refs[2].value.is_some(),
            };
            out.stats.events += 1;
            out.stats.with_source += usize::from(record.source.is_some());
            out.stats.with_spawned += usize::from(record.spawned.is_some());
            out.stats.reference_2 += usize::from(record.reference_2_present);
            out.records.push(record);
        }
    }
    Ok(out)
}

#[cfg(test)]
mod equipment_packet_index_tests {
    use super::*;
    use crate::clients::hi::models::{FilmChunk, FilmChunkData};
    use crate::theater::*;

    #[test]
    fn equipment_outputs_keep_native_packet_ordinals() {
        let mut data = Vec::new();
        for (i, kind) in [2u16, 0, 0].into_iter().enumerate() {
            data.extend_from_slice(&kind.to_le_bytes());
            data.extend_from_slice(&[0, 0]);
            data.extend_from_slice(&1u32.to_le_bytes());
            data.extend_from_slice(&(1000 + i as u64).to_le_bytes());
            data.push(0x40);
        }
        let chunks = [FilmChunkData {
            metadata: FilmChunk {
                index: 1,
                chunk_type: 2,
                start_time_offset_ms: 0,
                duration_ms: 1,
                size: data.len() as i64,
                file_relative_path: String::new(),
            },
            data,
        }];
        let packets = crate::theater::fire_events::native_chunk_packets(&chunks[0]);
        let heads: Vec<_> = packets[1..]
            .iter()
            .map(|source| FilmHeadEvent {
                source: *source,
                event: DecodedHeadEvent {
                    config: false,
                    kind: 103,
                    references: (0..3)
                        .map(|_| EventReference {
                            domain: 0,
                            start_bit: 0,
                            end_bit: 1,
                            value: None,
                        })
                        .collect(),
                    payload: None,
                    end_bit: 3,
                    stop: HeadEventStop::ReferenceFieldsRead,
                },
            })
            .collect();
        let spawns = equipment_spawns_from_heads(&chunks, &heads).unwrap();
        assert_eq!(
            spawns
                .records
                .iter()
                .map(|r| r.packet_index)
                .collect::<Vec<_>>(),
            vec![Some(1), Some(2)]
        );
        assert_eq!(spawns.records[0].packet.payload_offset, 33);
        let restored: EquipmentSpawnStream =
            serde_json::from_value(serde_json::to_value(&spawns).unwrap()).unwrap();
        assert_eq!(restored, spawns);

        let channels = BipedChannels {
            abilities: packets[1..]
                .iter()
                .enumerate()
                .map(|(i, source)| BipedChannelRead {
                    packet_index: None,
                    source: *source,
                    slot: 512,
                    record_start_bit: 0,
                    component: DecodedComponent {
                        references: vec![],
                        diagnostics: Default::default(),
                        name: "biped-desired-ability-set-component".into(),
                        start_bit: 0,
                        end_bit: 0,
                        fields: [("counter", 5 + i as u64), ("rank", 22 + i as u64)]
                            .into_iter()
                            .map(|(name, raw)| ComponentField {
                                name: name.into(),
                                raw,
                                bit: 0,
                                width: 0,
                            })
                            .collect(),
                    },
                })
                .collect(),
            ..Default::default()
        };
        let map = FilmMapBounds {
            module: String::new(),
            min: [0.; 3],
            max: [1.; 3],
            axis_widths: [13, 13, 14],
            region: 0,
            region_index_bits: 1,
        };
        let positions = scan_biped_positions(&chunks, &map, Default::default(), &[]).unwrap();
        let registry = FilmRegistry {
            archetypes: (0..36)
                .map(|index| FilmArchetype {
                    index,
                    components: vec![],
                    levels: vec![],
                })
                .collect(),
            major_version: 41,
            format_version: 27,
            end_byte: 0,
            truncated: false,
        };
        let changes =
            scan_equipment_changes(&chunks, &registry, &positions, &channels, &map).unwrap();
        assert_eq!(
            changes
                .assembly
                .records
                .iter()
                .map(|r| r.packet_index)
                .collect::<Vec<_>>(),
            vec![Some(1), Some(2)]
        );
        assert_eq!(changes.assembly.records[0].source.payload_offset, 33);
        let restored: EquipmentChangeStream =
            serde_json::from_value(serde_json::to_value(&changes).unwrap()).unwrap();
        assert_eq!(restored, changes);
        let emissions: Vec<_> = channels
            .ability_emissions()
            .map(|ability| EquipmentEmission {
                chunk_number: None,
                ability,
                recovery: None,
            })
            .collect();
        let standalone = assemble_equipment_changes(&emissions, &Default::default());
        assert!(standalone.records.iter().all(|r| r.packet_index.is_none()));
    }
}
