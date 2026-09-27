//! Record headers and registry-driven delta traversal. The caller supplies a checked
//! boundary and entity binding; this layer never searches for a plausible header.
use super::{ComponentDecode, DecodedComponent, FilmRegistry, bits::Cursor, decode_component};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecordKind {
    End,
    New,
    Delete,
    Delta,
}

/// Entity ID widths belong to the replication view, not a universal film constant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordIdLayout {
    pub low_bits: usize,
    pub base: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordHeader {
    pub kind: RecordKind,
    pub id: Option<u32>,
    pub start_bit: i64,
    pub end_bit: i64,
}

/// Read the prefix-coded kind followed by the view's low ID and two generation bits.
pub fn decode_record_header(
    data: &[u8],
    bit: usize,
    layout: RecordIdLayout,
) -> Option<RecordHeader> {
    if layout.low_bits > 30 || layout.base > 0x3fff_ffff {
        return None;
    }
    decode_header_cursor(Cursor::new(data, bit)?, layout)
}

pub(crate) fn decode_header_cursor(
    mut r: Cursor<'_>,
    layout: RecordIdLayout,
) -> Option<RecordHeader> {
    let bit = r.position;
    let kind = if r.bit()? {
        RecordKind::Delta
    } else {
        match r.read(2)? {
            0 => RecordKind::End,
            1 => RecordKind::New,
            2 => RecordKind::Delete,
            _ => RecordKind::Delta,
        }
    };
    let id = if kind == RecordKind::End {
        None
    } else {
        let low = (r.read(layout.low_bits)? as u32).checked_add(layout.base)?;
        if low > 0x3fff_ffff {
            return None;
        }
        Some(((r.read(2)? as u32) << 30) | low)
    };
    Some(RecordHeader {
        kind,
        id,
        start_bit: bit as i64,
        end_bit: r.position as i64,
    })
}

/// Decode a native header with signed cursor coordinates and native ID widths.
/// Nonpositive ID widths omit the low-ID read. Positive widths use the pinned
/// 64-bit width domain, independent of the host pointer size. A native read
/// panic leaves the caller's reader at its reached position.
pub fn decode_native_record_header(
    reader: &mut super::NativeFilmBits<'_>,
    width: i64,
    base: u32,
) -> RecordHeader {
    let start_bit = reader.position();
    let kind = if reader.read_bit() {
        RecordKind::Delta
    } else {
        match reader.read(2) {
            0 => RecordKind::End,
            1 => RecordKind::New,
            2 => RecordKind::Delete,
            _ => RecordKind::Delta,
        }
    };
    let id = if kind == RecordKind::End {
        None
    } else {
        let low = if width > 0 {
            reader.read_wide(width as u64) as u32
        } else {
            0
        };
        let low = low.wrapping_add(base) & 0x3fff_ffff;
        let tag = reader.read(2) as u32;
        Some((tag << 30) | low)
    };
    RecordHeader {
        kind,
        id,
        start_bit,
        end_bit: reader.position(),
    }
}

pub(crate) fn decode_frame_header_signed(
    data: &[u8],
    bit: i64,
    encoding: &super::FrameEncoding,
) -> Option<RecordHeader> {
    let width = encoding
        .native_id_low_bits
        .or_else(|| i64::try_from(encoding.ids.low_bits).ok())?;
    let mut reader = super::NativeFilmBits::new(data);
    reader.set_position(bit);
    Some(decode_native_record_header(
        &mut reader,
        width,
        encoding.ids.base,
    ))
}

pub(crate) fn decode_native_header_cursor(
    r: Cursor<'_>,
    width: i64,
    base: u32,
) -> Result<RecordHeader, super::NativeWidthRefusal> {
    Ok(decode_native_record_header(
        &mut r.native_reader(),
        width,
        base,
    ))
}

