use super::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct Case {
    name: String,
    hex: String,
    bit_len: usize,
    record_end: usize,
    orientation_bit: usize,
    orientation_end: usize,
}
#[test]
fn engine_one_teleports_match_config_orientation_and_preserve_continuation() {
    #[derive(Deserialize)]
    struct Oracle {
        name: String,
        end: usize,
        trace: Vec<(usize, usize)>,
    }
    #[derive(Deserialize)]
    struct Export {
        commit: String,
        blob: String,
        cases: Vec<Oracle>,
    }
    let cases: Vec<Case> = serde_json::from_str(include_str!(
        "../../../../../../../fixtures/events-v41/teleport-engine-one.json"
    ))
    .unwrap();
    let oracle: Export = serde_json::from_str(include_str!(
        "../../../../../../../fixtures/events-v41/teleport-engine-one-reference.json"
    ))
    .unwrap();
    assert_eq!(oracle.commit, "413971bcc485be53a2fa8cf229e170539a5146f8");
    assert_eq!(oracle.blob, "a32dc647e20c3d32146358145ad93b979adbe83e");
    assert_eq!(cases.len(), 48);
    assert_eq!(oracle.cases.len(), 48);
    let encoding = PrecisionDescriptor {
        index_bits: 1,
        region: 0,
        axis_bits: [15, 15, 17],
        regions: Default::default(),
    };
    for (case, oracle) in cases.into_iter().zip(oracle.cases) {
        assert_eq!(case.name, oracle.name);
        assert_eq!(case.orientation_end, oracle.end);
        let bytes: Vec<_> = case
            .hex
            .as_bytes()
            .chunks(2)
            .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
            .collect();
        let layout = |runtime_defaults| EventLayout {
            event_vector: Some(&encoding),
            variant: VariantContext {
                engine_type: Some(1),
                ..Default::default()
            },
            runtime_defaults,
            ..Default::default()
        };
        for runtime_defaults in [false, true] {
            let read = read_event_list(&bytes, 0, 100, layout(runtime_defaults));
            assert_eq!(read.stop, EventListStop::Terminator, "{}", case.name);
            assert_eq!(read.end_bit, case.bit_len);
            assert_eq!(read.records.len(), 2);
            assert_eq!(read.records[0].end_bit, case.record_end);
            assert_eq!(read.records[1].code, Some(21));
            assert!(read.assumptions.is_empty());
            let trace: Vec<_> = read.records[0]
                .fields
                .iter()
                .filter(|f| f.bit >= case.orientation_bit && f.bit < case.orientation_end)
                .map(|f| (f.bit, f.width))
                .collect();
            assert_eq!(trace, oracle.trace);
        }
        let unknown = read_event_list(
            &bytes,
            0,
            100,
            EventLayout {
                event_vector: Some(&encoding),
                ..Default::default()
            },
        );
        assert_eq!(unknown.stop, EventListStop::MissingVariantContext);
        assert_eq!(unknown.end_bit, case.orientation_bit);
        assert!(unknown.assumptions.is_empty());
        for cut in 0..case.bit_len {
            let shift = (8 - cut % 8) % 8;
            let mut prefix = vec![0; (cut + shift) / 8];
            for i in 0..cut {
                prefix[(i + shift) / 8] |=
                    ((bytes[i / 8] >> (7 - i % 8)) & 1) << (7 - (i + shift) % 8);
            }
            let read = read_event_list(&prefix, shift, 100, layout(false));
            assert_ne!(
                read.stop,
                EventListStop::Terminator,
                "{} cut {cut}",
                case.name
            );
            assert!(read.end_bit <= prefix.len() * 8);
            if cut < case.record_end {
                assert!(!read.records.first().is_some_and(|r| r.layout_complete));
            }
            assert!(read.assumptions.is_empty());
        }
    }
}
