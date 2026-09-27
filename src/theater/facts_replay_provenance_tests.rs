use super::*;
use serde_json::{Value, json};
use std::io::Read;
fn unhex(s: &str) -> Vec<u8> {
    s.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
#[test]
fn native_facts_entry_provenance_and_raw_fallbacks() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/facts-provenance-v41.json.zlib")[..])
        .read_to_end(&mut raw)
        .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 1024);
    let mut invalid = 0;
    let mut wide = 0;
    for (i, r) in rows.iter().enumerate() {
        let mut identity = FactsIdentityJsonReader::default()
            .read(&serde_json::to_vec(&r["identity"]).unwrap())
            .unwrap();
        if let Some(id) = &mut identity {
            id.build = unhex(r["build_hex"].as_str().unwrap());
            invalid += usize::from(std::str::from_utf8(&id.build).is_err());
        }
        let file = NativeFilmFactsFile {
            identity,
            fallbacks: Some(
                r["hits"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|h| FactsFallback {
                        name: unhex(h["name_hex"].as_str().unwrap()),
                        count: h["hits"].as_i64().unwrap(),
                    })
                    .collect(),
            ),
            coverage: FactsDecoderCoverage {
                build: b"stale-header".to_vec(),
                ..Default::default()
            },
            ..Default::default()
        };
        let counter = FallbackCounter::default();
        counter.trigger_n("repli_manche_zero_decretee", 3);
        let mut restored = None;
        let logs = super::log_test_support::capture_logs(|| {
            restored = restore_facts_replay_provenance(&file, &counter);
        });
        assert_eq!(
            serde_json::to_value(logs).unwrap(),
            r["entry_logs"],
            "entry logs {i}"
        );
        assert_eq!(
            restored.as_ref().map(|v| &v.build.0),
            file.identity.as_ref().map(|v| &v.build),
            "raw build {i}"
        );
        let decoder = build_replay_decoder_coverage(restored.as_ref());
        assert_eq!(
            serde_json::to_value(decoder).unwrap(),
            r["decoder"],
            "decoder {i}"
        );
        let report = counter.report();
        let values: Vec<_> = report
            .iter()
            .map(|h| json!({"name_hex":hex(&h.name.0),"hits":h.hits}))
            .collect();
        assert_eq!(json!(values), r["report"], "raw counter ordering {i}");
        assert_eq!(
            hex(&fallback_report_text(&report).0),
            r["text_hex"].as_str().unwrap(),
            "raw text {i}"
        );
        wide += report.iter().filter(|h| h.hits > u32::MAX as i64).count();
        let mut doc = ReplayDocument {
            coverage: Some(ReplayCoverage::default()),
            ..Default::default()
        };
        let logs = super::log_test_support::capture_logs(|| {
            attach_replay_fallback_coverage(&mut doc, Some(&counter))
        });
        // These four seeded float fields are written as 0 by Go and 0.0 by
        // serde_json. Normalize only their numeric spelling, never integer counts.
        let mut expected = r["final"].clone();
        for key in ["minX", "minY", "maxX", "maxY"] {
            assert_eq!(expected["bounds"][key], json!(0));
            expected["bounds"][key] = json!(0.0);
        }
        assert_eq!(
            serde_json::to_value(&doc).unwrap(),
            expected,
            "final JSON {i}"
        );
        assert_eq!(
            serde_json::to_value(logs).unwrap(),
            r["final_logs"],
            "final logs {i}"
        );
        assert_eq!(
            doc.coverage
                .unwrap()
                .fallbacks
                .iter()
                .map(|h| &h.name.0)
                .collect::<Vec<_>>(),
            report.iter().map(|h| &h.name.0).collect::<Vec<_>>(),
            "raw published names {i}"
        );
    }
    assert!(invalid > 0 && wide > 0);
}
