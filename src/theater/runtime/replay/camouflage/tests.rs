use super::*;
fn source() -> PoseSource {
    PoseSource {
        chunk_index: Some(1),
        payload_byte: 0,
        start_bit: 0,
        end_bit: 0,
    }
}
fn read(bytes: &[u8], start: usize, name: &str) -> (Option<bool>, i64, Vec<ComponentField>) {
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
    let result = dispatch::component(&mut r, name, 0, 35);
    (result, r.cursor.position, r.fields)
}
fn record(name: &str, fields: Vec<ComponentField>, start: i64, end: i64) -> EntityRecord {
    EntityRecord {
        header: RecordHeader {
            prefix: None,
            kind: RecordKind::Delta,
            id: Some(1),
            start_bit: 0,
            end_bit: start,
        },
        archetype: Some(35),
        mask: None,
        fields: vec![],
        default_state: None,
        components: vec![EntityComponentRead {
            index: 4,
            name: name.into(),
            start_bit: start,
            end_bit: end,
            status: ComponentReadStatus::Complete,
            fields,
            weapon_id: None,
        }],
        end_bit: end,
        stop: EntityViewStop::Complete,
    }
}

#[test]
fn component_channel_is_optional_and_only_binary_states_are_interpreted() {
    for q in [None, Some(0_u16), Some(4095), Some(2048)] {
        // state=0, flag0=1 (no main fraction), six independently gated subchannels.
        let mut bits = vec![false, false, false, true];
        for i in 0..6 {
            bits.push(i == 1 && q.is_some());
            if i == 1
                && let Some(q) = q
            {
                for bit in (0..12).rev() {
                    bits.push(q & (1 << bit) != 0);
                }
            }
        }
        let mut bytes = vec![0u8; bits.len().div_ceil(8)];
        for (i, bit) in bits.iter().enumerate() {
            if *bit {
                bytes[i / 8] |= 1 << (7 - i % 8);
            }
        }
        let name = "unit-active-camo-state-component";
        let (result, end, fields) = read(&bytes, 0, name);
        assert_eq!(result, Some(true));
        assert_eq!(end as usize, bits.len());
        let mut r = record(name, fields, 0, end);
        let o = observe(&r, 100, source());
        assert_eq!(o.as_ref().map(|o| o.raw_q), q);
        assert_eq!(o.and_then(|o| o.active), q.and_then(active));
        r.stop = EntityViewStop::Truncated;
        assert!(observe(&r, 100, source()).is_none());
    }
}

#[test]
fn timeline_resets_on_gaps_unknown_values_and_entity_life_changes() {
    let lives = [
        PlayerLife {
            start_us: 10,
            end_us: 100,
            entity_id: 1,
            source: source(),
            end_reason: LifeEnd::Delete,
        },
        PlayerLife {
            start_us: 100,
            end_us: 200,
            entity_id: 2,
            source: source(),
            end_reason: LifeEnd::Delete,
        },
    ];
    let mut obs: Vec<_> = [
        (20, 1, 4095),
        (30, 1, 0),
        (40, 1, 4095),
        (60, 1, 2048),
        (70, 1, 4095),
        (110, 1, 4095),
        (120, 2, 0),
    ]
    .into_iter()
    .map(|(timestamp_us, entity_id, raw_q)| CamouflageObservation {
        timestamp_us,
        entity_id,
        raw_q,
        active: active(raw_q),
        source: source(),
    })
    .collect();
    let samples = timeline(&mut obs, vec![(50, 3), (70, 4)], &lives);
    let at = |time| {
        samples
            .iter()
            .rev()
            .find(|s| s.timestamp_us <= time)
            .and_then(|s| s.active)
    };
    assert_eq!(at(19), None);
    assert_eq!(at(29), Some(true));
    assert_eq!(at(39), Some(false));
    assert_eq!(at(49), Some(true));
    assert_eq!(at(59), None);
    assert_eq!(at(69), None);
    assert_eq!(at(99), Some(true));
    assert_eq!(at(119), None);
    assert_eq!(at(199), Some(false));
    assert_eq!(at(200), None);
    assert_eq!(obs.len(), 6);
}
