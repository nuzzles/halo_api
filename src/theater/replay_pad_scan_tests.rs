use super::*;
use serde_json::{Value, json};
use std::{collections::BTreeMap, io::Read};
fn fixture(raw: &[u8]) -> Value {
    let mut bytes = Vec::new();
    flate2::read::ZlibDecoder::new(raw)
        .read_to_end(&mut bytes)
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}
fn list<T: serde::de::DeserializeOwned>(v: &Value) -> Vec<T> {
    serde_json::from_value(v.as_array().cloned().unwrap_or_default().into()).unwrap()
}
fn widths(v: &Value) -> FilmMppWidths {
    FilmMppWidths {
        lead: v["Lead"].as_i64().unwrap(),
        index: v["Index"].as_i64().unwrap(),
    }
}
#[test]
fn native_pad_scan_phase() {
    let rows = fixture(include_bytes!("fixtures/pad-scan-v41.json.zlib"));
    let seeds = fixture(include_bytes!(
        "fixtures/ground-creation-profile-v41.json.zlib"
    ));
    assert_eq!(rows.as_array().unwrap().len(), 96);
    let mut successes = [0; 2];
    let mut accepted = [0; 2];
    for (case, row) in rows.as_array().unwrap().iter().enumerate() {
        let mut buffers = Vec::new();
        let mut meta = Vec::new();
        for c in row["inputs"].as_array().unwrap() {
            let h = c["hex"].as_str().unwrap();
            buffers.push(
                (0..h.len())
                    .step_by(2)
                    .map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap())
                    .collect::<Vec<_>>(),
            );
            meta.push(FilmSourceMetadata {
                index: c["index"].as_i64().unwrap(),
                chunk_type: 0,
                start_ms: 0,
            });
        }
        let source = FilmSource::load(&buffers, &meta).unwrap();
        let mut context = NativeFilmContext::new(Some(&source));
        let mut p = NativeScanProfile {
            mpp: widths(&row["inherited"]),
            ..Default::default()
        };
        p.movement.world_object = NativePrecisionDescriptor {
            index_bits: 1,
            axis_bits: [16, 17, 18],
            region: 0,
        };
        let mut map = FilmMapBounds {
            module: String::new(),
            min: [-100., -200., -300.],
            max: [300., 400., 500.],
            axis_widths: [16, 17, 18],
            region: 0,
            region_index_bits: 1,
        };
        if row["source_kind"] == true {
            let seed = &seeds[row["seed"].as_u64().unwrap() as usize];
            map = serde_json::from_value(seed["map"].clone()).unwrap();
            p.movement.world_object = NativePrecisionDescriptor {
                index_bits: seed["index"].as_u64().unwrap(),
                axis_bits: serde_json::from_value(seed["axes"].clone()).unwrap(),
                region: seed["region"].as_u64().unwrap() as u32,
            };
            p.grammar.simulation_complete = seed["simulation"] == true;
            p.grammar.corruption_check = seed["corruption"] == true;
            let shared = |name: &str| {
                NativeSharedWidths::from_map(
                    serde_json::from_value::<BTreeMap<String, i64>>(
                        seed["component_widths"][name].clone(),
                    )
                    .unwrap(),
                )
            };
            p.grammar.calibrated_widths = Some(shared("calibrated"));
            p.grammar.stub_widths = Some(shared("stubs"));
        }
        context.set_scan_profile(p).unwrap();
        let mut result = None;
        let mut observed = Vec::new();
        let logs = super::log_test_support::capture_logs(|| {
            result = Some(scan_replay_pad_inputs(
                b"oracle-pad",
                &mut context,
                (row["no_bounds"] != true).then_some(&map),
                widths(&row["calibrated"]),
                |arch, a| {
                    assert_eq!(meta[a.source.chunk_index as usize].index, a.chunk);
                    observed.push(arch);
                },
            ));
        });
        let logs: Vec<_> = logs
            .into_iter()
            .filter(|v| {
                v["msg"]
                    .as_str()
                    .unwrap_or_default()
                    .starts_with("socles :")
            })
            .collect();
        assert_eq!(json!(logs), row["logs"], "logs {case}");
        let out = result.unwrap();
        assert_eq!(
            context.scan_profile().unwrap().mpp,
            widths(&row["inherited"]),
            "restore {case}"
        );
        assert_eq!(row["restored"], true);
        assert!(observed.windows(2).all(|w| w[0] >= w[1]));
        let format = source
            .registry_chunk()
            .map(|b| u32::from_le_bytes(b[4..8].try_into().unwrap()))
            .unwrap_or(0);
        assert_eq!(
            out.unknown_formats,
            if resolve_film_mpp(format).unknown_format {
                vec![format; 2]
            } else {
                vec![]
            }
        );
        for (i, (scan, name)) in [(&out.weapons, "weapons"), (&out.powerups, "powerups")]
            .into_iter()
            .enumerate()
        {
            let e = &row[name];
            let creations: Vec<EquipmentCreation> = list(&e["creations"]);
            let tracks: Vec<WorldObjectTrack> = list(&e["tracks"]);
            let stats: EquipmentCreationStats = serde_json::from_value(e["stats"].clone()).unwrap();
            let expected = FactsWorldObjectScan {
                scanned: e["scanned"].as_bool().unwrap(),
                creations: creations.iter().map(FactsEquipmentCreation::from).collect(),
                stats: FactsCreationStats::from(&stats),
                keyframes: FactsWorldKeyframes {
                    times_us: list(&e["times"]),
                    seen_us: e["seen"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|v| {
                            (
                                (
                                    v["slot"].as_u64().unwrap() as u32,
                                    v["generation"].as_u64().unwrap() as u32,
                                ),
                                list(&v["times_us"]),
                            )
                        })
                        .collect(),
                },
                tracks: tracks.iter().map(FactsProjectileTrack::from).collect(),
            };
            assert_eq!(scan.published, expected, "publication {case} {name}");
            assert_eq!(scan.error.is_none(), expected.scanned);
            if expected.scanned {
                successes[i] += 1;
                accepted[i] += creations.len();
                assert_eq!(
                    scan.census.archetypes[&scan.archetype]
                        .band
                        .iter()
                        .copied()
                        .collect::<Vec<_>>(),
                    list::<u32>(&e["band"])
                );
            } else {
                assert_eq!(scan.published, FactsWorldObjectScan::default());
            }
            assert_eq!(
                observed
                    .iter()
                    .filter(|&&arch| arch == scan.archetype)
                    .count(),
                scan.creations
                    .as_ref()
                    .map(|c| c.stats.anchors)
                    .unwrap_or(0)
            );
        }
    }
    assert!(successes.iter().all(|&n| n > 0));
    assert!(accepted.iter().all(|&n| n > 0));
}
