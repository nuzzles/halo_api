use super::*;
use serde::Deserialize;
use std::{collections::BTreeSet, io::Read};
#[derive(Deserialize)]
struct Position {
    time: u64,
    slot: u32,
    has_shield: bool,
    quantum: u8,
}
#[derive(Deserialize)]
struct Case {
    positions: Vec<Position>,
    camo: Vec<FactsCamoState>,
    origin: u64,
    step: u64,
    tracks: Vec<ReplayTrack>,
    deaths: BTreeSet<usize>,
    output: ReplayEquipmentEpisodes,
}
#[test]
fn native_facts_equipment_episode_presence_and_lifetimes() {
    let mut data = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/facts-equipment-episodes-v41.json.zlib")[..],
    )
    .read_to_end(&mut data)
    .unwrap();
    let rows: Vec<Case> = serde_json::from_slice(&data).unwrap();
    assert_eq!(rows.len(), 1024);
    let mut inactive = 0;
    let mut wide_camo = 0;
    let mut episodes = 0;
    for (i, row) in rows.into_iter().enumerate() {
        let positions: Vec<_> = row
            .positions
            .into_iter()
            .map(|p| FactsBipedPosition {
                timestamp_us: p.time,
                slot: p.slot,
                has_shield: p.has_shield,
                shield_quantum: p.quantum,
                // Episode state is carried by the quantum and presence flag, not this scalar.
                shield: -999.0,
                ..Default::default()
            })
            .collect();
        inactive += positions
            .iter()
            .filter(|p| !p.has_shield && p.shield_quantum > 64)
            .count();
        wide_camo += row.camo.iter().filter(|c| c.quantum > 4095).count();
        let got = build_facts_replay_equipment_episodes(
            &positions,
            &row.camo,
            row.origin,
            row.step,
            &row.tracks,
            &row.deaths,
        );
        episodes += got.episodes.len();
        assert_eq!(got, row.output, "case {i}");
    }
    assert!(inactive > 0 && wide_camo > 0 && episodes > 0);
}
