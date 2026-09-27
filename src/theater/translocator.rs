//! Type-117 teleport events from LevelUp's transloc_events.go.
//! See reference/LEVELUP_LICENSE.txt. Failed position reads retain the known event.
use super::{DecodeError, FilmMapBounds, FilmPacket, bits::Cursor, packets};
use crate::clients::hi::models::FilmChunkData;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TranslocatorStop {
    PositionsRead,
    Truncated,
    UnsupportedReferences,
    MissingMap,
    InvalidMap,
    UnknownRegion(u32),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TeleportPosition {
    pub start_bit: usize,
    pub end_bit: usize,
    /// None is the engine's default box (22 bits, +/-20000), not an inferred map.
    pub region: Option<u32>,
    pub axis_bits: [usize; 3],
    pub quantized: [u32; 3],
    pub world: [f32; 3],
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TranslocatorEvent {
    pub config: bool,
    pub slot: u32,
    pub generation: u8,
    pub other_reference_gates: Option<[bool; 2]>,
    pub effect_present: Option<bool>,
    pub effect: Option<u32>,
    pub from: Option<TeleportPosition>,
    pub to: Option<TeleportPosition>,
    pub end_bit: usize,
    pub stop: TranslocatorStop,
}
impl TranslocatorEvent {
    /// A jump is usable only after both positions were read. A partial first
    /// position is retained for inspection but never supplied as a complete jump.
    pub fn positions(&self) -> Option<[[f32; 3]; 2]> {
        if self.stop != TranslocatorStop::PositionsRead {
            return None;
        }
        Some([self.from.as_ref()?.world, self.to.as_ref()?.world])
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilmTranslocatorEvent {
    pub source: FilmPacket,
    pub event: TranslocatorEvent,
}

/// Decode the first event when it is type 117 and has a complete unit reference.
/// Missing map context or an unsupported payload does not discard slot/time data.
pub fn decode_translocator_head(
    data: &[u8],
    map: Option<&FilmMapBounds>,
) -> Option<TranslocatorEvent> {
    decode_translocator_with_cursor(data, map, Cursor::new(data, 0)?)
}

fn decode_translocator_with_cursor(
    data: &[u8],
    map: Option<&FilmMapBounds>,
    mut r: Cursor<'_>,
) -> Option<TranslocatorEvent> {
    let config = r.read(1)? != 0;
    if r.read(1)? == 0 || r.read(7)? != 117 || r.read(1)? == 0 {
        return None;
    }
    let slot = 512 + r.read(8)? as u32;
    let generation = r.read(2)? as u8;
    let mut event = TranslocatorEvent {
        config,
        slot,
        generation,
        other_reference_gates: None,
        effect_present: None,
        effect: None,
        from: None,
        to: None,
        end_bit: r.position,
        stop: TranslocatorStop::Truncated,
    };
    event.stop =
        payload(&mut r, map, &mut event, data.len() * 8).unwrap_or(TranslocatorStop::Truncated);
    event.end_bit = r.position;
    Some(event)
}

fn payload(
    r: &mut Cursor<'_>,
    map: Option<&FilmMapBounds>,
    event: &mut TranslocatorEvent,
    source_bits: usize,
) -> Option<TranslocatorStop> {
    let gates = [r.read(1)? != 0, r.read(1)? != 0];
    event.other_reference_gates = Some(gates);
    if gates.iter().any(|v| *v) {
        return Some(TranslocatorStop::UnsupportedReferences);
    }
    let effect_present = r.read(1)? != 0;
    event.effect_present = Some(effect_present);
    if effect_present {
        event.effect = Some(r.read(32)? as u32);
    }
    match position(r, map, source_bits) {
        Ok(from) => event.from = Some(from),
        Err(stop) => return Some(stop),
    }
    match position(r, map, source_bits) {
        Ok(to) => event.to = Some(to),
        Err(stop) => return Some(stop),
    }
    Some(TranslocatorStop::PositionsRead)
}

fn position(
    r: &mut Cursor<'_>,
    map: Option<&FilmMapBounds>,
    source_bits: usize,
) -> Result<TeleportPosition, TranslocatorStop> {
    let start_bit = r.position;
    let mut axis_bits = [22; 3];
    let mut min = [-20000.0f32; 3];
    let mut max = [20000.0f32; 3];
    let region = if r.read(1).ok_or(TranslocatorStop::Truncated)? == 0 {
        let map = map.ok_or(TranslocatorStop::MissingMap)?;
        if map.region_index_bits.max(1) > 8
            || (0..3).any(|i| {
                !(1..=26).contains(&map.axis_widths[i])
                    || !map.min[i].is_finite()
                    || !map.max[i].is_finite()
                    || map.max[i] <= map.min[i]
            })
        {
            return Err(TranslocatorStop::InvalidMap);
        }
        let region = r
            .read(map.region_index_bits.max(1))
            .ok_or(TranslocatorStop::Truncated)? as u32;
        if region != map.region {
            return Err(TranslocatorStop::UnknownRegion(region));
        }
        axis_bits = map.axis_widths;
        min = map.min;
        max = map.max;
        Some(region)
    } else {
        None
    };
    let mut quantized = [0; 3];
    let mut world = [0.0; 3];
    for i in 0..3 {
        quantized[i] = r.read(axis_bits[i]).ok_or(TranslocatorStop::Truncated)? as u32;
        let step = (f64::from(max[i]) - f64::from(min[i])) / ((1u64 << axis_bits[i]) as f64);
        world[i] = (f64::from(min[i]) + step * (f64::from(quantized[i]) + 0.5)) as f32;
    }
    // Native readTranslocVec consumes padded axes, then rejects the vector
    // when its logical end lies beyond the physical source.
    if r.position > source_bits {
        return Err(TranslocatorStop::Truncated);
    }
    Ok(TeleportPosition {
        start_bit,
        end_bit: r.position,
        region,
        axis_bits,
        quantized,
        world,
    })
}

/// Native decoder outcome, including its synthetic zero-tail convention.
/// Slot/reference values crossing source_bits are not fully recorded identities.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NativeTranslocatorEvent {
    pub event: TranslocatorEvent,
    pub source_bits: usize,
    /// Logical reader extent beyond the payload, including skipped tail fields.
    pub padded_bits: usize,
}
impl NativeTranslocatorEvent {
    pub fn unit_reference_recorded(&self) -> bool {
        self.source_bits >= 20
    }
}

pub fn decode_native_translocator_head(
    data: &[u8],
    map: Option<&FilmMapBounds>,
) -> Option<NativeTranslocatorEvent> {
    let event = decode_translocator_with_cursor(data, map, Cursor::new_padded(data, 0))?;
    Some(NativeTranslocatorEvent {
        padded_bits: event.end_bit.saturating_sub(data.len() * 8),
        source_bits: data.len() * 8,
        event,
    })
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilmNativeTranslocatorEvent {
    pub source: FilmPacket,
    pub packet_index: usize,
    pub read: NativeTranslocatorEvent,
}

/// Native source selection and head admission; padded reads retain provenance.
pub fn scan_native_translocator_events(
    chunks: &[FilmChunkData],
    map: Option<&FilmMapBounds>,
) -> Vec<FilmNativeTranslocatorEvent> {
    let mut out = Vec::new();
    for chunk in super::fire_events::native_chunk_prefix(chunks).unwrap_or_default() {
        for (packet_index, source) in super::fire_events::native_chunk_packets(chunk)
            .into_iter()
            .enumerate()
        {
            if source.packet_type != 0 || source.payload_size < 2 {
                continue;
            }
            let data =
                &chunk.data[source.payload_offset..source.payload_offset + source.payload_size];
            if data[0] != 0xfa {
                continue;
            }
            if let Some(read) = decode_native_translocator_head(data, map) {
                out.push(FilmNativeTranslocatorEvent {
                    source,
                    packet_index,
                    read,
                });
            }
        }
    }
    super::native_sort::sort_by(&mut out, |a, b| {
        a.source.timestamp_us.cmp(&b.source.timestamp_us)
    });
    out
}

/// Scan packet heads and order them by the film's absolute timestamp, as in Go.
pub fn scan_translocator_events(
    chunks: &[FilmChunkData],
    map: Option<&FilmMapBounds>,
) -> Result<Vec<FilmTranslocatorEvent>, DecodeError> {
    let packets = packets::index(chunks)?;
    let bytes: BTreeMap<_, _> = chunks.iter().map(|c| (c.metadata.index, &c.data)).collect();
    let mut events: Vec<_> = packets
        .into_iter()
        .filter(|p| p.packet_type == 0)
        .filter_map(|source| {
            let data = &bytes[&source.chunk_index]
                [source.payload_offset..source.payload_offset + source.payload_size];
            // The reference scanner requires the config bit through this family byte.
            if data.first() != Some(&0xfa) {
                return None;
            }
            decode_translocator_head(data, map).map(|event| FilmTranslocatorEvent { source, event })
        })
        .collect();
    events.sort_by_key(|e| e.source.timestamp_us);
    Ok(events)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn native_go_teleport_values_and_failures_match() {
        let mut data = String::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/translocator-levelup-v41.json.zlib")[..],
        )
        .read_to_string(&mut data)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_str(&data).unwrap();
        let mut count = 0;
        let mut captured = 0;
        for row in rows {
            let map: Option<FilmMapBounds> = serde_json::from_value(row["map"].clone()).unwrap();
            let hex = row["hex"].as_str().unwrap();
            let bytes: Vec<u8> = (0..hex.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                .collect();
            let event = decode_translocator_head(&bytes, map.as_ref());
            let native = decode_native_translocator_head(&bytes, map.as_ref());
            assert_eq!(native.is_some(), row["ok"].as_bool().unwrap());
            if let Some(native) = native {
                assert_eq!(
                    native.event.slot as u64,
                    row["event"]["Slot"].as_u64().unwrap()
                );
                assert_eq!(
                    native.event.positions().is_some(),
                    row["event"]["HasPositions"].as_bool().unwrap()
                );
                if let Some(positions) = native.event.positions() {
                    for (actual, name) in positions.iter().zip(["From", "To"]) {
                        for (axis, value) in actual.iter().enumerate() {
                            assert_eq!(
                                value.to_bits(),
                                (row["event"][name][axis].as_f64().unwrap() as f32).to_bits()
                            );
                        }
                    }
                }
            }
            assert_eq!(event.is_some(), row["ok"].as_bool().unwrap(), "{row}");
            if !row["source"].as_str().unwrap().starts_with("synthetic:") {
                captured += 1;
            }
            let Some(event) = event else { continue };
            let bits = super::super::bits::Bits(&bytes);
            assert_eq!(Some(u64::from(event.generation)), bits.read(18, 2));
            if let Some(present) = event.effect_present {
                assert_eq!(Some(u64::from(present)), bits.read(22, 1));
                if let Some(effect) = event.effect {
                    assert_eq!(Some(u64::from(effect)), bits.read(23, 32));
                }
            }
            for position in event.from.iter().chain(event.to.iter()) {
                let mut at = position.start_bit + 1;
                if position.region.is_some() {
                    at += map.as_ref().unwrap().region_index_bits.max(1);
                }
                for i in 0..3 {
                    assert_eq!(
                        Some(u64::from(position.quantized[i])),
                        bits.read(at, position.axis_bits[i])
                    );
                    at += position.axis_bits[i];
                }
                assert_eq!(at, position.end_bit);
            }
            let expected = &row["event"];
            assert_eq!(u64::from(event.slot), expected["Slot"].as_u64().unwrap());
            assert_eq!(
                event.positions().is_some(),
                expected["HasPositions"].as_bool().unwrap(),
                "{row}"
            );
            if let Some(positions) = event.positions() {
                for (actual, name) in positions.iter().zip(["From", "To"]) {
                    for (i, value) in actual.iter().enumerate() {
                        assert_eq!(
                            value.to_bits(),
                            (expected[name][i].as_f64().unwrap() as f32).to_bits(),
                            "{row}"
                        );
                    }
                }
                count += 1;
            }
            if row["end_bit"].as_u64().unwrap() <= bytes.len() as u64 * 8 {
                assert_eq!(
                    event.end_bit as u64,
                    row["end_bit"].as_u64().unwrap(),
                    "{row}"
                );
            }
            assert!(event.end_bit <= bytes.len() * 8);
            let encoded = serde_json::to_vec(&event).unwrap();
            assert_eq!(
                serde_json::from_slice::<TranslocatorEvent>(&encoded).unwrap(),
                event
            );
        }
        assert!(count >= 79 * 4);
        println!(
            "verified 632 map/profile cases and {captured} captured teleports; {count} complete jumps"
        );
    }

    #[test]
    fn native_translocator_padding_matches_reference() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/translocator-padding-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(rows.len(), 2048);
        let mut padded_references = 0;
        for (case, row) in rows.iter().enumerate() {
            let h = row["hex"].as_str().unwrap();
            let data: Vec<_> = (0..h.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap())
                .collect();
            let map: Option<FilmMapBounds> = serde_json::from_value(row["map"].clone()).unwrap();
            let read = decode_native_translocator_head(&data, map.as_ref());
            assert_eq!(read.is_some(), row["ok"].as_bool().unwrap(), "case {case}");
            if let Some(read) = read {
                assert_eq!(
                    u64::from(read.event.slot),
                    row["event"]["Slot"].as_u64().unwrap()
                );
                assert_eq!(
                    read.event.positions().is_some(),
                    row["event"]["HasPositions"].as_bool().unwrap()
                );
                assert_eq!(
                    read.event.end_bit as u64,
                    row["end"].as_u64().unwrap(),
                    "case {case}"
                );
                assert_eq!(
                    read.padded_bits,
                    read.event.end_bit.saturating_sub(data.len() * 8)
                );
                assert_eq!(read.source_bits, data.len() * 8);
                if !read.unit_reference_recorded() {
                    padded_references += 1;
                    assert!(decode_translocator_head(&data, map.as_ref()).is_none());
                    assert!(read.padded_bits > 0);
                }
                let restored: NativeTranslocatorEvent =
                    serde_json::from_slice(&serde_json::to_vec(&read).unwrap()).unwrap();
                assert_eq!(restored, read);
            }
        }
        assert!(padded_references > 0);
    }

    #[test]
    fn native_translocator_source_order_and_admission() {
        use crate::clients::hi::models::FilmChunk;
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/translocator-source-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(rows.len(), 128);
        let mut padded = 0;
        let mut complete = 0;
        for (case, row) in rows.iter().enumerate() {
            let chunks: Vec<_> = row["inputs"]
                .as_array()
                .unwrap()
                .iter()
                .map(|input| {
                    let h = input["hex"].as_str().unwrap();
                    let data: Vec<u8> = (0..h.len())
                        .step_by(2)
                        .map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap())
                        .collect();
                    FilmChunkData {
                        metadata: FilmChunk {
                            index: input["index"].as_i64().unwrap() as i32,
                            chunk_type: 2,
                            start_time_offset_ms: 98765,
                            duration_ms: 1,
                            size: data.len() as i64,
                            file_relative_path: String::new(),
                        },
                        data,
                    }
                })
                .collect();
            let reads = scan_native_translocator_events(&chunks, None);
            let expected = row["output"].as_array().unwrap();
            assert_eq!(reads.len(), expected.len(), "case {case}");
            for (read, expected) in reads.iter().zip(expected) {
                assert_eq!(
                    i64::from(read.source.chunk_index),
                    expected["chunk"].as_i64().unwrap()
                );
                assert_eq!(
                    read.packet_index as u64,
                    expected["packet"].as_u64().unwrap()
                );
                assert_eq!(
                    read.source.payload_offset as u64,
                    expected["start"].as_u64().unwrap()
                );
                assert_eq!(
                    read.source.payload_size as u64,
                    expected["size"].as_u64().unwrap()
                );
                let event = &expected["event"];
                assert_eq!(
                    read.source.timestamp_us,
                    event["TimestampUS"].as_u64().unwrap()
                );
                assert_eq!(read.read.event.slot as u64, event["Slot"].as_u64().unwrap());
                assert_eq!(
                    read.read.event.positions().is_some(),
                    event["HasPositions"].as_bool().unwrap()
                );
                if let Some(positions) = read.read.event.positions() {
                    complete += 1;
                    for (position, name) in positions.iter().zip(["From", "To"]) {
                        for (axis, value) in position.iter().enumerate() {
                            assert_eq!(
                                value.to_bits(),
                                (event[name][axis].as_f64().unwrap() as f32).to_bits()
                            );
                        }
                    }
                }
                padded += usize::from(read.read.padded_bits > 0);
            }
            let restored: Vec<FilmNativeTranslocatorEvent> =
                serde_json::from_slice(&serde_json::to_vec(&reads).unwrap()).unwrap();
            assert_eq!(restored, reads);
        }
        assert!(padded > 0 && complete > 0);
    }

    #[test]
    fn incomplete_unit_reference_never_fabricates_a_teleport() {
        assert!(decode_translocator_head(&[0xfa, 0x80], None).is_none());
        assert!(decode_translocator_head(&[0xfa, 0], None).is_none());
        assert!(decode_translocator_head(&[0xba, 0xff, 0xff], None).is_none());
        let event = decode_translocator_head(&[0xfa, 0xc0, 0x11], None).unwrap();
        assert_eq!(event.slot, 512);
        assert_eq!(event.generation, 1);
        assert_eq!(event.stop, TranslocatorStop::Truncated);
        assert!(event.positions().is_none());
    }
}
