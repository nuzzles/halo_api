use super::*;

fn source() -> PoseSource {
    PoseSource {
        chunk_index: Some(1),
        payload_byte: 0,
        start_bit: 0,
        end_bit: 10,
    }
}
fn life(start_us: u64, end_us: u64, entity_id: u32) -> PlayerLife {
    PlayerLife {
        start_us,
        end_us,
        entity_id,
        source: source(),
        end_reason: LifeEnd::SummaryDeath,
    }
}
fn obs(
    timestamp_us: u64,
    entity_id: u32,
    selection: Option<EquipmentSelection>,
) -> EquipmentObservation {
    EquipmentObservation {
        timestamp_us,
        entity_id,
        source: source(),
        counter: Some(5),
        channel: EquipmentChannel::Selection,
        objects: None,
        selection,
        fields: vec![],
    }
}

#[test]
fn equipment_changes_gaps_and_lives_are_causal() {
    use EquipmentSelection::*;
    let lives = [life(100, 400, 1), life(400, 800, 0x40000001)];
    let mut observations = vec![
        obs(150, 1, Some(Rank(8))),
        obs(250, 1, Some(Empty)),
        obs(300, 1, Some(Rank(9))),
        obs(400, 1, Some(Rank(8))),          // old generation at respawn
        obs(500, 0x40000001, Some(Rank(0))), // zero is not empty
        obs(600, 0x40000001, None),          // malformed update invalidates selection
    ];
    let samples = timeline(&mut observations, vec![(200, 1)], &lives);
    let at = |t| samples.iter().rev().find(|s| s.timestamp_us <= t).cloned();
    assert!(at(99).is_none());
    assert_eq!(at(149).unwrap().selection, None);
    assert_eq!(at(150).unwrap().selection, Some(Rank(8)));
    assert_eq!(at(199).unwrap().selection, Some(Rank(8)));
    assert_eq!(at(200).unwrap().selection, None);
    assert_eq!(at(250).unwrap().selection, Some(Empty));
    assert_eq!(at(399).unwrap().selection, Some(Rank(9)));
    assert_eq!(at(400).unwrap().selection, None);
    assert_eq!(at(500).unwrap().selection, Some(Rank(0)));
    assert_eq!(at(600).unwrap().selection, None);
    assert_eq!(at(800).unwrap().selection, None);
    assert!(!observations.iter().any(|o| o.timestamp_us == 400));
}

#[test]
fn coincident_gaps_preserve_packet_order_and_raw_rank() {
    let lives = [life(100, 400, 1)];
    for (preceding, expected) in [(0, Some(EquipmentSelection::Rank(20))), (1, None)] {
        let mut observations = vec![obs(100, 1, Some(EquipmentSelection::Rank(20)))];
        let samples = timeline(&mut observations, vec![(100, preceding)], &lives);
        assert_eq!(samples[0].selection, expected);
    }
    let mut observations = vec![obs(100, 1, Some(EquipmentSelection::Rank(20)))];
    assert_eq!(
        timeline(&mut observations, vec![], &lives)[0].selection,
        Some(EquipmentSelection::Rank(20))
    );
}

fn read_named(bytes: &[u8], start: usize, name: &str) -> (Option<bool>, i64, Vec<ComponentField>) {
    use crate::theater::parser::v41::chunks::replication::components::{
        cursor::ComponentCursor, dispatch, reader::ComponentReader,
    };
    let mut r = ComponentReader {
        reference_widths: None,
        width_error: None,
        live_grammar: None,
        movement_slot: None,
        references: vec![],
        diagnostics: Default::default(),
        cursor: ComponentCursor::new(bytes, start).unwrap(),
        fields: vec![],
        position_encoding: None,
    };
    let result = if name == "defaults" {
        crate::theater::parser::v41::chunks::replication::components::defaults::state(
            &mut r,
            37,
            [9, 5],
        )
    } else {
        dispatch::component(&mut r, name, 0, 35)
    };
    (result, r.cursor.position, r.fields)
}

#[test]
fn selection_gate_full_six_bit_rank_and_truncation() {
    // Counter 5, closed gate, rank 9. Complete ten-bit body.
    let (status, end, fields) = read(&[0xa2, 0x40], 0);
    assert_eq!(status, Some(true));
    assert_eq!(end, 10);
    assert_eq!(value(&fields, "counter", 3), Some(5));
    assert_eq!(selection(&fields), Some(EquipmentSelection::Rank(9)));
    // Counter 6 and open gate consume four bits, explicitly clear the selection.
    let (status, end, fields) = read(&[0xd0], 0);
    assert_eq!((status, end), (Some(true), 4));
    assert_eq!(selection(&fields), Some(EquipmentSelection::Empty));
    let (status, _, fields) = read(&[0xa2], 0);
    assert_eq!(status, None);
    assert_eq!(selection(&fields), None);
    // All observed ranks stay intact without assigning equipment names.
    let (_, _, fields) = read(&[0xaf, 0xc0], 0);
    assert_eq!(selection(&fields), Some(EquipmentSelection::Rank(63)));
}

