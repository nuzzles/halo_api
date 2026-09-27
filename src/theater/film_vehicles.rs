//! Map-aware vehicle observations and replay-layer integration.
use super::*;
use crate::clients::hi::models::FilmChunkData;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FilmVehicleFacts {
    pub scanned: bool,
    pub keyframes: WorldObjectKeyframes,
    pub creations: Option<EquipmentCreationStream>,
    pub positions: Option<BipedPositionStream>,
    pub events: Vec<VehicleEvent>,
    pub aims: Vec<BipedAim>,
    /// Native optional scanner failures are retained instead of silently discarded.
    pub issues: Vec<String>,
}
/// The native layer requires a census band and successful creation/position scans.
/// Event and aim failures are additive and do not suppress the rest of the layer.
pub fn scan_film_vehicle_facts(
    chunks: &[FilmChunkData],
    registry: &FilmRegistry,
    map: &FilmMapBounds,
    widths: [usize; 2],
) -> FilmVehicleFacts {
    scan_film_vehicle_facts_with_position(chunks, registry, map, widths, &map.position_encoding())
}
/// Preserve calibrated component widths through vehicle creation defaults.
pub fn scan_film_vehicle_facts_with_position(
    chunks: &[FilmChunkData],
    registry: &FilmRegistry,
    map: &FilmMapBounds,
    widths: [usize; 2],
    position: &PositionEncoding,
) -> FilmVehicleFacts {
    scan_film_vehicle_facts_observed(chunks, registry, map, widths, position, None)
}

/// Source-stage observations are enabled by the LegacyFilm constructor, which owns the match ID.
pub(crate) fn scan_film_vehicle_facts_observed(
    chunks: &[FilmChunkData],
    registry: &FilmRegistry,
    map: &FilmMapBounds,
    widths: [usize; 2],
    position: &PositionEncoding,
    match_id: Option<&str>,
) -> FilmVehicleFacts {
    let mut out = FilmVehicleFacts::default();
    let census = scan_world_object_keyframes(chunks, 40);
    if census.band.is_empty() {
        if let Some(match_id) = match_id {
            tracing::info!(
                match_id,
                imagesCles = census.times_us.len(),
                "vehicules : aucun slot ti=40 aux images-cles — rejeu sans ce calque"
            );
        }
        return out;
    }
    let layout = I0Layout {
        gate_bits: (4 + map.region_index_bits.max(1)) as i64,
        axis_widths: map.axis_widths.map(|w| w as u64),
        region: map.region,
    };
    let creations = match scan_vehicle_creations_for_band_report(
        chunks,
        &census.band,
        VehicleCreationScanConfig {
            map: Some(map),
            layout: Ok(&layout),
            registry: Ok(registry),
            position,
            mpp_widths: widths,
        },
    ) {
        Ok(v) => v,
        Err(failure) => {
            if let Some(match_id) = match_id {
                let error = match &failure.error {
                    DecodeError::Missing("vehicle archetype") => {
                        "archetype vehicule 40 absent du registre".to_owned()
                    }
                    error => error.to_string(),
                };
                tracing::warn!(
                    err = error.as_str(),
                    match_id,
                    "vehicules : records de creation illisibles — rejeu sans ce calque"
                );
            }
            out.issues
                .push(format!("vehicle creations: {}", failure.error));
            out.creations = Some(*failure.scan);
            return out;
        }
    };
    let positions = match scan_vehicle_positions(chunks, &census.band, map) {
        Ok(v) => v,
        Err(e) => {
            out.issues.push(format!("vehicle positions: {e}"));
            return out;
        }
    };
    out.scanned = true;
    out.keyframes = census;
    out.creations = Some(creations);
    out.positions = Some(positions);
    match super::biped_scan::biped_slot_band(chunks) {
        Ok(band) => {
            let band_absent = band.is_none();
            let band: BTreeSet<_> = band.map(|[lo, hi]| (lo..=hi).collect()).unwrap_or_default();
            match scan_vehicle_events_for_band(chunks, &band) {
                Ok(v) => out.events = v,
                Err(e) => out.issues.push(format!("vehicle events: {e}")),
            }
            if band_absent {
                let error = "aucun slot biped (ti=35) dans les keyframes du film";
                out.issues.push(format!("occupant aims: {error}"));
                if let Some(match_id) = match_id {
                    tracing::warn!(
                        err = error,
                        match_id,
                        "vehicules : visees sans position illisibles — episodes d occupation sans serie de visee, le cone retombe sur le cap du chassis"
                    );
                }
            } else {
                match scan_biped_aim_for_band(chunks, &band) {
                    Ok(v) => out.aims = v,
                    Err(e) => out.issues.push(format!("occupant aims: {e}")),
                }
            }
        }
        Err(e) => out.issues.push(format!("biped band: {e}")),
    }
    out
}
/// Assemble the vehicle layer on the same clock and identity registry as players.
/// None means the LegacyFilm was decoded without the required map-aware vehicle pass.
#[allow(dead_code)]
pub(crate) fn build_film_replay_vehicles(
    film: &LegacyFilm,
    players: &FilmReplayPlayers,
) -> Option<ReplayVehiclePublication> {
    let facts = film.native_vehicles.as_ref()?;
    let positions: Vec<_> = facts
        .positions
        .as_ref()
        .map(|p| p.accepted().map(ReplayVehiclePosition::from).collect())
        .unwrap_or_default();
    let bipeds: Vec<_> = film
        .biped_positions
        .as_ref()
        .map(|p| {
            p.accepted()
                .map(|p| ReplayVehiclePosition::from(p).position)
                .collect()
        })
        .unwrap_or_default();
    let deaths: Vec<_> = film
        .native_march_facts
        .as_ref()
        .map(|m| {
            m.facts
                .deaths
                .iter()
                .filter_map(ObjectDeath::vehicle_evidence)
                .collect()
        })
        .unwrap_or_default();
    let occupancy = film
        .native_march_facts
        .as_ref()
        .map_or(&[][..], |m| m.facts.occupancy.as_slice());
    Some(build_replay_vehicle_publication(
        ReplayVehicleScan {
            scanned: facts.scanned,
            keyframes: &facts.keyframes,
            creations: facts
                .creations
                .as_ref()
                .map_or(&[], |c| c.records.as_slice()),
            positions: &positions,
            events: &facts.events,
            aims: &facts.aims,
            deaths: &deaths,
            occupancy,
            march_default_retained: film
                .native_march_facts
                .as_ref()
                .and_then(|m| m.calibration.as_ref())
                .is_some_and(|c| c.retained_default),
        },
        &bipeds,
        &players.registry.owners.state,
        players.clock.origin_us,
        players.clock.step_us,
        players.clock.frame_count,
    ))
}

