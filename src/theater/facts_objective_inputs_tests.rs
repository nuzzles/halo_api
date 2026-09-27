use super::*;
use std::collections::BTreeMap;
use std::io::Read;

#[test]
fn native_facts_flag_layer() {
    let mut bytes = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/facts-flag-layer-v41.json.zlib")[..])
        .read_to_end(&mut bytes)
        .unwrap();
    let rows: Vec<serde_json::Value> = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(rows.len(), 128);
    let mut published = 0;
    let mut warnings = 0;
    let mut gauge_scans = 0;
    for (i, row) in rows.iter().enumerate() {
        let records: Vec<StatborgRecord> = serde_json::from_value(row["records"].clone()).unwrap();
        let deaths: Vec<IdentityDeath> = serde_json::from_value(row["deaths"].clone()).unwrap();
        let deaths: Vec<_> = deaths
            .iter()
            .map(|d| FactsDeath {
                xuid: d.xuid,
                time_ms: d.time_ms,
                gamertag: vec![0xff, 0, 0x80],
            })
            .collect();
        let tracks = serde_json::from_value::<Vec<ReplayTrack>>(row["tracks"].clone()).unwrap();
        let bursts = serde_json::from_value::<Vec<i64>>(row["bursts"].clone()).unwrap();
        let spawns = serde_json::from_value::<Vec<ReplayFlagSpawn>>(row["spawns"].clone()).unwrap();
        let gauges =
            serde_json::from_value::<Vec<ManagedPropertyRead>>(row["gauges"].clone()).unwrap();
        let identity = match row["identity_mode"].as_u64().unwrap() {
            0 => StatborgRoundIdentity {
                publication: IdentityStatborgPublication::default(),
                starts: Vec::new(),
            },
            1 => StatborgRoundIdentity::from_flat(BTreeMap::from([(10, "100".into())])),
            _ => StatborgRoundIdentity::from_flat(BTreeMap::new()),
        };
        assert_eq!(
            identity.resolved(),
            row["identity_mode"].as_u64().unwrap() != 0
        );
        let clock: ReplayMatchClock = serde_json::from_value(row["clock"].clone()).unwrap();
        let (out, diagnostics) = build_facts_replay_flag_layer(
            FactsReplayFlagLayerInput {
                scanned: row["scanned"].as_bool().unwrap(),
                records: &records,
                bursts: &bursts,
                identity: &identity,
                teams: &BTreeMap::from([("100".into(), 0)]),
                marks: &FactsCarrierMarkScan::default(),
                spawns: &spawns,
                free: &[],
                gauge: &gauges,
                gauge_scanned: row["gauge_scanned"].as_bool().unwrap(),
            },
            ReplayFlagContext {
                clock,
                tracks: &tracks,
                deaths: &deaths,
                bridge: &BTreeMap::new(),
                ambiguous_slots: &BTreeMap::new(),
            },
        );
        assert_eq!(
            out.carries,
            serde_json::from_value::<Vec<ReplayFlagCarry>>(row["carries"].clone()).unwrap(),
            "carries {i}"
        );
        assert_eq!(
            out.coverage,
            serde_json::from_value::<Option<ReplayFlagCoverage>>(row["coverage"].clone()).unwrap(),
            "coverage {i}"
        );
        assert_eq!(
            out.tracks_without_bridge as u64,
            row["no_bridge"].as_u64().unwrap(),
            "bridge fallback {i}"
        );
        assert_eq!(
            out.drops_using_pickup as u64,
            row["drop_pickup"].as_u64().unwrap(),
            "pickup fallback {i}"
        );
        assert_eq!(
            diagnostics,
            serde_json::from_value::<Vec<StatborgDiagnostic>>(row["logs"].clone()).unwrap(),
            "diagnostics {i}"
        );
        published += out.carries.len();
        warnings += diagnostics.len();
        gauge_scans += usize::from(out.coverage.as_ref().is_some_and(|c| c.gauge_scanned));
    }
    assert!(published > 0 && warnings > 0 && gauge_scans > 0);
    println!("{published} flags; {warnings} diagnostics; {gauge_scans} gauge scans");
}

