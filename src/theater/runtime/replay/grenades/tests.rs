use super::*;

fn source() -> PoseSource {
    PoseSource {
        chunk_index: Some(1),
        payload_byte: 0,
        start_bit: 0,
        end_bit: 35,
    }
}
fn fields(values: &[u8]) -> Vec<ComponentField> {
    let mut fields = vec![ComponentField {
        name: "count".into(),
        bit: 0,
        width: 3,
        raw: RawBits::from_low(values.len() as u64, 3),
    }];
    fields.extend(values.iter().enumerate().map(|(i, v)| ComponentField {
        name: format!("grenades[{i}]"),
        bit: 3 + 8 * i,
        width: 8,
        raw: RawBits::from_low(u64::from(*v), 8),
    }));
    fields
}
fn obs(time: u64, entity: u32, values: &[u8]) -> GrenadeObservation {
    GrenadeObservation {
        timestamp_us: time,
        entity_id: entity,
        source: source(),
        raw_counts: counts(&fields(values)),
        counts: counts(&fields(values)),
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

#[test]
fn four_ranks_preserve_zero_and_full_byte_counts() {
    assert_eq!(
        counts(&fields(&[0, 2, 4, 255])),
        Some(GrenadeCounts {
            frag: 0,
            plasma: 2,
            dynamo: 4,
            spike: 255
        })
    );
    assert_eq!(counts(&fields(&[0; 4])).unwrap().spike, 0);
    for values in [vec![], vec![1; 3], vec![1; 5], vec![1; 7]] {
        assert!(counts(&fields(&values)).is_none());
    }
    let mut malformed = fields(&[1; 4]);
    malformed[4].name = "grenades[2]".into();
    assert!(counts(&malformed).is_none());
    let mut truncated = fields(&[1; 4]);
    truncated[4].raw.bytes.clear();
    assert!(counts(&truncated).is_none());
}

#[test]
fn snapshots_gaps_and_lives_remain_causal() {
    let lives = [life(100, 400, 1), life(400, 800, 0x40000001)];
    let mut obs = vec![
        obs(100, 1, &[2, 0, 0, 0]),
        obs(150, 1, &[1, 0, 0, 0]),
        obs(250, 1, &[0; 4]),
        obs(300, 1, &[1; 3]), // Unsupported shape clears, rather than retaining zero.
        obs(400, 1, &[2; 4]), // Old generation at death must be filtered.
        obs(500, 0x40000001, &[0, 2, 1, 0]),
    ];
    let samples = timeline(&mut obs, vec![(200, 2)], &lives);
    let at = |time| {
        samples
            .iter()
            .rev()
            .find(|s| s.timestamp_us <= time)
            .and_then(|s| s.counts)
    };
    assert!(at(99).is_none());
    assert_eq!(at(149).unwrap().frag, 2);
    assert_eq!(at(150).unwrap().frag, 1);
    assert!(at(200).is_none());
    assert_eq!(at(250).unwrap().frag, 0);
    assert!(at(300).is_none());
    assert!(at(400).is_none());
    assert_eq!(at(500).unwrap().dynamo, 1);
    assert_eq!(at(799).unwrap().plasma, 2);
    assert!(at(800).is_none());
    assert!(!obs.iter().any(|o| o.timestamp_us == 400));
}

#[test]
fn same_time_packet_order_and_respawn_clear_prior_counts() {
    let lives = [life(100, 200, 1), life(200, 400, 2)];
    let mut observations = vec![obs(100, 1, &[2; 4]), obs(200, 2, &[0; 4])];
    let samples = timeline(&mut observations, vec![(100, 1)], &lives);
    assert!(samples[0].counts.is_none()); // Gap after observation.
    assert_eq!(samples[1].counts.unwrap().frag, 0); // Respawn wins over death clear.
    let mut observations = vec![obs(100, 1, &[2; 4])];
    assert!(
        timeline(&mut observations, vec![(100, 0)], &lives)[0]
            .counts
            .is_some()
    );
}

#[test]
fn admission_limit_rejects_whole_snapshot_without_clamping_and_supports_custom_modes() {
    let raw = counts(&fields(&[16, 8, 4, 0]));
    assert!(admitted_counts(raw, 2).is_none());
    assert_eq!(raw.unwrap().frag, 16);
    let raw = counts(&fields(&[4, 0, 0, 0]));
    assert!(admitted_counts(raw, 2).is_none());
    assert_eq!(admitted_counts(raw, 4).unwrap().frag, 4);
    assert!(admitted_counts(counts(&fields(&[0; 4])), 0).is_some());
}

#[test]
fn captured_nonzero_and_rejected_count_components_preserve_source_bits() {
    use crate::theater::parser::v41::chunks::replication::components::{
        cursor::ComponentCursor, dispatch, reader::ComponentReader,
    };
    let metadata: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/grenade-counts-v41/evidence.json"
    ))
    .unwrap();
    for (bytes, item) in [
        &include_bytes!("../../../fixtures/grenade-counts-v41/frag-plasma.bin")[..],
        &include_bytes!("../../../fixtures/grenade-counts-v41/outlier-16.bin")[..],
        &include_bytes!("../../../fixtures/grenade-counts-v41/outlier-144.bin")[..],
    ]
    .into_iter()
    .zip(metadata.as_array().unwrap())
    {
        let start = item["start_bit"].as_u64().unwrap() as usize;
        let read = |bytes, start| {
            let mut reader = ComponentReader {
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
            let result = dispatch::component(&mut reader, "unit-grenade-counts-component", 0, 35);
            (result, reader.cursor.position, reader.fields)
        };
        let (result, end, fields) = read(bytes, start);
        assert_eq!(result, Some(true));
        assert_eq!(end as usize, start + 35);
        let raw = counts(&fields);
        assert_eq!(serde_json::to_value(raw).unwrap(), item["counts"]);
        assert_eq!(
            admitted_counts(raw, 2).is_some(),
            item["name"] == "frag-plasma"
        );
        assert_eq!(read(&bytes[..bytes.len() - 1], start).0, None);
    }
}
