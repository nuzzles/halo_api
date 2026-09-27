//! Native long fire-event records and modal aim grammar (LevelUp fire_events.go).
//! A fire event does not establish a hit or identify a victim.
use super::{
    DecodeError,
    bits::{Bits, Cursor},
};
use crate::clients::hi::models::FilmChunkData;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct FilmFireEvent {
    pub chunk: i64,
    pub packet_index: usize,
    #[serde(rename = "TimestampUS")]
    pub timestamp_us: u64,
    /// Recorded five-bit shooter index, or -1 when its gate is closed.
    pub film_index: i32,
    pub has_shooter: bool,
    pub fire_number: u8,
    pub unit: FireUnitReference,
    pub short: bool,
    pub bloc: bool,
    #[serde(rename = "WeaponID")]
    pub weapon_id: u64,
    pub has_aim: bool,
    pub aim: [f32; 3],
}
impl FilmFireEvent {
    pub fn aim_heading_degrees(&self) -> Option<f64> {
        self.has_aim.then(|| {
            let h = (self.aim[1] as f64).atan2(self.aim[0] as f64) * 180. / std::f64::consts::PI;
            if h < 0. { h + 360. } else { h }
        })
    }
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct FireUnitReference {
    pub present: bool,
    pub slot: u32,
    #[serde(rename = "Gen")]
    pub generation: u32,
    pub probe: bool,
}

/// The former four-bit helper now reads the actual optional five-bit field.
pub fn read_fire_attacker_index(payload: &[u8]) -> Option<u8> {
    read_fire_shooter_index(payload)
}
pub fn read_fire_shooter_index(payload: &[u8]) -> Option<u8> {
    let event = decode_fire_event(payload)?;
    event.has_shooter.then_some(event.film_index as u8)
}
/// Decode the native type-36 head and supported modal aim. This does not
/// establish the end of the full event body or decode non-modal target loops.
pub fn decode_fire_event(payload: &[u8]) -> Option<FilmFireEvent> {
    read_native_fire_event(payload).event
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativeFireAimMethod {
    /// Historical exports only. New reads use the grammar-derived modal offset.
    Fixed,
    Modal,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeFireAimAttempt {
    pub method: NativeFireAimMethod,
    pub bit: usize,
    pub locator_padding_bits: usize,
    pub raw: Option<u32>,
    pub accepted: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeFireField {
    pub field: super::ComponentField,
    pub opaque: bool,
    pub padded_bits: usize,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativeFireHeaderStop {
    Read,
    Truncated,
    OtherHead,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativeFireAimStop {
    NoHeader,
    Short,
    TimestampBlock,
    NonModal,
    TruncatedCounts,
    Located,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NativeFireRead {
    pub source_bits: usize,
    pub end_bit: usize,
    pub header_end_bit: Option<usize>,
    pub header_stop: NativeFireHeaderStop,
    pub aim_stop: NativeFireAimStop,
    /// None means a refused/truncated head, never an absent recorded action.
    pub event: Option<FilmFireEvent>,
    /// Ordered header/count/aim fields. Unparsed body bytes stay in the source.
    pub fields: Vec<NativeFireField>,
    pub aim_attempts: Vec<NativeFireAimAttempt>,
}
struct FireReader<'a> {
    cursor: Cursor<'a>,
    source_bits: usize,
    fields: Vec<NativeFireField>,
}
impl FireReader<'_> {
    fn read(&mut self, name: &str, width: usize, opaque: bool) -> u64 {
        let bit = self.cursor.position;
        let raw = self.cursor.read(width).expect("native zero-tail fire read");
        self.fields.push(NativeFireField {
            field: super::ComponentField {
                name: name.into(),
                bit: bit as i64,
                width: width as u64,
                raw,
            },
            opaque,
            padded_bits: self
                .cursor
                .position
                .saturating_sub(bit.max(self.source_bits)),
        });
        raw
    }
    fn bit(&mut self, name: &str) -> bool {
        self.read(name, 1, false) != 0
    }
    fn reference(&mut self, name: &str, probe: bool) -> FireUnitReference {
        if !self.bit(&format!("{name}.present")) {
            return FireUnitReference::default();
        }
        let narrow = probe && self.bit(&format!("{name}.probe"));
        let index = self.read(&format!("{name}.index"), if narrow { 9 } else { 13 }, false) as u32;
        let generation = self.read(&format!("{name}.generation"), 2, false) as u32;
        FireUnitReference {
            present: true,
            slot: 512 + index,
            generation,
            probe: narrow,
        }
    }
}

/// Read the actual optional-field grammar, retaining refusals and zero-tail
/// provenance. The modal aim locator is accepted only within recorded bytes.
pub fn read_native_fire_event(payload: &[u8]) -> NativeFireRead {
    let mut r = FireReader {
        cursor: Cursor::new_padded(payload, 0),
        source_bits: payload.len() * 8,
        fields: vec![],
    };
    let mut out = NativeFireRead {
        source_bits: r.source_bits,
        end_bit: 0,
        header_end_bit: None,
        header_stop: NativeFireHeaderStop::Truncated,
        aim_stop: NativeFireAimStop::NoHeader,
        event: None,
        fields: vec![],
        aim_attempts: vec![],
    };
    if payload.len() < 2 {
        return out;
    }
    r.bit("header.configuration");
    let more = r.bit("header.continuation");
    let kind = r.read("header.code", 7, false);
    if !more || kind != 36 {
        out.header_stop = NativeFireHeaderStop::OtherHead;
    } else {
        let unit = r.reference("unit", true);
        r.reference("reference1", false);
        r.reference("reference2", false);
        let short = r.bit("head.short");
        let bloc = r.bit("head.bloc");
        let low = r.read("head.fire_number.low", 7, false) as u8;
        let high = r.bit("head.fire_number.high");
        let film_index = if !r.bit("head.shooter.absent") {
            r.read("head.shooter.index", 5, false) as i32
        } else {
            -1
        };
        if !r.bit("head.field_e.absent") {
            r.read("head.field_e", 2, true);
        }
        let upper = if r.bit("head.weapon_upper.present") {
            r.read("head.weapon_upper", 32, false)
        } else {
            0
        };
        let lower = r.read("head.weapon_lower", 32, false);
        r.read("head.flags_ij", 2, true);
        out.header_end_bit = Some(r.cursor.position);
        if r.cursor.position <= r.source_bits {
            out.header_stop = NativeFireHeaderStop::Read;
            let mut event = FilmFireEvent {
                chunk: 0,
                packet_index: 0,
                timestamp_us: 0,
                film_index,
                has_shooter: film_index >= 0,
                fire_number: low | ((high as u8) << 7),
                unit,
                short,
                bloc,
                weapon_id: upper << 32 | lower,
                has_aim: false,
                aim: [0.; 3],
            };
            out.aim_stop = locate_after_header(&mut r, short, bloc);
            if out.aim_stop == NativeFireAimStop::Located {
                let bit = r.cursor.position + 2;
                let raw = Bits(payload).read(bit, 30).map(|v| v as u32);
                let aim = raw.and_then(fire_aim_vector);
                out.aim_attempts.push(NativeFireAimAttempt {
                    method: NativeFireAimMethod::Modal,
                    bit,
                    locator_padding_bits: 0,
                    raw,
                    accepted: aim.is_some(),
                });
                if raw.is_some() {
                    r.read("modal.aim_gap", 2, true);
                    r.read("modal.aim", 30, false);
                }
                if let Some(aim) = aim {
                    event.has_aim = true;
                    event.aim = aim;
                }
            }
            out.event = Some(event);
        }
    }
    out.end_bit = r.cursor.position;
    out.fields = r.fields;
    out
}
fn locate_after_header(r: &mut FireReader<'_>, short: bool, bloc: bool) -> NativeFireAimStop {
    if bloc {
        r.read("body.bloc.flag", 1, true);
        if r.bit("body.bloc.timestamp") {
            return NativeFireAimStop::TimestampBlock;
        }
    }
    if short {
        return NativeFireAimStop::Short;
    }
    let (mut targets, mut components) = (0, 0);
    if !r.bit("body.counts.empty") {
        targets = if r.bit("body.targets.one") {
            1
        } else {
            r.read("body.targets.count", 4, false)
        };
        if !r.bit("body.components.empty") {
            components = if r.bit("body.components.one") {
                1
            } else {
                r.read("body.components.count", 4, false)
            };
        }
    }
    if targets != 0 || components != 0 {
        NativeFireAimStop::NonModal
    } else if r.cursor.position > r.source_bits {
        NativeFireAimStop::TruncatedCounts
    } else {
        NativeFireAimStop::Located
    }
}
pub fn modal_fire_aim_bit(payload: &[u8]) -> Option<usize> {
    locate_modal_fire_aim(payload).map(|v| v.bit)
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModalFireAimLocation {
    pub bit: usize,
    pub padding_bits: usize,
}
pub fn locate_modal_fire_aim(payload: &[u8]) -> Option<ModalFireAimLocation> {
    read_native_fire_event(payload)
        .aim_attempts
        .first()
        .map(|v| ModalFireAimLocation {
            bit: v.bit,
            padding_bits: v.locator_padding_bits,
        })
}

// Preserve the pinned native float32 fused arithmetic and float64 square root.
fn fire_aim_vector(code: u32) -> Option<[f32; 3]> {
    let face = code / 178_956_970;
    let rem = code % 178_956_970;
    let step = 2.0_f32 / 13375.;
    let coord = |i: u32| {
        if i * 2 == 13374 {
            0.
        } else {
            (i as f32).mul_add(step, -1.) + step * 0.5
        }
    };
    let (a, b) = (coord(rem / 13376), coord(rem % 13376));
    let v = match face {
        0 => [1., a, b],
        1 => [a, 1., b],
        2 => [a, b, 1.],
        3 => [-1., a, b],
        4 => [a, -1., b],
        5 => [a, b, -1.],
        _ => return None,
    };
    let norm = (v[2].mul_add(v[2], v[1].mul_add(v[1], v[0] * v[0])) as f64).sqrt() as f32;
    Some(if norm < 1e-4 { v } else { v.map(|x| x / norm) })
}
/// Scan the native contiguous metadata prefix starting at chunk 1.
pub fn scan_fire_events(chunks: &[FilmChunkData]) -> Result<Vec<FilmFireEvent>, DecodeError> {
    let mut out = Vec::new();
    for c in native_chunk_prefix(chunks)? {
        for (index, p) in native_chunk_packets(c).into_iter().enumerate() {
            let pay = &c.data[p.payload_offset..p.payload_offset + p.payload_size];
            if p.packet_type == 0
                && let Some(mut e) = decode_fire_event(pay)
            {
                e.chunk = i64::from(c.metadata.index);
                e.packet_index = index;
                e.timestamp_us = p.timestamp_us;
                out.push(e);
            }
        }
    }
    Ok(out)
}
/// Shared native source ordering, distinct from the strict legacy LegacyFilm admission.
pub(super) fn native_chunk_prefix(
    chunks: &[FilmChunkData],
) -> Result<Vec<&FilmChunkData>, DecodeError> {
    let mut wanted = 1;
    let mut selected = Vec::new();
    for c in chunks {
        if c.metadata.index < 1 {
            continue;
        }
        if c.metadata.index != wanted {
            break;
        }
        selected.push(c);
        wanted += 1;
    }
    if selected.is_empty() {
        return Err(DecodeError::Missing("readable film chunks"));
    }
    Ok(selected)
}
/// Native numbered lookup uses the first metadata entry, even when its bytes
/// are empty or malformed. A later duplicate is not a replacement or fallback.
pub(super) fn native_chunk_data(
    chunks: &[FilmChunkData],
) -> std::collections::BTreeMap<i32, &Vec<u8>> {
    let mut selected = std::collections::BTreeMap::new();
    for chunk in chunks {
        selected.entry(chunk.metadata.index).or_insert(&chunk.data);
    }
    selected
}

/// Native packet ordinals count every framed packet, not only a selected kind.
pub(super) fn native_packet_indices(
    chunks: &[FilmChunkData],
) -> std::collections::BTreeMap<(i32, usize), usize> {
    native_chunk_data(chunks)
        .into_iter()
        .flat_map(|(number, data)| {
            native_packet_bytes(data, number)
                .into_iter()
                .enumerate()
                .map(move |(index, p)| ((number, p.payload_offset), index))
        })
        .collect()
}
pub(super) fn native_chunk_packets(c: &FilmChunkData) -> Vec<super::FilmPacket> {
    native_packet_bytes(&c.data, c.metadata.index)
}
pub(super) fn native_packet_bytes(data: &[u8], chunk_index: i32) -> Vec<super::FilmPacket> {
    let mut out = Vec::new();
    let mut offset = 0usize;
    while let Some(header) = data.get(offset..offset.saturating_add(16)) {
        let kind = u16::from_le_bytes(header[..2].try_into().unwrap());
        let size = u32::from_le_bytes(header[4..8].try_into().unwrap()) as usize;
        let stamp = u64::from_le_bytes(header[8..16].try_into().unwrap());
        let start = offset + 16;
        let Some(end) = start.checked_add(size).filter(|end| *end <= data.len()) else {
            break;
        };
        if size == 0 && kind != 7 {
            break;
        }
        out.push(super::FilmPacket {
            chunk_index,
            packet_type: kind,
            byte_2: header[2],
            byte_3: header[3],
            payload_offset: start,
            payload_size: size,
            timestamp_us: stamp,
        });
        offset = end;
        if kind == 7 {
            break;
        }
    }
    out
}

#[cfg(test)]
pub(super) fn test_payload_after_keyframe(payload: &[u8], timestamp_us: u64) -> Vec<u8> {
    let mut out = Vec::new();
    for (kind, data) in [(2u16, &[0x40][..]), (0, payload)] {
        out.extend_from_slice(&kind.to_le_bytes());
        out.extend_from_slice(&[0, 0]);
        out.extend_from_slice(&(data.len() as u32).to_le_bytes());
        out.extend_from_slice(&timestamp_us.to_le_bytes());
        out.extend_from_slice(data);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn native_fire_heads_and_aim() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/fire-d61443e-v41.json.zlib")[..])
            .read_to_end(&mut raw)
            .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        for (i, row) in rows.iter().enumerate() {
            let hex = row["hex"].as_str().unwrap();
            let pay: Vec<_> = (0..hex.len())
                .step_by(2)
                .map(|p| u8::from_str_radix(&hex[p..p + 2], 16).unwrap())
                .collect();
            let mut actual = decode_fire_event(&pay);
            let read = read_native_fire_event(&pay);
            assert_eq!(read.event, actual);
            let mut at = 0;
            for field in &read.fields {
                let f = &field.field;
                assert_eq!(f.bit as usize, at);
                let source_width = (f.width as usize) - field.padded_bits;
                if source_width == 0 {
                    assert_eq!(f.raw, 0);
                } else {
                    assert_eq!(
                        Bits(&pay).read(f.bit as usize, source_width),
                        Some(f.raw >> field.padded_bits)
                    );
                }
                if field.padded_bits > 0 {
                    assert_eq!(f.raw & ((1u64 << field.padded_bits) - 1), 0);
                }
                at += f.width as usize;
            }
            assert_eq!(at, read.end_bit);
            for aim in &read.aim_attempts {
                assert_eq!(aim.raw, Bits(&pay).read(aim.bit, 30).map(|v| v as u32));
                assert_eq!(aim.accepted, aim.raw.and_then(fire_aim_vector).is_some());
            }
            assert_eq!(
                serde_json::from_value::<NativeFireRead>(serde_json::to_value(&read).unwrap())
                    .unwrap(),
                read
            );
            assert_eq!(
                actual.is_some(),
                row["valid"].as_bool().unwrap(),
                "valid {i}"
            );
            let expected: FilmFireEvent = serde_json::from_value(row["event"].clone()).unwrap();
            if let Some(e) = actual.as_mut() {
                e.chunk = expected.chunk;
                e.packet_index = expected.packet_index;
                e.timestamp_us = expected.timestamp_us;
                assert_eq!(*e, expected, "event {i}");
            }
            let modal = modal_fire_aim_bit(&pay);
            assert_eq!(
                modal.is_some(),
                row["modal"].as_bool().unwrap(),
                "modal {i}"
            );
            if let Some(bit) = modal {
                assert_eq!(bit, row["modal_bit"].as_u64().unwrap() as usize, "bit {i}");
            }
            assert_eq!(
                read_fire_attacker_index(&pay).map_or(-1, i64::from),
                row["attacker"].as_i64().unwrap()
            );
            assert_eq!(
                read_fire_shooter_index(&pay).map_or(-1, i64::from),
                row["shooter"].as_i64().unwrap()
            );
        }
    }

    #[test]
    fn native_fire_retains_rejected_quantized_aim() {
        let mut payload = vec![0; 24];
        payload[0] = 0xd2;
        let bit = modal_fire_aim_bit(&payload).unwrap();
        for at in bit..bit + 30 {
            payload[at / 8] |= 1 << (7 - at % 8);
        }
        let read = read_native_fire_event(&payload);
        assert_eq!(read.aim_attempts[0].method, NativeFireAimMethod::Modal);
        assert_eq!(read.aim_attempts[0].raw, Some((1 << 30) - 1));
        assert!(!read.aim_attempts[0].accepted);
        let truncated = read_native_fire_event(&payload[..2]);
        assert!(truncated.event.is_none());
        assert_eq!(truncated.header_stop, NativeFireHeaderStop::Truncated);
        assert!(truncated.fields.iter().any(|f| f.padded_bits > 0));
    }

    #[test]
    fn native_fire_scan_partial_chunks() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/fire-scanner-d61443e-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        for (i, row) in rows.iter().enumerate() {
            let chunks: Vec<_> = row["chunks"]
                .as_array()
                .unwrap()
                .iter()
                .map(|c| {
                    let hex = c["hex"].as_str().unwrap();
                    let data: Vec<_> = (0..hex.len())
                        .step_by(2)
                        .map(|p| u8::from_str_radix(&hex[p..p + 2], 16).unwrap())
                        .collect();
                    FilmChunkData {
                        metadata: crate::clients::hi::models::FilmChunk {
                            index: c["index"].as_i64().unwrap() as i32,
                            chunk_type: 2,
                            start_time_offset_ms: 0,
                            duration_ms: 0,
                            size: data.len() as i64,
                            file_relative_path: String::new(),
                        },
                        data,
                    }
                })
                .collect();
            let actual = scan_fire_events(&chunks);
            assert_eq!(actual.is_err(), row["error"].as_bool().unwrap(), "scan {i}");
            if let Ok(actual) = actual {
                let expected: Vec<FilmFireEvent> =
                    serde_json::from_value(if row["events"].is_null() {
                        serde_json::json!([])
                    } else {
                        row["events"].clone()
                    })
                    .unwrap();
                assert_eq!(actual, expected, "events {i}");
            }
        }
    }
}
