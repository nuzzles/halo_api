use super::*;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::io::Read;
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Raw {
    #[serde(rename = "TimestampUS")]
    time: u64,
    slot: u32,
    has_positions: bool,
    from: [f32; 3],
    to: [f32; 3],
}
#[derive(Deserialize)]
struct Case {
    log: serde_json::Value,
    raw: Vec<Raw>,
    origin: u64,
    step: u64,
    slots: BTreeSet<u32>,
    output: ReplayTranslocations,
}
#[test]
fn native_facts_translocation_publication() {
    let mut data = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/replay-translocations-v41.json.zlib")[..],
    )
    .read_to_end(&mut data)
    .unwrap();
    let cases: Vec<Case> = serde_json::from_slice(&data).unwrap();
    for (i, c) in cases.into_iter().enumerate() {
        assert_eq!(
            super::log_test_support::capture_log(|| c.output.coverage.log()),
            c.log,
            "log {i}"
        );
        let raw: Vec<_> = c
            .raw
            .into_iter()
            .map(|r| FactsTranslocation {
                timestamp_us: r.time,
                slot: r.slot,
                has_positions: r.has_positions,
                from: r.from,
                to: r.to,
            })
            .collect();
        assert_eq!(
            build_facts_replay_translocations(&raw, &c.slots, c.origin, c.step),
            c.output,
            "cache projection {i}"
        );
    }
}
