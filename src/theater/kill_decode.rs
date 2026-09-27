//! Complete native v41 kill-source orchestration; publication remains explicitly gated.
use super::*;
use crate::clients::hi::models::FilmChunkData;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KillDecodeOptions {
    pub bots: bool,
    pub self_source: bool,
    /// Native signed count; nonpositive values select the default.
    pub multiplicity_max: i64,
    pub strong_tag_required: bool,
    /// Native signed count; nonpositive values select the default.
    pub bijection_restarts: i64,
    /// Native signed count; nonpositive values select the default.
    pub views: i64,
}
impl Default for KillDecodeOptions {
    fn default() -> Self {
        Self {
            bots: true,
            self_source: true,
            multiplicity_max: 2,
            strong_tag_required: true,
            bijection_restarts: 40,
            views: 8,
        }
    }
}
impl KillDecodeOptions {
    /// Apply native defaults without changing the caller's supplied settings.
    /// Booleans are retained, including an explicitly all-false configuration.
    pub fn normalized(&self) -> Self {
        let mut o = self.clone();
        let d = Self::default();
        if o.multiplicity_max <= 0 {
            o.multiplicity_max = d.multiplicity_max;
        }
        if o.bijection_restarts <= 0 {
            o.bijection_restarts = d.bijection_restarts;
        }
        if o.views <= 0 {
            o.views = d.views;
        }
        o
    }
}

/// An explicit portability boundary, not a native source-data refusal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("kill-source option {field}={value} exceeds this target's execution index domain")]
pub struct KillDecodeOptionError {
    pub field: &'static str,
    pub value: i64,
}

