//! Captured all-create prefix, independently traced without upstream recovery.
use crate::theater::parser::test_support::component_registry;
use crate::theater::parser::v41::chunks::replication::replication_stream::frame::entities::{
    ProductionReaderContext, decode_production_frame_contextual,
};
use crate::theater::parser::v41::chunks::replication::source::components::field::RawBits;
use crate::theater::parser::v41::chunks::replication::source::{
    ComponentReadStatus, EntityViewStop, ProductionEntityEnd, RecordKind,
};
use crate::theater::parser::v41::chunks::replication::state::{
    DecodeBinding, ReplicationDecodeState,
};
use crate::theater::parser::v41::context::{V41DecodeConfig, map::level_precision};
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Deserialize)]
struct Component {
    index: usize,
    start: i64,
    end: i64,
    trace: Vec<(usize, usize)>,
}

#[derive(Deserialize)]
struct Record {
    start: i64,
    end: i64,
    index: u32,
    generation: u32,
    archetype: u32,
    components: Vec<Component>,
}

#[derive(Deserialize)]
struct Registry {
    archetype: usize,
    rows: Vec<(String, u32)>,
}

#[derive(Deserialize)]
struct Slot {
    #[serde(rename = "TypeIndex")]
    archetype: u32,
    #[serde(rename = "FullID")]
    full_id: u32,
    #[serde(rename = "Soft")]
    soft: bool,
    #[serde(rename = "GenAny")]
    generation_any: bool,
    #[serde(rename = "Vue")]
    view: i8,
}

#[derive(Deserialize)]
struct Context {
    namespace: u8,
    slots: BTreeMap<u32, Slot>,
}

#[derive(Deserialize)]
struct Fixture {
    commit: String,
    build_id: u32,
    branch: String,
    level_tag: u32,
    axis_bits: [u64; 3],
    index_bits: u64,
    initial_context: Context,
    registry: Vec<Registry>,
    records: Vec<Record>,
    stop: String,
    stop_bit: i64,
}

#[test]
fn captured_device_records_match_upstream_without_inventing_the_successor() {
    let data = include_bytes!("../../../fixtures/device-record-v41/packet.bin");
    let fixture: Fixture = serde_json::from_str(include_str!(
        "../../../fixtures/device-record-v41/reference.json"
    ))
    .unwrap();
    assert_eq!(fixture.commit, "18067987ebc3628ad499c819c6aee814692df43b");
    let registry = component_registry(
        &fixture
            .registry
            .into_iter()
            .map(|r| (r.archetype, r.rows))
            .collect::<Vec<_>>(),
    );
    let level = crate::theater::film::catalog::maps::levels::resolve_recorded(
        fixture.build_id,
        &fixture.branch,
        fixture.level_tag,
    )
    .unwrap();
    let mut config = V41DecodeConfig::default();
    config.profile.movement.world_object = level_precision(level, 16).unwrap();
    assert_eq!(
        config.profile.movement.world_object.axis_bits,
        fixture.axis_bits
    );
    assert_eq!(
        config.profile.movement.world_object.index_bits,
        fixture.index_bits
    );
    let mut world = ReplicationDecodeState {
        keyframe_namespace: Some(fixture.initial_context.namespace),
        slots: fixture
            .initial_context
            .slots
            .into_iter()
            .map(|(index, s)| {
                (
                    index,
                    DecodeBinding {
                        archetype: s.archetype,
                        full_id: s.full_id,
                        soft: s.soft,
                        generation_any: s.generation_any,
                        view: Some(s.view),
                    },
                )
            })
            .collect(),
        ..Default::default()
    };
    let frame = decode_production_frame_contextual(
        data,
        2,
        &registry,
        &config.record_layout(),
        &mut world,
        ProductionReaderContext {
            reader: Some(&config.profile),
            schema_recovery: None,
            preamble_bits: 2,
            control_record_limit: config.control_record_limit,
            explicit_delta_schema: None,
            reference_controls_after_rejection: false,
        },
        |_, _, _| {},
    )
    .unwrap();
    assert_eq!(fixture.records.len(), 90);
    assert_eq!(
        frame.records.len(),
        fixture.records.len(),
        "{:?}",
        frame.entity_end
    );
    for (actual, expected) in frame.records.iter().zip(&fixture.records) {
        assert_eq!(actual.header.start_bit, expected.start);
        assert_eq!(actual.header.kind, RecordKind::New);
        assert_eq!(
            actual.header.id,
            Some(expected.index | (expected.generation << 30))
        );
        assert_eq!(actual.archetype, Some(expected.archetype));
        assert_eq!(actual.end_bit, expected.end, "record {}", expected.start);
        assert_eq!(actual.stop, EntityViewStop::Complete);
        assert_eq!(actual.components.len(), expected.components.len());
        for (component, traced) in actual.components.iter().zip(&expected.components) {
            assert_eq!(component.index, traced.index);
            assert_eq!(
                (component.start_bit, component.end_bit),
                (traced.start, traced.end)
            );
            assert_eq!(component.status, ComponentReadStatus::Complete);
            // Local fields can split an upstream skip into words; each upstream
            // primitive must still begin and end on a retained field boundary.
            for &(start, width) in &traced.trace {
                assert!(
                    component.fields.iter().any(|f| f.bit == start),
                    "{} primitive at {start}",
                    component.name
                );
                assert!(
                    component
                        .fields
                        .iter()
                        .any(|f| f.bit + f.width == start + width),
                    "{} primitive end {}",
                    component.name,
                    start + width
                );
            }
        }
        for field in actual
            .fields
            .iter()
            .chain(actual.default_state.iter().flat_map(|d| &d.fields))
            .chain(actual.components.iter().flat_map(|c| &c.fields))
        {
            assert_eq!(
                Some(field.raw.clone()),
                RawBits::from_source(data, field.bit, field.width)
            );
        }
    }
    assert_eq!(fixture.stop, "UnknownEntity");
    assert_eq!(fixture.stop_bit, 64496);
    let Some(ProductionEntityEnd::Rejected { header, reason }) = frame.entity_end else {
        panic!("expected unresolved successor, got {:?}", frame.entity_end);
    };
    assert_eq!(header.start_bit, fixture.stop_bit);
    assert_eq!(header.kind, RecordKind::Delta);
    assert_eq!(
        reason,
        crate::theater::parser::v41::chunks::replication::source::DeltaRejectionReason::Unbound
    );
    // Canonical parsing retains the known header before refusing the body.
    assert_eq!(frame.end_bit, header.end_bit);
    assert!(frame.controls.is_none());
    assert!(world.schema_recoveries.is_empty());
}
