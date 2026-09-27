use super::*;
use serde_json::Value;
use std::io::Read;
#[test]
fn native_large_component_widths() {
    compare_large_component_widths(false);
}
#[test]
fn native_large_static_component_widths() {
    compare_large_component_widths(true);
}
fn compare_large_component_widths(static_widths: bool) {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/large-width-v41.json.zlib")[..])
        .read_to_end(&mut raw)
        .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 18);
    for row in rows {
        let mode = row["mode"].as_u64().unwrap();
        let width = row["width"].as_i64().unwrap();
        let name = row["name"].as_str().unwrap();
        let hex = row["hex"].as_str().unwrap();
        let bytes: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect();
        let registry = FilmRegistry {
            major_version: 41,
            format_version: 27,
            end_byte: 0,
            truncated: false,
            archetypes: vec![FilmArchetype {
                index: 0,
                components: vec![name.into()],
                levels: vec![0],
            }],
        };
        let mut context = NativeReaderContext::default();
        let widths = NativeSharedWidths::from_map([(name.into(), width)].into());
        if mode == 0 {
            context.profile.grammar.calibrated_widths = Some(widths.clone())
        } else {
            context.profile.grammar.stub_widths = Some(widths.clone())
        };
        let mut bindings = EntityBindings::default();
        bindings.bind(50, 0);
        let record = components::decode_entity_record_with_capture_slots(
            &bytes,
            0,
            &registry,
            &FrameEncoding {
                ids: RecordIdLayout {
                    low_bits: 11,
                    base: 0,
                },
                keyframe_layout: Default::default(),
                keyframe_simulation_complete: None,
                native_id_low_bits: None,
                component_widths: if static_widths {
                    let width = usize::try_from(width).expect("native width fits this host");
                    let mut overrides = ComponentWidthOverrides::default();
                    if mode == 0 {
                        overrides.calibrated.insert(name.into(), width);
                    } else {
                        overrides.stubs.insert(name.into(), width);
                    }
                    overrides
                } else {
                    Default::default()
                },
                new_record: Default::default(),
                position_capture: None,
                mpp_widths: [9, 5],
                position: None,
                extra_fields: false,
                corruption_check: false,
            },
            &bindings,
            true,
            components::RecordCaptureSlots {
                context: (!static_widths).then_some(context),
                movement: None,
                position: None,
            },
        )
        .unwrap();
        let expected = row["end"].as_i64().unwrap();
        for adjustment in &record.diagnostics.width_adjustments {
            assert_eq!(
                adjustment.native_end_bit(),
                Some(expected),
                "native endpoint mode={mode} width={width}"
            );
        }
        if expected >= 0 {
            assert_eq!(
                serde_json::json!(record.end_bit),
                serde_json::json!(expected),
                "mode {mode} width {width}"
            );
            assert_eq!(record.stop, EntityViewStop::Complete);
            if width > 4096 && mode != 2 {
                let adjustment = &record.diagnostics.width_adjustments[0];
                assert_eq!(adjustment.width, width);
                assert_eq!(adjustment.end_bit, usize::try_from(expected).ok());
                assert_eq!(adjustment.bit, 25);
                let available = bytes.len() * 8 - 25;
                assert_eq!(adjustment.retained_bits, Some(available));
                let fields: Vec<_> = record.fields.iter().filter(|f| f.bit >= 25).collect();
                assert_eq!(
                    fields
                        .iter()
                        .map(|f| usize::try_from(f.width).unwrap())
                        .sum::<usize>(),
                    available
                );
                for field in fields {
                    assert_eq!(
                        serde_json::json!(bits::Bits(&bytes).read(field.bit, field.width)),
                        serde_json::json!(Some(field.raw))
                    );
                }
                assert!(record.fields.len() < 16, "padding expansion is unbounded");
            } else {
                assert!(record.diagnostics.width_adjustments.is_empty());
            }
        } else {
            assert_eq!(record.stop, EntityViewStop::Complete);
            assert_eq!(record.end_bit, expected);
            let adjustment = &record.diagnostics.width_adjustments[0];
            assert_eq!(adjustment.width, width);
            assert_eq!(adjustment.bit, 25);
            assert_eq!(adjustment.end_bit, None);
            assert_eq!(adjustment.native_end_bit(), Some(expected));
        }
        assert_eq!(widths.get(name), Some(width));
        let restored: EntityRecord =
            serde_json::from_value(serde_json::to_value(&record).unwrap()).unwrap();
        assert_eq!(restored, record);
        for adjustment in &restored.diagnostics.width_adjustments {
            assert_eq!(adjustment.native_end_bit(), Some(expected));
        }
        let mut merged = FilmReadDiagnostics::default();
        merged.merge(&record.diagnostics);
        assert_eq!(
            merged.width_adjustments,
            record.diagnostics.width_adjustments
        );
    }
}
