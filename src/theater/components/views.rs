//! Message and control views. Unsupported paths mirror explicit LevelUp gaps.
use super::Cursor;
use super::{ComponentField, Reader, control};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FrameViewStop {
    Complete,
    Truncated,
    Unsupported { reason: String },
    RecordLimit,
}

/// One completely read native kind-0 control payload. Optional fields remain
/// absent when their gates are closed. Partial payload fields stay in the view.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeControlEntry {
    pub start_bit: i64,
    pub end_bit: i64,
    pub index: u8,
    pub baseline: Option<u8>,
    pub short: Option<u8>,
    pub analog: Option<[u8; 2]>,
    pub third_analog: Option<u8>,
    pub extra: Option<u8>,
    pub flags: Option<u8>,
    pub action: Option<super::NativeActionBlock>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecodedFrameView {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub control_entries: Vec<NativeControlEntry>,
    /// Includes the precise failed bounded read or grouped guard, when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diagnostics: Option<Box<crate::theater::FilmReadDiagnostics>>,
    pub start_bit: i64,
    /// First unread bit, including when an unsupported branch is encountered.
    pub end_bit: i64,
    /// Synthetic bits consumed by native action decoding beyond the source.
    /// A padded view remains truncated rather than becoming a complete view.
    #[serde(default, skip_serializing_if = "no_padding")]
    pub padded_bits: usize,
    pub kinds: Vec<u8>,
    pub fields: Vec<ComponentField>,
    pub stop: FrameViewStop,
}

fn no_padding(bits: &usize) -> bool {
    *bits == 0
}

/// Read view A at a known boundary. The reference supports its empty terminator
/// and message selectors, but has not ported the 123 message payload grammars.
pub fn decode_message_view(data: &[u8], bit: usize) -> DecodedFrameView {
    decode_message_view_signed(data, bit as i64)
}

/// Read view C using the reference's short, quantized control profile.
/// Caller must establish this profile and the view's boundary independently.
pub fn decode_control_view(data: &[u8], bit: usize) -> DecodedFrameView {
    decode_control_view_signed(data, bit as i64)
}

/// Native view A reader with signed positions and wrapping source guards.
/// Invalid negative reads retain the native panic behavior.
pub fn decode_message_view_signed(data: &[u8], bit: i64) -> DecodedFrameView {
    decode_view(data, bit, false, None, None)
}
/// Native view C reader with signed positions and wrapping source guards.
/// Invalid negative reads retain the native panic behavior.
pub fn decode_control_view_signed(data: &[u8], bit: i64) -> DecodedFrameView {
    decode_control_view_contextual(data, bit, None, None)
}

/// Native control view with the caller's film precision and lazy native widths.
pub(crate) fn decode_control_view_contextual(
    data: &[u8],
    bit: i64,
    encoding: Option<&super::PositionEncoding>,
    context: Option<&crate::theater::NativeReaderContext>,
) -> DecodedFrameView {
    let default_encoding = crate::theater::NativeScanProfile::default()
        .component_encoding()
        .expect("default native widths");
    decode_view(
        data,
        bit,
        true,
        Some(encoding.unwrap_or(&default_encoding)),
        context,
    )
}

fn decode_view(
    data: &[u8],
    bit: i64,
    is_control: bool,
    encoding: Option<&super::PositionEncoding>,
    context: Option<&crate::theater::NativeReaderContext>,
) -> DecodedFrameView {
    let mut out = DecodedFrameView {
        control_entries: Vec::new(),
        diagnostics: Default::default(),
        start_bit: bit,
        end_bit: bit,
        padded_bits: 0,
        kinds: Vec::new(),
        fields: Vec::new(),
        stop: FrameViewStop::Truncated,
    };
    let cursor = Cursor::guarded(data, bit);
    let mut r = Reader {
        native_widths: context.map(|c| super::NativeComponentWidths {
            movement: &c.profile.movement,
            mpp: c.profile.mpp,
            maximum: u64::MAX,
        }),
        width_error: None,
        live_observer: context.and_then(|c| c.observer.clone()),
        live_grammar: None,
        position_capture: None,
        position_start: 0,
        position_slot: 0,
        position_fallback: false,
        movement_slot: None,
        references: Vec::new(),
        diagnostics: Default::default(),
        cursor,
        fields: Vec::new(),
        position_encoding: encoding,
    };
    let result = walk(
        &mut r,
        is_control,
        &mut out.kinds,
        &mut out.control_entries,
        data,
    );
    out.end_bit = r.cursor.position;
    out.padded_bits = usize::try_from(out.end_bit)
        .unwrap_or(0)
        .saturating_sub((data.len() * 8).max(usize::try_from(bit).unwrap_or(0)));
    out.fields = r.fields;
    out.diagnostics = (!r.diagnostics.is_empty()).then(|| Box::new(r.diagnostics));
    out.stop = result.unwrap_or(FrameViewStop::Truncated);
    out
}

