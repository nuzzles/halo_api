//! Boarding and exit evidence from the pinned native packet-head scanner.
use super::{DecodeError, bits::Cursor, fire_events};
use crate::clients::hi::models::FilmChunkData;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct VehicleEvent {
    pub kind: u8,
    pub chunk: i64,
    pub packet_index: usize,
    #[serde(rename = "TimestampUS")]
    pub timestamp_us: u64,
    pub occupant_present: bool,
    pub occupant_sonde: u8,
    pub occupant_slot: u32,
    pub occupant_in_band: bool,
    pub vehicle_slot: u32,
    pub vehicle_slot_valid: bool,
    pub vehicle_gen: u32,
    pub seat: u32,
    pub seat_valid: bool,
}

#[derive(Default)]
struct Reference {
    present: bool,
    sonde: u8,
    index: u32,
    generation: u32,
}
fn reference(r: &mut Cursor<'_>, domain_one: bool, width: usize) -> Option<Reference> {
    if r.read(1)? == 0 {
        return Some(Reference::default());
    }
    let sonde = if domain_one { r.read(1)? as u8 } else { 0 };
    Some(Reference {
        present: true,
        sonde,
        index: r.read(if sonde == 1 { 9 } else { width })? as u32,
        generation: r.read(2)? as u32,
    })
}

/// Decode types 8 (board) and 22 (exit). A missing seat is valid evidence.
/// Truncated references return an explicit error; they must not be zero-filled
/// or confused with absent guarded references. The native head reader separately
/// retains the d61443e reference-refusal marker.
/// Type 53 is deliberately not interpreted by this native scanner.
pub fn decode_vehicle_event(
    payload: &[u8],
    base: u32,
    band: &BTreeSet<u32>,
) -> Result<Option<VehicleEvent>, DecodeError> {
    if payload.is_empty() {
        return Ok(None);
    }
    let mut head = Cursor::new_padded(payload, 1);
    let more = head.read(1).unwrap() != 0;
    let kind = head.read(7).unwrap() as u8;
    if !more || !matches!(kind, 8 | 22) {
        return Ok(None);
    }
    let truncated = || DecodeError::Missing("complete vehicle event references");
    let mut r = Cursor::new(payload, 9).ok_or_else(truncated)?;
    let exit = kind == 22;
    let occupant = reference(&mut r, exit, if exit { 13 } else { 8 }).ok_or_else(truncated)?;
    let vehicle = reference(&mut r, exit, if exit { 13 } else { 7 }).ok_or_else(truncated)?;
    reference(&mut r, false, 13).ok_or_else(truncated)?;
    let mut ev = VehicleEvent {
        kind,
        ..Default::default()
    };
    if occupant.present {
        ev.occupant_present = true;
        ev.occupant_sonde = occupant.sonde;
        ev.occupant_slot = if !exit || occupant.sonde == 1 {
            base.wrapping_add(occupant.index)
        } else {
            occupant.index
        };
        ev.occupant_in_band = band.contains(&ev.occupant_slot);
    }
    if exit && vehicle.present {
        ev.vehicle_slot_valid = true;
        ev.vehicle_gen = vehicle.generation;
        ev.vehicle_slot = if vehicle.sonde == 1 {
            base.wrapping_add(vehicle.index)
        } else {
            vehicle.index
        };
    }
    if let Some(seat) = r.read(6) {
        ev.seat = seat as u32;
        ev.seat_valid = true;
    }
    Ok(Some(ev))
}

/// Scan readable native delta packets with an explicit biped band. Packet indices
/// count every packet kind. Base is the minimum band slot, or zero for an empty band.
pub fn scan_vehicle_events_for_band(
    chunks: &[FilmChunkData],
    band: &BTreeSet<u32>,
) -> Result<Vec<VehicleEvent>, DecodeError> {
    let chunks = fire_events::native_chunk_prefix(chunks)?;
    let base = band.first().copied().unwrap_or(0);
    let mut events = Vec::new();
    for c in chunks {
        for (index, p) in fire_events::native_chunk_packets(c).into_iter().enumerate() {
            if p.packet_type != 0 || p.payload_size == 0 {
                continue;
            }
            let payload = &c.data[p.payload_offset..p.payload_offset + p.payload_size];
            if let Some(mut event) =
                decode_vehicle_event(payload, base, band).map_err(|_| DecodeError::Truncated {
                    chunk: c.metadata.index,
                    offset: p.payload_offset,
                })?
            {
                event.chunk = i64::from(c.metadata.index);
                event.packet_index = index;
                event.timestamp_us = p.timestamp_us;
                events.push(event);
            }
        }
    }
    Ok(events)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn packet_attribution_and_partial_chunks() {
        use crate::clients::hi::models::FilmChunk;
        let mut data = Vec::new();
        for (kind, time, payload) in [
            (1u16, 12u64, vec![0]),
            (0, 13, vec![0xc4, 0]),
            (0, 14, vec![0xcb, 0, 0]),
            (7, 15, vec![]),
        ] {
            data.extend(kind.to_le_bytes());
            data.extend([0; 2]);
            data.extend((payload.len() as u32).to_le_bytes());
            data.extend(time.to_le_bytes());
            data.extend(payload);
        }
        // Bytes after the native End packet are not evidence.
        data.extend([255; 32]);
        let chunk = FilmChunkData {
            metadata: FilmChunk {
                index: 1,
                chunk_type: 2,
                start_time_offset_ms: 0,
                duration_ms: 0,
                size: data.len() as i64,
                file_relative_path: String::new(),
            },
            data,
        };
        let events = scan_vehicle_events_for_band(&[chunk], &BTreeSet::new()).unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(
            (
                events[0].chunk,
                events[0].packet_index,
                events[0].timestamp_us
            ),
            (1, 1, 13)
        );
        assert!(!events[0].seat_valid);
        assert_eq!(
            (
                events[1].chunk,
                events[1].packet_index,
                events[1].timestamp_us
            ),
            (1, 2, 14)
        );
        assert!(events[1].seat_valid);
        assert!(scan_vehicle_events_for_band(&[], &BTreeSet::new()).is_err());
    }
    #[derive(Deserialize)]
    struct Case {
        payload: Vec<u8>,
        base: u32,
        band: BTreeSet<u32>,
        event: Option<VehicleEvent>,
        panicked: bool,
    }
    #[test]
    fn native_vehicle_events() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/vehicle-events-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&raw).unwrap();
        for (i, c) in cases.into_iter().enumerate() {
            let result = decode_vehicle_event(&c.payload, c.base, &c.band);
            if c.panicked {
                assert!(result.is_err(), "panic {i}");
            } else {
                assert_eq!(result.unwrap(), c.event, "case {i}");
            }
        }
    }
}
