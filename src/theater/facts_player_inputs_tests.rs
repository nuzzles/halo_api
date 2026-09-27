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
fn native_facts_player_document_projection() {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Document {
        frame_count: i64,
        bounds: ReplayBounds,
        tracks: Option<Vec<ReplayTrack>>,
        #[serde(default)]
        frame_interval_ms: i64,
        #[serde(default)]
        duration_ms: i64,
        #[serde(default)]
        origin_ms: Option<i64>,
        #[serde(default)]
        roster: Vec<ReplayRosterEntry>,
        identity: Option<IdentitySection>,
    }
    #[derive(Deserialize)]
    struct Row {
        input: String,
        interval: i64,
        min_points: i64,
        document: Document,
    }
    let rows: Vec<Row> = serde_json::from_slice(&fixture(include_bytes!(
        "fixtures/facts-players-v41.json.zlib"
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
        assert_eq!(g.clock.frame_count, d.frame_count, "frames {i}");
        assert_eq!(g.frame_interval_ms, d.frame_interval_ms, "interval {i}");
        assert_eq!(g.duration_ms, d.duration_ms, "duration {i}");
        assert_eq!(g.origin_ms, d.origin_ms, "origin {i}");
        assert_eq!(g.players.bounds, d.bounds, "bounds {i}");
        assert_eq!(
            g.players.publication.tracks,
            d.tracks.unwrap_or_default(),
            "tracks {i}"
        );
        assert_eq!(g.players.roster, d.roster, "roster {i}");
        assert_eq!(g.players.identity, d.identity.unwrap(), "identity {i}");
    }
}
#[test]
fn native_facts_scope_signed_levels() {
    #[derive(Deserialize)]
    struct Event {
        slot: u32,
        time: u64,
        level: i64,
    }
    #[derive(Deserialize)]
    struct Row {
        events: Vec<Event>,
        lives: Vec<IdentityLife>,
        hold: u64,
        queries: Vec<Event>,
    }
    let rows: Vec<Row> = serde_json::from_slice(&fixture(include_bytes!(
        "fixtures/facts-scope-v41.json.zlib"
    )))
    .unwrap();
    assert_eq!(rows.len(), 1024);
    for (i, row) in rows.into_iter().enumerate() {
        let events: Vec<_> = row
            .events
            .into_iter()
            .map(|e| FactsZoomEvent {
                timestamp_us: e.time,
                slot: e.slot,
                level: e.level,
            })
            .collect();
        let lookup = ReplayScopeLookup::from_facts(&events, &row.lives, row.hold);
        for q in row.queries {
            assert_eq!(
                lookup.at(q.slot, q.time),
                q.level,
                "scope {i} slot {} time {}",
                q.slot,
                q.time
            );
        }
    }
}
#[test]
fn facts_player_projection_rejects_unrepresentable_legacy_values() {
    for value in [i64::MIN, -1, 0, 1, 1i64 << 32, i64::MAX] {
        let indices = FactsPlayerIndices {
            readings: value,
            disagreements: value,
            ..Default::default()
        };
        let projected = PlayerIndexTable::try_from(&indices).unwrap();
        assert_eq!(projected.readings, value);
        assert_eq!(projected.disagreements, value);
    }
    let death = FactsDeath {
        gamertag: vec![0xff],
        ..Default::default()
    };
    assert_eq!(
        IdentityDeath::try_from(&death).unwrap().gamertag.0,
        vec![0xff]
    );
    assert_eq!(death.gamertag, vec![0xff]);
    let table = FactsPlayerTable {
        occupied: -1,
        ..Default::default()
    };
    assert_eq!(
        ReplayFilmPlayerTable::try_from(&table).unwrap().occupied,
        -1
    );
}