/// Observe vehicle death-mask losses from the shared chronological march.
/// Counts use archetype 40, including entities outside the keyframe census band.
pub(crate) fn log_vehicle_death_reads(match_id: &str, march: &FilmMarchFacts) {
    let coverage = &march.facts.coverage;
    let declared = coverage.mask_declared.get(&40).copied().unwrap_or(0);
    let lost = coverage.mask_declared_desync.get(&40).copied().unwrap_or(0);
    let records = coverage.records.get(&40).copied().unwrap_or(0);
    let clean = coverage.clean_records.get(&40).copied().unwrap_or(0);
    let vehicles = march
        .facts
        .deaths
        .iter()
        .filter(|d| d.type_index == 40)
        .count();
    let retained_default = march
        .calibration
        .as_ref()
        .is_some_and(|c| c.retained_default);
    macro_rules! emit {
        ($level:ident) => {
            tracing::$level!(
                match_id,
                mortsToutesEntites = march.facts.deaths.len(),
                mortsVehicules = vehicles,
                masqueDeclareLeDeadState = declared,
                dontDesynchronises = lost,
                recordsAtteints = records,
                recordsEntierementPortes = clean,
                paquetsAEvenements = march.event_packets,
                paquetsLocalises = march.located_packets,
                cadreParDefaut = retained_default,
                "vehicules : lecture des morts ecrites"
            );
        };
    }
    if lost > 0 {
        emit!(warn);
    } else {
        emit!(info);
    }
}

