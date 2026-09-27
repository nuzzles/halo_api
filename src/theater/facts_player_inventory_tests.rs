use super::*;
use serde::Deserialize;
use std::io::Read;
fn fixture(bytes: &[u8]) -> Vec<u8> {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(bytes)
        .read_to_end(&mut raw)
        .unwrap();
    raw
}
fn unhex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
#[test]
fn native_facts_player_inventory_document_projection() {
    #[derive(Deserialize)]
    struct Coverage {
        inventory: Option<ReplayInventoryCoverage>,
    }
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Document {
        frame_count: i64,
        #[serde(default)]
        inventory: Vec<ReplayInventory>,
        coverage: Option<Coverage>,
    }
    #[derive(Deserialize)]
    struct Row {
        input: String,
        interval: i64,
        min_points: i64,
        document: Document,
    }
    let rows: Vec<Row> = serde_json::from_slice(&fixture(include_bytes!(
        "fixtures/facts-player-inventory-v41.json.zlib"
    )))
    .unwrap();
    assert_eq!(rows.len(), 128);
    let entry = FactsMapEntry {
        module: b"map".to_vec(),
        axis_widths: [13, 14, 15],
        bounds: [[-10., 100.], [-30., 300.], [-50., 500.]],
        ..Default::default()
    };
    for (i, row) in rows.into_iter().enumerate() {
        let file = decode_film_facts_file(&unhex(&row.input), &entry).unwrap();
        let got = build_facts_replay_players(
            &file.facts,
            &[],
            &[],
            FilmReplayPlayerOptions {
                frame_interval_ms: row.interval,
                min_points: row.min_points,
                ..Default::default()
            },
        )
        .unwrap();
        let d = row.document;
        let Some(g) = got else {
            assert_eq!(d.frame_count, 0, "empty {i}");
            continue;
        };
        let inventory = build_facts_player_inventory(&file.facts.inventory, &g).unwrap();
        assert_eq!(inventory.reads, d.inventory, "inventory {i}");
        assert_eq!(
            inventory.coverage,
            d.coverage.and_then(|c| c.inventory),
            "inventory coverage {i}"
        );
    }
}
