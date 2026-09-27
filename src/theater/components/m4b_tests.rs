use super::*;
use std::io::Read;

#[test]
fn native_m4b_death_and_mobility_tails() {
    #[derive(Deserialize)]
    struct Case {
        name: String,
        archetype: u32,
        input: String,
        start: usize,
        end: i64,
        next: u64,
        ported: bool,
        encoding: PositionEncoding,
        dead: Option<crate::theater::ObjectDeadState>,
        actions: Vec<[bool; 2]>,
        indices: Option<std::collections::BTreeMap<i32, u64>>,
    }
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        include_bytes!("../fixtures/m4b-tails-d61443e-v41.json.zlib").as_slice(),
    )
    .read_to_end(&mut raw)
    .unwrap();
    let cases: Vec<Case> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(cases.len(), 1536);
    let mut velocity_cases = [0; 2];
    for (index, case) in cases.into_iter().enumerate() {
        let data: Vec<_> = (0..case.input.len())
            .step_by(2)
            .map(|at| u8::from_str_radix(&case.input[at..at + 2], 16).unwrap())
            .collect();
        let (status, read) = decode_native_component(
            &data,
            case.start,
            &case.name,
            1,
            case.archetype,
            Some(&case.encoding),
        );
        assert_eq!(status, Some(case.ported), "{index}: {}", case.name);
        assert_eq!(read.end_bit, case.end, "{index}: {}", case.name);
        let bits = crate::theater::bits::Bits(&data);
        assert_eq!(bits.read(case.end as usize, 8), Some(case.next));
        let mut end = case.start as i64;
        for field in &read.fields {
            assert_eq!(field.bit, end, "{index}: {}", field.name);
            assert_eq!(
                bits.read(field.bit as usize, field.width as usize),
                Some(field.raw)
            );
            end += field.width as i64;
        }
        assert_eq!(end, read.end_bit);
        assert_eq!(read.diagnostics.mobility_actions, case.actions);
        assert_eq!(
            read.diagnostics.absolute_indices,
            case.indices.unwrap_or_default()
        );
        if let Some(dead) = case.dead {
            assert_eq!(
                crate::theater::ObjectDeadState::from_fields(&read.fields),
                Some(dead)
            );
            if let Some(flags) = read.fields.iter().find(|f| f.name == "byte") {
                let has_velocity = flags.raw & 0x10 != 0;
                velocity_cases[usize::from(has_velocity)] += 1;
                assert_eq!(
                    read.fields
                        .iter()
                        .filter(|f| f.name.starts_with("velocity_axis["))
                        .count(),
                    if has_velocity { 3 } else { 0 }
                );
                assert!(read.fields.iter().all(|f| f.name != "value_40a"));
            }
        }
        let json = serde_json::to_vec(&read).unwrap();
        assert_eq!(
            serde_json::from_slice::<DecodedComponent>(&json).unwrap(),
            read
        );
    }
    assert!(velocity_cases.iter().all(|&n| n > 0));
}

#[test]
fn native_m4b_component_oracle() {
    #[derive(Deserialize)]
    struct Case {
        name: String,
        input: String,
        start: usize,
        end: i64,
        ported: bool,
        next: u64,
    }
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        include_bytes!("../fixtures/m4b-components-d61443e-v41.json.zlib").as_slice(),
    )
    .read_to_end(&mut raw)
    .unwrap();
    let cases: Vec<Case> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(cases.len(), 3584);
    for (index, case) in cases.into_iter().enumerate() {
        let data: Vec<_> = (0..case.input.len())
            .step_by(2)
            .map(|at| u8::from_str_radix(&case.input[at..at + 2], 16).unwrap())
            .collect();
        let (status, read) = decode_native_component(&data, case.start, &case.name, 1, 0, None);
        assert_eq!(status, Some(case.ported), "{index}: {}", case.name);
        assert_eq!(read.end_bit, case.end, "{index}: {}", case.name);
        let bits = crate::theater::bits::Bits(&data);
        assert_eq!(bits.read(case.end as usize, 8), Some(case.next));
        let mut end = case.start as i64;
        for field in &read.fields {
            assert_eq!(field.bit, end, "{index}: {}", field.name);
            assert_eq!(
                bits.read(field.bit as usize, field.width as usize),
                Some(field.raw)
            );
            end += field.width as i64;
        }
        assert_eq!(end, read.end_bit, "all consumed bits must be retained");
        assert_eq!(
            read.diagnostics.vehicle_type_physics_assumed,
            u64::from(case.name == "vehicle-type-physics-component"),
        );
        let json = serde_json::to_vec(&read).unwrap();
        assert_eq!(
            serde_json::from_slice::<DecodedComponent>(&json).unwrap(),
            read
        );
    }
}

#[test]
fn native_m4b_runtime_assumption_retention() {
    let (_, read) = decode_native_component(
        &[0xff; 64],
        0,
        "vehicle-type-physics-component",
        1,
        40,
        None,
    );
    let mut diagnostics = crate::theater::FilmReadDiagnostics::default();
    assert!(diagnostics.is_empty());
    diagnostics.merge(&read.diagnostics);
    diagnostics.merge(&read.diagnostics);
    assert_eq!(diagnostics.vehicle_type_physics_assumed, 2);
    assert!(!diagnostics.is_empty());
    let old: crate::theater::FilmReadDiagnostics = serde_json::from_str("{}").unwrap();
    assert_eq!(old.vehicle_type_physics_assumed, 0);
    assert!(
        serde_json::to_value(old)
            .unwrap()
            .get("vehicle_type_physics_assumed")
            .is_none()
    );
}
