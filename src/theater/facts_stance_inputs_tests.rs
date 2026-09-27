use super::*;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::io::Read;
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Raw {
    timestamp: u64,
    slot: u32,
    kind_hex: String,
    on: bool,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Stance {
    slot: u32,
    kind_hex: String,
    t0: i64,
    t1: i64,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Case {
    raw: Vec<Raw>,
    out_raw: Vec<Stance>,
    counts: Vec<(String, i64)>,
    stats: FactsMovementStats,
    origin: u64,
    step: u64,
    tracks: Vec<ReplayTrack>,
    deaths: BTreeSet<usize>,
    stances: serde_json::Value,
    coverage: serde_json::Value,
}
fn bytes(hex: &str) -> Vec<u8> {
    hex.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}
#[test]
fn native_facts_stance_raw_identity_and_counters() {
    let mut data = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/facts-stances-v41.json.zlib")[..])
        .read_to_end(&mut data)
        .unwrap();
    let cases: Vec<Case> = serde_json::from_slice(&data).unwrap();
    assert_eq!(cases.len(), 1024);
    for (i, c) in cases.into_iter().enumerate() {
        let reads: Vec<_> = c
            .raw
            .into_iter()
            .map(|r| FactsMovementState {
                timestamp_us: r.timestamp,
                slot: r.slot,
                kind: bytes(&r.kind_hex),
                on: r.on,
                ..Default::default()
            })
            .collect();
        let got =
            build_facts_replay_stances(&reads, &c.stats, c.origin, c.step, &c.tracks, &c.deaths);
        let expected: Vec<_> = c
            .out_raw
            .into_iter()
            .map(|r| ReplayStance {
                slot: r.slot,
                kind: ReplayByteString(bytes(&r.kind_hex)),
                t0: r.t0,
                t1: r.t1,
            })
            .collect();
        assert_eq!(got.stances, expected, "raw stances {i}");
        assert_eq!(
            got.coverage.by_kind,
            c.counts
                .into_iter()
                .map(|(k, v)| (ReplayByteString(bytes(&k)), v))
                .collect(),
            "raw counts {i}"
        );
        assert_eq!(
            serde_json::to_value(&got.stances).unwrap(),
            c.stances,
            "JSON stances {i}"
        );
        // Serialize through text to match duplicate rendered map-key semantics.
        let json: serde_json::Value =
            serde_json::from_slice(&serde_json::to_vec(&got.coverage).unwrap()).unwrap();
        assert_eq!(json, c.coverage, "coverage {i}");
    }
}