fn unsupported(reason: &str) -> FrameViewStop {
    FrameViewStop::Unsupported {
        reason: reason.into(),
    }
}

fn walk<'a>(
    r: &mut Reader<'a>,
    is_control: bool,
    kinds: &mut Vec<u8>,
    entries: &mut Vec<NativeControlEntry>,
    data: &'a [u8],
) -> Option<FrameViewStop> {
    for record in 0..64 {
        if !r.bit(&format!("record[{record}].present"))? {
            return Some(FrameViewStop::Complete);
        }
        let kind = r.r(
            &format!("record[{record}].kind"),
            if is_control { 2 } else { 7 },
        )? as u8;
        kinds.push(kind);
        if !is_control {
            return Some(unsupported("message payload"));
        }
        match kind {
            3 => continue,
            0 => {}
            _ => return Some(unsupported("control handler kind 1 or 2")),
        }
        let start_bit = r.cursor.position;
        let baseline = r.gated_value("control.baseline", 7, true)?.map(|v| v as u8);
        r.guard_source("control.index_and_input", 6)?;
        let index = r.r("control.index", 5)? as u8;
        let mut entry = NativeControlEntry {
            start_bit,
            end_bit: start_bit,
            index,
            baseline,
            short: None,
            analog: None,
            third_analog: None,
            extra: None,
            flags: None,
            action: None,
        };
        if r.bit("control.input.present")? {
            entry.short = r
                .gated_value("control.input.second_field", 2, true)?
                .map(|v| v as u8);
            // Native consumeCoupleAnalogique guards both scalars and the
            // following presence bit as one 13-bit group.
            r.guard_source("control.input.analog_group", 13)?;
            entry.analog = Some([
                r.r("control.input.analog[0]", 6)? as u8,
                r.r("control.input.analog[1]", 6)? as u8,
            ]);
            if r.bit("control.input.third_analog.present")? {
                entry.third_analog = Some(r.r("control.input.third_analog", 5)? as u8);
            }
            if r.bit("control.input.extra.present")? {
                entry.extra = Some(r.r("control.input.extra", 6)? as u8);
            }
            if r.bit("control.input.alternate")? {
                entry.flags = Some(r.r("control.input.flags", 5)? as u8);
            }
            // The native action decoder guards its first bit, then uses the
            // zero-tail reader and checks the consumed endpoint afterward.
            r.guard_source("control.actions.first_bit", 1)?;
            r.cursor = Cursor::signed(data, r.cursor.position, None);
            let action = control::actions(r)?;
            if r.cursor.position > (data.len() as i64 * 8) {
                return None;
            }
            entry.action = Some(match action {
                control::ActionBlockRead::Decoded(block) => block,
                control::ActionBlockRead::MissingPositionContext => {
                    return Some(unsupported("action aim position context unavailable"));
                }
            });
            r.cursor = Cursor::guarded(data, r.cursor.position);
        }
        if r.bit("control.secondary.present")? {
            return Some(unsupported("secondary control block"));
        }
        entry.end_bit = r.cursor.position;
        entries.push(entry);
    }
    Some(FrameViewStop::RecordLimit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn view_boundaries_and_termination_match_reference() {
        let mut json = String::new();
        flate2::read::ZlibDecoder::new(
            include_bytes!("../fixtures/views-d61443e-v41.json.zlib").as_slice(),
        )
        .read_to_string(&mut json)
        .unwrap();
        let cases: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
        assert_eq!(cases.len(), 2048);
        for case in cases {
            let hex = case["hex"].as_str().unwrap();
            let bytes: Vec<u8> = (0..hex.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                .collect();
            let bit = case["start_bit"].as_u64().unwrap() as usize;
            let result = if case["control"] == true {
                decode_control_view(&bytes, bit)
            } else {
                decode_message_view(&bytes, bit)
            };
            assert_eq!(result.end_bit, case["end_bit"], "{case}");
            assert_eq!(
                result.stop == FrameViewStop::Complete,
                case["complete"],
                "{case}"
            );
            assert_eq!(serde_json::to_value(&result.kinds).unwrap(), case["kinds"]);
            assert_ne!(result.stop, FrameViewStop::Truncated, "{case}");
            for prefix in case["prefixes"].as_array().unwrap() {
                let cut = prefix["bytes"].as_u64().unwrap() as usize;
                let short = &bytes[..cut];
                let view = if case["control"] == true {
                    decode_control_view(short, bit)
                } else {
                    decode_message_view(short, bit)
                };
                assert_eq!(
                    view.end_bit, prefix["end_bit"],
                    "control={} start={bit} bytes={cut} input={hex}",
                    case["control"]
                );
                assert_eq!(
                    view.stop == FrameViewStop::Complete,
                    prefix["complete"].as_bool().unwrap()
                );
                assert_eq!(serde_json::to_value(&view.kinds).unwrap(), prefix["kinds"]);
                let restored: DecodedFrameView =
                    serde_json::from_value(serde_json::to_value(&view).unwrap()).unwrap();
                assert_eq!(restored, view);
                assert_eq!(
                    serde_json::json!(view.padded_bits),
                    serde_json::json!(crate::theater::bits::padded_from_native(
                        view.end_bit,
                        (cut * 8).max(bit)
                    ))
                );
                if view.padded_bits > 0 {
                    assert_eq!(view.stop, FrameViewStop::Truncated);
                }
                let mut pos = bit;
                for field in &view.fields {
                    assert_eq!(serde_json::json!(field.bit), serde_json::json!(pos));
                    for offset in 0..field.width {
                        let source_pos = field.bit + offset as i64;
                        let expected = short
                            .get(crate::theater::bits::native_address(source_pos / 8))
                            .map_or(0, |b| (b >> (7 - source_pos % 8)) & 1);
                        assert_eq!(
                            ((field.raw >> (field.width - 1 - offset)) & 1) as u8,
                            expected
                        );
                    }
                    pos += usize::try_from(field.width).unwrap();
                }
                assert_eq!(serde_json::json!(pos), serde_json::json!(view.end_bit));
            }
            let mut position = bit;
            for field in result.fields {
                assert_eq!(serde_json::json!(field.bit), serde_json::json!(position));
                position += usize::try_from(field.width).unwrap();
            }
            assert_eq!(
                serde_json::json!(position),
                serde_json::json!(result.end_bit)
            );
        }
    }

    #[test]
    fn production_control_prefixes_match_native_frame_oracle() {
        use crate::theater::{
            FilmArchetype, FilmRegistry, FilmWorld, NativeFrameConfig, ProductionFrame,
        };
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            include_bytes!("../fixtures/views-d61443e-v41.json.zlib").as_slice(),
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        let registry = FilmRegistry {
            major_version: 41,
            format_version: 27,
            end_byte: 0,
            truncated: false,
            archetypes: vec![FilmArchetype {
                index: 0,
                components: vec![],
                levels: vec![],
            }],
        };
        let mut count = 0;
        for row in rows {
            for prefix in row["prefixes"].as_array().unwrap() {
                let Some(expected) = prefix.get("production") else {
                    continue;
                };
                count += 1;
                let hex = expected["hex"].as_str().unwrap();
                let data: Vec<_> = (0..hex.len())
                    .step_by(2)
                    .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                    .collect();
                let cfg = NativeFrameConfig {
                    packet_preamble_bits: expected["preamble"].as_i64().unwrap(),
                    ..Default::default()
                };
                let mut world = FilmWorld {
                    current_view: 2,
                    ..Default::default()
                };
                world.bind_full(50, 0);
                world.set_position(50, [1., 2., 3.]);
                let mut expected_world = world.clone();
                expected_world.current_view = expected["world_view"].as_i64().unwrap() as i8;
                let frame = cfg
                    .decode_production_views(
                        &data,
                        cfg.packet_preamble_bits as usize,
                        &registry,
                        &mut world,
                    )
                    .unwrap();
                assert_eq!(frame.end_bit, expected["end"], "case {count}");
                assert_eq!(frame.views_completed, expected["views"], "case {count}");
                assert!(frame.records.is_empty());
                assert_eq!(world, expected_world);
                crate::theater::native_frame_accumulator_tests::check_world(
                    &world,
                    &expected["slots"],
                    "control prefixes",
                );
                assert_eq!(
                    serde_json::json!(frame.padded_bits),
                    serde_json::json!(crate::theater::bits::padded_from_native(
                        frame.end_bit,
                        data.len() * 8
                    ))
                );
                if let Some(controls) = &frame.controls {
                    assert_eq!(
                        serde_json::json!(
                            controls.padded_bits
                                + crate::theater::bits::padded_from_native(
                                    controls.start_bit,
                                    data.len() * 8
                                )
                        ),
                        serde_json::json!(frame.padded_bits)
                    );
                    if controls.padded_bits > 0 {
                        assert_eq!(controls.stop, FrameViewStop::Truncated);
                    }
                }
                let restored: ProductionFrame =
                    serde_json::from_value(serde_json::to_value(&frame).unwrap()).unwrap();
                assert_eq!(restored, frame);
            }
        }
        assert_eq!(count, 2583);
    }

    #[test]
    fn incomplete_selector_is_truncated_without_reading_past_payload() {
        let result = decode_message_view(&[1], 7);
        assert_eq!(result.stop, FrameViewStop::Truncated);
        assert_eq!(result.end_bit, 8);
        assert_eq!(decode_control_view(&[], 0).stop, FrameViewStop::Truncated);
    }
}