/// Source summary follows the optional shared death/occupancy reading.
pub(crate) fn log_vehicle_scan_summary(facts: &FilmVehicleFacts, march: Option<&FilmMarchFacts>) {
    let empty = EquipmentCreationStats::default();
    let stats = facts.creations.as_ref().map_or(&empty, |s| &s.stats);
    let deaths = march.map_or(0, |m| {
        m.facts.deaths.iter().filter(|d| d.type_index == 40).count()
    });
    tracing::info!(
        slots = stats.slots,
        ancres = stats.anchors,
        creationsAcceptees = stats.accepted,
        imagesCles = facts.keyframes.times_us.len(),
        viesRecensees = facts.keyframes.seen_us.len(),
        echantillons = facts.positions.as_ref().map_or(0, |p| p.accepted().count()),
        evenements = facts.events.len(),
        viseesSansPosition = facts.aims.len(),
        mortsEcrites = deaths,
        lecturesDOccupation = march.map_or(0, |m| m.facts.occupancy.len()),
        "vehicules : balayage ti=40"
    );
}

#[cfg(test)]
mod observation_tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn native_vehicle_source_observations() {
        use crate::clients::hi::models::FilmChunk;
        let mut header = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/film-player-table-source-input-v41.zlib")[..],
        )
        .read_to_end(&mut header)
        .unwrap();
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/vehicle-source-logs-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(rows.len(), 152);
        for (i, row) in rows.into_iter().enumerate() {
            let hex = row["hex"].as_str().unwrap();
            let data: Vec<_> = (0..hex.len())
                .step_by(2)
                .map(|j| u8::from_str_radix(&hex[j..j + 2], 16).unwrap())
                .collect();
            let registry_bytes = header[..row["header_len"].as_u64().unwrap() as usize].to_vec();
            let registry = parse_registry(&registry_bytes).unwrap();
            let chunks: Vec<_> = [registry_bytes, data]
                .into_iter()
                .enumerate()
                .map(|(index, data)| FilmChunkData {
                    metadata: FilmChunk {
                        index: index as i32,
                        chunk_type: i32::from(index == 0),
                        start_time_offset_ms: 0,
                        duration_ms: 0,
                        size: data.len() as i64,
                        file_relative_path: String::new(),
                    },
                    data,
                })
                .collect();
            let map: FilmMapBounds = serde_json::from_value(row["map"].clone()).unwrap();
            let id = row["match_id"].as_str().unwrap();
            assert!(
                row["profile_restored"].as_bool().unwrap(),
                "native profile restore {i}"
            );
            let inherited: [usize; 2] = serde_json::from_value(row["initial_mpp"].clone()).unwrap();
            let calibrated: [usize; 2] = serde_json::from_value(row["passed_mpp"].clone()).unwrap();
            let widths = resolve_film_mpp(registry.format_version)
                .creation_widths(Some(calibrated), inherited);
            let logs = super::super::log_test_support::capture_logs(|| {
                let facts = scan_film_vehicle_facts_observed(
                    &chunks,
                    &registry,
                    &map,
                    widths,
                    &map.position_encoding(),
                    Some(id),
                );
                assert_eq!(facts.scanned, row["scanned"].as_bool().unwrap(), "scan {i}");
                if row["missing_vehicle_archetype"].as_bool().unwrap() {
                    assert!(!facts.scanned);
                    assert!(facts.issues.iter().any(|s| s.contains("vehicle archetype")));
                    assert!(facts.creations.as_ref().is_some_and(|s| s.stats.slots > 0));
                }
                if facts.scanned {
                    assert_eq!(
                        facts.issues.iter().any(|s| s.contains("aucun slot biped")),
                        !row["has_biped"].as_bool().unwrap(),
                        "aim availability {i}"
                    );
                    assert_eq!(
                        facts.creations.as_ref().unwrap().records,
                        serde_json::from_value::<Vec<EquipmentCreation>>(row["creations"].clone())
                            .unwrap(),
                        "creations {i}"
                    );
                    assert_eq!(
                        facts.creations.as_ref().unwrap().stats,
                        serde_json::from_value::<EquipmentCreationStats>(row["stats"].clone())
                            .unwrap(),
                        "creation stats {i}"
                    );
                    let positions: Vec<_> = facts
                        .positions
                        .as_ref()
                        .unwrap()
                        .accepted()
                        .map(|p| ReplayVehiclePosition::from(p).position)
                        .collect();
                    assert_eq!(
                        positions,
                        serde_json::from_value::<Vec<ReplayPlayerPosition>>(
                            row["positions"].clone()
                        )
                        .unwrap(),
                        "positions {i}"
                    );
                    assert_eq!(
                        facts.events,
                        serde_json::from_value::<Vec<VehicleEvent>>(row["events"].clone()).unwrap(),
                        "events {i}"
                    );
                    assert_eq!(
                        facts.aims,
                        serde_json::from_value::<Vec<BipedAim>>(row["aims"].clone()).unwrap(),
                        "aims {i}"
                    );
                    // Earlier cases inject native march facts for source composition.
                    // The positive death/occupancy cases below run the Rust march.
                    let m = &row["march"];
                    let march = FilmMarchFacts {
                        facts: MarchRecordFacts {
                            deaths: serde_json::from_value(m["deaths"].clone()).unwrap(),
                            occupancy: serde_json::from_value(m["occupancy"].clone()).unwrap(),
                            coverage: serde_json::from_value(m["coverage"].clone()).unwrap(),
                        },
                        event_packets: m["events"].as_u64().unwrap() as usize,
                        located_packets: m["located"].as_u64().unwrap() as usize,
                        calibration: Some(MarchCalibration {
                            encoding_error: None,
                            encoding: Some(FrameEncoding {
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
                                position: None,
                                extra_fields: false,
                                corruption_check: false,
                            }),
                            retained_default: m["retained"].as_bool().unwrap(),
                            best: Default::default(),
                            runner_up: Default::default(),
                        }),
                        ..Default::default()
                    };
                    let march = if i >= 120 {
                        let mut base = march
                            .calibration
                            .as_ref()
                            .unwrap()
                            .encoding
                            .as_ref()
                            .unwrap()
                            .clone();
                        base.position = Some(map.position_encoding());
                        let actual =
                            scan_film_march_facts(&chunks, &registry, &base, false).unwrap();
                        assert_eq!(actual.facts, march.facts, "actual source march facts {i}");
                        assert_eq!(
                            actual.event_packets, march.event_packets,
                            "event packets {i}"
                        );
                        assert_eq!(
                            actual.located_packets, march.located_packets,
                            "located packets {i}"
                        );
                        assert_eq!(
                            actual.calibration.as_ref().unwrap().retained_default,
                            march.calibration.as_ref().unwrap().retained_default,
                            "march calibration {i}"
                        );
                        actual
                    } else {
                        march
                    };
                    log_vehicle_death_reads(id, &march);
                    log_vehicle_scan_summary(&facts, Some(&march));
                }
            });
            assert_eq!(
                serde_json::to_value(logs).unwrap(),
                row["logs"],
                "source logs {i}"
            );
        }
    }

    #[test]
    fn native_vehicle_death_observations() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/vehicle-death-logs-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(rows.len(), 512);
        for (i, row) in rows.into_iter().enumerate() {
            let mut march = FilmMarchFacts {
                facts: MarchRecordFacts {
                    deaths: serde_json::from_value(row["deaths"].clone()).unwrap(),
                    coverage: serde_json::from_value(row["coverage"].clone()).unwrap(),
                    ..Default::default()
                },
                event_packets: row["events"].as_u64().unwrap() as usize,
                located_packets: row["located"].as_u64().unwrap() as usize,
                ..Default::default()
            };
            if let Some(retained) = row["retained"].as_bool() {
                march.calibration = Some(MarchCalibration {
                    encoding_error: None,
                    encoding: Some(FrameEncoding {
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
                        position: None,
                        extra_fields: false,
                        corruption_check: false,
                    }),
                    retained_default: retained,
                    best: Default::default(),
                    runner_up: Default::default(),
                });
            }
            let log = super::super::log_test_support::capture_log(|| {
                log_vehicle_death_reads(row["match_id"].as_str().unwrap(), &march);
            });
            assert_eq!(log, row["log"], "vehicle death observation {i}");
        }
    }
}
