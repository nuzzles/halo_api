use super::*;

fn source() -> PoseSource {
    PoseSource {
        chunk_index: Some(1),
        payload_byte: 0,
        start_bit: 0,
        end_bit: 11,
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
fn obs(time: u64, entity_id: u32, kind: VitalityKind, q: u8) -> VitalityObservation {
    let decoded = scalar(q, kind);
    VitalityObservation {
        timestamp_us: time,
        entity_id,
        source: source(),
        kind,
        fields: vec![],
        raw_q: Some(q),
        decoded,
        value: decoded.filter(|v| *v >= 0.0),
    }
}

#[test]
fn scalar_endpoints_midpoint_and_overfill() {
    use VitalityKind::*;
    assert_eq!(scalar(0, Health), Some(-1.0));
    assert_eq!(scalar(127, Health), Some(0.0));
    assert_eq!(scalar(254, Health), Some(1.0));
    assert_eq!(scalar(255, Health), None);
    assert_eq!(scalar(0, Shields), Some(0.0));
    assert_eq!(scalar(64, Shields), Some(1.0));
    assert_eq!(scalar(255, Shields), Some(4.0));
    assert!(scalar(65, Shields).unwrap() > 1.0);
    for q in 0..=255 {
        let sample = VitalitySample {
            timestamp_us: 0,
            health: None,
            shields: scalar(q, Shields),
        };
        assert_eq!(sample.is_overshield_active(), Some(q > 64));
    }
    assert_eq!(
        VitalitySample {
            timestamp_us: 0,
            health: None,
            shields: None
        }
        .is_overshield_active(),
        None
    );
    for kind in [Health, Shields] {
        let max_q = if kind == Health { 254 } else { 255 };
        for q in 0..max_q {
            assert!(scalar(q, kind).unwrap() < scalar(q + 1, kind).unwrap());
        }
    }
    // Captured normal endpoint was historically misread as the low seven bits (126).
    assert!(scalar(126, Health).unwrap() < 0.0);
    assert!(obs(0, 1, Health, 126).value.is_none());
    assert_eq!(obs(0, 1, Health, 127).value, Some(0.0));
}

#[test]
fn independent_updates_unknowns_gaps_and_generation_boundaries() {
    use VitalityKind::*;
    let lives = [life(100, 400, 1), life(400, 800, 0x40000001)];
    let mut observations = vec![
        obs(100, 1, Health, 254),
        obs(100, 1, Shields, 64),
        obs(150, 1, Shields, 32),
        obs(200, 1, Health, 127),
        obs(250, 1, Health, 255), // Invalid health clears only health.
        obs(300, 1, Shields, 0),  // After gap: no resurrected health.
        obs(400, 1, Health, 254), // Old generation after death ignored.
        obs(500, 0x40000001, Shields, 65),
        obs(600, 0x40000001, MaximumModifiers, 0),
    ];
    let samples = timeline(&mut observations, vec![(275, 5)], &lives);
    let at = |t| samples.iter().rev().find(|s| s.timestamp_us <= t).copied();
    assert!(at(99).is_none());
    assert_eq!(at(149).unwrap().health, Some(1.0));
    assert_eq!(at(150).unwrap().health, Some(1.0));
    assert!(at(150).unwrap().shields.unwrap() < 1.0);
    assert_eq!(at(200).unwrap().health, Some(0.0));
    assert_eq!(at(250).unwrap().health, None);
    assert_eq!(at(250).unwrap().shields, scalar(32, Shields));
    assert_eq!(at(275).unwrap().shields, None);
    assert_eq!(at(300).unwrap().shields, Some(0.0));
    assert_eq!(at(300).unwrap().health, None);
    assert_eq!(at(400).unwrap().shields, None);
    assert_eq!(at(799).unwrap().shields, scalar(65, Shields));
    assert_eq!(at(799).unwrap().health, None);
    assert_eq!(at(800).unwrap().shields, None);
    assert!(!observations.iter().any(|o| o.timestamp_us == 400));
}

#[test]
fn same_time_packet_order_clears_before_or_after_observation() {
    use VitalityKind::*;
    let lives = [life(100, 200, 1), life(200, 400, 2)];
    let mut observations = vec![
        obs(100, 1, Health, 254),
        obs(100, 1, Shields, 64),
        obs(200, 2, Shields, 0),
    ];
    let samples = timeline(&mut observations, vec![(100, 1)], &lives);
    assert_eq!(samples[0].health, None); // Gap falls between the two readings.
    assert_eq!(samples[0].shields, Some(1.0));
    assert_eq!(samples[1].health, None);
    assert_eq!(samples[1].shields, Some(0.0)); // New life wins over boundary clear.
    let mut observations = vec![obs(100, 1, Health, 254)];
    assert_eq!(
        timeline(&mut observations, vec![(100, 1)], &lives)[0].health,
        None
    );
    let mut observations = vec![obs(100, 1, Health, 254)];
    assert_eq!(
        timeline(&mut observations, vec![(100, 0)], &lives)[0].health,
        Some(1.0)
    );
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
fn captured_components_preserve_source_flags_and_scalar_values() {
    let metadata: serde_json::Value =
        serde_json::from_str(include_str!("../../../fixtures/vitality-v41/evidence.json")).unwrap();
    let fixtures: &[&[u8]] = &[
        include_bytes!("../../../fixtures/vitality-v41/body-damage.bin"),
        include_bytes!("../../../fixtures/vitality-v41/body-full.bin"),
        include_bytes!("../../../fixtures/vitality-v41/shield-damage.bin"),
        include_bytes!("../../../fixtures/vitality-v41/shield-empty.bin"),
        include_bytes!("../../../fixtures/vitality-v41/shield-full.bin"),
        include_bytes!("../../../fixtures/vitality-v41/shield-recovery.bin"),
    ];
    for (bytes, item) in fixtures.iter().zip(metadata.as_array().unwrap()) {
        let kind = if item["kind"] == "Health" {
            VitalityKind::Health
        } else {
            VitalityKind::Shields
        };
        let name = if kind == VitalityKind::Health {
            "object-body-vitality-component"
        } else {
            "object-shield-vitality-component"
        };
        let start = item["start_bit"].as_u64().unwrap() as usize;
        let (result, end, fields) = read(bytes, start, name);
        assert_eq!(result, Some(true));
        assert_eq!(
            end as usize,
            start + item["bits"].as_u64().unwrap() as usize
        );
        assert_eq!(
            quantum(&fields, kind),
            Some(item["q"].as_u64().unwrap() as u8)
        );
        assert_eq!(fields.len(), item["fields"].as_array().unwrap().len());
        for (actual, expected) in fields.iter().zip(item["fields"].as_array().unwrap()) {
            assert_eq!(actual.name, expected["name"]);
            assert_eq!(actual.width as u64, expected["width"].as_u64().unwrap());
            assert_eq!(serde_json::to_value(&actual.raw).unwrap(), expected["raw"]);
        }
        let mut r = record(name, fields, start as i64, end);
        let o = observe(&r, 100, source());
        assert_eq!(
            f64::from(o[0].value.unwrap()),
            item["decoded"].as_f64().unwrap()
        );
        assert_eq!(o[0].source.start_bit, start as i64);
        assert_eq!(o[0].source.end_bit, end);
        r.components[0].fields.pop();
        assert!(observe(&r, 100, source())[0].value.is_none());
        r.components[0].status = ComponentReadStatus::Unsupported;
        assert!(observe(&r, 100, source()).is_empty());
        assert_eq!(read(&bytes[..bytes.len() - 1], start, name).0, None);
    }
}

#[test]
fn optional_regeneration_shapes_are_read_without_changing_scale() {
    // Exercise all wire branches, explicitly synthetic (real fixture has no optional words).
    for regen in [false, true] {
        for a in [false, true] {
            for b in [false, true] {
                let mut bits = String::from("01000000"); // q64 = normal full.
                bits.push(if regen { '1' } else { '0' });
                if regen {
                    for (gate, word) in [(a, 123), (b, 2047)] {
                        bits.push(if gate { '1' } else { '0' });
                        if gate {
                            bits.push_str(&format!("{word:012b}"));
                        }
                    }
                }
                bits.push_str("0000000100101011"); // block64 = 299
                bits.push_str("1100");
                let length = bits.len();
                let mut bytes = vec![0u8; length.div_ceil(8)];
                for (i, bit) in bits.bytes().enumerate() {
                    bytes[i / 8] |= (bit - b'0') << (7 - i % 8);
                }
                let (result, end, fields) = read(&bytes, 0, "object-shield-vitality-component");
                assert_eq!(result, Some(true));
                assert_eq!(end as usize, length);
                assert_eq!(
                    length,
                    29 + if regen {
                        2 + 12 * (usize::from(a) + usize::from(b))
                    } else {
                        0
                    }
                );
                assert_eq!(quantum(&fields, VitalityKind::Shields), Some(64));
                assert_eq!(value(&fields, "block64", 16), Some(299));
                assert_eq!(
                    read(
                        &bytes[..bytes.len() - 1],
                        0,
                        "object-shield-vitality-component"
                    )
                    .0,
                    None
                );
            }
        }
    }
}
