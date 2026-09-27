use super::*;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    io::Read,
    sync::{Arc, Mutex},
};
fn unhex(h: &str) -> Vec<u8> {
    (0..h.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap())
        .collect()
}
fn list<T: serde::de::DeserializeOwned>(v: &Value) -> Vec<T> {
    serde_json::from_value(v.as_array().cloned().unwrap_or_default().into()).unwrap()
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
#[derive(Default)]
struct Timeline {
    counters: BTreeMap<String, usize>,
    entries: Vec<Value>,
}
impl Timeline {
    fn log(&mut self, value: Value) {
        self.entries
            .push(json!({"log":value,"counters":self.counters}));
    }
    fn diagnostic(&mut self, d: &StatborgDiagnostic) {
        let mut v = serde_json::Map::from_iter(d.attributes.iter().cloned());
        v.insert("level".into(), json!(d.level));
        v.insert("msg".into(), json!(d.message));
        self.log(v.into());
    }
    fn counter(&mut self, name: &str, amount: usize) {
        *self.counters.entry(name.into()).or_default() += amount;
    }
}
#[test]
fn native_world_scan_phase() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/world-scan-v41.json.zlib")[..])
        .read_to_end(&mut raw)
        .unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(rows.len(), 144);
    let mut totals = [0; 6];
    let mut conclusive = 0;
    let mut conclusive_unknown = 0;
    for (case, row) in rows.iter().enumerate() {
        assert_eq!(row["panicked"], false, "native panic {case}");
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
        let map: FilmMapBounds = serde_json::from_value(row["map"].clone()).unwrap();
        let layout = map.i0_layout();
        let mut context = NativeFilmContext::with_imposed_layout(
            Some(&source),
            (row["forced"] == true).then_some(&layout),
        );
        let mut p = NativeScanProfile {
            mpp: serde_json::from_value(row["inherited"].clone()).unwrap(),
            ..Default::default()
        };
        p.movement.world_object = NativePrecisionDescriptor {
            index_bits: map.region_index_bits.max(1) as u64,
            axis_bits: serde_json::from_value(row["axes"].clone()).unwrap(),
            region: map.region,
        };
        p.grammar.simulation_complete = row["simulation"] == true;
        p.grammar.generation_strict = row["strict"] == true;
        p.grammar.corruption_check = row["corruption"] == true;
        p.grammar.calibrated_widths =
            serde_json::from_value(row["calibrated_widths"].clone()).unwrap();
        p.grammar.stub_widths = serde_json::from_value(row["stub_widths"].clone()).unwrap();
        let original = p.mpp;
        context.set_scan_profile(p).unwrap();
        let records = list(&row["records"]);
        let bursts = list(&row["bursts"]);
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
        let timeline = Arc::new(Mutex::new(Timeline::default()));
        for key in row["counters"].as_object().unwrap().keys() {
            timeline.lock().unwrap().counters.insert(key.clone(), 0);
        }
        let logs = timeline.clone();
        let mut result = None;
        super::log_test_support::capture_logs_observed(
            || {
                result = Some(scan_replay_world_inputs(
                    b"oracle-world",
                    &mut context,
                    &map,
                    ReplayGuardedScanOptions {
                        flag_scanned: row["flag"] == true,
                        flag_records: &records,
                        flag_bursts: &bursts,
                        zones: &zones,
                        bomb_scanned: row["bomb"] == true,
                        chunk_start_ms: &start,
                    },
                    |event| {
                        let mut t = timeline.lock().unwrap();
                        if let Some(name) = event.channel_name() {
                            let mut value = json!({"step":name,"counters":t.counters});
                            let nil = match &event {
                                ReplayWorldObservation::ZoomEvents(v) => Some(v.is_none()),
                                ReplayWorldObservation::Placements(v) => Some(v.is_none()),
                                ReplayWorldObservation::SpawnEvents(v) => Some(v.is_none()),
                                ReplayWorldObservation::Guarded(
                                    ReplayGuardedObservation::ZoneReads(v)
                                    | ReplayGuardedObservation::FlagGauge(v),
                                ) => Some(v.is_none()),
                                ReplayWorldObservation::Guarded(
                                    ReplayGuardedObservation::BombReads(v),
                                ) => Some(v.is_none()),
                                _ => None,
                            };
                            if let Some(nil) = nil {
                                value["nil"] = json!(nil);
                            }
                            t.entries.push(value);
                            return;
                        }
                        match event {
                            ReplayWorldObservation::Diagnostic(d)
                            | ReplayWorldObservation::Guarded(
                                ReplayGuardedObservation::Diagnostic(d),
                            ) => t.diagnostic(d),
                            ReplayWorldObservation::Counter { name, amount }
                            | ReplayWorldObservation::Guarded(
                                ReplayGuardedObservation::Counter { name, amount },
                            ) => t.counter(name, amount),
                            _ => {}
                        }
                    },
                ));
            },
            move |v| logs.lock().unwrap().log(v),
        );
        let out = result.unwrap();
        let t = timeline.lock().unwrap();
        assert_eq!(json!(t.entries), row["timeline"], "timeline {case}");
        assert_eq!(json!(t.counters), row["counters"], "counters {case}");
        let calibration = &out.placements.stats.calibration;
        let widths = FilmMppWidths {
            lead: calibration.widths.lead as i64,
            index: calibration.widths.index as i64,
        };
        assert_eq!(
            serde_json::to_value(widths).unwrap(),
            row["calibration"],
            "calibration {case}"
        );
        assert_eq!(json!(calibration.agree), row["calibration_agree"]);
        assert_eq!(
            json!(calibration.runner_agree),
            row["calibration_runner_agree"]
        );
        if widths.is_valid() {
            conclusive += 1;
            assert!(
                out.placements
                    .tracks
                    .as_ref()
                    .is_some_and(|t| !t.tracks.is_empty())
            );
            if out.pads.weapons.format.unknown_format {
                conclusive_unknown += 1;
                assert_ne!(widths, original);
                assert_eq!(out.pads.weapons.installed_widths, Some(widths));
                assert_eq!(out.pads.powerups.installed_widths, Some(widths));
                assert_eq!(out.vehicles.installed_widths, Some(widths));
            }
        }
        assert_eq!(context.scan_profile().unwrap().mpp, original);
        assert_eq!(row["restored"], true);
        let mut facts = NativeFilmFacts::default();
        out.apply_to_facts(&mut facts);
        let mut writer = NativeFactsWriter::default();
        encode_facts_zoom_events(&mut writer, &facts.zoom_events);
        encode_facts_world_section(&mut writer, &facts.world);
        encode_facts_vehicle_scan(&mut writer, &facts.vehicles);
        encode_facts_mode_guards(&mut writer, &out.guarded.published);
        assert!(writer.error().is_none());
        assert_eq!(
            writer.bytes(),
            unhex(row["encoded"].as_str().unwrap()),
            "complete phase cache {case}"
        );
        for (i, (name, n)) in [
            ("zoom", out.published_zoom.len()),
            ("placements", out.published.placements.len()),
            ("spawns", out.published.spawn_events.len()),
            ("weapon_creations", out.published.weapons.creations.len()),
            ("powerup_creations", out.published.powerups.creations.len()),
            ("vehicle_creations", out.vehicles.published.creations.len()),
        ]
        .into_iter()
        .enumerate()
        {
            assert_eq!(json!(n), row[name], "{name} {case}");
            totals[i] += n;
        }
    }
    assert_eq!(conclusive, 5);
    assert!(conclusive_unknown > 0);
    assert!(
        totals.into_iter().all(|n| n > 0),
        "positive channels: {totals:?}"
    );
}
