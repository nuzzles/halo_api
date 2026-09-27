//! Native kill-source map precision, axis diagnostic and handle-width calibration.
use super::*;
use crate::clients::hi::models::FilmChunkData;
use serde::{Deserialize, Serialize};
/// Remaining native returned-profile settings for the v41 kill decoder. These
/// describe its fixed reader policy; they are not mutable scan options.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KillCalibrationPolicy {
    pub traversal_region: u32,
    pub chain_inference: bool,
    pub generation_strict: bool,
    pub view_tables: bool,
    pub view_classes: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KillCalibration {
    /// Complete calibrated native profile. Kept alongside the legacy projection;
    /// absent in older exports rather than reconstructed from omitted settings.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native_profile: Option<NativeScanProfile>,
    /// None identifies older exports without retained native policy metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reader_policy: Option<KillCalibrationPolicy>,
    /// Dequantization values carried by the native returned profile, independent
    /// of whether the kill scan installs a position observer. The native observer
    /// range is not the played map's geometry. None marks older exports.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position_observer_context: Option<PositionCaptureEncoding>,
    pub read_axis_widths: [usize; 3],
    pub read_index_width: usize,
    pub axis_width: usize,
    pub handle_index_width: usize,
    pub handle_score: usize,
    pub handle_median: usize,
    pub handle_discriminated: bool,
    pub map_read: bool,
    pub corruption_control_read: bool,
    pub disagreements: usize,
    pub score: usize,
    pub median: usize,
    pub flat: bool,
    pub profile: KillWalkProfile,
}
impl std::fmt::Display for KillCalibration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let source = if self.flat {
            format!(
                "PROFIL PLAT (score {}, mediane {})",
                self.score, self.median
            )
        } else {
            format!("score {}, mediane {}", self.score, self.median)
        };
        let map = if self.map_read {
            "CARTE"
        } else {
            "DEFAUT (carte absente)"
        };
        let corruption = if self.corruption_control_read {
            format!(
                "controle_corruption={} [FILM]",
                self.profile.encoding.corruption_check
            )
        } else {
            "controle_corruption=INVARIANT (section d identification absente)".into()
        };
        let handle = if self.handle_discriminated {
            format!(
                "decidee [score {}, mediane {}]",
                self.handle_score, self.handle_median
            )
        } else {
            format!(
                "INVARIANT (non discriminee) [score {}, mediane {}]",
                self.handle_score, self.handle_median
            )
        };
        write!(
            f,
            "LU axisW=[{} {} {}] indexW_plage={} [{}] {} | ORACLE axisW={} [{}] desaccords={} | DECIDE indexW_poignee={} {}",
            self.read_axis_widths[0],
            self.read_axis_widths[1],
            self.read_axis_widths[2],
            self.read_index_width,
            map,
            corruption,
            self.axis_width,
            source,
            self.disagreements,
            self.handle_index_width,
            handle
        )
    }
}
/// Select the smallest best-scoring width only when it reaches twice max(median,1).
/// A flat measurement retains the invariant, even if another width scores higher.
pub fn select_kill_handle_width(scores: &[usize], invariant: usize) -> (usize, usize, usize, bool) {
    let mut order: Vec<_> = scores
        .iter()
        .copied()
        .enumerate()
        .map(|(i, s)| (i + 1, s))
        .collect();
    order.sort_by_key(|&(i, s)| (std::cmp::Reverse(s), i));
    let Some(&(width, best)) = order.first() else {
        return (invariant, 0, 0, false);
    };
    let median = order[order.len() / 2].1;
    if (best as f64) < 2.0 * (median.max(1) as f64) {
        (invariant, best, median, false)
    } else {
        (width, best, median, true)
    }
}
fn starting_calibration(chunks: &[FilmChunkData], map: Option<&FilmMapBounds>) -> KillCalibration {
    let mut native_profile = NativeScanProfile::default();
    native_profile.grammar.generation_strict = true;
    let mut map_read = false;
    if let Some(map) = map.filter(|m| m.axis_widths.iter().all(|&w| w > 0)) {
        map_read =
            map.axis_widths != [13, 13, 14] || map.region_index_bits.max(1) != 1 || map.region != 0;
        native_profile.set_world_precision(NativePrecisionDescriptor {
            index_bits: map.effective_region_index_bits() as u64,
            axis_bits: map.axis_widths.map(|width| width as u64),
            region: map.region,
        });
        native_profile.grammar.simulation_complete = true;
    }
    let position = native_profile
        .component_encoding()
        .expect("starting kill profile uses address-sized map widths");
    let simulation_complete = native_profile.grammar.simulation_complete;
    // Native GrammaireSousFilm resolves numbered chunk zero independently of
    // the kill timeline's first-buffer registry. First duplicate number wins.
    let identity = chunks.iter().find(|c| c.metadata.index == 0).and_then(|c| {
        let registry = parse_registry(&c.data)?;
        decode_film_identity(&c.data, &registry).ok().flatten()
    });
    let corruption =
        FilmCorruptionControl::from_recorded(identity.as_ref().map(|i| i.corruption_checks), false);
    let corruption_control_read = corruption.declared;
    let corruption_check = corruption.enabled;
    native_profile.grammar.corruption_check = corruption_check;
    KillCalibration {
        native_profile: Some(native_profile),
        // ProfilDeDepart keeps the native invariants and enables strict IDs;
        // calibration only changes precision and the recorded corruption gate.
        reader_policy: Some(KillCalibrationPolicy {
            traversal_region: 0,
            chain_inference: false,
            generation_strict: true,
            view_tables: true,
            view_classes: true,
        }),
        position_observer_context: Some(PositionCaptureEncoding {
            min_bits: NATIVE_QUANT_RANGE_CE_BIPED.map(|axis| axis[0].to_bits()),
            max_bits: NATIVE_QUANT_RANGE_CE_BIPED.map(|axis| axis[1].to_bits()),
            quantum_bits: NATIVE_DELTA_QUANTUM.to_bits(),
            region: *position.region_axis_bits.first_key_value().unwrap().0,
            axis_widths: position.world_axis_bits.unwrap(),
            region_index_bits: position.index_bits,
        }),
        read_axis_widths: position.world_axis_bits.unwrap(),
        read_index_width: position.index_bits,
        axis_width: 0,
        handle_index_width: 0,
        handle_score: 0,
        handle_median: 0,
        handle_discriminated: false,
        map_read,
        corruption_control_read,
        disagreements: 0,
        score: 0,
        median: 0,
        flat: false,
        profile: KillWalkProfile {
            encoding: FrameEncoding {
                keyframe_layout: Default::default(),
                keyframe_simulation_complete: None,
                native_id_low_bits: None,
                component_widths: Default::default(),
                new_record: Default::default(),
                position_capture: None,
                ids: RecordIdLayout {
                    low_bits: 13,
                    base: 0,
                },
                mpp_widths: [9, 5],
                position: Some(position),
                extra_fields: false,
                corruption_check,
            },
            simulation_complete,
        },
    }
}
/// Native replay fallback: the kill decoder's starting profile followed by map precision.
pub fn kill_replay_starting_profile(
    _registry: &FilmRegistry,
    map: Option<&FilmMapBounds>,
) -> KillWalkProfile {
    starting_calibration(&[], map).profile
}
/// Complete native starting settings for the replay fallback after kill refusal.
pub fn kill_replay_starting_native_profile(map: Option<&FilmMapBounds>) -> NativeScanProfile {
    starting_calibration(&[], map).native_profile.unwrap()
}
fn count_bipeds(
    sample: &[&[u8]],
    timeline: &mut KillTimeline,
    profile: &KillWalkProfile,
    views: usize,
) -> usize {
    let mut count = 0;
    for data in sample {
        count += walk_march_with_policy(
            data,
            2,
            &timeline.registry,
            &profile.encoding,
            &timeline.world,
            MarchWalkPolicy {
                views,
                generation_strict: true,
                simulation_complete: profile.simulation_complete,
            },
        )
        .iter()
        .filter(|r| r.archetype == Some(35) && r.stop == EntityViewStop::Complete)
        .count();
        timeline.detach_snapshot_alias();
    }
    count
}
/// Calibrate using at most 400 eventless replication packets of at least 400
/// bytes, in source order. The axis sweep diagnoses the map and never replaces
/// its widths; only a discriminating handle-width sweep changes the profile.
pub fn calibrate_kill_walk(
    chunks: &[FilmChunkData],
    timeline: &mut KillTimeline,
    views: usize,
    map: Option<&FilmMapBounds>,
) -> KillCalibration {
    let mut result = starting_calibration(chunks, map);
    let mut sample = Vec::new();
    'chunks: for c in chunks {
        for p in fire_events::native_chunk_packets(c) {
            let data = &c.data[p.payload_offset..p.payload_offset + p.payload_size];
            if p.packet_type != 0 || bits::Bits(data).read(1, 1) == Some(1) || data.len() < 400 {
                continue;
            }
            sample.push(data);
            if sample.len() >= 400 {
                break 'chunks;
            }
        }
    }
    let mut scores = Vec::new();
    for width in 6..=26 {
        let mut profile = result.profile.clone();
        let pos = profile.encoding.position.as_mut().unwrap();
        pos.world_axis_bits = Some([width; 3]);
        for axes in pos.region_axis_bits.values_mut() {
            *axes = [width; 3];
        }
        scores.push((width, count_bipeds(&sample, timeline, &profile, views)));
    }
    scores.sort_by_key(|&(w, s)| (std::cmp::Reverse(s), w));
    result.axis_width = scores[0].0;
    result.score = scores[0].1;
    result.median = scores[scores.len() / 2].1;
    result.flat = (result.score as f64) < 2.0 * (result.median.max(1) as f64);
    if !result.flat
        && (result.axis_width < *result.read_axis_widths.iter().min().unwrap()
            || result.axis_width > *result.read_axis_widths.iter().max().unwrap())
    {
        result.disagreements += 1;
    }
    let mut handles = Vec::new();
    for width in 1..=3 {
        let mut profile = result.profile.clone();
        profile.encoding.position.as_mut().unwrap().handle_bits = width;
        handles.push(count_bipeds(&sample, timeline, &profile, views));
    }
    let (width, score, median, discriminated) = select_kill_handle_width(
        &handles,
        result
            .profile
            .encoding
            .position
            .as_ref()
            .unwrap()
            .handle_bits,
    );
    result.handle_index_width = width;
    result.handle_score = score;
    result.handle_median = median;
    result.handle_discriminated = discriminated;
    result
        .profile
        .encoding
        .position
        .as_mut()
        .unwrap()
        .handle_bits = width;
    result
        .native_profile
        .as_mut()
        .unwrap()
        .movement
        .traversal
        .index_bits = width as u64;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[derive(Deserialize)]
    struct Record {
        id: u32,
        kind: u8,
        ti: u32,
        end: usize,
        desync: i64,
        mask: u64,
    }
    #[derive(Deserialize)]
    struct Row {
        data: String,
        local_data: String,
        located: i64,
        views: usize,
        simulation: bool,
        records: Vec<Record>,
        deaths: Vec<KillDeadRecord>,
        scores: Vec<usize>,
        retained: usize,
        score: usize,
        median: usize,
        discriminated: bool,
    }
    #[test]
    fn native_kill_starting_profile_contract() {
        use serde_json::json;
        let oracle: serde_json::Value =
            serde_json::from_str(include_str!("fixtures/starting-profile-v41.json")).unwrap();
        let result = starting_calibration(&[], None);
        assert_eq!(
            result.native_profile.as_ref().unwrap(),
            &native_scan_profile::tests::profile(&oracle["starting"])
        );
        let frame = &result.profile.encoding;
        let position = frame.position.as_ref().unwrap();
        let policy = result.reader_policy.as_ref().unwrap();
        let capture = result.position_observer_context.as_ref().unwrap();
        let actual = json!({
            "Mouvement": {
                "Traversal": {"IndexW": position.handle_bits, "AxisW": position.traversal_axis_bits, "Region": policy.traversal_region},
                "WorldObject": {"IndexW": position.index_bits, "AxisW": position.world_axis_bits.unwrap(), "Region": capture.region},
                "DeltaQuantum": f32::from_bits(capture.quantum_bits),
                "DeltaAxisWidth": position.delta_axis_bits[0],
                "Range": (0..3).map(|axis| json!({"Min": f32::from_bits(capture.min_bits[axis]), "Max": f32::from_bits(capture.max_bits[axis])})).collect::<Vec<_>>(),
                "FullPrecision": position.full_precision,
                "DeltaHasHandleTail": position.delta_handle_tail,
                "CalibratedSkip": position.calibrated_skip,
                "MobilityActionExtraBits": position.bodies.mobility_extra_bits
            },
            "Cadre": {"EnTeteBits": frame.keyframe_layout.header_bits, "MotDeTailleBits": frame.keyframe_layout.size_word_bits},
            "MPP": {"Lead": frame.mpp_widths[0], "Index": frame.mpp_widths[1]},
            "Grammaire": {
                "ControleDeCorruption": frame.corruption_check,
                "BitsDeQueueRecordNew": frame.new_record.terminal_bits,
                "DeserEtatParArchetype": frame.new_record.deserialize_defaults,
                "SimStateComplet": result.profile.simulation_complete,
                "PorteeBaseline": position.baseline_scope,
                "GrammaireEcrivainI0": position.writer_absolute,
                "CorpsActionMobilite": position.bodies.mobility,
                "CorpsAncrageCapacite": position.bodies.ability_anchor,
                "InferenceChaine": policy.chain_inference,
                "LargeursCalibrees": if frame.component_widths.calibrated.is_empty() { serde_json::Value::Null } else { json!(frame.component_widths.calibrated) },
                "GenerationStricte": policy.generation_strict,
                "TablesParVue": policy.view_tables,
                "ClassesDeVue": policy.view_classes,
                "LargeursBouchon": if frame.component_widths.stubs.is_empty() { serde_json::Value::Null } else { json!(frame.component_widths.stubs) }
            }
        });
        let mut expected = oracle["starting"].clone();
        // Go emits shortest f32 decimals; compare their f32 values, not the
        // differing decimal-to-f64 intermediates of the JSON implementations.
        for axis in 0..3 {
            for key in ["Min", "Max"] {
                let v = &mut expected["Mouvement"]["Range"][axis][key];
                *v = json!(v.as_f64().unwrap() as f32);
            }
        }
        let quantum = &mut expected["Mouvement"]["DeltaQuantum"];
        *quantum = json!(quantum.as_f64().unwrap() as f32);
        assert_eq!(actual, expected);
        assert_eq!(position.delta_axis_bits, [position.delta_axis_bits[0]; 3]);
        assert_eq!(position.region_axis_bits.len(), 1);
        assert_eq!(
            position.region_axis_bits[&capture.region],
            capture.axis_widths
        );
        let mut default = oracle["default"].clone();
        assert_eq!(default["Grammaire"]["GenerationStricte"], false);
        default["Grammaire"]["GenerationStricte"] = json!(true);
        assert_eq!(default, oracle["starting"]);
    }

    #[test]
    fn native_calibration_corruption_source_selection() {
        let mut bootstrap = Vec::new();
        flate2::read::ZlibDecoder::new(include_bytes!("fixtures/bootstrap-v41.zlib").as_slice())
            .read_to_end(&mut bootstrap)
            .unwrap();
        let registry = parse_registry(&bootstrap).unwrap();
        let identity = decode_film_identity(&bootstrap, &registry)
            .unwrap()
            .unwrap();
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            include_bytes!("fixtures/corruption-source-v41.json.zlib").as_slice(),
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(rows.len(), 216);
        for (i, row) in rows.iter().enumerate() {
            let chunks: Vec<_> = row["order"]
                .as_array()
                .unwrap()
                .iter()
                .zip(row["variants"].as_array().unwrap())
                .map(|(number, variant)| {
                    let mut data = bootstrap.clone();
                    data[identity.build_offset + 0x48] &= 0x7f;
                    match variant.as_u64().unwrap() {
                        0 => {}
                        1 => data[identity.build_offset + 0x48] |= 0x80,
                        2 => data[identity.build_offset..identity.build_offset + 3].fill(0),
                        _ => unreachable!(),
                    }
                    FilmChunkData {
                        metadata: crate::clients::hi::models::FilmChunk {
                            index: number.as_i64().unwrap() as i32,
                            chunk_type: 1,
                            start_time_offset_ms: 0,
                            duration_ms: 0,
                            size: data.len() as i64,
                            file_relative_path: String::new(),
                        },
                        data,
                    }
                })
                .collect();
            let result = starting_calibration(&chunks, None);
            assert_eq!(
                serde_json::to_value(result.position_observer_context.as_ref().unwrap()).unwrap(),
                row["capture"],
                "observer profile {i}"
            );
            let mut json = serde_json::to_value(&result).unwrap();
            assert_eq!(
                serde_json::from_value::<KillCalibration>(json.clone()).unwrap(),
                result
            );
            json.as_object_mut()
                .unwrap()
                .remove("position_observer_context");
            json.as_object_mut().unwrap().remove("native_profile");
            json.as_object_mut().unwrap().remove("reader_policy");
            let legacy = serde_json::from_value::<KillCalibration>(json).unwrap();
            assert!(legacy.position_observer_context.is_none());
            assert!(legacy.native_profile.is_none());
            assert!(legacy.reader_policy.is_none());
            assert_eq!(result.corruption_control_read, row["read"], "read {i}");
            assert_eq!(
                result.profile.encoding.corruption_check, row["enabled"],
                "enabled {i}"
            );
        }
    }

    #[test]
    fn native_kill_walk_policy_oracle() {
        let mut json = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/kill-walk-policy-v41.json.zlib")[..],
        )
        .read_to_end(&mut json)
        .unwrap();
        let rows: Vec<Row> = serde_json::from_slice(&json).unwrap();
        assert_eq!(rows.len(), 768);
        let registry = FilmRegistry {
            archetypes: (0..36)
                .map(|index| FilmArchetype {
                    index,
                    components: if index == 35 {
                        vec![
                            "object-dead-state-component".into(),
                            "simulation-state".into(),
                            "high-frequency".into(),
                        ]
                    } else {
                        vec![]
                    },
                    levels: vec![0, 0, 1],
                })
                .collect(),
            major_version: 41,
            format_version: 0,
            end_byte: 0,
            truncated: false,
        };
        for (i, row) in rows.into_iter().enumerate() {
            let data: Vec<u8> = row
                .data
                .as_bytes()
                .as_chunks::<2>()
                .0
                .iter()
                .map(|c| u8::from_str_radix(std::str::from_utf8(c).unwrap(), 16).unwrap())
                .collect();
            let local_data: Vec<u8> = row
                .local_data
                .as_bytes()
                .as_chunks::<2>()
                .0
                .iter()
                .map(|c| u8::from_str_radix(std::str::from_utf8(c).unwrap(), 16).unwrap())
                .collect();
            let mut profile = starting_calibration(&[], None).profile;
            profile.simulation_complete = row.simulation;
            let mut world = FilmWorld::default();
            world.bind_full((1 << 30) | 520, 35);
            let mut local_world = world.clone();
            local_world.bind_full((1 << 30) | 123, 35);
            assert_eq!(
                locate_march_event_records_with_simulation_policy(
                    &local_data,
                    &registry,
                    &profile.encoding,
                    &local_world,
                    true,
                    row.simulation
                )
                .map(|p| p as i64)
                .unwrap_or(-1),
                row.located,
                "locator {i}"
            );
            let recs = walk_march_with_policy(
                &data,
                2,
                &registry,
                &profile.encoding,
                &world,
                MarchWalkPolicy {
                    views: row.views,
                    generation_strict: true,
                    simulation_complete: row.simulation,
                },
            );
            assert_eq!(recs.len(), row.records.len(), "count {i}");
            for (j, (r, e)) in recs.iter().zip(row.records).enumerate() {
                let desync = match r.stop {
                    EntityViewStop::Complete => -1,
                    EntityViewStop::UnsupportedComponent { index, .. }
                    | EntityViewStop::InvalidComponent { index } => index as i64,
                    _ => 0,
                };
                assert_eq!(
                    match r.header.kind {
                        RecordKind::End => 0,
                        RecordKind::New => 1,
                        RecordKind::Delete => 2,
                        RecordKind::Delta => 3,
                    },
                    e.kind
                );
                assert_eq!(
                    serde_json::json!((
                        r.header.id.unwrap_or(0),
                        r.archetype.unwrap_or(0),
                        // Native delete records leave Trace.EndBit unset.
                        if r.header.kind == RecordKind::Delete {
                            0
                        } else {
                            r.end_bit
                        },
                        desync,
                        r.mask.unwrap_or(0)
                    )),
                    serde_json::json!((e.id, e.ti, e.end, e.desync, e.mask)),
                    "record {i}/{j}"
                );
            }
            assert_eq!(
                walk_kill_packet(
                    &data,
                    &world,
                    &registry,
                    &profile,
                    KillPacketWalkOptions {
                        start: 2,
                        views: row.views,
                        time_ms: 100,
                        packet: KillPacketIdentity {
                            chunk: 2,
                            packet: 7
                        }
                    }
                ),
                row.deaths,
                "deaths {i}"
            );
            assert_eq!(
                select_kill_handle_width(&row.scores, 2),
                (row.retained, row.score, row.median, row.discriminated),
                "width {i}"
            );
        }
    }
}