#[test]
fn native_facts_vip_layer() {
    let mut bytes = Vec::new();
    flate2::read::ZlibDecoder::new(
        &include_bytes!("fixtures/objective-diagnostics-v41.json.zlib")[..],
    )
    .read_to_end(&mut bytes)
    .unwrap();
    let rows: Vec<serde_json::Value> = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(rows.len(), 128);
    let mut published = 0;
    let mut warnings = 0;
    for (i, row) in rows.iter().enumerate() {
        let records =
            serde_json::from_value::<Vec<StatborgRecord>>(row["records"].clone()).unwrap();
        let deaths = serde_json::from_value::<Vec<IdentityDeath>>(row["deaths"].clone()).unwrap();
        let deaths: Vec<_> = deaths
            .iter()
            .map(|d| FactsDeath {
                xuid: d.xuid,
                time_ms: d.time_ms,
                gamertag: vec![0xff, 0x80],
            })
            .collect();
        let clock = serde_json::from_value(row["clock"].clone()).unwrap();
        let (out, diagnostics) = build_facts_replay_vip_crown(
            &records,
            &deaths,
            clock,
            row["recognized"].as_bool().unwrap(),
        );
        assert_eq!(
            out,
            serde_json::from_value::<ReplayVipCrown>(row["vip"].clone()).unwrap(),
            "VIP {i}"
        );
        assert_eq!(
            diagnostics,
            serde_json::from_value::<Vec<StatborgDiagnostic>>(row["vip_logs"].clone()).unwrap(),
            "VIP diagnostics {i}"
        );
        published += out.periods.len();
        warnings += diagnostics.len();
    }
    assert!(published > 0 && warnings > 0);
    println!("cached VIP: {published} periods; {warnings} diagnostics");
}

#[test]
fn native_facts_skull_layer() {
    let mut bytes = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/facts-skull-layer-v41.json.zlib")[..])
        .read_to_end(&mut bytes)
        .unwrap();
    let rows: Vec<serde_json::Value> = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(rows.len(), 128);
    let mut published = [0; 3];
    for (i, row) in rows.iter().enumerate() {
        let records =
            serde_json::from_value::<Vec<StatborgRecord>>(row["records"].clone()).unwrap();
        let deaths = serde_json::from_value::<Vec<IdentityDeath>>(row["deaths"].clone()).unwrap();
        let deaths: Vec<_> = deaths
            .iter()
            .map(|d| FactsDeath {
                xuid: d.xuid,
                time_ms: d.time_ms,
                gamertag: vec![0xff, 0x80],
            })
            .collect();
        let tracks = serde_json::from_value::<Vec<ReplayTrack>>(row["tracks"].clone()).unwrap();
        let deduced: BTreeMap<usize, bool> =
            serde_json::from_value(row["deduced"].clone()).unwrap();
        let deduced = deduced
            .into_iter()
            .filter_map(|(i, yes)| yes.then_some(i))
            .collect();
        let mode = row["identity_mode"].as_u64().unwrap() as usize;
        let identity = match mode {
            0 => StatborgRoundIdentity {
                publication: IdentityStatborgPublication::default(),
                starts: Vec::new(),
            },
            1 => StatborgRoundIdentity::from_flat(BTreeMap::from([(10, "100".into())])),
            _ => StatborgRoundIdentity::from_flat(BTreeMap::new()),
        };
        assert_eq!(identity.resolved(), mode != 0);
        let clock = serde_json::from_value(row["clock"].clone()).unwrap();
        let out = build_facts_replay_skull_carries(
            FactsReplaySkullInput {
                scanned: row["scanned"].as_bool().unwrap(),
                records: &records,
                identity: &identity,
                deaths: &deaths,
            },
            clock,
            &ReplayCarrierPresence::from_tracks(&tracks, &deduced),
        );
        assert_eq!(
            out.carries,
            serde_json::from_value::<Vec<ReplaySkullCarry>>(row["carries"].clone()).unwrap(),
            "skull carries {i}"
        );
        assert_eq!(
            out.coverage,
            serde_json::from_value::<Option<ReplaySkullCoverage>>(row["coverage"].clone()).unwrap(),
            "skull coverage {i}"
        );
        published[mode] += out.carries.len();
    }
    assert!(published[0] > 0 && published[1] > 0);
    assert_eq!(published[2], 0);
    println!("cached skull carries by identity mode: {published:?}");
}
