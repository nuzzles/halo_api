//! Native packet-head events. These readers do not imply complete event-list coverage.
use super::{DecodeError, FilmPacket, bits::Cursor, packets};
use crate::clients::hi::models::FilmChunkData;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A guarded reference. Domain 3 uses the reference parser's measured seven bits;
/// component handle categories are a separate table and must not be substituted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventReference {
    pub domain: u8,
    pub start_bit: usize,
    pub end_bit: usize,
    pub value: Option<EventReferenceValue>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventReferenceValue {
    pub index: u32,
    pub generation: u8,
    /// Domain-one probe: true selects a nine-bit index, false thirteen bits.
    pub narrow: Option<bool>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HeadEventPayload {
    Pickup {
        class: u8,
        catalog_id: Option<u32>,
        /// Read only when the catalog gate is set, matching the reference scanner.
        more_events: Option<bool>,
    },
    VehicleSeat {
        seat: u8,
    },
    Zoom {
        level: u8,
    },
    /// The reference exposes three references without interpreting the later body.
    SpawnedObjectReferences,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HeadEventStop {
    /// The fields exposed by the corresponding reference head scanner were read.
    /// This does not establish the boundary of the complete event or event list.
    ReferenceFieldsRead,
    UnsupportedType,
    Truncated,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecodedHeadEvent {
    pub config: bool,
    pub kind: u8,
    pub references: Vec<EventReference>,
    pub payload: Option<HeadEventPayload>,
    /// First unread bit, or source_bits + 1 on a bounded reference refusal.
    /// That refusal marker is not a consumed bit and never proves a body boundary.
    pub end_bit: usize,
    pub stop: HeadEventStop,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilmHeadEvent {
    pub source: FilmPacket,
    pub event: DecodedHeadEvent,
}

pub(super) fn reference(r: &mut Cursor<'_>, domain: u8) -> Option<EventReference> {
    let result = (|| {
        let start_bit = r.position;
        let value = if r.read(1)? == 0 {
            None
        } else {
            let narrow = if domain == 1 {
                Some(r.read(1)? != 0)
            } else {
                None
            };
            let width = match domain {
                1 if narrow == Some(true) => 9,
                0 | 1 | 7 | 8 => 13,
                2 | 5 => 8,
                3 => 7,
                4 | 6 => 9,
                _ => return None,
            };
            // The native bounded helper guards index and generation together.
            let packed = r.read(width + 2)?;
            Some(EventReferenceValue {
                index: (packed >> 2) as u32,
                generation: (packed & 3) as u8,
                narrow,
            })
        };
        Some(EventReference {
            domain,
            start_bit,
            end_bit: r.position,
            value,
        })
    })();
    if result.is_none() {
        // Native refTronquee sentinel, not a consumed or zero-filled source bit.
        r.position = r.source_bits().saturating_add(1);
    }
    result
}

/// Read the first event using the pinned reference's domains and payload fields.
/// None means an empty list or an incomplete nine-bit head. Unknown types remain
/// explicit. Failed payload reads never zero-fill absent bits as valid data.
pub fn decode_packet_head_event(data: &[u8]) -> Option<DecodedHeadEvent> {
    decode_head_with_cursor(Cursor::new(data, 0)?)
}

pub(super) fn decode_head_with_cursor(mut r: Cursor<'_>) -> Option<DecodedHeadEvent> {
    let config = r.read(1)? != 0;
    if r.read(1)? == 0 {
        return None;
    }
    let kind = r.read(7)? as u8;
    let mut out = DecodedHeadEvent {
        config,
        kind,
        references: Vec::new(),
        payload: None,
        end_bit: r.position,
        stop: HeadEventStop::UnsupportedType,
    };
    let domains = match kind {
        8 => [2, 3, 7],
        9 => [2, 7, 8],
        21 => [4, 8, 7],
        22 => [1, 1, 7],
        103 => [0, 0, 7],
        _ => return Some(out),
    };
    out.stop = HeadEventStop::Truncated;
    for domain in domains {
        let Some(value) = reference(&mut r, domain) else {
            out.end_bit = r.position;
            return Some(out);
        };
        out.references.push(value);
    }
    out.payload = match kind {
        9 => pickup_payload(&mut r),
        8 | 22 => r
            .read(6)
            .map(|seat| HeadEventPayload::VehicleSeat { seat: seat as u8 }),
        21 => r
            .read(2)
            .map(|level| HeadEventPayload::Zoom { level: level as u8 }),
        103 => Some(HeadEventPayload::SpawnedObjectReferences),
        _ => unreachable!(),
    };
    out.end_bit = r.position;
    if out.payload.is_some() {
        out.stop = HeadEventStop::ReferenceFieldsRead;
    }
    Some(out)
}

fn pickup_payload(r: &mut Cursor<'_>) -> Option<HeadEventPayload> {
    let class = r.read(3)? as u8;
    let (catalog_id, more_events) = if r.read(1)? != 0 {
        (Some(r.read(32)? as u32), Some(r.read(1)? != 0))
    } else {
        (None, None)
    };
    Some(HeadEventPayload::Pickup {
        class,
        catalog_id,
        more_events,
    })
}

/// Scan packet heads in chunk order, retaining unsupported types and source times.
/// Later events in the same list are not scanned without established boundaries.
pub fn scan_packet_head_events(
    chunks: &[FilmChunkData],
) -> Result<Vec<FilmHeadEvent>, DecodeError> {
    let packets = packets::index(chunks)?;
    let bytes: BTreeMap<_, _> = chunks.iter().map(|c| (c.metadata.index, &c.data)).collect();
    Ok(packets
        .into_iter()
        .filter(|p| p.packet_type == 0)
        .filter_map(|source| {
            let data = &bytes[&source.chunk_index]
                [source.payload_offset..source.payload_offset + source.payload_size];
            decode_packet_head_event(data).map(|event| FilmHeadEvent { source, event })
        })
        .collect())
}

impl DecodedHeadEvent {
    /// Picker slot, catalog identifier and class. Classes zero and one identify
    /// weapon pickups in the reference; other classes remain unnamed.
    pub fn pickup(&self) -> Option<(u32, u32, u8)> {
        let HeadEventPayload::Pickup {
            class,
            catalog_id: Some(catalog),
            ..
        } = self.payload.as_ref()?
        else {
            return None;
        };
        let picker = self.references.first()?.value.as_ref()?;
        Some((picker.index.checked_add(512)?, *catalog, *class))
    }
    /// Reference unit_zoom's measured domain-four base. Generation remains in
    /// references[0]; level zero means leaving scope.
    pub fn zoom(&self) -> Option<(u32, u8)> {
        let HeadEventPayload::Zoom { level } = self.payload.as_ref()? else {
            return None;
        };
        let unit = self.references.first()?.value.as_ref()?;
        Some((unit.index.checked_add(512)?, *level))
    }
    /// Spawned object's slot and generation (reference 1, measured base 512).
    pub fn spawned_object(&self) -> Option<(u32, u8)> {
        if self.payload != Some(HeadEventPayload::SpawnedObjectReferences) {
            return None;
        }
        let object = self.references.get(1)?.value.as_ref()?;
        Some((object.index.checked_add(512)?, object.generation))
    }
    /// Occupant slot using the independently recovered minimum biped slot.
    /// A wide domain-one reference is already an absolute slot.
    pub fn vehicle_occupant(&self, biped_base: u32) -> Option<u32> {
        if !matches!(self.payload, Some(HeadEventPayload::VehicleSeat { .. })) {
            return None;
        }
        let occupant = self.references.first()?.value.as_ref()?;
        if self.kind == 22 && occupant.narrow == Some(false) {
            Some(occupant.index)
        } else {
            biped_base.checked_add(occupant.index)
        }
    }
    /// Only exit events identify the vehicle through reference 1 in this port.
    pub fn exited_vehicle(&self, biped_base: u32) -> Option<(u32, u8)> {
        if self.kind != 22 || !matches!(self.payload, Some(HeadEventPayload::VehicleSeat { .. })) {
            return None;
        }
        let vehicle = self.references.get(1)?.value.as_ref()?;
        let slot = if vehicle.narrow == Some(false) {
            vehicle.index
        } else {
            biped_base.checked_add(vehicle.index)?
        };
        Some((slot, vehicle.generation))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn captured_and_synthetic_heads_match_native_go_outputs() {
        let mut json = String::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/event-heads-levelup-v41.json.zlib")[..],
        )
        .read_to_string(&mut json)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
        let mut counts = BTreeMap::<u8, usize>::new();
        let mut captured = 0;
        for row in rows {
            let hex = row["hex"].as_str().unwrap();
            let data: Vec<u8> = (0..hex.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                .collect();
            let event = decode_packet_head_event(&data).unwrap();
            assert_eq!(event.stop, HeadEventStop::ReferenceFieldsRead, "{row}");
            assert_eq!(u64::from(event.kind), row["kind"].as_u64().unwrap());
            assert_eq!(event.end_bit as u64, row["end"].as_u64().unwrap());
            *counts.entry(event.kind).or_default() += 1;
            if row["source"] != "synthetic" {
                captured += 1;
            }
            for (actual, expected) in event.references.iter().zip(row["refs"].as_array().unwrap()) {
                assert_eq!(
                    u64::from(actual.domain),
                    expected["domain"].as_u64().unwrap()
                );
                assert_eq!(actual.start_bit as u64, expected["start"].as_u64().unwrap());
                let expected = &expected["ref"];
                assert_eq!(actual.end_bit as u64, expected["EndBit"].as_u64().unwrap());
                assert_eq!(
                    actual.value.is_some(),
                    expected["Present"].as_bool().unwrap()
                );
                if let Some(value) = &actual.value {
                    assert_eq!(u64::from(value.index), expected["Index"].as_u64().unwrap());
                    assert_eq!(
                        u64::from(value.generation),
                        expected["Gen"].as_u64().unwrap()
                    );
                    assert_eq!(
                        u64::from(value.narrow.unwrap_or(false)),
                        expected["Sonde"].as_u64().unwrap()
                    );
                }
            }
            match event.payload.as_ref().unwrap() {
                HeadEventPayload::Pickup { more_events, .. } => {
                    assert_eq!(
                        event.pickup().is_some(),
                        row["pickup_valid"].as_bool().unwrap()
                    );
                    if let Some((slot, catalog, class)) = event.pickup() {
                        assert_eq!(u64::from(slot), row["pickup"]["Slot"].as_u64().unwrap());
                        assert_eq!(
                            u64::from(catalog),
                            row["pickup"]["CatalogID"].as_u64().unwrap()
                        );
                        assert_eq!(u64::from(class), row["pickup"]["Class"].as_u64().unwrap());
                        assert_eq!(
                            u64::from(more_events.unwrap()),
                            row["pickup_stats"]["MultiEvent"].as_u64().unwrap()
                        );
                    }
                }
                HeadEventPayload::VehicleSeat { seat } => {
                    let expected = &row["vehicle"];
                    assert_eq!(u64::from(*seat), expected["Seat"].as_u64().unwrap());
                    assert_eq!(
                        event.vehicle_occupant(512).is_some(),
                        expected["OccupantPresent"].as_bool().unwrap()
                    );
                    if let Some(slot) = event.vehicle_occupant(512) {
                        assert_eq!(u64::from(slot), expected["OccupantSlot"].as_u64().unwrap());
                    }
                    assert_eq!(
                        event.exited_vehicle(512).is_some(),
                        expected["VehicleSlotValid"].as_bool().unwrap()
                    );
                    if let Some((slot, generation)) = event.exited_vehicle(512) {
                        assert_eq!(u64::from(slot), expected["VehicleSlot"].as_u64().unwrap());
                        assert_eq!(
                            u64::from(generation),
                            expected["VehicleGen"].as_u64().unwrap()
                        );
                    }
                }
                HeadEventPayload::Zoom { .. } => {
                    assert_eq!(event.zoom().is_some(), row["zoom_valid"].as_bool().unwrap());
                    if let Some((slot, level)) = event.zoom() {
                        assert_eq!(u64::from(slot), row["zoom"]["Slot"].as_u64().unwrap());
                        assert_eq!(u64::from(level), row["zoom"]["Level"].as_u64().unwrap());
                    }
                }
                HeadEventPayload::SpawnedObjectReferences => {
                    assert_eq!(
                        event.spawned_object().is_some(),
                        row["spawn"]["SpawnedValid"].as_bool().unwrap()
                    );
                    if let Some((slot, generation)) = event.spawned_object() {
                        assert_eq!(
                            u64::from(slot),
                            row["spawn"]["Spawned"]["Slot"].as_u64().unwrap()
                        );
                        assert_eq!(
                            u64::from(generation),
                            row["spawn"]["Spawned"]["Gen"].as_u64().unwrap()
                        );
                    }
                }
            }
            for n in 0..event.end_bit.div_ceil(8) {
                if let Some(short) = decode_packet_head_event(&data[..n]) {
                    assert_eq!(short.stop, HeadEventStop::Truncated);
                    assert!(
                        short.end_bit <= n * 8
                            || (short.references.len() < 3 && short.end_bit == n * 8 + 1)
                    );
                    assert!(short.payload.is_none());
                }
            }
            let json = serde_json::to_vec(&event).unwrap();
            assert_eq!(
                serde_json::from_slice::<DecodedHeadEvent>(&json).unwrap(),
                event
            );
        }
        assert!(captured > 0);
        assert_eq!(counts.len(), 5);
        assert!(counts.values().all(|n| *n >= 256));
        println!("event-head cases={counts:?}, captured={captured}");
    }

    #[test]
    fn native_bounded_event_references_d61443e() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/bounded-event-refs-d61443e-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        let bytes = |s: &str| -> Vec<u8> {
            (0..s.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
                .collect()
        };
        let check = |actual: &EventReference, expected: &serde_json::Value| {
            assert_eq!(actual.end_bit, expected["EndBit"]);
            assert_eq!(actual.value.is_some(), expected["Present"]);
            if let Some(v) = &actual.value {
                assert_eq!(v.index, expected["Index"]);
                assert_eq!(v.generation, expected["Gen"]);
                assert_eq!(u8::from(v.narrow.unwrap_or(false)), expected["Sonde"]);
            }
        };
        assert_eq!(rows["refs"].as_array().unwrap().len(), 36864);
        for row in rows["refs"].as_array().unwrap() {
            let data = bytes(row["hex"].as_str().unwrap());
            let mut cursor = Cursor::new(&data, 0).unwrap();
            cursor.position = row["start"].as_u64().unwrap() as usize;
            let actual = reference(&mut cursor, row["domain"].as_u64().unwrap() as u8);
            let expected = &row["ref"];
            assert_eq!(actual.is_none(), expected["Tronquee"]);
            assert_eq!(cursor.position, expected["EndBit"]);
            if let Some(r) = actual {
                assert_eq!(r.start_bit, row["start"]);
                check(&r, expected);
                assert!(r.end_bit <= data.len() * 8);
            }
        }
        assert_eq!(rows["heads"].as_array().unwrap().len(), 12288);
        for (i, row) in rows["heads"].as_array().unwrap().iter().enumerate() {
            let data = bytes(row["hex"].as_str().unwrap());
            let head = decode_packet_head_event(&data).unwrap();
            for (j, r) in row["refs"].as_array().unwrap().iter().enumerate() {
                if r["Tronquee"] == true {
                    assert_eq!(head.references.len(), j);
                    assert_eq!(head.end_bit, r["EndBit"]);
                    assert_eq!(head.stop, HeadEventStop::Truncated);
                    assert!(head.payload.is_none());
                    break;
                }
                check(&head.references[j], r);
            }
            if i % 64 == 0 {
                use crate::theater::{FilmSource, KeyframeRecoveryPolicy, NativeFilmData};
                let mut packet = vec![0; 4];
                packet.extend_from_slice(&(data.len() as u32).to_le_bytes());
                packet.extend_from_slice(&0u64.to_le_bytes());
                packet.extend_from_slice(&data);
                let chunks = [[41u32.to_le_bytes(), 27u32.to_le_bytes()].concat(), packet];
                let source = FilmSource::load(&chunks, &[]).unwrap();
                let parsed = NativeFilmData::parse_v41_with_recovery(
                    &source,
                    KeyframeRecoveryPolicy::SequentialOnly,
                )
                .unwrap();
                assert_eq!(parsed.chunks[1].data, chunks[1]);
                assert_eq!(parsed.chunks[1].packets[0].event_head.as_ref(), Some(&head));
            }
        }
    }

    #[test]
    fn unknown_event_types_are_retained_without_guessing_payloads() {
        let event = decode_packet_head_event(&[0xc0, 0]).unwrap();
        assert_eq!(event.kind, 0);
        assert_eq!(event.end_bit, 9);
        assert_eq!(event.stop, HeadEventStop::UnsupportedType);
        assert!(event.references.is_empty());
        assert!(decode_packet_head_event(&[0x80]).is_none());
        assert!(decode_packet_head_event(&[0xc0]).is_none());
    }
}