pub(crate) fn decode_frame_header_cursor(
    r: Cursor<'_>,
    encoding: &super::FrameEncoding,
) -> Option<RecordHeader> {
    decode_frame_header_cursor_report(r, encoding, &mut Default::default())
}

pub(crate) fn decode_frame_header_cursor_report(
    r: Cursor<'_>,
    encoding: &super::FrameEncoding,
    diagnostics: &mut super::FilmReadDiagnostics,
) -> Option<RecordHeader> {
    let width = encoding
        .native_id_low_bits
        .or_else(|| i64::try_from(encoding.ids.low_bits).ok())?;
    match decode_native_header_cursor(r, width, encoding.ids.base) {
        Ok(header) => Some(header),
        Err(refusal) => {
            diagnostics.width_refusals.push(refusal);
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecordStop {
    Complete,
    Truncated { bit: usize },
    UnsupportedComponent { index: usize, name: String },
    InvalidComponent { index: usize },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeltaRecord {
    pub header: RecordHeader,
    pub archetype: usize,
    pub baseline: Option<u8>,
    pub mask: u64,
    pub components: Vec<DecodedComponent>,
    /// End of the last completely decoded component, never a guessed record end.
    pub end_bit: usize,
    pub stop: RecordStop,
}

/// Decode one delta using a separately established entity-to-archetype binding.
/// Unsupported components terminate the walk without consuming their payload.
pub fn decode_delta_record(
    data: &[u8],
    bit: usize,
    layout: RecordIdLayout,
    registry: &FilmRegistry,
    archetype: usize,
) -> Option<DeltaRecord> {
    delta_record(data, bit, layout, registry, archetype, None)
}

/// Delta traversal with a separately established position encoding.
pub fn decode_delta_record_with_encoding(
    data: &[u8],
    bit: usize,
    layout: RecordIdLayout,
    registry: &FilmRegistry,
    archetype: usize,
    encoding: &super::PositionEncoding,
) -> Option<DeltaRecord> {
    delta_record(data, bit, layout, registry, archetype, Some(encoding))
}

fn delta_record(
    data: &[u8],
    bit: usize,
    layout: RecordIdLayout,
    registry: &FilmRegistry,
    archetype: usize,
    encoding: Option<&super::PositionEncoding>,
) -> Option<DeltaRecord> {
    let header = decode_record_header(data, bit, layout)?;
    if header.kind != RecordKind::Delta {
        return None;
    }
    let arch = registry.archetype(archetype)?;
    let mut r = Cursor::new(data, header.end_bit)?;
    let baseline = if r.bit()? {
        Some(r.read(7)? as u8)
    } else {
        None
    };
    let mask = r.mask()?;
    let mut record = DeltaRecord {
        header,
        archetype,
        baseline,
        mask,
        components: Vec::new(),
        end_bit: r.position,
        stop: RecordStop::Complete,
    };
    for index in 0..64 {
        if mask & (1 << index) == 0 {
            continue;
        }
        let Some(name) = arch.components.get(index) else {
            record.stop = RecordStop::InvalidComponent { index };
            break;
        };
        let level = arch.levels.get(index).copied().unwrap_or(0);
        let component = if let Some(encoding) = encoding {
            super::decode_component_with_encoding(
                data,
                record.end_bit,
                name,
                level,
                archetype as u32,
                encoding,
            )
        } else {
            decode_component(data, record.end_bit, name, level, archetype as u32)
        };
        match component {
            ComponentDecode::Decoded(component) => {
                record.end_bit = usize::try_from(component.end_bit).ok()?;
                record.components.push(component);
            }
            ComponentDecode::Truncated { bit } => {
                record.stop = RecordStop::Truncated {
                    bit: usize::try_from(bit).ok()?,
                };
                break;
            }
            ComponentDecode::Unsupported => {
                record.stop = RecordStop::UnsupportedComponent {
                    index,
                    name: name.clone(),
                };
                break;
            }
        }
    }
    Some(record)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn registry_delta_walk_matches_captured_ammo_component_boundaries() {
        let mut bootstrap = Vec::new();
        flate2::read::ZlibDecoder::new(include_bytes!("fixtures/bootstrap-v41.zlib").as_slice())
            .read_to_end(&mut bootstrap)
            .unwrap();
        let registry = crate::theater::parse_registry(&bootstrap).unwrap();
        let cases: serde_json::Value =
            serde_json::from_str(include_str!("fixtures/ranked_ammo_records.json")).unwrap();
        let mut checked = 0;
        for case in cases["records"].as_array().unwrap() {
            if case["accepted"] != true {
                continue;
            }
            let layout: crate::theater::CoordinateLayout =
                serde_json::from_value(case["layout"].clone()).unwrap();
            let encoding = crate::theater::PositionEncoding {
                bodies: Default::default(),
                world_axis_bits: None,
                index_bits: 1,
                handle_bits: 1,
                traversal_axis_bits: [6; 3],
                default_axis_bits: [22; 3],
                region_axis_bits: [(0, layout.axis_bits())].into(),
                delta_axis_bits: [14; 3],
                calibrated_skip: false,
                baseline_scope: false,
                full_precision: false,
                writer_absolute: false,
                delta_handle_tail: false,
            };
            let hex = case["hex"].as_str().unwrap();
            let data: Vec<u8> = (0..hex.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                .collect();
            let bit = case["source"]["bit"].as_u64().unwrap() as usize;
            let result = decode_delta_record_with_encoding(
                &data,
                bit,
                RecordIdLayout {
                    low_bits: 13,
                    base: 0,
                },
                &registry,
                35,
                &encoding,
            )
            .unwrap();
            assert_eq!(result.stop, RecordStop::Complete, "{}", case["label"]);
            assert_eq!(
                result.end_bit, case["source"]["end_bit"],
                "{}",
                case["label"]
            );
            let fields = case["fields"].as_array().unwrap();
            assert_eq!(result.components.len(), fields.len());
            for (actual, expected) in result.components.iter().zip(fields) {
                assert_eq!(actual.start_bit, expected["bit"], "{}", case["label"]);
                assert_eq!(actual.end_bit, expected["end_bit"], "{}", case["label"]);
            }
            checked += 1;
        }
        assert!(checked >= 10);
    }

    #[test]
    fn both_delta_prefixes_and_end_preserve_exact_boundaries() {
        let layout = RecordIdLayout {
            low_bits: 3,
            base: 0x200,
        };
        // Short delta, slot 5, generation 2; long delta, same identity.
        let a = decode_record_header(&[0b11011000], 0, layout).unwrap();
        let b = decode_record_header(&[0b01110110], 0, layout).unwrap();
        assert_eq!(a.id, Some(0x8000_0205));
        assert_eq!(a.id, b.id);
        assert_eq!((a.end_bit, b.end_bit), (6, 8));
        let end = decode_record_header(&[0], 2, layout).unwrap();
        assert_eq!(end.kind, RecordKind::End);
        assert_eq!((end.id, end.end_bit), (None, 5));
    }

    #[test]
    fn sparse_and_dense_masks_and_signed_widths() {
        let mut sparse = Cursor::new(&[0b00100000, 0b01111111], 0).unwrap();
        assert_eq!(sparse.mask(), Some((1 << 1) | (1 << 63)));
        let mut data = vec![0xff; 9];
        let mut dense = Cursor::new(&data, 0).unwrap();
        assert_eq!(dense.mask(), Some(u64::MAX));
        for (sel, width) in [(0, 8), (1, 16), (2, 32), (3, 64)] {
            data[0] = (sel << 6) | 0x3f;
            let mut reader = Cursor::new(&data, 0).unwrap();
            assert_eq!(reader.signed_variable(), Some(-1));
            assert_eq!(reader.position, width + 2);
        }
    }
}
