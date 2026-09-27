//! Native i26 equipment-reference emissions, attributed to biped lives and packet time.
use super::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnitEquipmentEntry {
    pub value: u32,
    pub tail: u32,
    /// A closed entry remains in the list, with value and tail zero.
    pub present: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnitEquipmentRead {
    pub head: u32,
    pub entries: Vec<UnitEquipmentEntry>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnitEquipmentEmission {
    pub source: FilmPacket,
    pub slot: u32,
    pub record_start_bit: usize,
    pub read: UnitEquipmentRead,
    pub component: DecodedComponent,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnitEquipmentStream {
    /// Failed intermediate/target reads with raw fields, source and callbacks.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rejected_components: Vec<BipedChannelRejection>,
    /// Every attempted component in scan order, including intermediate fields and callbacks.
    /// Absence in old exports means this trace was unavailable.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub component_attempts: Vec<super::BipedComponentAttempt>,
    pub records: Vec<UnitEquipmentEmission>,
    pub stats: BipedChannelStats,
}
/// Recover the native list from decoded fields, including older portable exports
/// that predate typed reference observations. Values remain uninterpreted indexes.
pub fn read_unit_equipment(component: &DecodedComponent) -> Option<UnitEquipmentRead> {
    if component.name != "unit-equipment-component" {
        return None;
    }
    let field = |name: &str| {
        component
            .fields
            .iter()
            .find(|f| f.name == name)
            .map(|f| f.raw)
    };
    let head = field("head")? as u32;
    let count = field("count")?;
    if count > 7 {
        return None;
    }
    let mut entries = Vec::with_capacity(count as usize);
    for i in 0..count {
        let present = field(&format!("equipment[{i}].present"))? != 0;
        let (value, tail) = if present {
            (
                field(&format!("equipment[{i}].value"))? as u32,
                field(&format!("equipment[{i}].generation"))? as u32,
            )
        } else {
            (0, 0)
        };
        entries.push(UnitEquipmentEntry {
            value,
            tail,
            present,
        });
    }
    Some(UnitEquipmentRead { head, entries })
}
/// Native ScanUnitEquipment over existing raw position anchors. Includes anchors
/// rejected by movement filtering, zero-entry lists and closed entries. Retains
/// scan order; slots identify biped lives and are not resolved to player identities.
pub fn scan_unit_equipment(
    chunks: &[crate::clients::hi::models::FilmChunkData],
    registry: &FilmRegistry,
    positions: &BipedPositionStream,
    encoding: &PositionEncoding,
) -> Result<UnitEquipmentStream, DecodeError> {
    scan_unit_equipment_impl(
        chunks,
        registry,
        positions,
        super::biped_channels::BipedComponentReader::Legacy(encoding),
    )
}

/// Native direct biped walk under a complete inherited reader context.
/// Traversal width overrides and corruption guards do not apply here.
pub fn scan_unit_equipment_with_context(
    chunks: &[crate::clients::hi::models::FilmChunkData],
    registry: &FilmRegistry,
    positions: &BipedPositionStream,
    context: &super::FrameEncoding,
) -> Result<UnitEquipmentStream, DecodeError> {
    scan_unit_equipment_impl(
        chunks,
        registry,
        positions,
        super::biped_channels::BipedComponentReader::Context(context),
    )
}

fn scan_unit_equipment_impl(
    chunks: &[crate::clients::hi::models::FilmChunkData],
    registry: &FilmRegistry,
    positions: &BipedPositionStream,
    reader: super::biped_channels::BipedComponentReader<'_>,
) -> Result<UnitEquipmentStream, DecodeError> {
    if chunks.is_empty() {
        return Err(DecodeError::Missing("film chunks"));
    }
    if positions.slot_band.is_none() {
        return Err(DecodeError::Missing("biped slot band"));
    }
    let arch = registry
        .archetype(35)
        .ok_or(DecodeError::Missing("biped archetype"))?;
    let target = arch
        .components
        .iter()
        .take(64)
        .position(|n| n == "unit-equipment-component")
        .ok_or(DecodeError::Missing("biped equipment component"))? as u8;
    let packet_indices = super::fire_events::native_packet_indices(chunks);
    let bytes = super::fire_events::native_chunk_data(chunks);
    let mut out = UnitEquipmentStream::default();
    for candidate in &positions.candidates {
        // Native delta walking skips absent chunks before counting records.
        if !bytes.contains_key(&candidate.source.chunk_index) {
            continue;
        }
        out.stats.records += 1;
        let r = &candidate.record;
        if !r.component_indices.contains(&target) {
            continue;
        }
        out.stats.with_component += 1;
        let source = candidate.source;
        let data = bytes
            .get(&source.chunk_index)
            .and_then(|b| {
                b.get(
                    source.payload_offset
                        ..source.payload_offset.checked_add(source.payload_size)?,
                )
            })
            .ok_or(DecodeError::Truncated {
                chunk: source.chunk_index,
                offset: source.payload_offset,
            })?;
        let component = super::biped_channels::walk_biped_components_retaining_rejections(
            data,
            r,
            arch,
            reader,
            target,
            (
                source,
                packet_indices
                    .get(&(source.chunk_index, source.payload_offset))
                    .copied(),
            ),
            (&mut out.rejected_components, &mut out.component_attempts),
        )
        .into_iter()
        .find(|(id, _)| *id == target)
        .map(|(_, c)| c);
        let Some((component, read)) =
            component.and_then(|c| read_unit_equipment(&c).map(|r| (c, r)))
        else {
            out.stats.unread += 1;
            continue;
        };
        out.stats.read += 1;
        out.records.push(UnitEquipmentEmission {
            source,
            slot: r.slot,
            record_start_bit: r.start_bit,
            read,
            component,
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clients::hi::models::{FilmChunk, FilmChunkData};
    use std::io::Read;
    fn read_json(bytes: &[u8]) -> Vec<serde_json::Value> {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(bytes)
            .read_to_end(&mut raw)
            .unwrap();
        serde_json::from_slice(&raw).unwrap()
    }
    fn wire(r: &UnitEquipmentRead) -> serde_json::Value {
        serde_json::json!({"Head":r.head,"Entries":r.entries.iter().map(|e|serde_json::json!({"Val":e.value,"Tail":e.tail,"Present":e.present})).collect::<Vec<_>>()})
    }
    #[test]
    fn native_unit_equipment_scan() {
        let cases = read_json(include_bytes!("fixtures/unit-equipment-v41.json.zlib"));
        assert_eq!(cases.len(), 512);
        for (i, c) in cases.into_iter().enumerate() {
            let hex = c["hex"].as_str().unwrap();
            let bytes: Vec<u8> = (0..hex.len())
                .step_by(2)
                .map(|n| u8::from_str_radix(&hex[n..n + 2], 16).unwrap())
                .collect();
            let source = FilmPacket {
                chunk_index: 1,
                packet_type: 0,
                byte_2: 0,
                byte_3: 0,
                payload_offset: 0,
                payload_size: bytes.len(),
                timestamp_us: i as u64 * 1000,
            };
            let chunk = FilmChunkData {
                metadata: FilmChunk {
                    index: 1,
                    chunk_type: 2,
                    start_time_offset_ms: 0,
                    duration_ms: 0,
                    size: bytes.len() as i64,
                    file_relative_path: String::new(),
                },
                data: bytes,
            };
            let positions = BipedPositionStream {
                record_masks: vec![],
                slot_band: Some([512, 512]),
                options: Default::default(),
                candidates: vec![BipedPositionCandidate {
                    packet_index: None,
                    source,
                    record: BipedPositionRecord {
                        start_bit: 0,
                        position_bit: 0,
                        end_bit: c["at"].as_u64().unwrap() as usize - 2,
                        slot: 512,
                        generation: 1,
                        component_indices: serde_json::from_value(c["mask"].clone()).unwrap(),
                        quantized: [0; 3],
                        world: [0.; 3],
                        companions: Default::default(),
                    },
                    rejection: Some(BipedPositionRejection::Saturated),
                }],
            };
            let registry = FilmRegistry {
                major_version: 41,
                format_version: 27,
                end_byte: 0,
                truncated: false,
                archetypes: (0..36)
                    .map(|index| FilmArchetype {
                        index,
                        levels: vec![0; 64],
                        components: if index == 35 {
                            serde_json::from_value(c["names"].clone()).unwrap()
                        } else {
                            vec![]
                        },
                    })
                    .collect(),
            };
            let encoding = film_map_catalog().maps["cliffhanger"].position_encoding();
            let scan = scan_unit_equipment(&[chunk], &registry, &positions, &encoding).unwrap();
            assert_eq!(
                !scan.records.is_empty(),
                c["ok"].as_bool().unwrap(),
                "case {i}"
            );
            assert_eq!(scan.stats.records, 1);
            if let Some(r) = scan.records.first() {
                assert_eq!(wire(&r.read), c["read"], "read {i}");
                assert_eq!(r.source, source);
                assert_eq!(r.slot, 512);
                let mut legacy = r.component.clone();
                legacy.references.clear();
                assert_eq!(read_unit_equipment(&legacy).as_ref(), Some(&r.read));
            }
            let restored: UnitEquipmentStream =
                serde_json::from_slice(&serde_json::to_vec(&scan).unwrap()).unwrap();
            assert_eq!(restored, scan);
        }
    }
    #[test]
    #[ignore = "requires six downloaded v41 films"]
    fn local_unit_equipment_corpus() {
        compare_corpus(false);
    }
    #[test]
    #[ignore = "requires captured Cadet Blue v41 film; verifies positive Film export"]
    fn local_unit_equipment_film_export() {
        compare_corpus(true);
    }
    fn compare_corpus(film_export: bool) {
        for c in read_json(include_bytes!(
            "fixtures/unit-equipment-corpus-v41.json.zlib"
        )) {
            let folder = c["folder"].as_str().unwrap();
            if film_export && folder != "appearance/01-cadet-blue" {
                continue;
            }
            let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("experiments/films")
                .join(folder);
            let meta: serde_json::Value =
                serde_json::from_slice(&std::fs::read(root.join("film.json")).unwrap()).unwrap();
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
                    data: std::fs::read(root.join(c["file"].as_str().unwrap())).unwrap(),
                })
                .collect();
            let registry =
                parse_registry(&chunks.iter().find(|c| c.metadata.index == 0).unwrap().data)
                    .unwrap();
            let map: FilmMapBounds = serde_json::from_value(c["map"].clone()).unwrap();
            let encoding: PositionEncoding = serde_json::from_value(c["encoding"].clone()).unwrap();
            let positions =
                scan_biped_positions(&chunks, &map, BipedScanOptions::default(), &[]).unwrap();
            let scan = scan_unit_equipment(&chunks, &registry, &positions, &encoding).unwrap();
            if film_export {
                let film = LegacyFilm::try_from_chunks_with_map(
                    &chunks,
                    DecodeOptions::v41(),
                    "high ground",
                    RecordIdLayout {
                        low_bits: 11,
                        base: 0,
                    },
                )
                .unwrap();
                assert_eq!(film.unit_equipment_error, None);
                // LegacyFilm uses the inherited context, which also retains observations
                // from preceding components. The legacy scan above does not install
                // those hooks; compare full retention against the context path.
                let precision = &film.scan_precision.as_ref().unwrap().profile;
                let context = super::super::FrameEncoding {
                    keyframe_simulation_complete: Some(precision.simulation_complete),
                    native_id_low_bits: None,
                    ..precision.encoding.clone()
                };
                let contextual =
                    scan_unit_equipment_with_context(&chunks, &registry, &positions, &context)
                        .unwrap();
                assert_eq!(film.unit_equipment.as_ref(), Some(&contextual));
                let native_records: Vec<_> = contextual.records.iter().map(|r| {
                    serde_json::json!({"Slot":r.slot,"TimestampUS":r.source.timestamp_us,"Read":wire(&r.read)})
                }).collect();
                assert_eq!(serde_json::json!(native_records), c["scan"]);
                assert_eq!(contextual.stats, scan.stats);
                assert!(!scan.records.is_empty());
                let restored: LegacyFilm =
                    serde_json::from_slice(&serde_json::to_vec(&film).unwrap()).unwrap();
                assert_eq!(restored.unit_equipment, film.unit_equipment);
                assert_eq!(restored.unit_equipment_error, film.unit_equipment_error);
            }
            let records:Vec<_>=scan.records.iter().map(|r|serde_json::json!({"Slot":r.slot,"TimestampUS":r.source.timestamp_us,"Read":wire(&r.read)})).collect();
            assert_eq!(
                serde_json::to_value(records).unwrap(),
                c["scan"],
                "film {folder}"
            );
            eprintln!("equipment lists match {folder}: {}", scan.records.len());
        }
    }
}
