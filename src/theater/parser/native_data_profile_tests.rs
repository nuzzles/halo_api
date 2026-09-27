//! Native-entry precision routing against independently read Go frames.
use super::*;
use serde_json::{Value, json};
use std::io::Read;

fn bytes(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
fn source(payload: &[u8]) -> FilmSource {
    let mut packet = vec![0; 4];
    packet.extend((payload.len() as u32).to_le_bytes());
    packet.extend(123u64.to_le_bytes());
    packet.extend(payload);
    FilmSource::load(
        &[[41u32.to_le_bytes(), 27u32.to_le_bytes()].concat(), packet],
        &[],
    )
    .unwrap()
}

#[test]
fn native_data_explicit_frame_profile_d61443e() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        include_bytes!("../fixtures/native-entry-profile-d61443e-v41.json.zlib").as_slice(),
    )
    .read_to_end(&mut raw)
    .unwrap();
    let cases: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(cases.len(), 768);
    let mut changed = 0;
    for (i, c) in cases.iter().enumerate() {
        let payload = bytes(c["hex"].as_str().unwrap());
        let source = source(&payload);
        let mut profile = NativeScanProfile::default();
        profile.movement.world_object.index_bits = c["index"].as_u64().unwrap();
        profile.movement.world_object.axis_bits =
            serde_json::from_value(c["axes"].clone()).unwrap();
        profile.movement.full_precision = c["full"].as_bool().unwrap();
        profile.grammar.baseline_scope = c["baseline"].as_bool().unwrap();
        let options = ParseOptions {
            frame_profile: Some(profile.clone()),
            recovery_policy: KeyframeRecoveryPolicy::SequentialOnly,
            ..Default::default()
        };
        assert_eq!(
            serde_json::from_slice::<ParseOptions>(&serde_json::to_vec(&options).unwrap()).unwrap(),
            options
        );
        let parsed = Film::parse_v41_with_options(&source, options).unwrap();
        assert_eq!(parsed.frame_config.profile, profile);
        let NativeFilmPacketBody::Frame(frame) = &parsed.chunks[1].packets[0].body else {
            panic!("frame {i}");
        };
        assert_eq!(json!(frame.views_completed), c["views"], "views {i}");
        assert_eq!(json!(frame.end_bit), c["end"], "end {i}");
        let verdict = frame.control_verdict(&payload);
        assert!(verdict.reached && verdict.closed, "closed {i}");
        let expected = c["verdict"]["Entrees"].as_array().unwrap();
        assert_eq!(verdict.entries.len(), expected.len());
        for (actual, expected) in verdict.entries.iter().zip(expected) {
            assert_eq!(json!(actual.index), expected["Index"]);
            let action = actual.action.as_ref().unwrap();
            assert_eq!(
                json!({"Present":action.present,"Gachettes":action.triggers,
                "Barillets":action.barrels,"Arme":action.weapons}),
                expected["Action"]
            );
        }
        let control = frame.controls.as_ref().unwrap();
        for field in &control.fields {
            assert_eq!(
                crate::theater::bits::Bits(&payload).read(field.bit, field.width),
                Some(field.raw),
                "source field {i}: {}",
                field.name
            );
        }
        let default =
            Film::parse_v41_with_recovery(&source, KeyframeRecoveryPolicy::SequentialOnly).unwrap();
        changed +=
            usize::from(default.chunks[1].packets[0].body != parsed.chunks[1].packets[0].body);
        assert_eq!(parsed.chunks[1].data, default.chunks[1].data);
        assert_eq!(
            serde_json::from_slice::<Film>(&serde_json::to_vec(&parsed).unwrap()).unwrap(),
            parsed
        );
    }
    assert!(changed > 0, "fixture must exercise width-dependent parsing");
}

#[test]
fn native_data_profile_compatibility_and_snapshot() {
    let mut old = serde_json::to_value(ParseOptions::default()).unwrap();
    old.as_object_mut().unwrap().remove("frame_profile");
    assert_eq!(
        serde_json::from_value::<ParseOptions>(old).unwrap(),
        ParseOptions::default()
    );
    let mut profile = NativeScanProfile::default();
    profile.grammar.corruption_check = !profile.grammar.corruption_check;
    let widths = NativeSharedWidths::from_map(std::collections::BTreeMap::from([(
        "unused-component".to_string(),
        17,
    )]));
    profile.grammar.calibrated_widths = Some(widths.clone());
    let parsed = Film::parse_v41_with_options(
        &source(&[0x80]),
        ParseOptions {
            frame_profile: Some(profile),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        parsed.frame_config.profile.grammar.corruption_check,
        NativeScanProfile::default().grammar.corruption_check
    );
    widths.insert("unused-component".into(), 99);
    assert_eq!(
        parsed
            .frame_config
            .profile
            .grammar
            .calibrated_widths
            .as_ref()
            .unwrap()
            .get("unused-component"),
        Some(17)
    );
}
