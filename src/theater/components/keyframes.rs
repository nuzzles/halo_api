//! Full-state keyframe records use signed guards and all named components,
//! not the presence masks used by delta records.
use super::Cursor;
use super::{ComponentField, PositionEncoding, Reader, defaults};
use crate::theater::FilmRegistry;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyframeStop {
    Complete,
    Truncated,
    InvalidArchetype,
    UnsupportedDefault,
    UnsupportedComponent { index: usize, name: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyframeComponentSpan {
    /// Native returned variant; None marks an incomplete read or older export.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variant: Option<u32>,
    /// Native ported result; None marks an incomplete read or older export.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ported: Option<bool>,
    /// Ordered retained fields, independent of overlapping source bit ranges.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub field_range: Option<[usize; 2]>,
    /// Ordered diagnostic observations emitted by this attempt. Absent in older serialized data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observation_range: Option<[usize; 2]>,
    pub index: usize,
    pub name: String,
    pub start_bit: i64,
    /// Includes the optional corruption check after the component.
    pub end_bit: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyframeRecord {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub references: Vec<crate::theater::NativeUnitReference>,
    #[serde(
        default,
        skip_serializing_if = "crate::theater::FilmReadDiagnostics::is_empty"
    )]
    pub diagnostics: crate::theater::FilmReadDiagnostics,
    pub start_bit: i64,
    pub end_bit: i64,
    pub id: u32,
    /// Decoded type. The native isolated reader uses six low bits; the complete
    /// raw header word remains in fields. Table readers admit the full word first.
    pub archetype: u32,
    pub fields: Vec<ComponentField>,
    pub components: Vec<KeyframeComponentSpan>,
    /// Ordered attempts, including the final unsupported or incomplete reader.
    /// Empty in older exports where this trace was unavailable.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attempts: Vec<KeyframeComponentSpan>,
    pub stop: KeyframeStop,
}

/// Read a full-state record at an independently established boundary. The
/// corruption-check flag comes from bootstrap identity. No anchor scanning or
/// inferred component skips occur here. Returns None for an incomplete header
/// or an invalid precision context.
pub fn decode_keyframe_record(
    data: &[u8],
    bit: usize,
    registry: &FilmRegistry,
    mpp_widths: [usize; 2],
    encoding: Option<&PositionEncoding>,
    corruption_check: bool,
) -> Option<KeyframeRecord> {
    decode_keyframe_record_inner(
        data,
        bit,
        registry,
        mpp_widths,
        encoding,
        corruption_check,
        (
            false,
            None,
            None,
            true,
            super::KeyframeLayout::default(),
            None,
        ),
    )
}

/// Bounded full-state reader with explicit frame/profile capture context.
pub fn decode_keyframe_record_with_encoding(
    data: &[u8],
    bit: usize,
    registry: &FilmRegistry,
    encoding: &super::FrameEncoding,
) -> Option<KeyframeRecord> {
    decode_keyframe_record_inner(
        data,
        bit,
        registry,
        encoding.mpp_widths,
        encoding.position.as_ref(),
        encoding.corruption_check,
        (
            false,
            encoding.position_capture.as_ref(),
            Some(&encoding.component_widths),
            encoding.keyframe_simulation_complete.unwrap_or(true),
            encoding.keyframe_layout,
            None,
        ),
    )
}

/// Native full-state reader, including zero-padded reads and optional position capture.
pub fn decode_native_keyframe_record(
    data: &[u8],
    bit: impl TryInto<i64>,
    registry: &FilmRegistry,
    encoding: &super::FrameEncoding,
) -> Option<KeyframeRecord> {
    decode_native_keyframe_record_contextual(data, bit, registry, encoding, None)
}

pub(crate) fn decode_native_keyframe_record_contextual(
    data: &[u8],
    bit: impl TryInto<i64>,
    registry: &FilmRegistry,
    encoding: &super::FrameEncoding,
    context: Option<&crate::theater::NativeReaderContext>,
) -> Option<KeyframeRecord> {
    decode_keyframe_record_inner(
        data,
        bit,
        registry,
        encoding.mpp_widths,
        encoding.position.as_ref(),
        encoding.corruption_check,
        (
            true,
            encoding.position_capture.as_ref(),
            Some(&encoding.component_widths),
            encoding.keyframe_simulation_complete.unwrap_or(true),
            encoding.keyframe_layout,
            context,
        ),
    )
}

fn decode_keyframe_record_inner(
    data: &[u8],
    bit: impl TryInto<i64>,
    registry: &FilmRegistry,
    mpp_widths: [usize; 2],
    encoding: Option<&PositionEncoding>,
    corruption_check: bool,
    policy: (
        bool,
        Option<&crate::theater::PositionCaptureEncoding>,
        Option<&super::ComponentWidthOverrides>,
        bool,
        super::KeyframeLayout,
        Option<&crate::theater::NativeReaderContext>,
    ),
) -> Option<KeyframeRecord> {
    let (padded, capture_encoding, component_widths, simulation_complete, layout, context) = policy;
    let bit = bit.try_into().ok()?;
    if context.is_none() && !layout.valid() {
        return None;
    }
    let header_bits = match context {
        Some(c) => c.profile.keyframe.header_bits,
        None => i64::try_from(layout.header_bits).ok()?,
    };
    let size_word_bits = context.map_or(layout.size_word_bits as u64, |c| {
        c.profile.keyframe.size_word_bits as u64
    });
    let capture_map = capture_encoding.map(|c| c.map());
    if mpp_widths.iter().any(|w| !(1..=32).contains(w)) || encoding.is_some_and(|e| !e.valid()) {
        return None;
    }
    let mut r = Reader {
        native_widths: context.map(|c| super::NativeComponentWidths {
            movement: &c.profile.movement,
            mpp: c.profile.mpp,
            maximum: u64::MAX,
        }),
        width_error: None,
        live_observer: context.and_then(|c| c.observer.clone()),
        live_grammar: context.map(|c| c.profile.grammar.clone()),
        position_capture: capture_encoding
            .zip(capture_map.as_ref())
            .map(|(c, map)| c.reader(map, 0)),
        position_start: 0,
        position_slot: 0,
        position_fallback: false,
        // The native full-state walker creates a fresh reader without setting a slot.
        movement_slot: Some(0),
        references: Vec::new(),
        diagnostics: Default::default(),
        cursor: if padded {
            Cursor::signed(data, bit, None)
        } else {
            Cursor::new(data, usize::try_from(bit).ok()?)?
        },
        fields: Vec::new(),
        position_encoding: encoding,
    };
    let (id, archetype) = if padded {
        let archetype = crate::theater::native_bits_at(data, bit.wrapping_add(58), 6) as u32;
        let id = crate::theater::native_bits_tolerant(data, bit, 32) as u32;
        let mut remaining = header_bits.max(64);
        for (name, width) in [
            ("header.id", 32),
            ("header.archetype", 32),
            ("header.word", 32),
            ("header.kind", 4),
            ("header.byte", 8),
        ] {
            let width = width.min(remaining);
            if width > 0 {
                r.fields.push(ComponentField {
                    name: name.into(),
                    bit: r.cursor.position,
                    width: width as u64,
                    raw: crate::theater::native_bits_tolerant(data, r.cursor.position, width),
                });
                r.cursor.skip_signed(width);
                remaining -= width;
            }
        }
        if remaining > 0 {
            super::widths::signed_skip(
                &mut r,
                "header.extension",
                remaining,
                false,
                None,
                ("header.extension", "header.extension.tail"),
            )?;
        }
        if header_bits < 64 {
            super::widths::signed_skip(
                &mut r,
                "header.layout_adjustment",
                header_bits.wrapping_sub(64),
                false,
                None,
                ("header.layout_adjustment", "header.layout_adjustment"),
            )?;
        }
        (id, archetype)
    } else {
        let id = r.r("header.id", 32)? as u32;
        let raw_archetype = r.r("header.archetype", 32)? as u32;
        // WalkKeyframeFullState reads only record+58..64. The sequential table
        // validates the whole word separately and handles its no-archetype sentinel.
        let archetype = raw_archetype;
        let mut remaining = layout.header_bits - 64;
        for (name, width) in [("header.word", 32), ("header.kind", 4), ("header.byte", 8)] {
            let width = width.min(remaining);
            if width > 0 {
                r.r(name, width)?;
                remaining -= width;
            }
        }
        r.words("header.extension", remaining / 64, 64)?;
        if !remaining.is_multiple_of(64) {
            r.r("header.extension.tail", remaining % 64)?;
        }
        (id, archetype)
    };
    let mut components = Vec::new();
    let mut attempts = Vec::new();
    let stop = body(
        &mut r,
        archetype,
        registry,
        mpp_widths,
        corruption_check,
        (&mut components, &mut attempts),
        (
            padded,
            component_widths,
            simulation_complete,
            size_word_bits,
        ),
    )
    .unwrap_or(KeyframeStop::Truncated);
    Some(KeyframeRecord {
        start_bit: bit,
        end_bit: r.cursor.position,
        id,
        archetype,
        fields: r.fields,
        references: r.references,
        diagnostics: r.diagnostics,
        components,
        attempts,
        stop,
    })
}

fn body(
    r: &mut Reader<'_>,
    ti: u32,
    registry: &FilmRegistry,
    widths: [usize; 2],
    check: bool,
    outputs: (
        &mut Vec<KeyframeComponentSpan>,
        &mut Vec<KeyframeComponentSpan>,
    ),
    policy: (bool, Option<&super::ComponentWidthOverrides>, bool, u64),
) -> Option<KeyframeStop> {
    let (components, attempts) = outputs;
    let (native_policy, component_widths, simulation_complete, size_word_bits) = policy;
    // The writer omits n1, n2 and the entire body for this sentinel archetype.
    if ti == u32::MAX {
        return Some(KeyframeStop::Complete);
    }
    let Some(arch) = registry.archetype(ti as usize).filter(|_| ti < 50) else {
        return Some(KeyframeStop::InvalidArchetype);
    };
    let n1 = r.r_wide("default_guard", size_word_bits)? as u32 as i32;
    if n1 > 0 {
        if defaults::has_native_deserializer(ti) && !defaults::state(r, ti, widths)? {
            return Some(KeyframeStop::UnsupportedDefault);
        }
        if check {
            r.r_wide("default_corruption_check", size_word_bits)?;
        }
    }
    let n2 = r.r_wide("components_guard", size_word_bits)? as u32 as i32;
    if n2 <= 0 {
        return Some(KeyframeStop::Complete);
    }
    for (index, name) in arch.components.iter().enumerate() {
        if name.is_empty() && !native_policy && component_widths.is_none() {
            continue;
        }
        let start_bit = r.cursor.position;
        let field_start = r.fields.len();
        let observation_start = r.diagnostics.component_observations.len();
        let calibrated = super::widths::is_calibrated(r, name, component_widths);
        let status = super::widths::read_component(
            r,
            name,
            arch.levels.get(index).copied().unwrap_or(0),
            ti,
            component_widths,
            (simulation_complete, None),
        );
        attempts.push(KeyframeComponentSpan {
            variant: status
                .map(|_| super::widths::result_variant(name, &r.fields[field_start..], calibrated)),
            ported: status,
            field_range: Some([field_start, r.fields.len()]),
            observation_range: Some([
                observation_start,
                r.diagnostics.component_observations.len(),
            ]),
            index,
            name: name.clone(),
            start_bit,
            end_bit: r.cursor.position,
        });
        if !status? {
            return Some(KeyframeStop::UnsupportedComponent {
                index,
                name: name.clone(),
            });
        }
        if check {
            r.gate("component_corruption_check", 32, true)?;
        }
        components.push(KeyframeComponentSpan {
            variant: attempts.last().unwrap().variant,
            ported: status,
            field_range: Some([field_start, r.fields.len()]),
            observation_range: Some([
                observation_start,
                r.diagnostics.component_observations.len(),
            ]),
            index,
            name: name.clone(),
            start_bit,
            end_bit: r.cursor.position,
        });
        *attempts.last_mut().unwrap() = components.last().unwrap().clone();
    }
    Some(KeyframeStop::Complete)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyframeTableStop {
    /// Like the reference walker, fewer than 64 bits remain for an identifying
    /// header. The remainder stays explicit and is not declared padding.
    PayloadEnd {
        remaining_bits: usize,
    },
    Sentinel,
    Truncated,
    InvalidEncoding,
    InvalidHeader,
    NonIncreasingSlot,
    Record {
        reason: KeyframeStop,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecodedKeyframeTable {
    pub configuration: Option<bool>,
    pub records: Vec<KeyframeRecord>,
    pub end_bit: i64,
    pub stop: KeyframeTableStop,
}

/// Sequential keyframe traversal. Never scans forward for plausible headers.
/// Completely decoded records seed bindings; unresolved records stop the table.
pub fn decode_keyframe_table(
    data: &[u8],
    registry: &FilmRegistry,
    encoding: &super::FrameEncoding,
    bindings: &mut super::EntityBindings,
) -> DecodedKeyframeTable {
    let mut out = DecodedKeyframeTable {
        configuration: None,
        records: Vec::new(),
        end_bit: 0,
        stop: KeyframeTableStop::Truncated,
    };
    if !encoding.valid() || !encoding.keyframe_layout.valid() {
        out.stop = KeyframeTableStop::InvalidEncoding;
        return out;
    }
    let mut r = Cursor::new(data, 0).expect("zero offset is valid");
    out.configuration = r.bit();
    if out.configuration.is_none() {
        return out;
    }
    out.end_bit = 1;
    let mut previous_slot = None;
    loop {
        let remaining = (data.len() * 8) as i64 - out.end_bit;
        if remaining < 64 {
            out.stop = KeyframeTableStop::PayloadEnd {
                remaining_bits: usize::try_from(remaining)
                    .expect("bounded keyframe remaining bits"),
            };
            return out;
        }
        r.position = out.end_bit;
        let id = r.read(32).expect("header bounded") as u32;
        if id == u32::MAX {
            out.stop = KeyframeTableStop::Sentinel;
            return out;
        }
        let ti = r.read(32).expect("header bounded") as u32;
        let slot = id & 0x3fff_ffff;
        if id >> 30 == 0 || slot >= 8192 || (ti >= 50 && ti != u32::MAX) {
            out.stop = KeyframeTableStop::InvalidHeader;
            return out;
        }
        if previous_slot.is_some_and(|previous| slot <= previous) {
            out.stop = KeyframeTableStop::NonIncreasingSlot;
            return out;
        }
        let Some(record) = decode_keyframe_record_with_encoding(
            data,
            crate::theater::bits::native_address(out.end_bit),
            registry,
            encoding,
        ) else {
            out.stop = KeyframeTableStop::Truncated;
            return out;
        };
        out.end_bit = record.end_bit;
        let reason = record.stop.clone();
        if reason == KeyframeStop::Complete && ti != u32::MAX {
            bindings.bind_with_origin(
                id,
                ti,
                super::BindingOrigin::SequentialKeyframe {
                    bit: crate::theater::bits::native_address(record.start_bit),
                },
            );
        }
        out.records.push(record);
        if reason != KeyframeStop::Complete {
            out.stop = KeyframeTableStop::Record { reason };
            return out;
        }
        previous_slot = Some(slot);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theater::FilmArchetype;
    use std::io::Read;

    #[test]
    fn native_keyframe_type_word() {
        #[derive(Deserialize)]
        struct Case {
            hex: String,
            table_hex: String,
            start: usize,
            word: u32,
            id: u32,
            layout: super::super::KeyframeLayout,
            corruption: bool,
            ti: u32,
            end: usize,
            desync: i64,
            observations: Option<Vec<crate::theater::FilmComponentObservation>>,
            table_count: usize,
            table_end: usize,
            table_desync: i64,
            table_ti: u32,
            table_stop: String,
        }
        let mut bytes = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("../fixtures/keyframe-type-word-v41.json.zlib")[..],
        )
        .read_to_end(&mut bytes)
        .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(cases.len(), 512);
        let unhex = |s: &str| {
            s.as_bytes()
                .as_chunks::<2>()
                .0
                .iter()
                .map(|c| u8::from_str_radix(std::str::from_utf8(c).unwrap(), 16).unwrap())
                .collect::<Vec<_>>()
        };
        let registry = FilmRegistry {
            major_version: 41,
            format_version: 27,
            end_byte: 0,
            truncated: false,
            archetypes: (0..50)
                .map(|index| FilmArchetype {
                    index,
                    components: if index == 5 {
                        vec!["biped-emp-timer-component".into()]
                    } else {
                        Vec::new()
                    },
                    levels: if index == 5 { vec![0] } else { Vec::new() },
                })
                .collect(),
        };
        let mut high_word_success = 0;
        let mut sentinel_table_success = 0;
        let mut absent_defaults = 0;
        for (i, c) in cases.into_iter().enumerate() {
            let encoding = crate::theater::FrameEncoding {
                keyframe_layout: c.layout,
                keyframe_simulation_complete: None,
                native_id_low_bits: None,
                component_widths: Default::default(),
                new_record: Default::default(),
                position_capture: None,
                ids: crate::theater::RecordIdLayout {
                    low_bits: 11,
                    base: 0,
                },
                mpp_widths: [9, 5],
                position: None,
                extra_fields: false,
                corruption_check: c.corruption,
            };
            let record =
                decode_native_keyframe_record(&unhex(&c.hex), c.start, &registry, &encoding)
                    .unwrap();
            assert_eq!(record.archetype, c.ti, "type {i}");
            assert_eq!(record.id, c.id, "id {i}");
            assert_eq!(
                serde_json::json!(record.end_bit),
                serde_json::json!(c.end),
                "end {i}"
            );
            assert_eq!(
                record
                    .fields
                    .iter()
                    .find(|f| f.name == "header.archetype")
                    .unwrap()
                    .raw,
                u64::from(c.word)
            );
            assert_eq!(
                record.stop,
                if c.desync == -1 {
                    KeyframeStop::Complete
                } else {
                    KeyframeStop::InvalidArchetype
                },
                "stop {i}"
            );
            assert_eq!(
                record.diagnostics.component_observations,
                c.observations.unwrap_or_default(),
                "observations {i}"
            );
            assert_eq!(
                serde_json::from_value::<KeyframeRecord>(serde_json::to_value(&record).unwrap())
                    .unwrap(),
                record
            );
            high_word_success += usize::from(c.word > 63 && c.desync == -1);
            absent_defaults += usize::from(
                matches!(c.word, 23 | 41 | 44) && c.layout.size_word_bits > 0 && c.desync == -1,
            );
            let mut bindings = crate::theater::EntityBindings::default();
            let table =
                decode_keyframe_table(&unhex(&c.table_hex), &registry, &encoding, &mut bindings);
            assert_eq!(table.records.len(), c.table_count, "table count {i}");
            let stop = match table.stop {
                KeyframeTableStop::PayloadEnd { .. } | KeyframeTableStop::Sentinel => {
                    "fin-du-payload"
                }
                KeyframeTableStop::InvalidHeader => "en-tete-invalide",
                KeyframeTableStop::NonIncreasingSlot => "slot-non-croissant",
                KeyframeTableStop::Record { .. } => "composant-non-porte",
                _ => "unexpected",
            };
            assert_eq!(stop, c.table_stop, "table stop {i}");
            if let Some(first) = table.records.first() {
                assert_eq!(
                    serde_json::json!(first.end_bit),
                    serde_json::json!(c.table_end),
                    "table end {i}"
                );
                assert_eq!(first.archetype, c.table_ti, "table type {i}");
                assert_eq!(
                    first.stop == KeyframeStop::Complete,
                    c.table_desync == -1,
                    "table body {i}"
                );
                sentinel_table_success += usize::from(
                    first.archetype == u32::MAX && first.stop == KeyframeStop::Complete,
                );
            }
            assert_eq!(
                bindings.slots.contains_key(&(c.id & 0x3fff_ffff)),
                c.table_count > 0 && c.table_ti != u32::MAX && c.table_desync == -1,
                "binding {i}"
            );
        }
        assert!(high_word_success > 0 && sentinel_table_success > 0 && absent_defaults > 0);
    }

    #[test]
    fn captured_keyframe_reaches_the_same_upstream_gap() {
        fn inflate(data: &[u8]) -> Vec<u8> {
            let mut out = Vec::new();
            flate2::read::ZlibDecoder::new(data)
                .read_to_end(&mut out)
                .unwrap();
            out
        }
        let payload = inflate(include_bytes!("../fixtures/captured-keyframe-v41.zlib"));
        let oracle: serde_json::Value = serde_json::from_slice(&inflate(include_bytes!(
            "../fixtures/captured-keyframe-v41-oracle.json.zlib"
        )))
        .unwrap();
        let registry = crate::theater::parse_registry(&inflate(include_bytes!(
            "../fixtures/bootstrap-v41.zlib"
        )))
        .unwrap();
        compare_captured_table(&payload, &registry, &oracle);
    }

    #[test]
    #[ignore = "requires all 32 local films; compares 451 native sequential keyframe walks"]
    fn captured_keyframe_tables_match_native_corpus() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("../fixtures/keyframe-table-corpus-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(rows.len(), 451);
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("experiments/films");
        let mut records = 0;
        for row in rows {
            let path = root.join(row["file"].as_str().unwrap());
            let folder = path.parent().unwrap();
            let manifest: serde_json::Value =
                serde_json::from_slice(&std::fs::read(folder.join("film.json")).unwrap()).unwrap();
            let bootstrap = manifest["chunks"]
                .as_array()
                .unwrap()
                .iter()
                .find(|c| c["chunk_type"] == 1)
                .unwrap();
            let registry = crate::theater::parse_registry(
                &std::fs::read(folder.join(bootstrap["file"].as_str().unwrap())).unwrap(),
            )
            .unwrap();
            let data = std::fs::read(path).unwrap();
            let packets = crate::theater::walk_film_packets(&data);
            let packet = &packets[row["packet"].as_u64().unwrap() as usize];
            assert_eq!(packet.payload_offset, row["offset"]);
            assert_eq!(packet.payload_size, row["size"]);
            let payload = &data[packet.payload_offset..packet.payload_offset + packet.payload_size];
            compare_captured_table(payload, &registry, &row);
            records += row["records"].as_array().unwrap().len();
        }
        assert_eq!(records, 902);
    }

    fn compare_captured_table(payload: &[u8], registry: &FilmRegistry, oracle: &serde_json::Value) {
        fn inflate(data: &[u8]) -> Vec<u8> {
            let mut out = Vec::new();
            flate2::read::ZlibDecoder::new(data)
                .read_to_end(&mut out)
                .unwrap();
            out
        }
        let defaults: Vec<serde_json::Value> = serde_json::from_slice(&inflate(include_bytes!(
            "../fixtures/defaults-levelup-v41.json.zlib"
        )))
        .unwrap();
        // Compare with the exact reference context; these widths are not
        // asserted to be a universal production profile.
        let position = serde_json::from_value(defaults[0]["position_encoding"].clone()).unwrap();
        let encoding = super::super::FrameEncoding {
            keyframe_layout: Default::default(),
            keyframe_simulation_complete: None,
            native_id_low_bits: None,
            component_widths: Default::default(),
            new_record: Default::default(),
            position_capture: None,
            ids: crate::theater::RecordIdLayout {
                low_bits: 13,
                base: 0,
            },
            mpp_widths: [9, 5],
            position: Some(position),
            extra_fields: false,
            corruption_check: oracle["check"] == true,
        };
        let table = decode_keyframe_table(
            payload,
            registry,
            &encoding,
            &mut super::super::EntityBindings::default(),
        );
        assert_eq!(oracle["stop"], "composant-non-porte");
        let expected = oracle["records"].as_array().unwrap();
        assert_eq!(table.records.len(), expected.len());
        assert_eq!(
            table.stop,
            KeyframeTableStop::Record {
                reason: KeyframeStop::UnsupportedComponent {
                    index: 10,
                    name: "tacmap-mapdismissallock".into()
                }
            }
        );
        for (actual, expected) in table.records.iter().zip(expected) {
            assert!(actual.end_bit <= (payload.len() * 8) as i64);
            for field in &actual.fields {
                assert!(field.bit >= actual.start_bit);
                assert!(field.bit.wrapping_add(field.width as i64) <= actual.end_bit);
                assert_eq!(
                    serde_json::json!(
                        crate::theater::bits::Bits(payload).read(field.bit, field.width)
                    ),
                    serde_json::json!(Some(field.raw))
                );
            }
            assert_eq!(actual.start_bit, expected["BitStart"]);
            assert_eq!(actual.end_bit, expected["BitEnd"]);
            assert_eq!(actual.id & 0x3fff_ffff, expected["Slot"]);
            assert_eq!(actual.id >> 30, expected["Gen"]);
            assert_eq!(actual.archetype, expected["Archetype"]);
            let comps: Vec<_> = expected["Comps"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|c| c["Ported"] == true)
                .collect();
            assert_eq!(actual.components.len(), comps.len());
            for (actual, expected) in actual.components.iter().zip(comps) {
                assert_eq!(actual.start_bit, expected["StartBit"]);
                assert_eq!(actual.name, expected["Name"]);
            }
        }
    }

    #[test]
    fn full_state_guards_components_and_checks_match_reference() {
        let mut json = String::new();
        flate2::read::ZlibDecoder::new(
            include_bytes!("../fixtures/keyframes-d61443e-v41.json.zlib").as_slice(),
        )
        .read_to_string(&mut json)
        .unwrap();
        let cases: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
        let mut registry = FilmRegistry {
            archetypes: (0..10)
                .map(|index| FilmArchetype {
                    index,
                    components: Vec::new(),
                    levels: Vec::new(),
                })
                .collect(),
            major_version: 41,
            format_version: 27,
            end_byte: 0,
            truncated: false,
        };
        registry.archetypes[9].components = [
            "unit-actor-control-component",
            "unit-actor-state-component",
            "weapon-state-ammo",
        ]
        .map(String::from)
        .to_vec();
        registry.archetypes[9].levels = vec![2, 4, 1];
        assert_eq!(cases.len(), 256);
        for case in cases {
            let hex = case["hex"].as_str().unwrap();
            let bytes: Vec<u8> = (0..hex.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                .collect();
            let result = decode_keyframe_record(
                &bytes,
                case["start_bit"].as_u64().unwrap() as usize,
                &registry,
                [9, 5],
                Some(
                    &crate::theater::NativeScanProfile::default()
                        .component_encoding()
                        .unwrap(),
                ),
                case["check"] == true,
            )
            .unwrap();
            assert_eq!(result.stop, KeyframeStop::Complete, "{case}");
            assert_eq!(result.id, 0x40000203);
            assert_eq!(result.archetype, 9);
            assert_eq!(result.end_bit, case["end_bit"], "{case}");
            let expected = case["components"].as_array().cloned().unwrap_or_default();
            assert_eq!(result.components.len(), expected.len());
            for (actual, expected) in result.components.iter().zip(expected) {
                assert_eq!(actual.start_bit, expected["StartBit"]);
                assert_eq!(actual.name, expected["Name"]);
            }
            let mut bit = result.start_bit;
            for field in result.fields {
                assert_eq!(field.bit, bit);
                bit += field.width as i64;
            }
            assert_eq!(bit, result.end_bit);
        }
    }
}
