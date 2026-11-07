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
        end_reason: LifeEnd::FilmEnd,
    }
}
#[test]
fn expiry_gap_and_same_value_respawn_never_leak_state() {
    let mut b = Builder {
        lives: vec![life(100, 200, 1), life(200, 500_000, 2)],
        ..Default::default()
    };
    let p = Some([1., 2., 3.]);
    b.updates.extend([
        Update {
            time: 100,
            channel: 0,
            value: p,
        },
        Update {
            time: 200,
            channel: 0,
            value: p,
        },
        Update {
            time: 200,
            channel: 1,
            value: Some([1., 0., 0.]),
        },
        Update {
            time: 250,
            channel: 0,
            value: p,
        },
    ]);
    let track = b.finish(0);
    assert!(track.at(99).is_none());
    assert_eq!(track.at(200).unwrap().timestamp_us, 200);
    assert_eq!(track.at(200).unwrap().position, p);
    assert!(track.at(200).unwrap().aim.is_some());
    assert_eq!(track.at(100_201).unwrap().position, p); // older expiry cannot clear newer data
    assert!(track.at(100_201).unwrap().aim.is_none());
    assert_eq!(track.at(100_250).unwrap().position, p);
    assert!(track.at(100_251).unwrap().position.is_none());
    assert!(track.at(500_000).unwrap().position.is_none());
}
#[test]
fn death_filters_observations_and_clears_both_channels() {
    let mut b = Builder {
        lives: vec![life(100, 200, 1)],
        ..Default::default()
    };
    for time in [100, 199, 200, 201] {
        b.updates.push(Update {
            time,
            channel: 0,
            value: Some([time as f64, 0., 0.]),
        });
    }
    let track = b.finish(0);
    assert_eq!(track.at(199).unwrap().position.unwrap()[0], 199.);
    assert!(track.at(200).unwrap().position.is_none());
    assert!(track.at(201).unwrap().position.is_none());
}
#[test]
fn desired_aim_controlled_heading_convention() {
    assert_eq!(desired_direction(0, 1024), [1., 0., 0.]);
    assert!((desired_direction(1024, 1024)[1] - 1.).abs() < 1e-12);
    assert!((desired_direction(0, 1280)[2] - std::f64::consts::FRAC_1_SQRT_2).abs() < 1e-12);
    assert!(desired_direction(4, 1336)[2] > 0.); // captured pitch-up packet
}
#[test]
fn roster_rejects_missing_count_excess_entries_and_unknown_tail() {
    assert!(roster(&[], 1852 * 8).is_none());
    assert!(roster(&[0, 0, 0, 33], 1852 * 8).is_none());
    assert!(roster(&[0, 0, 0, 1], 1852 * 8).is_none());
    assert!(roster(&[0, 0, 0, 0, 0], 1852 * 8).is_none());
    assert_eq!(roster(&[0; 4], 1852 * 8).unwrap().len(), 0);
}