fn read(bytes: &[u8], start: usize) -> (Option<bool>, i64, Vec<ComponentField>) {
    read_named(bytes, start, "biped-desired-ability-set-component")
}

#[test]
fn captured_inventory_and_identity_are_source_backed() {
    let evidence: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/equipment-v41/evidence.json"
    ))
    .unwrap();
    let fixtures: &[&[u8]] = &[
        include_bytes!("../../../fixtures/equipment-v41/inventory-single.bin"),
        include_bytes!("../../../fixtures/equipment-v41/inventory-multiple.bin"),
        include_bytes!("../../../fixtures/equipment-v41/object-defaults.bin"),
    ];
    let mut reads = Vec::new();
    for (bytes, e) in fixtures.iter().zip(evidence.as_array().unwrap()) {
        let start = e["start_bit"].as_u64().unwrap() as usize;
        let name = if e["kind"] == "defaults" {
            "defaults"
        } else {
            "unit-equipment-component"
        };
        let (status, end, fields) = read_named(bytes, start, name);
        assert_eq!(status, Some(true));
        assert_eq!(end as u64, start as u64 + e["bits"].as_u64().unwrap());
        assert_eq!(fields.len(), e["fields"].as_array().unwrap().len());
        for (f, expected) in fields.iter().zip(e["fields"].as_array().unwrap()) {
            assert_eq!(f.name, expected["name"].as_str().unwrap());
            assert_eq!(f.width as u64, expected["width"].as_u64().unwrap());
            assert_eq!(serde_json::to_value(&f.raw).unwrap(), expected["raw"]);
        }
        reads.push(fields);
    }
    let id = 1073743267;
    let mut identities = ObjectIdentities::default();
    let first = inventory(&reads[0], &identities).unwrap();
    assert_eq!(first.len(), 1);
    assert_eq!(first[0].entity_id, id);
    assert_eq!(first[0].object_tag, None); // No future identity backfill.
    let mut r = EntityRecord {
        header: RecordHeader {
            prefix: None,
            kind: RecordKind::New,
            id: Some(id),
            start_bit: 0,
            end_bit: 0,
        },
        archetype: Some(37),
        mask: None,
        fields: vec![],
        default_state: Some(DefaultState {
            source: BitRange { start: 0, end: 92 },
            status: DefaultStateStatus::Complete,
            fields: reads[2].clone(),
        }),
        components: vec![],
        end_bit: 92,
        stop: EntityViewStop::Complete,
    };
    identities.observe(&r, source());
    let known = inventory(&reads[0], &identities).unwrap();
    assert_eq!(known[0].object_tag, Some(0xa53cd143));
    assert!(known[0].identity_source.is_some());
    assert_eq!(inventory(&reads[1], &identities).unwrap().len(), 2);
    r.header.kind = RecordKind::Delete;
    identities.observe(&r, source());
    assert_eq!(
        inventory(&reads[0], &identities).unwrap()[0].object_tag,
        None
    );
    r.header.kind = RecordKind::New;
    identities.observe(&r, source());
    r.header.id = Some(id ^ 0x40000000);
    r.header.kind = RecordKind::Delta;
    identities.observe(&r, source());
    assert_eq!(
        inventory(&reads[0], &identities).unwrap()[0].object_tag,
        None
    );
    r.header.id = Some(id);
    r.header.kind = RecordKind::New;
    identities.observe(&r, source());
    identities.clear();
    assert_eq!(
        inventory(&reads[0], &identities).unwrap()[0].object_tag,
        None
    );
    r.stop = EntityViewStop::Truncated;
    identities.observe(&r, source());
    assert_eq!(
        inventory(&reads[0], &identities).unwrap()[0].object_tag,
        None
    );
}

#[test]
fn inventory_empty_unknown_and_lifetime_resets_are_distinct() {
    let lives = [life(0, 100, 1), life(100, 200, 2)];
    let item = HeldEquipment {
        entity_id: 0x400005a3,
        object_tag: None,
        identity_source: None,
    };
    let mut observations = vec![
        obs(10, 1, None),
        obs(30, 1, None),
        obs(100, 1, None),
        obs(110, 2, None),
    ];
    for o in &mut observations {
        o.channel = EquipmentChannel::Inventory;
    }
    observations[0].objects = Some(vec![item]);
    observations[1].objects = Some(vec![]);
    observations[2].objects = Some(vec![item]); // dead owner's reading
    observations[3].objects = Some(vec![item]);
    let samples = timeline(&mut observations, vec![(20, 1), (110, 4)], &lives);
    let at = |t| samples.iter().rev().find(|s| s.timestamp_us <= t).unwrap();
    assert_eq!(at(10).objects, Some(vec![item]));
    assert_eq!(at(20).objects, None);
    assert_eq!(at(30).objects, Some(vec![]));
    assert_eq!(at(100).objects, None);
    assert_eq!(at(110).objects, None); // same-time later gap wins
    let (_, _, empty) = read_named(&[0], 0, "unit-equipment-component");
    assert_eq!(
        inventory(&empty, &ObjectIdentities::default()),
        Some(vec![])
    );
}
