use super::*;
use crate::theater::*;
use std::io::Read;

fn bytes(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|at| u8::from_str_radix(&hex[at..at + 2], 16).unwrap())
        .collect()
}
fn reader<'a>(data: &'a [u8], bit: usize, encoding: Option<&'a PositionEncoding>) -> Reader<'a> {
    Reader {
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
        cursor: Cursor::new(data, bit).unwrap(),
        fields: Vec::new(),
        position_encoding: encoding,
    }
}

#[test]
fn native_projectile_defaults_d61443e() {
    #[derive(serde::Deserialize)]
    struct Case {
        input: String,
        start: usize,
        end: i64,
        next: u64,
        param5: bool,
        encoding: PositionEncoding,
        mpp: [usize; 2],
        observations: Vec<FilmComponentObservation>,
        indices: Option<std::collections::BTreeMap<i32, u64>>,
        frame: Option<String>,
        frame_end: Option<i64>,
        record_end: Option<i64>,
    }
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        include_bytes!("../fixtures/projectile-defaults-d61443e-v41.json.zlib").as_slice(),
    )
    .read_to_end(&mut raw)
    .unwrap();
    let cases: Vec<Case> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(cases.len(), 1536);
    let registry = FilmRegistry {
        major_version: 41,
        format_version: 27,
        end_byte: 0,
        truncated: false,
        archetypes: (0..42)
            .map(|index| FilmArchetype {
                index,
                components: vec![],
                levels: vec![],
            })
            .collect(),
    };
    let mut kinds = [[0; 4]; 2];
    let mut positions = [0; 2];
    for (i, c) in cases.iter().enumerate() {
        let data = bytes(&c.input);
        let mut r = reader(&data, c.start, Some(&c.encoding));
        assert_eq!(projectile(&mut r, c.mpp, c.param5), Some(true), "{i}");
        assert_eq!(r.cursor.position, c.end, "{i}");
        assert_eq!(
            crate::theater::bits::Bits(&data).read(c.end as usize, 8),
            Some(c.next)
        );
        let mut at = c.start as i64;
        for f in &r.fields {
            assert_eq!(f.bit, at, "{i}: {}", f.name);
            assert_eq!(
                crate::theater::bits::Bits(&data).read(f.bit as usize, f.width as usize),
                Some(f.raw)
            );
            at += f.width as i64;
            if f.name == "projectile.target.kind" {
                kinds[usize::from(c.param5)][f.raw as usize] += 1;
            }
            if f.name == "projectile.relative" {
                positions[usize::from(f.raw != 0)] += 1;
            }
        }
        assert_eq!(at, c.end);
        assert_eq!(r.diagnostics.component_observations, c.observations, "{i}");
        assert_eq!(
            r.diagnostics.absolute_indices,
            c.indices.clone().unwrap_or_default()
        );
        let decoded = DecodedComponent {
            name: "default-state-41".into(),
            start_bit: c.start as i64,
            end_bit: c.end,
            fields: r.fields,
            references: r.references,
            diagnostics: r.diagnostics,
        };
        assert_eq!(
            serde_json::from_slice::<DecodedComponent>(&serde_json::to_vec(&decoded).unwrap())
                .unwrap(),
            decoded
        );
        // A complete source word cannot be replaced by synthetic zeros in the
        // bounded reader. Remove the byte containing the final consumed bit.
        let short = &data[..((c.end - 1) / 8) as usize];
        let mut r = reader(short, c.start, Some(&c.encoding));
        assert_eq!(projectile(&mut r, c.mpp, c.param5), None, "truncation {i}");
        if let Some(frame) = &c.frame {
            let encoding = FrameEncoding {
                keyframe_layout: Default::default(),
                keyframe_simulation_complete: None,
                component_widths: Default::default(),
                new_record: Default::default(),
                position_capture: None,
                native_id_low_bits: None,
                ids: RecordIdLayout {
                    low_bits: 5,
                    base: 0,
                },
                mpp_widths: c.mpp,
                position: Some(c.encoding.clone()),
                extra_fields: false,
                corruption_check: false,
            };
            let view = decode_native_entity_view(
                &bytes(frame),
                c.start,
                &registry,
                &encoding,
                &mut EntityBindings::default(),
                true,
            );
            assert_eq!(view.stop, EntityViewStop::Complete, "NEW {i}");
            assert_eq!(Some(view.end_bit), c.frame_end, "NEW end {i}");
            assert_eq!(view.records.len(), 2);
            let record = &view.records[0];
            assert_eq!(record.header.kind, RecordKind::New);
            assert_eq!(record.archetype, Some(41));
            assert_eq!(record.mask, Some(0));
            assert!(!record.default_state_fallback);
            assert_eq!(Some(record.end_bit), c.record_end, "NEW record end {i}");
        }
    }
    assert!(kinds.iter().flatten().all(|&n| n > 0));
    assert!(positions.iter().all(|&n| n > 0));
    // The generic/full-keyframe default route has not acquired NEW's grammar.
    assert!(!has_native_deserializer(41));
    assert_eq!(
        decode_default_state(&[0; 128], 0, 41, [9, 5], None),
        ComponentDecode::Unsupported
    );
}

#[test]
fn native_projectile_default_routing_d61443e() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        include_bytes!("../fixtures/projectile-default-routing-d61443e-v41.json.zlib").as_slice(),
    )
    .read_to_end(&mut raw)
    .unwrap();
    let cases: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(cases.len(), 8);
    let registry = FilmRegistry {
        major_version: 41,
        format_version: 27,
        end_byte: 0,
        truncated: false,
        archetypes: (0..42)
            .map(|index| FilmArchetype {
                index,
                components: vec![],
                levels: vec![],
            })
            .collect(),
    };
    for c in cases {
        let start = c["start"].as_u64().unwrap() as usize;
        let mut cfg = NativeFrameConfig {
            id_low_bits: 5,
            packet_preamble_bits: 0,
            new_default_state_bits: 7,
            ..Default::default()
        };
        cfg.context.profile.grammar.default_state_by_archetype = false;
        let data = bytes(c["frame"].as_str().unwrap());
        let mut r = NativeFilmReader::new(&data);
        r.set_bit_position(start);
        let view = r
            .read_frame_records(&registry, &mut FilmWorld::default(), &cfg)
            .unwrap();
        assert_eq!(view.stop, EntityViewStop::Complete);
        assert_eq!(serde_json::json!(r.native_bit_position()), c["frame_end"]);
        let record = &view.records[0];
        assert_eq!(record.archetype, Some(41));
        assert!(record.default_state_fallback);
        assert_eq!(record.mask, Some(0));
        assert_eq!(serde_json::json!(record.end_bit), c["record_end"]);
        let data = bytes(c["keyframe"].as_str().unwrap());
        let key = decode_keyframe_record(&data, start, &registry, [9, 5], None, false).unwrap();
        assert_eq!(key.archetype, 41);
        assert_eq!(key.stop, KeyframeStop::Complete);
        assert_eq!(serde_json::json!(key.end_bit), c["keyframe_end"]);
        assert!(
            key.fields
                .iter()
                .all(|f| !f.name.starts_with("projectile."))
        );
    }
}
