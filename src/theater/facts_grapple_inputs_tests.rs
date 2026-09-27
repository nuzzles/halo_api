use super::*;
use serde::Deserialize;
use std::io::Read;

#[derive(Deserialize)]
struct Case {
    raw: Vec<FactsGrappleRead>,
    origin: u64,
    step: u64,
    tracks: Vec<ReplayTrack>,
    map: FilmMapBounds,
    output: ReplayGrapple,
}
#[test]
fn native_facts_grapple_pairing_and_arrival() {
    let mut data = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/facts-grapple-v41.json.zlib")[..])
        .read_to_end(&mut data)
        .unwrap();
    let rows: Vec<Case> = serde_json::from_slice(&data).unwrap();
    assert_eq!(rows.len(), 1024);
    let mut wide = 0;
    let mut late = 0;
    let mut lines = 0;
    for (i, row) in rows.into_iter().enumerate() {
        wide += row
            .raw
            .iter()
            .filter(|r| r.position_quanta.iter().any(|&q| q > u16::MAX as u32))
            .count();
        late += row
            .raw
            .iter()
            .filter(|r| r.timestamp_us == u64::MAX)
            .count();
        let got = build_facts_replay_grapple(&row.raw, &row.map, row.origin, row.step, &row.tracks);
        lines += got.lines.len();
        assert_eq!(got, row.output, "case {i}");
    }
    assert!(wide > 0 && late > 0 && lines > 0);
}