#[test]
fn captured_event_vectors_preserve_the_spawn_boundary() {
    use crate::theater::parser::v41::chunks::replication::replication_stream::frame::events::{
        read_contextual_event_list, read_reference_event_list,
    };
    use crate::theater::parser::v41::context::movement::PrecisionDescriptor;
    let payload = include_bytes!("../../fixtures/oddball-event-vector-v41.bin");
    let encoding = PrecisionDescriptor {
        index_bits: 1,
        axis_bits: [18, 18, 15],
        region: 0,
    };
    let without = read_reference_event_list(payload, 1, Some(true), payload.len());
    assert_eq!(without.stop, EventListStop::UnsupportedBody);
    assert_eq!(without.end_bit, 304);
    let events = read_contextual_event_list(
        payload,
        1,
        Some(true),
        payload.len(),
        Some(&encoding),
        None,
        None,
    );
    assert_eq!(events.stop, EventListStop::Terminator);
    assert_eq!(events.end_bit, 1999);
    assert_eq!(
        events
            .records
            .iter()
            .map(|e| e.code.unwrap())
            .collect::<Vec<_>>(),
        [82, 15, 82, 82, 82, 82, 82, 82, 82, 82, 15, 15]
    );
    let vector: Vec<_> = events.records[2]
        .fields
        .iter()
        .filter(|f| (304..357).contains(&f.bit))
        .map(|f| (f.bit, f.width, f.value.clone()))
        .collect();
    assert_eq!(
        vector,
        vec![
            (304, 1, EventFieldValue::Scalar(0)),
            (305, 1, EventFieldValue::Scalar(0)),
            (306, 18, EventFieldValue::Scalar(97_243)),
            (324, 18, EventFieldValue::Scalar(129_662)),
            (342, 15, EventFieldValue::Scalar(16_630)),
        ]
    );
    // No byte-truncated prefix can claim the complete event list or seek ahead
    // to the biped NEW. The captured list terminator is bit 1998.
    for end in 0..250 {
        let truncated = read_contextual_event_list(
            &payload[..end],
            1,
            Some(true),
            payload.len(),
            Some(&encoding),
            None,
            None,
        );
        assert_ne!(truncated.stop, EventListStop::Terminator, "prefix {end}");
        assert!(truncated.records.iter().all(|r| r.end_bit <= end * 8));
    }
    // Unmapped/default regions must stop before vector words, even though the
    // correct-looking downstream NEW is still present in the packet.
    for bit in [304, 305] {
        let mut changed = payload.to_vec();
        changed[bit / 8] |= 1 << (7 - bit % 8);
        let refused = read_contextual_event_list(
            &changed,
            1,
            Some(true),
            changed.len(),
            Some(&encoding),
            None,
            None,
        );
        assert_eq!(refused.stop, EventListStop::UnsupportedBody);
        assert!(refused.end_bit <= 306);
    }
}

#[test]
fn firing_requires_a_bounded_header_and_matching_player_generation() {
    use crate::theater::runtime::interpretation::fire_events::read_fire_event;
    for narrow in [false, true] {
        let mut bits = Vec::new();
        let mut put = |value: u64, width: usize| {
            bits.extend((0..width).rev().map(|i| ((value >> i) & 1) as u8));
        };
        put(1, 1);
        put(1, 1);
        put(36, 7);
        put(1, 1);
        put(u64::from(narrow), 1);
        put(if narrow { 17 } else { 529 }, if narrow { 9 } else { 13 });
        put(2, 2); // generation
        put(0, 1);
        put(0, 1); // other references absent
        put(1, 1);
        put(0, 1); // short fire header; no optional block
        put(1, 7);
        put(0, 1); // fire sequence
        put(0, 1);
        put(3, 5); // shooter index 3
        put(1, 1);
        put(0, 1); // field E absent, upper weapon absent
        put(0x12345678, 32);
        put(0, 2);
        let mut data = vec![0; bits.len().div_ceil(8)];
        for (i, bit) in bits.iter().enumerate() {
            data[i / 8] |= bit << (7 - i % 8);
        }
        let read = read_fire_event(&data);
        let id = 529 | (2 << 30);
        let mut bindings = BTreeMap::from([(id, 3)]);
        assert_eq!(shot_owner(&read, &bindings), Some((3, id, bits.len())));
        bindings.insert(id, 4);
        assert!(shot_owner(&read, &bindings).is_none());
        bindings = BTreeMap::from([(529 | (1 << 30), 3)]);
        assert!(shot_owner(&read, &bindings).is_none());
        bindings = BTreeMap::from([(id, 3)]);
        for end in 0..data.len() {
            assert!(shot_owner(&read_fire_event(&data[..end]), &bindings).is_none());
        }
    }
}

#[test]
fn shot_lifetimes_reject_death_and_reused_generations() {
    let mut b = Builder {
        lives: vec![life(100, 200, 1), life(200, 300, 2)],
        ..Default::default()
    };
    for (time, id) in [
        (99, 1),
        (100, 1),
        (199, 1),
        (200, 1),
        (201, 1),
        (200, 2),
        (300, 2),
    ] {
        b.shots.push(ShotObservation {
            timestamp_us: time,
            entity_id: id,
            source: source(),
        });
    }
    assert_eq!(
        b.finish(0)
            .shots
            .iter()
            .map(|s| (s.timestamp_us, s.entity_id))
            .collect::<Vec<_>>(),
        [(100, 1), (199, 1), (200, 2)]
    );
}

