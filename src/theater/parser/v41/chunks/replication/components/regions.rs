use super::{
    cursor::ComponentCursor,
    reader::{ComponentReader, ComponentWidths},
    widths::read_component,
};
use crate::theater::{
    film::catalog::maps::levels,
    parser::v41::context::{map::level_precision, profile::DecodeProfile},
};

#[test]
fn catalogue_component_points_keep_all_regions_fields_and_boundaries() {
    let oracles = levels::fixtures::oracles();
    let mut count = 0;
    for case in levels::fixtures::cases()
        .into_iter()
        .filter(|c| c.kind != "event")
    {
        count += 1;
        let level = levels::resolve(levels::BUILD, case.tag).unwrap();
        let mut profile = DecodeProfile::default();
        profile.movement.world_object = level_precision(level, 16).unwrap();
        // The old aggregate skip must not supersede known, individually sized axes.
        if let Some(region) = case.region {
            profile
                .movement
                .opaque_position_region_bits
                .insert(region, 1);
        }
        let encoding = profile.component_encoding().unwrap();
        let read = |bytes: &[u8], start: i64, context: bool| {
            let mut r = ComponentReader {
                category_references: true,
                snapshot: false,
                reference_widths: context.then_some(ComponentWidths {
                    movement: &profile.movement,
                    mpp: profile.mpp,
                    maximum: u64::MAX,
                }),
                width_error: None,
                context_error: None,
                live_grammar: None,
                movement_slot: None,
                references: vec![],
                diagnostics: Default::default(),
                cursor: ComponentCursor::guarded(bytes, start),
                fields: vec![],
                position_encoding: context.then_some(&encoding),
            };
            let status = read_component(&mut r, case.component(), 4, 0, None, (true, None));
            (status, r.cursor.position, r.fields)
        };
        let bytes = case.bytes();
        let (status, end, fields) = read(&bytes, 0, true);
        assert_eq!(status, Some(true), "{}", case.name);
        assert_eq!(end as usize, case.bit_len, "{}", case.name);
        for axis in 0..3 {
            let f = fields
                .iter()
                .find(|f| f.name == format!("position[{axis}]"))
                .unwrap();
            assert_eq!(f.width, case.widths[axis], "{}", case.name);
            assert_eq!(f.raw.low_u64(), u64::from(case.codes[axis]));
        }
        let marker = crate::theater::parser::v41::chunks::replication::source::components::field::RawBits::from_source(&bytes, case.bit_len, 8).unwrap();
        assert_eq!(marker.low_u64(), 211);
        if case.upstream {
            let oracle = oracles.iter().find(|o| o.name == case.name).unwrap();
            assert_eq!(oracle.end, case.bit_len);
            let boundaries: std::collections::BTreeSet<_> = std::iter::once(0)
                .chain(fields.iter().map(|f| f.bit + f.width))
                .collect();
            for (at, width) in &oracle.trace {
                assert!(boundaries.contains(at));
                assert!(boundaries.contains(&(at + width)));
            }
        }
        for cut in 0..case.bit_len {
            let (bytes, shift) = case.cut(cut);
            let (status, end, prefix) = read(&bytes, shift as i64, true);
            assert_eq!(status, None, "{} cut {cut}", case.name);
            assert!(end <= bytes.len() as i64 * 8);
            assert_eq!(
                prefix.len(),
                fields.iter().take_while(|f| f.bit + f.width <= cut).count()
            );
            for (got, want) in prefix.iter().zip(&fields) {
                assert_eq!(
                    (&got.name, got.bit, got.width, &got.raw),
                    (&want.name, want.bit + shift, want.width, &want.raw)
                );
            }
        }
        if case.region.is_none() {
            let (status, end, _) = read(&bytes, 0, false);
            assert_eq!(status, Some(true));
            assert_eq!(end as usize, case.bit_len);
        }
    }
    assert_eq!(count, 131);
}

pub(crate) fn record(
    case: &levels::fixtures::Case,
    refuse_region: bool,
) -> crate::theater::parser::v41::chunks::replication::source::EntityRecord {
    use crate::theater::parser::{
        test_support::component_registry,
        v41::{
            chunks::replication::replication_stream::frame::records::{
                RecordBinding, RecordReadContext, decode_entity_record,
            },
            context::V41DecodeConfig,
        },
    };
    let registry = component_registry(&[(
        3,
        vec![
            (case.component().into(), 4),
            ("equipment-being-hacked-component".into(), 1),
        ],
    )]);
    let level = levels::resolve(levels::BUILD, case.tag).unwrap();
    let mut config = V41DecodeConfig::default();
    config.profile.movement.world_object = level_precision(level, 16).unwrap();
    if refuse_region {
        let region = case.region.unwrap();
        config.profile.movement.world_object.regions.remove(&region);
        if config.profile.movement.world_object.regions.is_empty() {
            config
                .profile
                .movement
                .world_object
                .regions
                .insert(u32::MAX, [1; 3]);
        }
        config
            .profile
            .movement
            .opaque_position_region_bits
            .insert(region, case.widths.iter().sum::<usize>() as u64);
        // A missing context must not be bypassed by the historical stub policy.
        config.profile.grammar.stub_widths = Some(
            crate::theater::parser::v41::context::profile::SharedWidths::from_map(
                [(case.component().into(), 3)].into(),
            ),
        );
    }
    let mut bits = format!("1{:013b}01{}{}{:064b}", 42, 0, 1, 3);
    let source = case.bytes();
    for i in 0..case.bit_len + 8 {
        bits.push(if source[i / 8] & (1 << (7 - i % 8)) != 0 {
            '1'
        } else {
            '0'
        });
    }
    let mut bytes = vec![0; bits.len().div_ceil(8)];
    for (i, b) in bits.bytes().enumerate() {
        bytes[i / 8] |= (b - b'0') << (7 - i % 8);
    }
    decode_entity_record(
        &bytes,
        0,
        &registry,
        &config.record_layout(),
        Some(RecordBinding {
            id: (1 << 30) | 42,
            archetype: 3,
        }),
        true,
        RecordReadContext {
            context: Some(&config.profile),
            movement: None,
        },
    )
    .unwrap()
}

#[test]
fn catalogue_regions_preserve_delta_component_continuation_and_refusal() {
    use crate::theater::parser::v41::chunks::replication::source::{
        ComponentReadStatus, EntityViewStop,
    };
    for case in levels::fixtures::cases()
        .into_iter()
        .filter(|c| c.kind != "event")
    {
        let parsed = record(&case, false);
        assert_eq!(parsed.stop, EntityViewStop::Complete, "{}", case.name);
        assert_eq!(parsed.components.len(), 2);
        assert_eq!(parsed.components[0].end_bit, 82 + case.bit_len as i64);
        assert_eq!(parsed.components[1].fields[0].raw.low_u64(), 211);
        if case.region.is_some() {
            let parsed = record(&case, true);
            assert!(
                matches!(
                    parsed.stop,
                    EntityViewStop::UnsupportedComponent { index: 0, .. }
                ),
                "{}: {:?}",
                case.name,
                parsed.stop
            );
            assert_eq!(parsed.components.len(), 1);
            assert_eq!(
                parsed.components[0].status,
                ComponentReadStatus::Unsupported
            );
            let gates = if case.kind == "dynamic" { 4 } else { 2 };
            let level = levels::resolve(levels::BUILD, case.tag).unwrap();
            assert_eq!(parsed.end_bit, 82 + gates + level.index_bits as i64);
        }
    }
}
