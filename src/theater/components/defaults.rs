//! v41 archetype defaults, before the component loop. LevelUp default_state*.go.
use super::Cursor;
use super::{ComponentDecode, DecodedComponent, PositionEncoding, Reader};

/// Decode a default-state at a known boundary for the v41 registry layout.
/// `mpp_widths` gives the multiplayer-property lead and index widths (nominally
/// 9 and 5); these belong to the film profile. Unknown archetypes are unsupported,
/// never treated as zero-length defaults. Position context is required for vehicles.
pub fn decode_default_state(
    data: &[u8],
    bit: usize,
    archetype: u32,
    mpp_widths: [usize; 2],
    encoding: Option<&PositionEncoding>,
) -> ComponentDecode {
    if mpp_widths.iter().any(|w| !(1..=32).contains(w)) || encoding.is_some_and(|e| !e.valid()) {
        return ComponentDecode::Unsupported;
    }
    let Some(cursor) = Cursor::new(data, bit) else {
        return ComponentDecode::Truncated { bit: bit as i64 };
    };
    let mut r = Reader {
        native_widths: None,
        width_error: None,
        live_observer: None,
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
    match state(&mut r, archetype, mpp_widths) {
        Some(true) => ComponentDecode::Decoded(DecodedComponent {
            name: format!("default-state-{archetype}"),
            start_bit: bit as i64,
            end_bit: r.cursor.position,
            fields: r.fields,
            references: r.references,
            diagnostics: r.diagnostics,
        }),
        Some(false) => ComponentDecode::Unsupported,
        None => ComponentDecode::Truncated {
            bit: r.cursor.position,
        },
    }
}

/// Native world-object default-state reader: live signed MPP widths and zero-tail reads.
/// The caller owns bounds admission after the complete default-state attempt.
pub(crate) fn decode_native_world_default(
    data: &[u8],
    bit: i64,
    archetype: u32,
    encoding: &super::PositionEncoding,
    context: &crate::theater::NativeReaderContext,
    mirror: &std::cell::Cell<i64>,
) -> (Option<bool>, DecodedComponent, Option<&'static str>) {
    let mut r = Reader {
        native_widths: Some(super::NativeComponentWidths {
            movement: &context.profile.movement,
            mpp: context.profile.mpp,
            maximum: u64::MAX,
        }),
        width_error: None,
        live_observer: context.observer.clone(),
        live_grammar: None,
        position_capture: None,
        position_start: 0,
        position_slot: 0,
        position_fallback: false,
        movement_slot: None,
        references: Vec::new(),
        diagnostics: Default::default(),
        cursor: Cursor::signed(data, bit, Some(mirror)),
        fields: Vec::new(),
        position_encoding: Some(encoding),
    };
    let status = state(&mut r, archetype, [9, 5]);
    (
        status,
        DecodedComponent {
            name: format!("default-state-{archetype}"),
            start_bit: bit,
            end_bit: r.cursor.position,
            fields: r.fields,
            references: r.references,
            diagnostics: r.diagnostics,
        },
        r.width_error,
    )
}

fn version(r: &mut Reader<'_>) -> Option<()> {
    r.gate("version", 8, true)
}

/// FUN_1408efb58 at d61443e. NEW calls with param5=true. The full-state
/// reference form is retained here, but is not wired into keyframe traversal.
pub(super) fn projectile(r: &mut Reader<'_>, widths: [usize; 2], param5: bool) -> Option<bool> {
    let version = if r.bit("version.present")? {
        r.r("version", 8)?
    } else {
        1
    };
    multiplayer(r, widths)?;
    r.gate("projectile.value_70", 5, true)?;
    if r.bit("projectile.flag2")? {
        r.optional_handle("projectile.reference_64", 1)?;
        r.gate("projectile.id2", 2, false)?;
    }
    if r.bit("projectile.target.present")? {
        let kind = r.r("projectile.target.kind", 2)?;
        if kind == 1 || kind == 2 {
            if param5 {
                r.handle("projectile.target.reference", kind as u8)?;
            } else {
                r.r("projectile.target.word", 32)?;
            }
            if kind == 1 {
                r.gate("projectile.target.index", 6, true)?;
            }
        }
    }
    r.bit("projectile.flag4")?;
    if r.bit("projectile.scales.present")? {
        r.words("projectile.scales", 2, 5)?;
    }
    r.r("projectile.scale_90", 5)?;
    if r.bit("projectile.flag8")? {
        if !projectile_position(r)? {
            return Some(false);
        }
        r.r("projectile.direction", 19)?;
        r.r("projectile.speed", 12)?;
    }
    if r.bit("projectile.flag10")? {
        r.optional_handle("projectile.reference_b0", 0)?;
    }
    if r.bit("projectile.flag20")? {
        if r.bit("projectile.relative")? {
            r.r("projectile.relative.kind", 2)?;
            r.words("projectile.relative.axes", 3, 13)?;
            r.gate("projectile.relative.tail", 16, true)?;
        } else if !projectile_position(r)? {
            return Some(false);
        }
    }
    if version > 2 {
        r.bit("projectile.flag40")?;
    }
    if r.bit("projectile.tail.present")? {
        r.gate("projectile.tail.index", 5, false)?;
        r.r("projectile.tail.value7", 7)?;
        r.bit("projectile.tail.flag")?;
        r.r("projectile.tail.value4", 4)?;
    }
    Some(true)
}

fn projectile_position(r: &mut Reader<'_>) -> Option<bool> {
    let Some(encoding) = r.position_encoding else {
        return Some(false);
    };
    if encoding.full_precision_gate() {
        r.words("projectile.position_bits", 3, 32)?;
    } else {
        super::position::absolute_payload(r, encoding)?;
    }
    Some(true)
}

#[cfg(test)]
#[path = "defaults_projectile_tests.rs"]
mod projectile_tests;

/// Entries in LevelUp's defaultStateDeserByTI (plus its special biped path).
/// Verified zero-bit stubs are intentionally absent: TraverseEntity applies its
/// configured fallback width to them even though their ordinary default is empty.
pub(super) fn has_native_deserializer(ti: u32) -> bool {
    matches!(
        ti,
        3 | 5
            | 6
            | 8
            | 9
            | 10
            | 11
            | 12
            | 13
            | 14
            | 17
            | 20
            | 21
            | 24
            | 28
            | 29
            | 35
            | 36
            | 37
            | 38
            | 39
            | 40
            | 42
            | 43
            | 47
            | 48
            | 49
    )
}

pub(super) fn state(r: &mut Reader<'_>, ti: u32, widths: [usize; 2]) -> Option<bool> {
    match ti {
        // Verified zero-bit vtable implementations; excluded from the fallback.
        0 | 1 | 2 | 4 | 7 | 15 | 16 | 18 | 19 | 22 | 25 | 26 | 27 | 30 | 31 | 32 | 33 | 34 | 45
        | 46 => {}
        3 => {
            version(r)?;
            r.optional_handle("reference", 0)?;
            r.gate("value", 8, true)?;
            r.bit("flag")?;
            r.r("tail", 8)?;
        }
        5 | 14 | 17 | 47 => {
            version(r)?;
            r.r(
                "index",
                match ti {
                    5 => 6,
                    17 => 7,
                    _ => 5,
                },
            )?;
        }
        6 => {
            r.r("index", 6)?;
        }
        8 => {
            let version = if r.bit("version.present")? {
                r.r("version", 8)?
            } else {
                0
            };
            r.r("value", if version > 1 { 16 } else { 8 })?;
        }
        9 => {
            version(r)?;
            r.r("player_index", 6)?;
            r.r("secondary_index", 6)?;
            r.bit("flag")?;
        }
        10 => {
            version(r)?;
            r.optional_handle("reference", 0)?;
        }
        11 | 12 | 20 | 29 | 49 => version(r)?,
        13 => {
            version(r)?;
            r.r("property_name", 32)?;
            let count = if r.bit("array")? { 32 } else { 1 };
            r.words("properties", count, 4)?;
        }
        21 => {
            r.r("value", 18)?;
        }
        24 => {
            r.gate("index", 2, false)?;
            r.r("state", 3)?;
        }
        28 => {
            r.r("value", 32)?;
        }
        35 => {
            let v = if r.bit("version.present")? {
                r.r("version", 8)?
            } else {
                13
            };
            r.gate("representation_name", 32, true)?;
            if v > 10 {
                r.gate("player_index", 5, false)?;
            }
            multiplayer(r, widths)?;
            r.gate("index", 6, true)?;
            r.bit("flag")?;
            r.optional_word_reference("reference", true)?;
            r.r("direction", 19)?;
            if v > 5 {
                r.bit("version_flag")?;
            }
            if v >= 12 {
                // d61443e: this word belongs to the film reader, before the
                // NEW component gate/mask (also present in full defaults).
                let start_bit = r.cursor.position;
                let present = r.bit("external_reference_gate")?;
                let value = if present {
                    r.r("external_reference_word", 32)? as u32
                } else {
                    0
                };
                r.publish_reference(crate::theater::NativeUnitReference {
                    kind: crate::theater::NativeUnitReferenceKind::GatedWord32,
                    start_bit,
                    end_bit: r.cursor.position,
                    present,
                    value,
                    tail: 0,
                    probe: false,
                });
            }
        }
        36..=40 | 42 | 43 => {
            if ti == 40 && r.position_encoding.is_none() {
                return Some(false);
            }
            version(r)?;
            if ti == 37 || ti == 42 {
                version(r)?;
            }
            multiplayer(r, widths)?;
            match ti {
                37 => {
                    let value = r.gated_value("player_index", 5, false)?;
                    publish_creation(
                        r,
                        crate::theater::NativeEquipmentCreationField::Reference,
                        value,
                    );
                    let value = r.gated_value("ability_enabled_id", 32, true)?;
                    publish_creation(
                        r,
                        crate::theater::NativeEquipmentCreationField::AbilityId,
                        value,
                    );
                }
                38 | 39 => r.optional_handle("reference", 0)?,
                40 => {
                    if r.bit("media_frame.present")? {
                        super::orientation::media_frame(r)?;
                        r.gate("media_frame.direction", 19, false)?;
                        r.r("media_frame.magnitude", 8)?;
                    }
                    r.r("direction", 19)?;
                    let count = if r.bit("reference_list")? {
                        r.r("reference_count", 2)?
                    } else {
                        1
                    };
                    for i in 0..count {
                        r.optional_word_reference(&format!("references[{i}]"), true)?;
                    }
                }
                42 => {
                    r.r("value12", 12)?;
                    r.r("value7", 7)?;
                    super::object::magazines(r)?;
                    let value = r.gated_value("player_index", 5, false)?;
                    publish_creation(
                        r,
                        crate::theater::NativeEquipmentCreationField::Reference,
                        value,
                    );
                }
                _ => {}
            }
        }
        48 => r.gate("player_index", 5, false)?,
        _ => return Some(false),
    }
    Some(true)
}

fn publish_mpp(r: &mut Reader<'_>, field: crate::theater::NativeMppField, value: Option<u64>) {
    r.publish_component(crate::theater::FilmComponentObservation::Mpp {
        field,
        value: value.unwrap_or(0),
        present: value.is_some(),
    });
}

fn publish_creation(
    r: &mut Reader<'_>,
    field: crate::theater::NativeEquipmentCreationField,
    value: Option<u64>,
) {
    r.publish_component(
        crate::theater::FilmComponentObservation::EquipmentCreation {
            field,
            value: value.unwrap_or(0),
            present: value.is_some(),
        },
    );
}

/// Native MPP widths are signed profile values cast to uint at the read. Admit
/// each only when reached, retaining that unsigned value on a domain refusal.
fn mpp_field(r: &mut Reader<'_>, lead: bool, fallback: usize) -> Option<u64> {
    let (name, field) = if lead {
        ("mpp.lead", "MPP lead")
    } else {
        ("mpp.index", "MPP index")
    };
    if let Some(native) = r.native_widths {
        let raw = if lead {
            native.mpp.lead
        } else {
            native.mpp.index
        } as u64;
        let width = r.native_width_limited(raw, field, u64::MAX)?;
        r.r_wide(name, width)
    } else {
        r.r(name, fallback)
    }
}

fn multiplayer(r: &mut Reader<'_>, widths: [usize; 2]) -> Option<()> {
    let value = mpp_field(r, true, widths[0])?;
    publish_mpp(r, crate::theater::NativeMppField::Word9, Some(value));
    let value = r.r("mpp.object_tag", 32)?;
    publish_mpp(r, crate::theater::NativeMppField::Word32, Some(value));
    let value = r.gated_value("mpp.variant_name", 32, false)?;
    publish_mpp(r, crate::theater::NativeMppField::VariantName, value);
    r.gate("mpp.value18", 18, true)?;
    r.gate("mpp.value13", 13, true)?;
    r.r("mpp.kind", 2)?;
    mpp_field(r, false, widths[1])?;
    let count = r.r("mpp.count", 3)?;
    if count <= 4 {
        for i in 0..count {
            r.r(&format!("mpp.entry[{i}].index"), 5)?;
            r.optional_word_reference(&format!("mpp.entry[{i}].reference"), true)?;
        }
    }
    if r.bit("mpp.extra.present")? {
        r.gate("mpp.extra.index", 5, false)?;
        r.gate("mpp.extra.value", 8, true)?;
        r.words("mpp.extra.bytes", 2, 8)?;
    }
    if r.bit("mpp.tail.present")? {
        let value = r.r("mpp.tail.name", 32)?;
        publish_mpp(r, crate::theater::NativeMppField::TailName, Some(value));
        r.optional_word_reference("mpp.tail.reference", true)?;
        r.r("mpp.tail.fraction", 14)?;
    } else {
        publish_mpp(r, crate::theater::NativeMppField::TailName, None);
    }
    Some(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn native_mpp_signed_domain() {
        use crate::theater::*;
        use serde_json::{Value, json};
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            include_bytes!("../fixtures/mpp-domain-v41.json.zlib").as_slice(),
        )
        .read_to_end(&mut raw)
        .unwrap();
        let fixture: Value = serde_json::from_slice(&raw).unwrap();
        for (i, row) in fixture["direct"].as_array().unwrap().iter().enumerate() {
            let hex = row["hex"].as_str().unwrap();
            let data: Vec<_> = (0..hex.len())
                .step_by(2)
                .map(|j| u8::from_str_radix(&hex[j..j + 2], 16).unwrap())
                .collect();
            let mut profile = NativeScanProfile::default();
            if row["axis"] == 0 {
                profile.mpp.lead = row["width"].as_i64().unwrap();
            } else {
                profile.mpp.index = row["width"].as_i64().unwrap();
            }
            let mut r = Reader {
                native_widths: Some(crate::theater::components::NativeComponentWidths {
                    movement: &profile.movement,
                    mpp: profile.mpp,
                    maximum: u64::MAX,
                }),
                width_error: None,
                live_observer: None,
                live_grammar: None,
                position_capture: None,
                position_start: 0,
                position_slot: 0,
                position_fallback: false,
                movement_slot: None,
                references: vec![],
                diagnostics: Default::default(),
                cursor: Cursor::signed(&data, row["start"].as_i64().unwrap(), None),
                fields: vec![],
                position_encoding: None,
            };
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                multiplayer(&mut r, [9, 5])
            }));
            assert_eq!(
                result.is_err(),
                row["panic"].as_bool().unwrap(),
                "panic {i}"
            );
            assert_eq!(json!(r.cursor.position), row["end"], "cursor {i}");
            if let Ok(result) = result {
                assert!(result.is_some(), "completion {i}");
            }
            let events: Vec<_> = r
                .diagnostics
                .component_observations
                .iter()
                .filter_map(|v| {
                    if let FilmComponentObservation::Mpp {
                        field,
                        value,
                        present,
                    } = v
                    {
                        Some(json!({"field":field,"value":value,"present":present}))
                    } else {
                        None
                    }
                })
                .collect();
            assert_eq!(json!(events), row["events"], "publications {i}");
            r.cursor.position = 0;
            assert_eq!(json!(r.cursor.read(8).unwrap()), row["recovery"]);
            assert_eq!(json!(r.cursor.position), row["recoveryEnd"]);
        }
    }

    #[test]
    fn defaults_match_reference_at_every_alignment() {
        let mut json = String::new();
        flate2::read::ZlibDecoder::new(
            include_bytes!("../fixtures/defaults-d61443e-v41.json.zlib").as_slice(),
        )
        .read_to_string(&mut json)
        .unwrap();
        let cases: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
        assert_eq!(cases.len(), 47 * 64);
        let mut biped_word_cases = [0; 2];
        for case in cases {
            let hex = case["hex"].as_str().unwrap();
            let bytes: Vec<u8> = (0..hex.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                .collect();
            let bit = case["start_bit"].as_u64().unwrap() as usize;
            let ti = case["archetype"].as_u64().unwrap() as u32;
            let widths = serde_json::from_value(case["mpp_widths"].clone()).unwrap();
            let encoding = serde_json::from_value(case["position_encoding"].clone()).unwrap();
            let ComponentDecode::Decoded(result) =
                decode_default_state(&bytes, bit, ti, widths, Some(&encoding))
            else {
                panic!("{case}");
            };
            assert_eq!(result.end_bit, case["end_bit"], "{case}");
            if ti == 35
                && let Some(gate) = result
                    .fields
                    .iter()
                    .find(|f| f.name == "external_reference_gate")
            {
                biped_word_cases[usize::from(gate.raw != 0)] += 1;
                let word = result
                    .fields
                    .iter()
                    .find(|f| f.name == "external_reference_word");
                assert_eq!(word.is_some(), gate.raw != 0);
                if let Some(word) = word {
                    assert_eq!(word.bit, gate.bit + 1);
                    assert_eq!(word.width, 32);
                }
            }
            let mut position = bit;
            for field in result.fields {
                assert_eq!(serde_json::json!(field.bit), serde_json::json!(position));
                assert_eq!(
                    crate::theater::bits::Bits(&bytes).read(position, field.width as usize),
                    Some(field.raw)
                );
                position += usize::try_from(field.width).unwrap();
            }
            assert_eq!(
                serde_json::json!(position),
                serde_json::json!(result.end_bit)
            );
            if result.end_bit > (bit + 8) as i64 {
                assert!(matches!(
                    decode_default_state(
                        &bytes[..crate::theater::bits::native_address((result.end_bit - 1) / 8)],
                        bit,
                        ti,
                        widths,
                        Some(&encoding)
                    ),
                    ComponentDecode::Truncated { .. }
                ));
            }
        }
        assert!(biped_word_cases.iter().all(|&n| n > 0));
    }

    #[test]
    fn unknown_defaults_are_not_zero_bit_stubs() {
        for ti in [23, 41, 44, 50, u32::MAX] {
            assert_eq!(
                decode_default_state(&[0; 256], 0, ti, [9, 5], None),
                ComponentDecode::Unsupported
            );
        }
        assert!(matches!(
            decode_default_state(&[], 0, 0, [9, 5], None),
            ComponentDecode::Decoded(_)
        ));
    }
}
