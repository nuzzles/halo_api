//! Slide and posture bodies from the reference movement-state deserializers.
use super::{Reader, orientation};

pub(super) fn slide(r: &mut Reader<'_>, level: u32) -> Option<bool> {
    let field_start = r.fields.len();
    if r.bit("active")? {
        if !r.bit("normal.constant")? {
            r.r("normal.direction", 19)?;
            r.r("normal.magnitude", 10)?;
        }
        r.r("fraction_a", 8)?;
        if level >= 1 {
            r.r("fraction_b", 8)?;
        }
        r.r("tail", 8)?;
    }
    let field = |name: &str| {
        r.fields[field_start..]
            .iter()
            .find(|f| f.name == name)
            .map_or(0, |f| f.raw)
    };
    let values = [
        "active",
        "normal.constant",
        "normal.direction",
        "normal.magnitude",
        "fraction_a",
        "fraction_b",
        "tail",
    ]
    .map(field)
    .to_vec();
    r.publish_movement(crate::theater::NativeMovementComponent::Slide, values);
    Some(true)
}

pub(super) fn posture(r: &mut Reader<'_>) -> Option<bool> {
    let field_start = r.fields.len();
    match r.r("tag", 2)? {
        0 => {
            if !r.bit("gate")? {
                match r.r("sub", 2)? {
                    1 => {
                        r.r("handle", 15)?;
                        r.words("flags", 2, 1)?;
                        r.r("kind", 2)?;
                    }
                    2 | 3 => {
                        r.r("handle", 15)?;
                        r.r("kind", 2)?;
                        r.bit("flag")?;
                        r.r("word", 32)?;
                        r.gate("direction", 19, true)?;
                    }
                    _ => {}
                }
            }
        }
        1 => {
            if r.r("sub", 2)? != 0 {
                r.r("handle", 15)?;
                r.words("kinds", 2, 2)?;
                if r.position_encoding.is_none() {
                    return Some(false);
                }
                orientation::media_frame(r)?;
                r.r("direction", 19)?;
            }
        }
        2 => {
            if r.position_encoding.is_none() {
                return Some(false);
            }
            orientation::media_frame(r)?;
            r.r("direction", 19)?;
            if r.bit("gate")? {
                r.optional_handle("reference", 0)?;
            } else {
                r.r("word", 32)?;
            }
            r.r("handle", 15)?;
        }
        3 => {
            r.r("handle", 15)?;
            if r.bit("gate")? {
                if r.position_encoding.is_none() {
                    return Some(false);
                }
                orientation::media_frame(r)?;
                r.r("direction", 19)?;
            }
            r.r("word", 32)?;
            r.words("flags", 2, 1)?;
        }
        _ => unreachable!(),
    }
    let field = |name: &str| {
        r.fields[field_start..]
            .iter()
            .find(|f| f.name == name)
            .map_or(0, |f| f.raw)
    };
    let tag = field("tag");
    let gate = if tag == 1 {
        u64::from(field("sub") != 0)
    } else {
        field("gate")
    };
    let word = if tag == 2 && gate != 0 {
        field("reference.value")
    } else {
        field("word")
    };
    let values = vec![
        tag,
        gate,
        field("sub"),
        field("handle"),
        field("direction"),
        word,
    ];
    r.publish_movement(crate::theater::NativeMovementComponent::Posture, values);

    Some(true)
}

#[cfg(test)]
mod tests {
    use crate::theater::{PositionEncoding, components::decode_component_attempt};
    use serde_json::Value;
    use std::io::Read;

    #[test]
    fn movement_bodies_and_published_values_match_reference() {
        let mut json = String::new();
        flate2::read::ZlibDecoder::new(
            include_bytes!("../fixtures/movement-components-v41.json.zlib").as_slice(),
        )
        .read_to_string(&mut json)
        .unwrap();
        let cases: Vec<Value> = serde_json::from_str(&json).unwrap();
        assert_eq!(cases.len(), 6144);
        for case in cases {
            let hex = case["hex"].as_str().unwrap();
            let data: Vec<_> = (0..hex.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                .collect();
            let name = case["name"].as_str().unwrap();
            let start = case["start"].as_u64().unwrap() as usize;
            let level = case["level"].as_u64().unwrap() as u32;
            let encoding: PositionEncoding =
                serde_json::from_value(case["encoding"].clone()).unwrap();
            let (status, body) =
                decode_component_attempt(&data, start, name, level, 35, Some(&encoding));
            if case["ported"] == false {
                assert_eq!(status, Some(false), "{case}");
                assert_eq!(serde_json::json!(body.end_bit), serde_json::json!(start));
                continue;
            }
            assert_eq!(status, Some(true), "{case}");
            assert_eq!(case["ported"], true);
            assert_eq!(body.end_bit, case["end"], "{case}");
            let field = |name: &str| {
                body.fields
                    .iter()
                    .find(|f| f.name == name)
                    .map(|f| f.raw)
                    .unwrap_or(0)
            };
            let actual = if name.starts_with("unit-crouch") {
                vec![field("flag"), field("progress")]
            } else if name.starts_with("biped-slide") {
                vec![
                    field("active"),
                    field("normal.constant"),
                    field("normal.direction"),
                    field("normal.magnitude"),
                    field("fraction_a"),
                    field("fraction_b"),
                    field("tail"),
                ]
            } else {
                let tag = field("tag");
                let gate = if tag == 1 {
                    u64::from(field("sub") != 0)
                } else {
                    field("gate")
                };
                let dir = field("direction");
                let word = if tag == 2 && gate != 0 {
                    field("reference.value")
                } else {
                    field("word")
                };
                vec![tag, gate, field("sub"), field("handle"), dir, word]
            };
            assert_eq!(serde_json::json!(actual), case["values"], "{case}");
            // A read missing the last actual bit cannot complete or publish a zero-filled body.
            let cut = (body.end_bit - 1) / 8;
            if cut * 8 > start as i64 {
                let (status, _) = decode_component_attempt(
                    &data[..crate::theater::bits::native_address(cut)],
                    start,
                    name,
                    level,
                    35,
                    Some(&encoding),
                );
                assert_eq!(status, None, "{case}");
            }
        }
    }
}
