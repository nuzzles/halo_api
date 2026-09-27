//! March traversal using the complete native scan context, without profile projection loss.
use super::*;
use serde::{Deserialize, Serialize};

/// Frozen parser configuration, not recorded wire data. Callback closures are not
/// serializable; their presence is explicit. ScanMarchFacts uses no observer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NativeFrameMetadata {
    pub extra_fields: bool,
    pub id_low_bits: i64,
    pub id_base: u32,
    pub new_default_state_bits: i64,
    pub packet_preamble_bits: i64,
    pub profile: NativeScanProfile,
    pub observer_present: bool,
}
impl NativeFrameConfig {
    pub fn snapshot(&self) -> NativeFrameMetadata {
        NativeFrameMetadata {
            extra_fields: self.extra_fields,
            id_low_bits: self.id_low_bits,
            id_base: self.id_base,
            new_default_state_bits: self.new_default_state_bits,
            packet_preamble_bits: self.packet_preamble_bits,
            profile: self.context.profile.snapshot(),
            observer_present: self.context.observer.is_some(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum NativeMarchError {
    #[error(transparent)]
    Registry(#[from] NativeContextRegistryError),
    #[error(transparent)]
    Profile(#[from] NativeProfileResolveError),
    #[error(transparent)]
    Reader(#[from] NativeReaderProfileError),
}
impl NativeFilmContext<'_> {
    /// Native ScanMarchFacts: chronological source traversal, width calibration,
    /// death/occupancy harvest and the full calibrated configuration. Uses the
    /// cached registry and numbered source prefix; the context observer is excluded.
    pub fn scan_march_facts(&self) -> Result<FilmMarchFacts, NativeMarchError> {
        let registry = &self.registry().map_err(|e| *e)?.registry;
        if registry.major_version != 41 {
            return Err(
                NativeProfileResolveError::UnsupportedVersion(registry.major_version).into(),
            );
        }
        let mut keys = Vec::new();
        let mut deltas = Vec::new();
        for &number in self.chunk_numbers() {
            let Some((data, packets)) = self.chunk_at(number) else {
                continue;
            };
            for packet in packets {
                let payload =
                    &data[packet.payload_offset..packet.payload_offset + packet.payload_size];
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
        deltas.sort_by_key(|delta| delta.timestamp_us);
        // Native early return does not resolve the scan profile.
        if deltas.is_empty() {
            return Ok(FilmMarchFacts {
                keyframes: keys.len(),
                ..Default::default()
            });
        }
        Ok(scan_native_march_packets(
            registry,
            &keys,
            &deltas,
            &self.scan_frame()?,
        )?)
    }
}

/// Native ScanMarchFacts with explicit full settings and already-decoded chunks.
/// This retains the existing constructor's source adapter and registry identity.
pub fn scan_film_march_facts_with_native_config(
    chunks: &[crate::clients::hi::models::FilmChunkData],
    registry: &FilmRegistry,
    config: &NativeFrameConfig,
) -> Result<FilmMarchFacts, NativeMarchError> {
    if registry.major_version != 41 {
        return Err(NativeProfileResolveError::UnsupportedVersion(registry.major_version).into());
    }
    let mut keys = Vec::new();
    let mut deltas = Vec::new();
    for chunk in fire_events::native_chunk_prefix(chunks).unwrap_or_default() {
        for packet in fire_events::native_chunk_packets(chunk) {
            let payload =
                &chunk.data[packet.payload_offset..packet.payload_offset + packet.payload_size];
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
    deltas.sort_by_key(|delta| delta.timestamp_us);
    Ok(scan_native_march_packets(registry, &keys, &deltas, config)?)
}

/// The reference installs the inherited profile first, then only the map's
/// world-object precision. Corruption is resolved separately from recorded input.
pub fn native_replay_march_config(
    map: &FilmMapBounds,
    inherited: Option<&NativeScanProfile>,
    corruption: FilmCorruptionControl,
) -> NativeFrameConfig {
    let mut profile = inherited.cloned().unwrap_or_else(|| {
        let mut profile = NativeScanProfile::default();
        profile.grammar.simulation_complete =
            resolve_imposed_i0_layout(None, Some(&map.i0_layout())).is_some();
        profile
    });
    profile.set_world_precision_from_layout(&map.i0_layout());
    profile.grammar.corruption_check = corruption.enabled;
    NativeFrameConfig {
        context: NativeReaderContext {
            profile,
            observer: None,
        },
        ..Default::default()
    }
}

fn locate(
    data: &[u8],
    registry: &FilmRegistry,
    world: &FilmWorld,
    cfg: &NativeFrameConfig,
) -> Result<Option<usize>, NativeReaderProfileError> {
    locate_with_policy(data, registry, world, cfg, false)
}

impl NativeFrameConfig {
    /// Native movement locator: first 35-bit, single-component delta for slot
    /// 123 preceded by zero. Trials suppress movement hooks, retain other hooks,
    /// and neither apply the death locator's generation recheck nor its fallback.
    pub fn locate_strict_entity_view(
        &self,
        data: &[u8],
        registry: &FilmRegistry,
        world: &FilmWorld,
    ) -> Result<Option<usize>, NativeReaderProfileError> {
        locate_with_policy(data, registry, world, self, true)
    }
}

fn locate_with_policy(
    data: &[u8],
    registry: &FilmRegistry,
    world: &FilmWorld,
    cfg: &NativeFrameConfig,
    strict_only: bool,
) -> Result<Option<usize>, NativeReaderProfileError> {
    let encoding = cfg.contextual_frame_encoding()?;
    let profile = KillWalkProfile {
        encoding,
        simulation_complete: cfg.context.profile.grammar.simulation_complete,
    };
    let _guard = cfg
        .context
        .observer
        .as_ref()
        .map(NativeFilmObserver::neutralize_movement);
    let trial = |at: usize| -> Option<EntityRecord> {
        if data.len().saturating_mul(8).saturating_sub(at) < 24 {
            return None;
        }
        let header_at = at.checked_add(if cfg.extra_fields { 32 } else { 0 })?;
        let header = records::decode_frame_header_cursor(
            bits::Cursor::new_padded(data, header_at),
            &profile.encoding,
        )?;
        if header.kind != RecordKind::Delta {
            return None;
        }
        let slot = header.id? & 0x3fff_ffff;
        // No observer on the FilmContext scan path: reject irrelevant slots before
        // walking potentially very large speculative bodies.
        if slot != 123 && cfg.context.observer.is_none() {
            return None;
        }
        let record = inference_frame::read_bound_record_contextual(
            data,
            header_at,
            &header,
            registry,
            &profile,
            world,
            components::RecordCaptureSlots {
                context: Some(cfg.context.clone()),
                movement: Some(slot),
                position: Some(slot),
            },
        )?;
        (record.stop == EntityViewStop::Complete).then_some(record)
    };
    let bits = bits::Bits(data);
    let accepted = |record: &EntityRecord| {
        world.generation_matches(
            record.header.id.unwrap(),
            cfg.context.profile.grammar.generation_strict,
        )
    };
    for at in 2..bits.len().saturating_sub(35) {
        if bits.read(at - 1, 1) != Some(0) {
            continue;
        }
        if trial(at).is_some_and(|r| {
            r.header.id.unwrap() & 0x3fff_ffff == 123
                && r.end_bit == (at + 35) as i64
                && r.components.len() == 1
        }) {
            if strict_only {
                return Ok(Some(at));
            }
            if trial(at).is_some_and(|r| accepted(&r)) {
                return Ok(Some(at));
            }
            break;
        }
    }
    if strict_only {
        return Ok(None);
    }
    for at in 2..bits.len().saturating_sub(16) {
        if bits.read(at - 1, 1) == Some(0)
            && trial(at).is_some_and(|r| r.header.id.unwrap() & 0x3fff_ffff == 123 && accepted(&r))
        {
            return Ok(Some(at));
        }
    }
    Ok(None)
}
fn start(
    data: &[u8],
    registry: &FilmRegistry,
    world: &FilmWorld,
    cfg: &NativeFrameConfig,
) -> Result<(Option<usize>, bool), NativeReaderProfileError> {
    if bits::Bits(data).read(1, 1) != Some(1) {
        return Ok((
            Some(
                usize::try_from(cfg.packet_preamble_bits)
                    .map_err(|_| NativeReaderProfileError::Width("march preamble"))?,
            ),
            false,
        ));
    }
    Ok((locate(data, registry, world, cfg)?, true))
}
fn walk(
    data: &[u8],
    at: usize,
    registry: &FilmRegistry,
    world: &FilmWorld,
    cfg: &NativeFrameConfig,
) -> Result<Vec<EntityRecord>, NativeReaderProfileError> {
    let mut working = world.clone();
    let mut reader = cfg.reader(data);
    reader.set_bit_position(at);
    let mut records = Vec::new();
    for _ in 0..8 {
        if reader.source_bits().saturating_sub(reader.bit_position()) < 8 {
            break;
        }
        let view = reader.read_frame_records(registry, &mut working, cfg)?;
        records.extend(
            view.records
                .into_iter()
                .filter(|r| r.header.kind != RecordKind::End),
        );
        if view.stop != EntityViewStop::Complete {
            break;
        }
    }
    Ok(records)
}

pub(crate) fn scan_native_march_packets(
    registry: &FilmRegistry,
    keys: &[MarchKeyframe],
    deltas: &[MarchDelta<'_>],
    base: &NativeFrameConfig,
) -> Result<FilmMarchFacts, NativeReaderProfileError> {
    let mut out = FilmMarchFacts {
        keyframes: keys.len(),
        deltas: deltas.len(),
        ..Default::default()
    };
    if deltas.is_empty() {
        return Ok(out);
    }
    let mut scores = Vec::new();
    for low in 10..=15 {
        let mut config = base.clone();
        config.id_low_bits = low;
        let mut timeline = MarchTimeline::new(keys);
        let mut score = MarchCalibrationScore {
            id_low_bits: low as usize,
            ..Default::default()
        };
        for delta in deltas.iter().take(3000) {
            if score.events >= 150 {
                break;
            }
            let world = timeline.advance_to(delta.timestamp_us);
            let (at, events) = start(delta.payload, registry, world, &config)?;
            score.events += usize::from(events);
            if let Some(at) = at {
                score.packets += 1;
                score.located += usize::from(events);
                walk(delta.payload, at, registry, world, &config)?;
            }
        }
        scores.push(score);
    }
    scores.sort_by_key(|s| {
        (
            std::cmp::Reverse(s.located),
            s.id_low_bits as i64 != base.id_low_bits,
            s.id_low_bits,
        )
    });
    let best = scores[0].clone();
    let runner_up = scores[1].clone();
    let retained_default = best.located < 2 * runner_up.located.max(1);
    let mut config = base.clone();
    if !retained_default {
        config.id_low_bits = best.id_low_bits as i64
    }
    let mut timeline = MarchTimeline::new(keys);
    for delta in deltas {
        let world = timeline.advance_to(delta.timestamp_us);
        let (at, events) = start(delta.payload, registry, world, &config)?;
        out.event_packets += usize::from(events);
        if let Some(at) = at {
            out.located_packets += usize::from(events);
            out.packets += 1;
            out.facts.harvest(
                &walk(delta.payload, at, registry, world, &config)?,
                delta.timestamp_us,
                registry,
            );
        }
    }
    out.facts.deduplicate();
    out.packet_preamble_bits = Some(
        usize::try_from(config.packet_preamble_bits)
            .map_err(|_| NativeReaderProfileError::Width("march preamble"))?,
    );
    out.walk_policy = Some(MarchWalkPolicy {
        views: 8,
        generation_strict: config.context.profile.grammar.generation_strict,
        simulation_complete: config.context.profile.grammar.simulation_complete,
    });
    let (encoding, encoding_error) = match config.frame_encoding() {
        Ok(encoding) => (Some(encoding), None),
        Err(error) => (None, Some(error.to_string())),
    };
    out.calibration = Some(MarchCalibration {
        encoding,
        encoding_error,
        retained_default,
        best,
        runner_up,
    });
    out.native_config = Some(config.snapshot());
    Ok(out)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use serde_json::{Value, json};
    use std::io::Read;

    pub(crate) fn assert_native_march_facts(out: &FilmMarchFacts, expected: &Value, folder: &str) {
        let stats = &expected["Stats"];
        assert_eq!(
            json!(out.facts.deaths),
            expected["Deaths"],
            "deaths {folder}"
        );
        assert_eq!(
            json!(out.facts.occupancy),
            expected["Occupancy"],
            "occupancy {folder}"
        );
        for (actual, key) in [
            (out.keyframes, "Keyframes"),
            (out.deltas, "Deltas"),
            (out.packets, "Packets"),
            (out.event_packets, "EventPackets"),
            (out.located_packets, "LocatedPackets"),
        ] {
            assert_eq!(json!(actual), stats[key], "{key} {folder}");
        }
        for (actual, key) in [
            (&out.facts.coverage.records, "Records"),
            (&out.facts.coverage.clean_records, "CleanRecords"),
            (&out.facts.coverage.mask_declared, "MaskDeclared"),
            (
                &out.facts.coverage.mask_declared_desync,
                "MaskDeclaredDesync",
            ),
        ] {
            assert_eq!(json!(actual), stats[key], "{key} {folder}");
        }
        if let Some(actual) = &out.native_config {
            let cfg = &stats["Config"];
            assert_eq!(json!(actual.extra_fields), cfg["HasExtraFields"]);
            assert_eq!(json!(actual.id_low_bits), cfg["IDLowBits"]);
            assert_eq!(json!(actual.id_base), cfg["IDBase"]);
            assert_eq!(
                json!(actual.new_default_state_bits),
                cfg["NewDefaultStateBits"]
            );
            assert_eq!(
                json!(actual.packet_preamble_bits),
                cfg["PacketPreambleBits"]
            );
            assert_eq!(actual.observer_present, !cfg["Obs"].is_null());
            assert_eq!(
                actual.profile,
                native_scan_profile::tests::profile(&cfg["Profil"])
            );
            let calibration = out.calibration.as_ref().unwrap();
            assert_eq!(json!(calibration.retained_default), stats["CadreParDefaut"]);
            assert_eq!(json!(calibration.best.located), stats["CadreLocalises"]);
            assert_eq!(json!(calibration.runner_up.located), stats["CadreDauphin"]);
            assert_eq!(json!(calibration.best.events), stats["CadreEvenements"]);
        } else {
            assert_eq!(out.deltas, 0);
        }
        assert_eq!(
            serde_json::from_value::<FilmMarchFacts>(json!(out)).unwrap(),
            *out
        );
    }

    #[test]
    fn native_march_configuration_snapshot_does_not_alias_live_maps() {
        let mut config = NativeFrameConfig::default();
        config.context.profile.grammar.calibrated_widths = Some(NativeSharedWidths::from_map(
            [("native-width".into(), -7)].into(),
        ));
        config.context.profile.grammar.stub_widths = Some(NativeSharedWidths::default());
        let snapshot = config.snapshot();
        config
            .context
            .profile
            .grammar
            .calibrated_widths
            .as_ref()
            .unwrap()
            .insert("native-width".into(), 41);
        config
            .context
            .profile
            .grammar
            .stub_widths
            .as_ref()
            .unwrap()
            .insert("late".into(), 9);
        assert_eq!(
            snapshot
                .profile
                .grammar
                .calibrated_widths
                .as_ref()
                .unwrap()
                .get("native-width"),
            Some(-7)
        );
        assert!(
            snapshot
                .profile
                .grammar
                .stub_widths
                .as_ref()
                .unwrap()
                .snapshot()
                .is_empty()
        );
        assert_eq!(
            serde_json::from_value::<NativeFrameMetadata>(json!(snapshot)).unwrap(),
            snapshot
        );
        assert!(!snapshot.observer_present);
        config.context.observer = Some(NativeFilmObserver::default());
        assert!(config.snapshot().observer_present);
        assert!(!snapshot.observer_present);
    }

    #[test]
    fn native_march_empty_source_preserves_absent_config_and_rejects_other_versions() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/bootstrap-v41.zlib")[..])
            .read_to_end(&mut raw)
            .unwrap();
        for major in [41u32, 75] {
            raw[..4].copy_from_slice(&major.to_le_bytes());
            let source = FilmSource::load(
                &[raw.clone()],
                &[FilmSourceMetadata {
                    index: 0,
                    chunk_type: 1,
                    start_ms: 0,
                }],
            )
            .unwrap();
            let mut context = NativeFilmContext::new(Some(&source));
            if major == 41 {
                // This metadata would fail width conversion if the scanner eagerly
                // constructed a reader, despite having no delta packets to read.
                let mut profile = NativeScanProfile::default();
                profile.mpp.lead = -1;
                context.set_scan_profile(profile).unwrap();
                let facts = context.scan_march_facts().unwrap();
                assert_eq!(facts, FilmMarchFacts::default());
                assert!(facts.native_config.is_none());
                assert!(
                    serde_json::to_value(facts)
                        .unwrap()
                        .get("native_config")
                        .is_none()
                );
            } else {
                assert!(matches!(
                    context.scan_march_facts(),
                    Err(NativeMarchError::Profile(
                        NativeProfileResolveError::UnsupportedVersion(75)
                    ))
                ));
            }
        }
    }

    #[test]
    #[ignore = "requires four captured films; complete native-context march facts and configuration"]
    fn native_context_march_corpus() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/march-corpus-v41.json.zlib")[..])
            .read_to_end(&mut raw)
            .unwrap();
        let rows: Value = serde_json::from_slice(&raw).unwrap();
        for row in rows.as_array().unwrap() {
            let folder = row["folder"].as_str().unwrap();
            let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("experiments/films")
                .join(folder);
            let manifest: Value =
                serde_json::from_slice(&std::fs::read(dir.join("film.json")).unwrap()).unwrap();
            let mut data = Vec::new();
            let mut metadata = Vec::new();
            for chunk in manifest["chunks"].as_array().unwrap() {
                data.push(std::fs::read(dir.join(chunk["file"].as_str().unwrap())).unwrap());
                metadata.push(FilmSourceMetadata {
                    index: chunk["index"].as_i64().unwrap(),
                    chunk_type: chunk["chunk_type"].as_i64().unwrap(),
                    start_ms: chunk["start_time_offset_ms"].as_i64().unwrap(),
                });
            }
            let source = FilmSource::load(&data, &metadata).unwrap();
            let name = match folder {
                "maps/01-bazaar-idle" => "bazaar",
                "maps/02-aquarius" => "aquarius",
                _ => "recharge",
            };
            let map = film_map_catalog().maps[name].clone();
            let mut context = NativeFilmContext::for_map(Some(&source), Some(&map), None).unwrap();
            context.set_world_precision_from_layout(&map.i0_layout());
            let out = context.scan_march_facts().unwrap();
            let expected = &row["facts"];
            assert_native_march_facts(&out, expected, folder);
            eprintln!("native context march passed: {folder}");
        }
    }
}
