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
