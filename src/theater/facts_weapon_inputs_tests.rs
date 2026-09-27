use super::*;
use serde::Deserialize;
use std::io::Read;
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Raw {
    #[serde(rename = "TimestampUS")]
    timestamp_us: u64,
    slot: u32,
    slot_index: i64,
    family: u32,
    previous: u32,
    kind_hex: String,
}
#[derive(Deserialize)]
struct Case {
    raw: Vec<Raw>,
    origin: u64,
    step: u64,
    output: ReplayWeaponChanges,
}
#[test]
fn native_facts_weapon_change_publication() {
    let mut data = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/facts-weapon-changes-v41.json.zlib")[..],
    )
    .read_to_end(&mut data)
    .unwrap();
    let rows: Vec<Case> = serde_json::from_slice(&data).unwrap();
    assert_eq!(rows.len(), 1024);
    let mut invalid = 0;
    let mut wide = 0;
    for (i, row) in rows.into_iter().enumerate() {
        let raw: Vec<_> = row
            .raw
            .into_iter()
            .map(|r| FactsWeaponChange {
                timestamp_us: r.timestamp_us,
                slot: r.slot,
                slot_index: r.slot_index,
                family: r.family,
                previous: r.previous,
                kind: (0..r.kind_hex.len())
                    .step_by(2)
                    .map(|j| u8::from_str_radix(&r.kind_hex[j..j + 2], 16).unwrap())
                    .collect(),
            })
            .collect();
        invalid += raw
            .iter()
            .filter(|r| std::str::from_utf8(&r.kind).is_err())
            .count();
        wide += raw
            .iter()
            .filter(|r| r.slot_index < 0 || r.slot_index > u32::MAX as i64)
            .count();
        assert_eq!(
            build_facts_replay_weapon_changes(&raw, row.origin, row.step),
            row.output,
            "publication {i}"
        );
    }
    assert!(invalid > 0 && wide > 0);
}