#[test]
fn captured_fire_events_reach_following_spawns() {
    use crate::theater::parser::v41::chunks::replication::replication_stream::frame::events::read_contextual_event_list;
    use crate::theater::parser::v41::context::movement::PrecisionDescriptor;
    let encoding = PrecisionDescriptor {
        index_bits: 1,
        axis_bits: [18, 18, 15],
        region: 0,
    };
    for (payload, expected_end, expected_codes) in [
        (
            include_bytes!("../../fixtures/oddball-fire-v41/230612473.bin").as_slice(),
            538,
            &[36, 0, 82][..],
        ),
        (
            include_bytes!("../../fixtures/oddball-fire-v41/359342177.bin").as_slice(),
            274,
            &[36, 82][..],
        ),
        (
            include_bytes!("../../fixtures/oddball-fire-v41/532868633.bin").as_slice(),
            597,
            &[36, 0, 9, 82][..],
        ),
        (
            include_bytes!("../../fixtures/oddball-fire-v41/763286544.bin").as_slice(),
            329,
            &[36, 82][..],
        ),
        (
            include_bytes!("../../fixtures/oddball-fire-v41/929442282.bin").as_slice(),
            538,
            &[36, 0, 82][..],
        ),
        (
            include_bytes!("../../fixtures/oddball-fire-v41/1070150702.bin").as_slice(),
            318,
            &[36, 82][..],
        ),
        (
            include_bytes!("../../fixtures/oddball-fire-v41/1085031941.bin").as_slice(),
            327,
            &[36, 82][..],
        ),
    ] {
        let events = read_contextual_event_list(
            payload,
            1,
            Some(true),
            payload.len(),
            Some(&encoding),
            Some(&encoding),
            None,
        );
        assert_eq!(events.stop, EventListStop::Terminator);
        assert_eq!(events.end_bit, expected_end);
        assert_eq!(
            events
                .records
                .iter()
                .map(|e| e.code.unwrap())
                .collect::<Vec<_>>(),
            expected_codes
        );
        // Rebase only for the standalone projection test, not for production
        // traversal. Both readers must agree on each captured damage endpoint.
        for damage in events.records.iter().filter(|e| e.code == Some(0)) {
            let width = damage.end_bit - damage.start_bit;
            let mut standalone = vec![0u8; (width + 1).div_ceil(8)];
            standalone[0] = 0x80; // configuration
            for bit in 0..width {
                let source = damage.start_bit + bit;
                let dest = 1 + bit;
                standalone[dest / 8] |=
                    ((payload[source / 8] >> (7 - source % 8)) & 1) << (7 - dest % 8);
            }
            let projected =
                crate::theater::runtime::interpretation::weapon_damage::read_weapon_damage(
                    &standalone,
                    0,
                );
            assert_eq!(projected.padding_bits, 0);
            assert_eq!(projected.end_bit, width + 1);
        }
        // Every byte-truncated event-list prefix must stop before any spawn.
        for end in 0..expected_end.div_ceil(8) {
            let truncated = read_contextual_event_list(
                &payload[..end],
                1,
                Some(true),
                payload.len(),
                Some(&encoding),
                Some(&encoding),
                None,
            );
            assert_ne!(truncated.stop, EventListStop::Terminator, "prefix {end}");
            assert!(truncated.records.iter().all(|e| e.end_bit <= end * 8));
        }
        let disabled = read_contextual_event_list(
            payload,
            1,
            Some(true),
            payload.len(),
            Some(&encoding),
            None,
            None,
        );
        assert_eq!(disabled.stop, EventListStop::UnsupportedBody);
        assert_eq!(disabled.end_bit, 24);
        let mut changed = payload.to_vec();
        changed[0] &= 0x7f;
        let refused = read_contextual_event_list(
            &changed,
            1,
            Some(true),
            payload.len(),
            Some(&encoding),
            Some(&encoding),
            None,
        );
        assert_eq!(refused.stop, EventListStop::UnsupportedBody);
        assert_eq!(refused.end_bit, 24);
    }
}
