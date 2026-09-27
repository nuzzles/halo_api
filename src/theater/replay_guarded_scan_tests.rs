use super::*;
use serde_json::{Value, json};
use std::io::Read;
fn unhex(h: &str) -> Vec<u8> {
    (0..h.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap())
        .collect()
}
fn zone() -> ObjectiveZone {
    let shape = ObjectiveShape {
        family: "box".into(),
        half_x: Some(1.),
        half_y: Some(1.),
        up: ObjectiveVec3 {
            z: 1.,
            ..Default::default()
        },
        forward: ObjectiveVec3 {
            x: 1.,
            ..Default::default()
        },
        ..Default::default()
    };
    ObjectiveZone {
        role: String::new(),
        instance_id: 0,
        object_idx: 0,
        team_index: 0,
        spatial_rank: 0,
        center: ObjectiveVec3::default(),
        volume: ObjectiveVolume::new(ObjectiveVec3::default(), Some(&shape)).unwrap(),
        shape,
    }
}
#[test]
fn native_guarded_scan_phase() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/guarded-scan-v41.json.zlib")[..])
        .read_to_end(&mut raw)
        .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 224);
    let mut allocated_empty = 0;
    let (mut marks, mut zones_read, mut gauges, mut bombs, mut both) = (0, 0, 0, 0, 0);
    for (case, row) in rows.iter().enumerate() {
        let mut buffers = Vec::new();
        let mut meta = Vec::new();
        for c in row["inputs"].as_array().unwrap() {
            buffers.push(unhex(c["Hex"].as_str().unwrap()));
            meta.push(FilmSourceMetadata {
                index: c["Index"].as_i64().unwrap(),
                chunk_type: 0,
                start_ms: 0,
            });
        }
        let source = FilmSource::load(&buffers, &meta).unwrap();
        let mut context = NativeFilmContext::new(Some(&source));
        let mut p = NativeScanProfile::default();
        p.grammar.simulation_complete = row["simulation"] == true;
        context.set_scan_profile(p).unwrap();
        let records: Vec<StatborgRecord> = serde_json::from_value(row["records"].clone()).unwrap();
        let bursts: Vec<i64> =
            serde_json::from_value(row["bursts"].as_array().map_or(json!([]), |a| json!(a)))
                .unwrap();
        let zones = if row["zones"] == true {
            vec![zone()]
        } else {
            vec![]
        };
        let start = serde_json::from_value(if row["start"].is_null() {
            json!({})
        } else {
            row["start"].clone()
        })
        .unwrap();
        let mut timeline = Vec::new();
        let mut counts = [0usize; 2];
        let mut names = Vec::new();
        let out = scan_replay_guarded_inputs(
            b"oracle-guarded",
            &context,
            ReplayGuardedScanOptions {
                flag_scanned: row["flag"] == true,
                flag_records: &records,
                flag_bursts: &bursts,
                zones: &zones,
                bomb_scanned: row["bomb"] == true,
                chunk_start_ms: &start,
            },
            |event| {
                let name = event.channel_name();
                let value = match event {
                    ReplayGuardedObservation::CarrierMarks(v) => json!(v),
                    ReplayGuardedObservation::ZoneReads(v)
                    | ReplayGuardedObservation::FlagGauge(v) => json!(v),
                    ReplayGuardedObservation::BombReads(v) => json!(v),
                    ReplayGuardedObservation::Diagnostic(d) => {
                        let mut log = serde_json::Map::from_iter(d.attributes.iter().cloned());
                        log.insert("level".into(), json!(d.level));
                        log.insert("msg".into(), json!(d.message));
                        timeline.push(json!({"log":log,"counters":counts}));
                        return;
                    }
                    ReplayGuardedObservation::Counter { name, amount } => {
                        let i = match name {
                            "filmdec_keyframe_ti12_closed" => 0,
                            "filmdec_keyframe_ti12_total" => 1,
                            _ => panic!("unexpected counter"),
                        };
                        counts[i] += amount;
                        return;
                    }
                };
                let name = name.unwrap();
                names.push(name);
                timeline.push(json!({"step":name,"value":value,"counters":counts}));
            },
        );
        assert_eq!(
            names,
            ["carrierMarks", "zoneReads", "flagGauge", "bombReads"]
        );
        assert_eq!(
            json!(timeline),
            row["timeline"],
            "ordered observations {case}"
        );
        assert_eq!(json!(counts), row["counters"], "counters {case}");
        assert_eq!(
            serde_json::to_value(&out.published).unwrap(),
            row["published"],
            "publication {case}"
        );
        let mut writer = NativeFactsWriter::default();
        encode_facts_mode_guards(&mut writer, &out.published);
        assert!(writer.error().is_none());
        assert_eq!(
            writer.bytes(),
            unhex(row["encoded"].as_str().unwrap()),
            "cache {case}"
        );
        assert_eq!(
            out.managed.is_some(),
            out.published.zone_scanned || out.published.flag_gauge_scanned
        );
        let managed_logs = out
            .diagnostics
            .iter()
            .filter(|d| d.message.starts_with("rejeu : proprietes ti=13 "))
            .count();
        assert_eq!(
            managed_logs,
            usize::from(out.managed.is_some()),
            "single shared scan {case}"
        );
        if out.published.zone_scanned && out.published.flag_gauge_scanned {
            both += 1;
            assert_eq!(out.published.zone_reads, out.published.flag_gauge);
        }
        marks += out.published.flag_marks.marks.as_ref().map_or(0, Vec::len);
        zones_read += out.published.zone_reads.as_ref().map_or(0, Vec::len);
        gauges += out.published.flag_gauge.as_ref().map_or(0, Vec::len);
        allocated_empty +=
            usize::from(out.published.bomb_reads.as_ref().is_some_and(Vec::is_empty));
        bombs += out.published.bomb_reads.as_ref().map_or(0, Vec::len);
    }
    assert_eq!(allocated_empty, 6);
    assert!(marks > 0 && zones_read > 0 && gauges > 0 && bombs > 0 && both > 0);
}
