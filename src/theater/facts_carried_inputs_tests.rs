use super::*;
use serde::Deserialize;
use std::{io::Read, num::NonZeroU64};
fn fixture(bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    flate2::read::ZlibDecoder::new(bytes)
        .read_to_end(&mut out)
        .unwrap();
    out
}
#[test]
fn native_facts_loadout_publication() {
    #[derive(Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct Loadout {
        #[serde(rename = "TimestampUS")]
        timestamp_us: u64,
        slot: u32,
        families: Option<Vec<u32>>,
    }
    #[derive(Deserialize)]
    struct Case {
        raw: Vec<Loadout>,
        origin: u64,
        step: u64,
        slots: Vec<u32>,
        built: Vec<ReplayLoadout>,
        published: Vec<ReplayLoadout>,
    }
    let rows: Vec<Case> = serde_json::from_slice(&fixture(include_bytes!(
        "fixtures/facts-loadouts-v41.json.zlib"
    )))
    .unwrap();
    assert_eq!(rows.len(), 1024);
    for (i, row) in rows.into_iter().enumerate() {
        let raw: Vec<_> = row
            .raw
            .into_iter()
            .map(|l| FactsLoadout {
                timestamp_us: l.timestamp_us,
                slot: l.slot,
                families: l.families,
            })
            .collect();
        let mut got =
            build_facts_replay_loadouts(&raw, row.origin, NonZeroU64::new(row.step).unwrap());
        assert_eq!(got, row.built, "loadouts {i}");
        let tracks: Vec<_> = row
            .slots
            .into_iter()
            .map(|slot| ReplayTrack {
                slot,
                ..Default::default()
            })
            .collect();
        retain_replay_loadouts(&mut got, &tracks);
        assert_eq!(got, row.published, "loadout publication {i}");
    }
}
#[test]
fn native_facts_grenade_selection_domain() {
    #[derive(Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct Keyframe {
        #[serde(rename = "TimestampUS")]
        timestamp_us: u64,
        slot: u32,
        grenades_read: bool,
        grenades: [u32; 4],
        selected_grenade_rank: i64,
    }
    #[derive(Deserialize)]
    struct Delta {
        time: u64,
        slot: u32,
        counts: Option<Vec<u32>>,
        selection_read: bool,
        selection: i64,
    }
    #[derive(Deserialize)]
    struct Case {
        kf: Vec<Keyframe>,
        deltas: Vec<Delta>,
        origin: u64,
        step: u64,
        slots: Vec<u32>,
        refused: bool,
        built: Vec<ReplayGrenadeRead>,
        output: ReplayGrenadeReads,
    }
    let rows: Vec<Case> = serde_json::from_slice(&fixture(include_bytes!(
        "fixtures/facts-grenade-reads-v41.json.zlib"
    )))
    .unwrap();
    assert_eq!(rows.len(), 1024);
    let mut empty_count = 0;
    let mut wide = 0;
    let mut negative = 0;
    for (i, row) in rows.into_iter().enumerate() {
        let kf: Vec<_> = row
            .kf
            .into_iter()
            .map(|r| FactsKeyframeInventory {
                timestamp_us: r.timestamp_us,
                slot: r.slot,
                grenades_read: r.grenades_read,
                grenades: r.grenades,
                selected_grenade_rank: r.selected_grenade_rank,
                ..Default::default()
            })
            .collect();
        let delta: Vec<_> = row
            .deltas
            .into_iter()
            .map(|r| FactsInventoryDelta {
                timestamp_us: r.time,
                slot: r.slot,
                grenades: r.counts,
                selection_read: r.selection_read,
                selection: r.selection,
                mask: 0,
            })
            .collect();
        let got = build_facts_replay_grenade_reads(
            &kf,
            &delta,
            row.origin,
            NonZeroU64::new(row.step).unwrap(),
        );
        empty_count += got.iter().filter(|r| r.g.is_none()).count();
        wide += got
            .iter()
            .filter(|r| r.gs.is_some_and(|s| s > i32::MAX as i64))
            .count();
        negative += got.iter().filter(|r| r.gs.is_some_and(|s| s < -1)).count();
        assert_eq!(got, row.built, "grenade reads {i}");
        let tracks: Vec<_> = row
            .slots
            .into_iter()
            .map(|slot| ReplayTrack {
                slot,
                ..Default::default()
            })
            .collect();
        assert_eq!(
            publish_replay_grenade_reads(got, &tracks, row.refused),
            row.output,
            "grenade publication {i}"
        );
    }
    assert!(
        empty_count > 0 && wide > 0 && negative > 0,
        "oracle must exercise all new domains"
    );
}