fn execution_count(field: &'static str, value: i64) -> Result<usize, KillDecodeOptionError> {
    usize::try_from(value).map_err(|_| KillDecodeOptionError { field, value })
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilmKillSourceResult {
    #[serde(default)]
    pub icons: FilmKillIcons,
    /// Raw per-line attribution and all hybrid diagnostics. Consult the publication gate.
    pub attribution: KillHybridResult,
    pub coverage: KillSourceCoverage,
    pub health: KillSourceHealth,
    pub assist_stats: KillAssistStats,
    pub pair_stats: KillPairStats,
    pub roster: KillRoster,
    pub calibration: KillCalibration,
    pub bijection_margin: i64,
    pub bijection_determined: bool,
    pub packets_with_events: usize,
    pub packets_located: usize,
    pub probe: Option<KillRelaxedProbe>,
}
impl FilmKillSourceResult {
    pub fn line_by_line_publishable(&self) -> bool {
        self.health
            .line_by_line_publishable(self.bijection_margin, self.bijection_determined)
    }
}
pub fn decode_film_kill_sources(
    name: &str,
    chunks: &[FilmChunkData],
    major_version: i32,
    map: Option<&FilmMapBounds>,
    options: &KillDecodeOptions,
) -> Result<FilmKillSourceResult, DecodeError> {
    if major_version != 41 {
        return Err(DecodeError::UnsupportedVersion(major_version));
    }
    super::kill_source_film::validate_kill_source_chunks(chunks)?;
    let options = options.normalized();
    let restarts = execution_count("bijection_restarts", options.bijection_restarts)?;
    let views = execution_count("views", options.views)?;
    let multiplicity_max = execution_count("multiplicity_max", options.multiplicity_max)?;
    let evidence = prepare_kill_film_evidence(chunks, major_version, options.bots, restarts)?;
    let mut timeline = KillTimeline::from_chunks(chunks)?;
    timeline.rewind();
    let calibration = calibrate_kill_walk(chunks, &mut timeline, views, map);
    let walk = run_kill_record_walk(
        chunks,
        &mut timeline,
        &calibration.profile,
        views,
        evidence.roster.player_count,
    )?;
    let ctx = KillMatchingContext {
        roster: &evidence.roster,
        feed: &evidence.pairs,
        scan: &evidence.candidates,
    };
    let (candidates, no_bit) = walk.candidates();
    let mut attribution = attribute_kill_sources(
        &ctx,
        &candidates,
        no_bit,
        options.self_source,
        multiplicity_max,
        options.strong_tag_required,
    );
    let coverage = kill_source_coverage(&ctx, &evidence.feed, &attribution);
    let mut health = measure_kill_source_health(name, &ctx, &walk, &attribution, &coverage);
    let names: Vec<_> = (0..evidence.roster.player_count)
        .map(|i| evidence.roster.name_of(i as i32).to_string())
        .collect();
    let events: Vec<_> = evidence
        .kill_events
        .events
        .iter()
        .map(|e| KillAssistEvent {
            time_ms: e.time_ms,
            packet: e.packet,
            fields: e.fields,
        })
        .collect();
    let mut targets: Vec<_> = attribution
        .kills
        .iter()
        .map(|k| KillAssistTarget {
            time_ms: k.time_ms,
            victim: k.victim.clone(),
            killer: k.feed_present.then(|| k.killer.clone()),
            packet: k.packet,
            ..Default::default()
        })
        .collect();
    let assist_stats =
        attach_kill_assists(&mut targets, &events, &names, evidence.kill_events.gate15);
    for (k, t) in attribution.kills.iter_mut().zip(targets) {
        k.assist = t.assist;
        k.killer_damage = t.killer_damage;
        k.assist_damage = t.assist_damage;
    }
    let probe = if (coverage.covered as i64) < coverage.real_pairs {
        let candidates =
            scan_film_relaxed_kill_sources(chunks, major_version, evidence.roster.player_count)?;
        let covered = attribution.kills.iter().map(|k| k.time_ms).collect();
        let probe = probe_relaxed_kill_candidates(&ctx, &candidates, &covered);
        health.tag_out_of_catalogue_scan = probe.uncovered;
        Some(probe)
    } else {
        None
    };
    Ok(FilmKillSourceResult {
        icons: FilmKillIcons::from_attribution(&attribution),
        attribution,
        coverage,
        health,
        assist_stats,
        pair_stats: evidence.pairs.stats,
        bijection_determined: evidence.roster.table.assignment_unique(),
        roster: evidence.roster,
        calibration,
        bijection_margin: evidence.bijection_margin,
        packets_with_events: walk.with_events,
        packets_located: walk.located,
        probe,
    })
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;
    use crate::clients::hi::models::FilmChunk;
    use serde_json::{Value, json};
    use std::{fs, io::Read, path::Path};
    #[test]
    fn native_kill_option_normalization() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            include_bytes!("fixtures/kill-options-v41.json.zlib").as_slice(),
        )
        .read_to_end(&mut raw)
        .unwrap();
        let fixture: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        assert_eq!(
            serde_json::to_value(KillDecodeOptions::default()).unwrap(),
            fixture["default"]
        );
        let rows = fixture["cases"].as_array().unwrap();
        assert_eq!(rows.len(), 512);
        for (i, row) in rows.iter().enumerate() {
            let input: KillDecodeOptions = serde_json::from_value(row["input"].clone()).unwrap();
            let original = input.clone();
            let expected: KillDecodeOptions =
                serde_json::from_value(row["normalized"].clone()).unwrap();
            assert_eq!(input.normalized(), expected, "case {i}");
            assert_eq!(input, original, "caller settings {i}");
            assert_eq!(
                serde_json::from_str::<KillDecodeOptions>(&serde_json::to_string(&input).unwrap())
                    .unwrap(),
                input
            );
            assert_eq!(expected.normalized(), expected, "idempotence {i}");
        }
    }
    #[test]
    fn kill_option_execution_domain() {
        let options = KillDecodeOptions {
            views: 1i64 << 32,
            ..Default::default()
        };
        assert!(matches!(
            decode_film_kill_sources("domain", &[], 41, None, &options).unwrap_err(),
            DecodeError::KillSource(KillSourceFilmError::NoChunk)
        ));
        let mut data = vec![0u8; 17];
        data[4] = 1; // one-byte type-zero packet, enough to reach preparation
        let chunk = crate::clients::hi::models::FilmChunkData {
            metadata: crate::clients::hi::models::FilmChunk {
                index: 0,
                chunk_type: 0,
                start_time_offset_ms: 0,
                duration_ms: 0,
                size: 0,
                file_relative_path: String::new(),
            },
            data,
        };
        let error = decode_film_kill_sources("domain", &[chunk], 41, None, &options).unwrap_err();
        #[cfg(target_pointer_width = "64")]
        assert!(matches!(
            error,
            DecodeError::KillSource(KillSourceFilmError::NoKillFeed)
        ));
        #[cfg(target_pointer_width = "32")]
        assert!(matches!(
            error,
            DecodeError::KillOption(KillDecodeOptionError {
                field: "views",
                value: 4294967296,
            })
        ));
        assert_eq!(options.views, 4294967296);
    }
    fn normalize(v: Value) -> Value {
        match v {
            Value::Array(a) => Value::Array(a.into_iter().map(normalize).collect()),
            Value::Object(o) => Value::Object(
                o.into_iter()
                    .map(|(k, v)| {
                        let alias = match k.as_str() {
                            "TimeMS" => "time_ms",
                            "BotID" => "bot_id",
                            "VictimXUID" => "victim_xuid",
                            "NoBit" => "no_bit",
                            "UnexplainedBotIdx" => "unexplained_bot",
                            "ParLaFenetre" => "window_fallback",
                            "MemeInstant" => "same_instant",
                            "Lus" => "read",
                            "Recolles" => "neighboring_fallback",
                            "Perdus" => "lost",
                            "VictimesBotLues" => "bot_victims_read",
                            "Muet" => "silent",
                            "Ambigu" => "ambiguous",
                            "Accord" => "agree",
                            "Contradiction" => "contradict",
                            "Identite" => "identity",
                            "Fenetre" => "window",
                            "BotFenetre" => "bot_window",
                            "NonRevendiqueeFenetre" => "unclaimed_window",
                            "CouplesSansIdentite" => "pairs_without_identity",
                            _ => "",
                        };
                        let key = if alias.is_empty() {
                            k.chars()
                                .enumerate()
                                .flat_map(|(i, c)| {
                                    let mut s = String::new();
                                    if i > 0 && c.is_uppercase() {
                                        s.push('_');
                                    }
                                    s.extend(c.to_lowercase());
                                    s.chars().collect::<Vec<_>>()
                                })
                                .collect()
                        } else {
                            alias.into()
                        };
                        (key, normalize(v))
                    })
                    .collect(),
            ),
            v => v,
        }
    }
    pub(crate) fn assert_native_result(
        result: &FilmKillSourceResult,
        expected_raw: &Value,
        publishable: bool,
        folder: &str,
    ) {
        let calibration = &result.calibration;
        assert_eq!(
            calibration.native_profile.as_ref().unwrap(),
            &native_scan_profile::tests::profile(&expected_raw["ProfilCalibre"])
        );
        let profile = &calibration.profile;
        let encoding = &profile.encoding;
        let position = encoding.position.as_ref().unwrap();
        let capture = calibration.position_observer_context.as_ref().unwrap();
        let policy = calibration.reader_policy.as_ref().unwrap();
        assert_eq!(position.delta_axis_bits, [position.delta_axis_bits[0]; 3]);
        assert_eq!(position.world_axis_bits, Some(capture.axis_widths));
        assert_eq!(position.index_bits, capture.region_index_bits);
        assert_eq!(
            position.region_axis_bits,
            [(capture.region, capture.axis_widths)].into()
        );
        let range: Vec<_> = (0..3)
            .map(|i| {
                json!({
                    "Min": f32::from_bits(capture.min_bits[i]),
                    "Max": f32::from_bits(capture.max_bits[i]),
                })
            })
            .collect();
        // Reconstruct every native field from retained Rust values. Whole-object
        // equality rejects extra or missing fields as well as mismatched values.
        let actual_profile = json!({
            "Mouvement": {
                "Traversal": {"IndexW": position.handle_bits, "AxisW": position.traversal_axis_bits, "Region": policy.traversal_region},
                "WorldObject": {"IndexW": capture.region_index_bits, "AxisW": capture.axis_widths, "Region": capture.region},
                "DeltaQuantum": f32::from_bits(capture.quantum_bits),
                "DeltaAxisWidth": position.delta_axis_bits[0], "Range": range,
                "FullPrecision": position.full_precision, "DeltaHasHandleTail": position.delta_handle_tail,
                "CalibratedSkip": position.calibrated_skip, "MobilityActionExtraBits": position.bodies.mobility_extra_bits,
            },
            "Cadre": {"EnTeteBits": encoding.keyframe_layout.header_bits, "MotDeTailleBits": encoding.keyframe_layout.size_word_bits},
            "MPP": {"Lead": encoding.mpp_widths[0], "Index": encoding.mpp_widths[1]},
            "Grammaire": {
                "ControleDeCorruption": encoding.corruption_check, "BitsDeQueueRecordNew": encoding.new_record.terminal_bits,
                "DeserEtatParArchetype": encoding.new_record.deserialize_defaults, "SimStateComplet": profile.simulation_complete,
                "PorteeBaseline": position.baseline_scope, "GrammaireEcrivainI0": position.writer_absolute,
                "CorpsActionMobilite": position.bodies.mobility, "CorpsAncrageCapacite": position.bodies.ability_anchor,
                "InferenceChaine": policy.chain_inference, "GenerationStricte": policy.generation_strict,
                "TablesParVue": policy.view_tables, "ClassesDeVue": policy.view_classes,
                "LargeursCalibrees": encoding.component_widths.calibrated, "LargeursBouchon": encoding.component_widths.stubs,
            },
        });
        let mut native_profile = expected_raw["ProfilCalibre"].clone();
        // Go and Rust JSON emit different shortest forms for float32 values.
        // Compare both through their actual float32 representation. Native nil
        // width maps and Rust empty maps represent the same no-override policy.
        for axis in native_profile["Mouvement"]["Range"].as_array_mut().unwrap() {
            for key in ["Min", "Max"] {
                axis[key] = json!(axis[key].as_f64().unwrap() as f32);
            }
        }
        native_profile["Mouvement"]["DeltaQuantum"] = json!(
            native_profile["Mouvement"]["DeltaQuantum"]
                .as_f64()
                .unwrap() as f32
        );
        for key in ["LargeursCalibrees", "LargeursBouchon"] {
            if native_profile["Grammaire"][key].is_null() {
                native_profile["Grammaire"][key] = json!({});
            }
        }
        assert_eq!(
            actual_profile, native_profile,
            "{folder}: complete returned profile"
        );
        let restored: FilmKillSourceResult =
            serde_json::from_value(serde_json::to_value(result).unwrap()).unwrap();
        assert_eq!(&restored, result, "{folder}: complete result export");
        let expected = normalize(expected_raw.clone());
        let h = &result.attribution;
        let kills:Vec<_>=h.kills.iter().map(|k|json!({"time_ms":k.time_ms,"victim":k.victim,"feed":{"killer":k.killer,"present":k.feed_present},"source":k.source,"diverges":k.diverges,"read":k.read,"assist":k.assist,"killer_damage":k.killer_damage,"assist_damage":k.assist_damage})).collect();
        let stats = json!({"walk":h.walk,"scan":h.scan,"self_walk":h.self_walk,"self_scan":h.self_scan,"bot":h.bot,"bot_killer":h.bot_killer,"unclaimed":h.unclaimed_stats,"redundant":h.redundant,"no_bit":h.no_bit,"agree":h.agree,"disagree":h.disagree,"multi_candidate":h.multi_candidate,"packets_with_events":result.packets_with_events,"packets_located":result.packets_located,"assist":result.assist_stats,"couples":result.pair_stats,"appariement":h.matching});
        let mut actual = json!({"kills":kills,"unclaimed_deaths":h.unclaimed,"coverage":result.coverage,"health":result.health,"stats":stats,"calibration":result.calibration.to_string(),"bijection_margin":result.bijection_margin,"bijection_determined":result.bijection_determined,"probe":result.probe});
        let roster = &result.roster;
        actual["roster"] = json!({"names":roster.evidence.names,"humans":roster.humans,"bots":roster.bots,"index_to_name":(0..roster.player_count).map(|i|roster.name_of(i as i32)).collect::<Vec<_>>(),"unpinned_bots":roster.unpinned_bots,"bots_succedes":roster.bots_succeeded,"index_source":(0..roster.player_count).map(|i|roster.origin_of(i as i32)).collect::<Vec<_>>(),"film_table":roster.table});
        // The complete returned profile was compared above.
        // Compare every kill, assist, statistic, and public roster field here.
        let mut wanted = expected.as_object().unwrap().clone();
        wanted.remove("profil_calibre");
        for field in ["bots", "unpinned_bots"] {
            if wanted["roster"][field].is_null() {
                wanted.get_mut("roster").unwrap()[field] = json!([]);
            }
        }
        if wanted["unclaimed_deaths"].is_null() {
            wanted.insert("unclaimed_deaths".into(), json!([]));
        }
        let wanted = Value::Object(wanted);
        fs::write(
            format!(
                "/private/tmp/halo-kill-decode-{}-actual.json",
                folder.replace('/', "-")
            ),
            serde_json::to_vec_pretty(&actual).unwrap(),
        )
        .unwrap();
        fs::write(
            format!(
                "/private/tmp/halo-kill-decode-{}-expected.json",
                folder.replace('/', "-")
            ),
            serde_json::to_vec_pretty(&wanted).unwrap(),
        )
        .unwrap();
        assert_eq!(actual, wanted, "{folder}");
        assert_eq!(result.line_by_line_publishable(), publishable);
        println!("kill decode matches {folder}: {} kills", h.kills.len());
    }
    #[test]
    #[ignore = "requires downloaded v41 captures; complete native kill-source decoder"]
    fn local_kill_source_decode() {
        compare_captured_kill_options(&KillDecodeOptions::default());
    }
    #[test]
    #[ignore = "requires downloaded v41 captures; native nonpositive-option defaults"]
    fn local_kill_source_decode_nonpositive_options() {
        compare_captured_kill_options(&KillDecodeOptions {
            multiplicity_max: i64::MIN,
            bijection_restarts: -40,
            views: 0,
            ..Default::default()
        });
    }
    fn compare_captured_kill_options(options: &KillDecodeOptions) {
        let mut bytes = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/kill-decode-v41.json.zlib")[..])
            .read_to_end(&mut bytes)
            .unwrap();
        let rows: Vec<Value> = serde_json::from_slice(&bytes).unwrap();
        for row in rows {
            let folder = row["folder"].as_str().unwrap();
            let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("experiments/films")
                .join(folder);
            let meta: Value =
                serde_json::from_slice(&fs::read(dir.join("film.json")).unwrap()).unwrap();
            let chunks: Vec<_> = meta["chunks"]
                .as_array()
                .unwrap()
                .iter()
                .map(|c| FilmChunkData {
                    metadata: FilmChunk {
                        index: c["index"].as_i64().unwrap() as i32,
                        chunk_type: c["chunk_type"].as_i64().unwrap() as i32,
                        start_time_offset_ms: c["start_time_offset_ms"].as_i64().unwrap(),
                        duration_ms: c["duration_ms"].as_i64().unwrap(),
                        size: 0,
                        file_relative_path: String::new(),
                    },
                    data: fs::read(dir.join(c["file"].as_str().unwrap())).unwrap(),
                })
                .collect();
            let catalog = film_map_catalog();
            let result = decode_film_kill_sources(
                folder,
                &chunks,
                41,
                Some(&catalog.maps["recharge"]),
                options,
            )
            .unwrap();
            assert_native_result(
                &result,
                &row["result"],
                row["publishable"].as_bool().unwrap(),
                folder,
            );
        }
    }
}
