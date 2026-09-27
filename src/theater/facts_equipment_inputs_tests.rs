use super::*;
use serde::Deserialize;
use std::{collections::BTreeSet, io::Read};
fn unhex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Raw {
    #[serde(rename = "TimestampUS")]
    timestamp_us: u64,
    slot: u32,
    rank: i64,
    previous: i64,
    gap: i64,
    recovered: bool,
    kind_hex: String,
}
#[derive(Deserialize)]
struct Case {
    raw: Vec<Raw>,
    stats: FactsEquipmentChangeStats,
    origin: u64,
    step: u64,
    slots: BTreeSet<u32>,
    output: serde_json::Value,
    kind_hex: Vec<String>,
    log: serde_json::Value,
}
#[test]
fn native_facts_equipment_domains_and_publication() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/facts-equipment-publication-v41.json.zlib")[..],
    )
    .read_to_end(&mut raw)
    .unwrap();
    let rows: Vec<Case> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 1024);
    let mut invalid = 0;
    let mut wide = 0;
    for (i, row) in rows.into_iter().enumerate() {
        let raw: Vec<_> = row
            .raw
            .into_iter()
            .map(|r| FactsEquipmentChange {
                timestamp_us: r.timestamp_us,
                slot: r.slot,
                rank: r.rank,
                previous: r.previous,
                gap: r.gap,
                recovered: r.recovered,
                kind: unhex(&r.kind_hex),
                counter: 0,
            })
            .collect();
        let got = build_facts_replay_equipment_changes(
            &raw, &row.stats, row.origin, row.step, &row.slots,
        );
        assert_eq!(
            serde_json::to_value(&got).unwrap(),
            row.output,
            "publication {i}"
        );
        assert_eq!(
            got.changes
                .iter()
                .map(|c| serde_json::to_value(c.kind)
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .as_bytes()
                    .to_vec())
                .collect::<Vec<_>>(),
            row.kind_hex.iter().map(|s| unhex(s)).collect::<Vec<_>>(),
            "native published kind bytes {i}"
        );
        assert_eq!(
            crate::theater::log_test_support::capture_logs(|| got.coverage.log()),
            vec![row.log],
            "coverage log {i}"
        );
        invalid += raw
            .iter()
            .filter(|c| std::str::from_utf8(&c.kind).is_err())
            .count();
        wide += got
            .changes
            .iter()
            .filter(|c| c.gap < 0 || c.gap > 255)
            .count();
    }
    assert!(invalid > 0 && wide > 0);
}
