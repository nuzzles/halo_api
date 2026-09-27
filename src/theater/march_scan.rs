//! Chronological keyframe binding, native frame-width calibration and march facts.
use super::*;
use crate::clients::hi::models::FilmChunkData;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarchKeyframe {
    pub timestamp_us: u64,
    /// Full entity ID (generation included) and archetype, in native anchor order.
    pub bindings: Vec<(u32, u32)>,
}
#[derive(Debug, Clone, Copy)]
pub struct MarchDelta<'a> {
    pub timestamp_us: u64,
    pub payload: &'a [u8],
}
/// First declarations seed even entities only observed in a later keyframe.
/// Subsequent keyframes replace bindings in stable timestamp order; omitted
/// slots are not removed. Advance calls must be chronological.
pub struct MarchTimeline {
    keyframes: Vec<MarchKeyframe>,
    cursor: usize,
    world: FilmWorld,
}
impl MarchTimeline {
    pub fn new(keyframes: &[MarchKeyframe]) -> Self {
        let mut keyframes = keyframes.to_vec();
        keyframes.sort_by_key(|k| k.timestamp_us);
        let mut seen = BTreeSet::new();
        let mut world = FilmWorld::default();
        for k in &keyframes {
            for &(id, ti) in &k.bindings {
                if seen.insert(id & 0x3fff_ffff) {
                    world.bind_full(id, ti);
                }
            }
        }
        Self {
            keyframes,
            cursor: 0,
            world,
        }
    }
    pub fn advance_to(&mut self, timestamp_us: u64) -> &FilmWorld {
        while let Some(k) = self
            .keyframes
            .get(self.cursor)
            .filter(|k| k.timestamp_us <= timestamp_us)
        {
            for &(id, ti) in &k.bindings {
                self.world.bind_full(id, ti);
            }
            self.cursor += 1;
        }
        &self.world
    }
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarchCalibrationScore {
    pub id_low_bits: usize,
    pub located: usize,
    pub events: usize,
    pub packets: usize,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarchCalibration {
    /// Checked convenience projection. Native settings may be unrepresentable;
    /// the complete calibrated settings remain in FilmMarchFacts.native_config.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encoding: Option<FrameEncoding>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encoding_error: Option<String>,
    pub retained_default: bool,
    pub best: MarchCalibrationScore,
    pub runner_up: MarchCalibrationScore,
}
fn packet_start(
    delta: MarchDelta<'_>,
    registry: &FilmRegistry,
    encoding: &FrameEncoding,
    world: &FilmWorld,
    generation_strict: bool,
    simulation_complete: bool,
) -> (Option<usize>, bool) {
    let events = super::bits::Bits(delta.payload).read(1, 1) == Some(1);
    if !events {
        (Some(MARCH_PACKET_PREAMBLE_BITS), false)
    } else {
        (
            locate_march_event_records_with_simulation_policy(
                delta.payload,
                registry,
                encoding,
                world,
                generation_strict,
                simulation_complete,
            ),
            true,
        )
    }
}
/// Deltas must be in stable chronological order. The native criterion is event
/// localization alone. Ties prefer the supplied width, then the smaller width.
/// A winner must have at least twice max(runner-up, 1) localized event packets.
pub fn calibrate_march_encoding(
    registry: &FilmRegistry,
    keyframes: &[MarchKeyframe],
    deltas: &[MarchDelta<'_>],
    base: &FrameEncoding,
    generation_strict: bool,
) -> MarchCalibration {
    calibrate_march_encoding_with_simulation_policy(
        registry,
        keyframes,
        deltas,
        base,
        generation_strict,
        true,
    )
}
pub fn calibrate_march_encoding_with_simulation_policy(
    registry: &FilmRegistry,
    keyframes: &[MarchKeyframe],
    deltas: &[MarchDelta<'_>],
    base: &FrameEncoding,
    generation_strict: bool,
    simulation_complete: bool,
) -> MarchCalibration {
    let mut scores = Vec::new();
    for low in 10..=15 {
        let mut encoding = base.clone();
        encoding.ids.low_bits = low;
        let mut timeline = MarchTimeline::new(keyframes);
        let mut score = MarchCalibrationScore {
            id_low_bits: low,
            ..Default::default()
        };
        for d in deltas.iter().take(3000) {
            if score.events >= 150 {
                break;
            }
            let world = timeline.advance_to(d.timestamp_us);
            let (start, events) = packet_start(
                *d,
                registry,
                &encoding,
                world,
                generation_strict,
                simulation_complete,
            );
            score.events += usize::from(events);
            if start.is_some() {
                score.packets += 1;
                score.located += usize::from(events);
            }
            // Native trial walks restore all bindings and discard every record.
            // No observation hook is installed here, so these walks have no output.
        }
        scores.push(score);
    }
    scores.sort_by_key(|s| {
        (
            std::cmp::Reverse(s.located),
            s.id_low_bits != base.ids.low_bits,
            s.id_low_bits,
        )
    });
    let best = scores[0].clone();
    let runner_up = scores[1].clone();
    let retained_default = best.located < 2 * runner_up.located.max(1);
    let mut encoding = base.clone();
    if !retained_default {
        encoding.ids.low_bits = best.id_low_bits;
    }
    MarchCalibration {
        encoding: Some(encoding),
        encoding_error: None,
        retained_default,
        best,
        runner_up,
    }
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FilmMarchFacts {
    /// Actual calibrated native input settings, frozen after the scan. Absent for
    /// legacy encoding-only callers, old exports, and scans with no delta packets.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native_config: Option<NativeFrameMetadata>,
    pub facts: MarchRecordFacts,
    /// Packet preamble used for packets without an event list. This scanner's
    /// native context uses two bits; old exports and uncalibrated scans retain
    /// None rather than inventing a configuration that was not retained.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub packet_preamble_bits: Option<usize>,
    /// Policy actually used by the calibrated walk. Unavailable for old exports
    /// and when no delta packet required calibration or traversal.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub walk_policy: Option<MarchWalkPolicy>,
    /// Absent if the film has no delta packets, matching native early return.
    pub calibration: Option<MarchCalibration>,
    pub keyframes: usize,
    pub deltas: usize,
    pub packets: usize,
    pub event_packets: usize,
    pub located_packets: usize,
}
const MARCH_PACKET_PREAMBLE_BITS: usize = 2;
/// Extract native death and occupancy facts in one chronological pass. The caller
/// supplies the v41 registry and map-dependent base encoding. Native default
/// generation checking is false; use the resolved profile value when available.
pub fn scan_film_march_facts(
    chunks: &[FilmChunkData],
    registry: &FilmRegistry,
    base: &FrameEncoding,
    generation_strict: bool,
) -> Result<FilmMarchFacts, DecodeError> {
    scan_film_march_facts_with_simulation_policy(chunks, registry, base, generation_strict, true)
}
pub fn scan_film_march_facts_with_simulation_policy(
    chunks: &[FilmChunkData],
    registry: &FilmRegistry,
    base: &FrameEncoding,
    generation_strict: bool,
    simulation_complete: bool,
) -> Result<FilmMarchFacts, DecodeError> {
    if registry.major_version != 41 {
        return Err(DecodeError::UnsupportedVersion(
            registry.major_version as i32,
        ));
    }
    if !base.valid() {
        return Err(DecodeError::Inconsistent("invalid march encoding".into()));
    }
    // Native ScanMarchFacts treats an empty contiguous data prefix as an empty
    // measurement after resolving the registry, not a failure to decode a packet.
    let chunks = super::fire_events::native_chunk_prefix(chunks).unwrap_or_default();
    let mut keyframes = Vec::new();
    let mut deltas = Vec::new();
    for c in chunks {
        for p in super::fire_events::native_chunk_packets(c) {
            let payload = &c.data[p.payload_offset..p.payload_offset + p.payload_size];
            match p.packet_type {
                2 => keyframes.push(MarchKeyframe {
                    timestamp_us: p.timestamp_us,
                    bindings: recover_keyframe_anchors(payload)
                        .into_iter()
                        .map(|a| (a.id, a.archetype))
                        .collect(),
                }),
                0 => deltas.push(MarchDelta {
                    timestamp_us: p.timestamp_us,
                    payload,
                }),
                _ => {}
            }
        }
    }
    deltas.sort_by_key(|d| d.timestamp_us);
    let mut out = FilmMarchFacts {
        keyframes: keyframes.len(),
        deltas: deltas.len(),
        ..Default::default()
    };
    if deltas.is_empty() {
        return Ok(out);
    }
    let calibration = calibrate_march_encoding_with_simulation_policy(
        registry,
        &keyframes,
        &deltas,
        base,
        generation_strict,
        simulation_complete,
    );
    let mut timeline = MarchTimeline::new(&keyframes);
    let policy = MarchWalkPolicy {
        views: 8,
        generation_strict,
        simulation_complete,
    };
    out.walk_policy = Some(policy);
    out.packet_preamble_bits = Some(MARCH_PACKET_PREAMBLE_BITS);
    let encoding = calibration
        .encoding
        .as_ref()
        .expect("encoding-only calibration always retains its input encoding");
    for d in deltas {
        let world = timeline.advance_to(d.timestamp_us);
        let (start, events) = packet_start(
            d,
            registry,
            encoding,
            world,
            generation_strict,
            simulation_complete,
        );
        out.event_packets += usize::from(events);
        if let Some(start) = start {
            out.packets += 1;
            out.located_packets += usize::from(events);
            let records =
                walk_march_with_policy(d.payload, start, registry, encoding, world, policy);
            out.facts.harvest(&records, d.timestamp_us, registry);
        }
    }
    out.facts.deduplicate();
    out.calibration = Some(calibration);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
    use std::io::Read;
    fn fixture(bytes: &[u8]) -> Value {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(bytes)
            .read_to_end(&mut raw)
            .unwrap();
        serde_json::from_slice(&raw).unwrap()
    }
    fn assert_native_march_config(actual: &NativeFrameMetadata, expected: &Value) {
        assert_eq!(json!(actual.extra_fields), expected["HasExtraFields"]);
        assert_eq!(json!(actual.id_low_bits), expected["IDLowBits"]);
        assert_eq!(json!(actual.id_base), expected["IDBase"]);
        assert_eq!(
            json!(actual.new_default_state_bits),
            expected["NewDefaultStateBits"]
        );
        assert_eq!(
            json!(actual.packet_preamble_bits),
            expected["PacketPreambleBits"]
        );
        assert_eq!(actual.observer_present, !expected["Obs"].is_null());
        assert_eq!(
            actual.profile,
            native_scan_profile::tests::profile(&expected["Profil"])
        );
    }
    #[test]
    fn native_loaded_march_reports() {
        use crate::clients::hi::models::FilmChunk;
        let rows = fixture(include_bytes!("fixtures/march-loaded-v41.json.zlib"));
        assert_eq!(rows.as_array().unwrap().len(), 256);
        let registry = FilmRegistry {
            archetypes: (0..41)
                .map(|index| {
                    let (components, levels) = match index {
                        3 => (vec!["high-frequency".into()], vec![1]),
                        35 | 40 => (
                            vec![
                                "object-dead-state-component".into(),
                                "object-parent-state-component".into(),
                                "not-ported".into(),
                            ],
                            vec![0; 3],
                        ),
                        _ => (vec![], vec![]),
                    };
                    FilmArchetype {
                        index,
                        components,
                        levels,
                    }
                })
                .collect(),
            major_version: 41,
            format_version: 27,
            end_byte: 0,
            truncated: false,
        };
        for (i, row) in rows.as_array().unwrap().iter().enumerate() {
            let chunks: Vec<_> = row["chunks"]
                .as_array()
                .unwrap()
                .iter()
                .map(|c| {
                    let hex = c["hex"].as_str().unwrap();
                    let data: Vec<_> = (0..hex.len())
                        .step_by(2)
                        .map(|k| u8::from_str_radix(&hex[k..k + 2], 16).unwrap())
                        .collect();
                    FilmChunkData {
                        metadata: FilmChunk {
                            index: c["index"].as_i64().unwrap() as i32,
                            chunk_type: 0,
                            start_time_offset_ms: 0,
                            duration_ms: 0,
                            size: data.len() as i64,
                            file_relative_path: String::new(),
                        },
                        data,
                    }
                })
                .collect();
            let base = FrameEncoding {
                keyframe_layout: Default::default(),
                keyframe_simulation_complete: Some(row["simulation"].as_bool().unwrap()),
                native_id_low_bits: None,
                component_widths: Default::default(),
                new_record: Default::default(),
                position_capture: None,
                ids: RecordIdLayout {
                    low_bits: row["base_low"].as_u64().unwrap() as usize,
                    base: row["base_id"].as_u64().unwrap() as u32,
                },
                mpp_widths: serde_json::from_value(row["mpp"].clone()).unwrap(),
                position: Some(serde_json::from_value(row["position"].clone()).unwrap()),
                extra_fields: row["extra"].as_bool().unwrap(),
                corruption_check: row["corruption"].as_bool().unwrap(),
            };
            let out = scan_film_march_facts_with_simulation_policy(
                &chunks,
                &registry,
                &base,
                row["strict"].as_bool().unwrap(),
                row["simulation"].as_bool().unwrap(),
            )
            .unwrap_or_else(|e| panic!("case {i}: {e}"));
            let e = &row["facts"];
            let stats = &e["Stats"];
            let mut config = NativeFrameConfig::default();
            config.context.profile.grammar.generation_strict = row["strict"].as_bool().unwrap();
            config.context.profile.grammar.simulation_complete =
                row["simulation"].as_bool().unwrap();
            config.context.profile.grammar.corruption_check = row["corruption"].as_bool().unwrap();
            config.context.profile.mpp = FilmMppWidths {
                lead: base.mpp_widths[0] as i64,
                index: base.mpp_widths[1] as i64,
            };
            let mut keys = Vec::new();
            let mut deltas = Vec::new();
            for chunk in fire_events::native_chunk_prefix(&chunks).unwrap_or_default() {
                for packet in fire_events::native_chunk_packets(chunk) {
                    let payload = &chunk.data
                        [packet.payload_offset..packet.payload_offset + packet.payload_size];
                    match packet.packet_type {
                        2 => keys.push(MarchKeyframe {
                            timestamp_us: packet.timestamp_us,
                            bindings: recover_keyframe_anchors(payload)
                                .into_iter()
                                .map(|a| (a.id, a.archetype))
                                .collect(),
                        }),
                        0 => deltas.push(MarchDelta {
                            timestamp_us: packet.timestamp_us,
                            payload,
                        }),
                        _ => {}
                    }
                }
            }
            deltas.sort_by_key(|d| d.timestamp_us);
            let native =
                native_march::scan_native_march_packets(&registry, &keys, &deltas, &config)
                    .unwrap();
            assert_eq!(native.facts, out.facts, "complete native context facts {i}");
            assert_eq!(
                (
                    native.keyframes,
                    native.deltas,
                    native.packets,
                    native.event_packets,
                    native.located_packets
                ),
                (
                    out.keyframes,
                    out.deltas,
                    out.packets,
                    out.event_packets,
                    out.located_packets
                ),
                "native counters {i}"
            );
            if let Some(metadata) = &native.native_config {
                assert_native_march_config(metadata, &stats["Config"]);
                let c = native.calibration.as_ref().unwrap();
                let legacy = out.calibration.as_ref().unwrap();
                assert_eq!(
                    (&c.best, &c.runner_up, c.retained_default),
                    (&legacy.best, &legacy.runner_up, legacy.retained_default)
                );
            } else {
                assert_eq!(native.deltas, 0);
            }
            assert_eq!(
                serde_json::from_value::<FilmMarchFacts>(json!(native)).unwrap(),
                native
            );
            assert_eq!(json!(out.facts.deaths), e["Deaths"], "deaths {i}");
            assert_eq!(json!(out.facts.occupancy), e["Occupancy"], "occupancy {i}");
            for (value, name) in [
                (out.keyframes, "Keyframes"),
                (out.deltas, "Deltas"),
                (out.packets, "Packets"),
                (out.event_packets, "EventPackets"),
                (out.located_packets, "LocatedPackets"),
            ] {
                assert_eq!(json!(value), stats[name], "{name} {i}");
            }
            for (value, name) in [
                (&out.facts.coverage.records, "Records"),
                (&out.facts.coverage.clean_records, "CleanRecords"),
                (&out.facts.coverage.mask_declared, "MaskDeclared"),
                (
                    &out.facts.coverage.mask_declared_desync,
                    "MaskDeclaredDesync",
                ),
            ] {
                assert_eq!(json!(value), stats[name], "{name} {i}");
            }
            if let Some(c) = &out.calibration {
                assert_eq!(
                    json!(out.packet_preamble_bits),
                    stats["Config"]["PacketPreambleBits"],
                    "packet preamble {i}"
                );
                let policy = out.walk_policy.expect("calibrated walk policy");
                let grammar = &stats["Config"]["Profil"]["Grammaire"];
                assert_eq!(policy.views, 8);
                assert_eq!(
                    json!(policy.generation_strict),
                    grammar["GenerationStricte"]
                );
                assert_eq!(
                    json!(policy.simulation_complete),
                    grammar["SimStateComplet"]
                );
                let mut expected = base.clone();
                expected.ids.low_bits = stats["Config"]["IDLowBits"].as_u64().unwrap() as usize;
                assert_eq!(c.encoding, Some(expected), "inherited context {i}");
                assert_eq!(json!(c.retained_default), stats["CadreParDefaut"]);
                for (value, name) in [
                    (c.best.located, "CadreLocalises"),
                    (c.runner_up.located, "CadreDauphin"),
                    (c.best.events, "CadreEvenements"),
                ] {
                    assert_eq!(json!(value), stats[name], "{name} {i}");
                }
            } else {
                assert_eq!(out.deltas, 0, "missing calibration {i}");
                assert_eq!(out.walk_policy, None);
                assert_eq!(out.packet_preamble_bits, None);
            }
            assert_eq!(
                serde_json::from_value::<FilmMarchFacts>(serde_json::to_value(&out).unwrap())
                    .unwrap(),
                out
            );
            let mut legacy = serde_json::to_value(&out).unwrap();
            legacy.as_object_mut().unwrap().remove("walk_policy");
            legacy
                .as_object_mut()
                .unwrap()
                .remove("packet_preamble_bits");
            let restored: FilmMarchFacts = serde_json::from_value(legacy).unwrap();
            assert_eq!(restored.walk_policy, None);
            assert_eq!(restored.packet_preamble_bits, None);
            let mut expected_legacy = out.clone();
            expected_legacy.walk_policy = None;
            expected_legacy.packet_preamble_bits = None;
            assert_eq!(restored, expected_legacy);
        }
    }
    #[test]
    fn native_timeline_and_calibration() {
        let rows = fixture(include_bytes!("fixtures/march-calibration-v41.json.zlib"));
        let registry = FilmRegistry {
            archetypes: (0..4)
                .map(|index| FilmArchetype {
                    index,
                    components: if index == 3 {
                        vec!["high-frequency".into()]
                    } else {
                        vec![]
                    },
                    levels: vec![1],
                })
                .collect(),
            major_version: 41,
            format_version: 0,
            end_byte: 0,
            truncated: false,
        };
        for (i, r) in rows.as_array().unwrap().iter().enumerate() {
            let keys: Vec<MarchKeyframe> = serde_json::from_value(r["keyframes"].clone()).unwrap();
            let mut timeline = MarchTimeline::new(&keys);
            for s in r["states"].as_array().unwrap() {
                let world = timeline.advance_to(s["time"].as_u64().unwrap());
                let bindings: Vec<_> = world
                    .slots
                    .values()
                    .map(|s| (s.full_id, s.archetype))
                    .collect();
                assert_eq!(json!(bindings), s["bindings"], "timeline {i}");
            }
            let payloads: Vec<Vec<u8>> = r["deltas"]
                .as_array()
                .unwrap()
                .iter()
                .map(|d| serde_json::from_value(d["payload"].clone()).unwrap())
                .collect();
            let deltas: Vec<_> = r["deltas"]
                .as_array()
                .unwrap()
                .iter()
                .zip(&payloads)
                .map(|(d, p)| MarchDelta {
                    timestamp_us: d["time"].as_u64().unwrap(),
                    payload: p,
                })
                .collect();
            let base = FrameEncoding {
                keyframe_layout: Default::default(),
                keyframe_simulation_complete: None,
                native_id_low_bits: None,
                component_widths: Default::default(),
                new_record: Default::default(),
                position_capture: None,
                ids: RecordIdLayout {
                    low_bits: r["base"].as_u64().unwrap() as usize,
                    base: 0,
                },
                mpp_widths: [9, 5],
                position: None,
                extra_fields: false,
                corruption_check: false,
            };
            let out = calibrate_march_encoding(
                &registry,
                &keys,
                &deltas,
                &base,
                r["strict"].as_bool().unwrap(),
            );
            assert_eq!(
                json!(out.encoding.as_ref().unwrap().ids.low_bits),
                r["low"],
                "low {i}"
            );
            assert_eq!(json!(out.retained_default), r["default"], "default {i}");
            assert_eq!(json!(out.best), r["best"], "best {i}");
            assert_eq!(json!(out.runner_up), r["second"], "second {i}");
        }
    }
    #[test]
    #[ignore = "requires downloaded v41 films"]
    fn local_native_march_corpus() {
        use crate::clients::hi::models::FilmChunk;
        let rows = fixture(include_bytes!("fixtures/march-corpus-v41.json.zlib"));
        for r in rows.as_array().unwrap() {
            let folder = r["folder"].as_str().unwrap();
            let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("experiments/films")
                .join(folder);
            let meta: Value =
                serde_json::from_slice(&std::fs::read(dir.join("film.json")).unwrap()).unwrap();
            let chunks: Vec<_> = meta["chunks"]
                .as_array()
                .unwrap()
                .iter()
                .map(|c| FilmChunkData {
                    metadata: FilmChunk {
                        index: c["index"].as_i64().unwrap() as i32,
                        chunk_type: c["chunk_type"].as_i64().unwrap() as i32,
                        start_time_offset_ms: c["start_time_offset_ms"].as_i64().unwrap(),
                        duration_ms: 0,
                        size: 0,
                        file_relative_path: String::new(),
                    },
                    data: std::fs::read(dir.join(c["file"].as_str().unwrap())).unwrap(),
                })
                .collect();
            let map_name = match folder {
                "maps/01-bazaar-idle" => "bazaar",
                "maps/02-aquarius" => "aquarius",
                _ => "recharge",
            };
            let film = LegacyFilm::try_from_chunks_with_map(
                &chunks,
                DecodeOptions::v41(),
                map_name,
                RecordIdLayout {
                    low_bits: 13,
                    base: 0,
                },
            )
            .unwrap();
            let out = film.native_march_facts.unwrap();
            let e = &r["facts"];
            native_march::tests::assert_native_march_facts(&out, e, folder);
            let stats = &e["Stats"];
            eprintln!(
                "march {folder}: {} deaths, {} occupancy",
                out.facts.deaths.len(),
                out.facts.occupancy.len()
            );
            assert_eq!(json!(out.facts.deaths), e["Deaths"], "deaths {folder}");
            assert_eq!(
                json!(out.facts.occupancy),
                e["Occupancy"],
                "occupancy {folder}"
            );
            for (v, name) in [
                (out.keyframes, "Keyframes"),
                (out.deltas, "Deltas"),
                (out.packets, "Packets"),
                (out.event_packets, "EventPackets"),
                (out.located_packets, "LocatedPackets"),
            ] {
                assert_eq!(json!(v), stats[name], "{name} {folder}");
            }
            for (v, name) in [
                (&out.facts.coverage.records, "Records"),
                (&out.facts.coverage.clean_records, "CleanRecords"),
                (&out.facts.coverage.mask_declared, "MaskDeclared"),
                (
                    &out.facts.coverage.mask_declared_desync,
                    "MaskDeclaredDesync",
                ),
            ] {
                assert_eq!(json!(v), stats[name], "{name} {folder}");
            }
            let policy = out.walk_policy.expect("captured march policy");
            let grammar = &stats["Config"]["Profil"]["Grammaire"];
            assert_eq!(policy.views, 8);
            assert_eq!(
                json!(policy.generation_strict),
                grammar["GenerationStricte"]
            );
            assert_eq!(
                json!(policy.simulation_complete),
                grammar["SimStateComplet"]
            );
            assert_eq!(
                serde_json::from_value::<FilmMarchFacts>(serde_json::to_value(&out).unwrap())
                    .unwrap(),
                out,
                "captured march export {folder}"
            );
            let c = out.calibration.unwrap();
            assert_eq!(
                json!(out.packet_preamble_bits),
                stats["Config"]["PacketPreambleBits"],
                "packet preamble {folder}"
            );
            assert_eq!(
                json!(c.encoding.as_ref().unwrap().ids.low_bits),
                stats["Config"]["IDLowBits"],
                "IDLowBits {folder}"
            );
            assert_eq!(json!(c.retained_default), stats["CadreParDefaut"]);
            for (v, name) in [
                (c.best.located, "CadreLocalises"),
                (c.runner_up.located, "CadreDauphin"),
                (c.best.events, "CadreEvenements"),
            ] {
                assert_eq!(json!(v), stats[name], "{name} {folder}");
            }
        }
    }
}
