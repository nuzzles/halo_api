//! Recorded equipment-object state (ti=37), not inferred equipment uses.
use super::fire_events::{native_chunk_packets, native_chunk_prefix};
use super::navpoint_radial_scan::{indices, read};
use super::*;
use crate::clients::hi::models::FilmChunkData;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
const NAMES: [&str; 6] = [
    "equipment-deployed-component",
    "equipment-activated-component",
    "equipment-creator-component",
    "equipment-energy-component",
    "equipment-energy-delay-ticks-left-component",
    "equipment-charges-remaining-component",
];
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct EquipmentStateSample {
    pub slot: u32,
    #[serde(rename = "Gen")]
    pub generation: u32,
    pub chunk: i32,
    pub packet_index: usize,
    #[serde(rename = "TimestampUS")]
    pub timestamp_us: u64,
    /// Native field order: deployed, activated, creator, energy, energy delay, charges.
    pub seen: [bool; 6],
    pub present: [bool; 6],
    /// Raw recorded value; meaningful only where `present` is true.
    pub val: [u64; 6],
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct EquipmentStateStats {
    pub records: usize,
    pub with_any: usize,
    pub walked: usize,
    pub broken: usize,
    pub with_field: [usize; 6],
    pub read: [usize; 6],
    pub gated: [usize; 6],
    pub slots: usize,
    /// Exactly 64 native six-bit component indices.
    pub mask_census: Vec<usize>,
}
impl Default for EquipmentStateStats {
    fn default() -> Self {
        Self {
            records: 0,
            with_any: 0,
            walked: 0,
            broken: 0,
            with_field: [0; 6],
            read: [0; 6],
            gated: [0; 6],
            slots: 0,
            mask_census: vec![0; 64],
        }
    }
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EquipmentStateStream {
    pub samples: Vec<EquipmentStateSample>,
    pub stats: EquipmentStateStats,
    /// Ordered component attempts, including partial walks that publish no sample.
    /// Missing traces in older exports do not establish absence of reads.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub component_attempts: Vec<EquipmentComponentAttempt>,
}
/// Source-attributed scan evidence, not an accepted equipment state sample.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EquipmentComponentAttempt {
    pub source: FilmPacket,
    pub packet_index: usize,
    pub record_start_bit: usize,
    pub slot: u32,
    pub generation: u32,
    pub component_index: usize,
    pub status: Option<bool>,
    pub in_bounds: bool,
    pub component: DecodedComponent,
}
impl EquipmentStateStream {
    fn delta(
        &mut self,
        pay: &[u8],
        band: &BTreeSet<u32>,
        arch: &FilmArchetype,
        encoding: &FrameEncoding,
        (source, packet_index): (FilmPacket, usize),
    ) {
        let position = encoding.position.as_ref();
        let pos_bits = position.map_or(4 + 1 + 13 + 13 + 14, |p| {
            4 + p.index_bits
                + p.world_axis_bits
                    .unwrap_or([13, 13, 14])
                    .iter()
                    .sum::<usize>()
        });
        let Some(limit) = (pay.len() * 8).checked_sub(27 + pos_bits) else {
            return;
        };
        let wanted = NAMES.map(|n| arch.components.iter().position(|c| c == n));
        let mut p = 0;
        while p <= limit {
            let start = p;
            p += 1;
            let slot = read(pay, start + 1, 13) as u32;
            if read(pay, start, 1) != 1 || !band.contains(&slot) || read(pay, start + 16, 2) != 0 {
                continue;
            }
            let count = read(pay, start + 18, 3) as usize;
            if count == 0 {
                continue;
            }
            let Some(ids) = indices(pay, start + 21, count) else {
                continue;
            };
            if ids[0] != 0 {
                continue;
            }
            p = start + 21 + 6 * count;
            self.stats.records += 1;
            for &id in &ids {
                self.stats.mask_census[id] += 1;
            }
            let Some(last) = ids
                .iter()
                .copied()
                .filter(|id| wanted.contains(&Some(*id)))
                .max()
            else {
                continue;
            };
            self.stats.with_any += 1;
            for (f, id) in wanted.iter().enumerate() {
                if id.is_some_and(|id| ids.contains(&id)) {
                    self.stats.with_field[f] += 1;
                }
            }
            let mut sample = EquipmentStateSample {
                slot,
                generation: read(pay, start + 14, 2) as u32,
                chunk: source.chunk_index,
                packet_index,
                timestamp_us: source.timestamp_us,
                ..Default::default()
            };
            let mut at = p;
            let mut complete = false;
            for id in ids {
                let Some(name) = arch.components.get(id).filter(|n| !n.is_empty()) else {
                    break;
                };
                if at > pay.len() * 8 {
                    break;
                }
                let (status, c) = consume_component_at(
                    pay,
                    at,
                    name,
                    37,
                    arch.levels.get(id).copied().unwrap_or(0),
                    encoding,
                );
                self.component_attempts.push(EquipmentComponentAttempt {
                    source,
                    packet_index,
                    record_start_bit: start,
                    slot,
                    generation: sample.generation,
                    component_index: id,
                    status,
                    in_bounds: c.end_bit >= 0 && c.end_bit <= (pay.len() * 8) as i64,
                    component: c.clone(),
                });
                if status != Some(true) || c.end_bit < 0 || c.end_bit > (pay.len() * 8) as i64 {
                    break;
                }
                at = usize::try_from(c.end_bit).expect("in-bounds component endpoint");
                for obs in c.diagnostics.component_observations {
                    if let FilmComponentObservation::EquipmentState {
                        field,
                        value,
                        present,
                    } = obs
                    {
                        let f = match field {
                            NativeEquipmentField::Deployed => 0,
                            NativeEquipmentField::Activated => 1,
                            NativeEquipmentField::Creator => 2,
                            NativeEquipmentField::Energy => 3,
                            NativeEquipmentField::EnergyDelay => 4,
                            NativeEquipmentField::Charges => 5,
                        };
                        sample.seen[f] = true;
                        sample.present[f] = present;
                        sample.val[f] = value;
                    }
                }
                if id == last {
                    complete = true;
                    break;
                }
            }
            if complete {
                self.stats.walked += 1;
                for f in 0..6 {
                    if sample.seen[f] {
                        self.stats.read[f] += 1;
                        if !sample.present[f] {
                            self.stats.gated[f] += 1;
                        }
                    }
                }
                self.samples.push(sample);
            } else {
                self.stats.broken += 1;
            }
        }
    }
}
/// Scan contiguous loaded chunks in packet/bit order with the complete reader context.
/// Failed walks contribute counters but never publish partial state samples.
pub fn scan_equipment_state_for_band(
    chunks: &[FilmChunkData],
    band: &BTreeSet<u32>,
    registry: &FilmRegistry,
    encoding: &FrameEncoding,
) -> Result<EquipmentStateStream, DecodeError> {
    let (stream, error) =
        scan_equipment_state_for_band_with_diagnostics(chunks, band, registry, encoding);
    error.map_or(Ok(stream), Err)
}
/// Preserve native counters even if the registry lookup fails after slot discovery.
pub fn scan_equipment_state_for_band_with_diagnostics(
    chunks: &[FilmChunkData],
    band: &BTreeSet<u32>,
    registry: &FilmRegistry,
    encoding: &FrameEncoding,
) -> (EquipmentStateStream, Option<DecodeError>) {
    let mut stream = EquipmentStateStream::default();
    let error = scan_into(chunks, band, registry, encoding, &mut stream).err();
    (stream, error)
}
fn scan_into(
    chunks: &[FilmChunkData],
    band: &BTreeSet<u32>,
    registry: &FilmRegistry,
    encoding: &FrameEncoding,
    out: &mut EquipmentStateStream,
) -> Result<(), DecodeError> {
    let chunks = native_chunk_prefix(chunks)?;
    if band.is_empty() {
        return Err(DecodeError::Missing("equipment object slot band"));
    }
    out.stats.slots = band.len();
    let arch = registry
        .archetype(37)
        .ok_or(DecodeError::Missing("equipment archetype 37"))?;
    for c in chunks {
        for (index, pk) in native_chunk_packets(c).iter().enumerate() {
            if pk.packet_type == 0 {
                out.delta(
                    &c.data[pk.payload_offset..pk.payload_offset + pk.payload_size],
                    band,
                    arch,
                    encoding,
                    (*pk, index),
                );
            }
        }
    }
    Ok(())
}
/// Resolve the native equipment slot band before scanning its state.
pub fn scan_equipment_state(
    chunks: &[FilmChunkData],
    registry: &FilmRegistry,
    encoding: &FrameEncoding,
) -> Result<EquipmentStateStream, DecodeError> {
    let (stream, error) = scan_equipment_state_with_diagnostics(chunks, registry, encoding);
    error.map_or(Ok(stream), Err)
}
/// Loaded native scan retaining the partial result alongside any source/registry error.
pub fn scan_equipment_state_with_diagnostics(
    chunks: &[FilmChunkData],
    registry: &FilmRegistry,
    encoding: &FrameEncoding,
) -> (EquipmentStateStream, Option<DecodeError>) {
    match world_object_slot_band(chunks, 37) {
        Ok(band) => {
            scan_equipment_state_for_band_with_diagnostics(chunks, &band, registry, encoding)
        }
        Err(error) => (EquipmentStateStream::default(), Some(error)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn equipment_error_diagnostics_survive() {
        use crate::clients::hi::models::FilmChunk;
        let encoding:FrameEncoding=serde_json::from_value(serde_json::json!({
            "ids":{"low_bits":13,"base":0},"mpp_widths":[9,5],"extra_fields":false,"corruption_check":false
        })).unwrap();
        let registry = FilmRegistry {
            archetypes: vec![],
            major_version: 41,
            format_version: 27,
            end_byte: 0,
            truncated: false,
        };
        let band = BTreeSet::from([512, 513]);
        let (empty, error) = scan_equipment_state_with_diagnostics(&[], &registry, &encoding);
        assert!(matches!(
            error,
            Some(DecodeError::Missing("readable film chunks"))
        ));
        assert_eq!(empty.stats, EquipmentStateStats::default());
        let chunks = [FilmChunkData {
            metadata: FilmChunk {
                index: 1,
                chunk_type: 2,
                start_time_offset_ms: 0,
                duration_ms: 0,
                size: 0,
                file_relative_path: String::new(),
            },
            data: vec![],
        }];
        let (partial, error) =
            scan_equipment_state_for_band_with_diagnostics(&chunks, &band, &registry, &encoding);
        assert!(matches!(
            error,
            Some(DecodeError::Missing("equipment archetype 37"))
        ));
        assert_eq!(partial.stats.slots, 2);
        assert_eq!(partial.stats.mask_census, vec![0; 64]);
        assert!(partial.samples.is_empty());
        let restored: EquipmentStateStream =
            serde_json::from_value(serde_json::to_value(&partial).unwrap()).unwrap();
        assert_eq!(restored, partial);
        let (empty, error) = scan_equipment_state_for_band_with_diagnostics(
            &chunks,
            &BTreeSet::new(),
            &registry,
            &encoding,
        );
        assert!(matches!(
            error,
            Some(DecodeError::Missing("equipment object slot band"))
        ));
        assert_eq!(empty.stats.slots, 0);
    }
    #[test]
    fn native_equipment_state() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/equipment-state-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(rows.len(), 512);
        let mut reads = [0; 6];
        let mut broken = 0;
        let mut gated = [0; 6];
        let mut emissions = 0;
        let mut rejected_callbacks = 0;
        for (case, row) in rows.iter().enumerate() {
            let h = row["hex"].as_str().unwrap();
            let pay: Vec<u8> = (0..h.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap())
                .collect();
            let encoding:FrameEncoding=serde_json::from_value(serde_json::json!({
                "ids":{"low_bits":13,"base":0},"mpp_widths":[9,5],"extra_fields":false,"corruption_check":false,
                "keyframe_simulation_complete":row["simulation"],
                "position":{"index_bits":1,"world_axis_bits":[6,7,8],"handle_bits":1,"traversal_axis_bits":[6,6,6],"default_axis_bits":[13,13,14],"region_axis_bits":{},"delta_axis_bits":[6,6,6],"full_precision":false,"writer_absolute":false,"delta_handle_tail":false}
            })).unwrap();
            let arch = FilmArchetype {
                index: 37,
                components: serde_json::from_value(row["names"].clone()).unwrap(),
                levels: vec![0; 7],
            };
            let mut got = EquipmentStateStream {
                stats: EquipmentStateStats {
                    slots: 2,
                    mask_census: vec![0; 64],
                    ..Default::default()
                },
                ..Default::default()
            };
            let source = FilmPacket {
                chunk_index: 2,
                packet_type: 0,
                byte_2: 0,
                byte_3: 0,
                payload_offset: 16,
                payload_size: pay.len(),
                timestamp_us: 123456,
            };
            got.delta(
                &pay,
                &BTreeSet::from([512, 513]),
                &arch,
                &encoding,
                (source, 3),
            );
            let mut actual = Vec::new();
            for attempt in &got.component_attempts {
                assert_eq!(attempt.source, source);
                assert_eq!(attempt.packet_index, 3);
                assert!(attempt.component.start_bit >= attempt.record_start_bit as i64);
                assert_eq!(
                    attempt.in_bounds,
                    attempt.component.end_bit >= 0
                        && attempt.component.end_bit <= (pay.len() * 8) as i64
                );
                for obs in &attempt.component.diagnostics.component_observations {
                    if let FilmComponentObservation::EquipmentState {
                        field,
                        value,
                        present,
                    } = obs
                    {
                        let field = match field {
                            NativeEquipmentField::Deployed => 0,
                            NativeEquipmentField::Activated => 1,
                            NativeEquipmentField::Creator => 2,
                            NativeEquipmentField::Energy => 3,
                            NativeEquipmentField::EnergyDelay => 4,
                            NativeEquipmentField::Charges => 5,
                        };
                        actual.push(serde_json::json!({"slot":attempt.slot,"generation":attempt.generation,"field":field,"value":value,"present":present}));
                        rejected_callbacks +=
                            usize::from(attempt.status != Some(true) || !attempt.in_bounds);
                    }
                }
            }
            emissions += actual.len();
            assert_eq!(
                serde_json::json!(actual),
                row["emissions"],
                "all callbacks {case}"
            );
            let mut json = serde_json::to_value(&got).unwrap();
            assert_eq!(
                serde_json::from_value::<EquipmentStateStream>(json.clone()).unwrap(),
                got
            );
            json.as_object_mut().unwrap().remove("component_attempts");
            let old: EquipmentStateStream = serde_json::from_value(json).unwrap();
            assert!(old.component_attempts.is_empty());
            assert_eq!(old.samples, got.samples);
            assert_eq!(
                serde_json::to_value(&got.samples).unwrap(),
                row["samples"],
                "samples {case}"
            );
            assert_eq!(
                serde_json::to_value(&got.stats).unwrap(),
                row["stats"],
                "stats {case}"
            );
            for f in 0..6 {
                reads[f] += got.stats.read[f];
                gated[f] += got.stats.gated[f];
            }
            broken += got.stats.broken;
        }
        assert!(reads.iter().all(|&n| n > 0));
        assert!(broken > 0);
        assert!(gated[1] > 0 && gated[2] > 0);
        assert_eq!(emissions, 5339);
        assert!(rejected_callbacks > 0);
        eprintln!("retained equipment callbacks={emissions}, rejected={rejected_callbacks}");
        eprintln!("equipment reads={reads:?} gated={gated:?} broken={broken}");
    }
    #[test]
    #[ignore = "requires four captured v41 films"]
    fn local_equipment_state_corpus() {
        use crate::clients::hi::models::{FilmChunk, FilmChunkData};
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/equipment-state-corpus-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(rows.len(), 8);
        let mut total = 0;
        for row in rows {
            let folder = row["folder"].as_str().unwrap();
            let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("experiments/films")
                .join(folder);
            let meta: serde_json::Value =
                serde_json::from_slice(&std::fs::read(root.join("film.json")).unwrap()).unwrap();
            assert_eq!(meta["film_major_version"], 41);
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
                    data: std::fs::read(root.join(c["file"].as_str().unwrap())).unwrap(),
                })
                .collect();
            let registry = decode_registry(&chunks).unwrap();
            let encoding:FrameEncoding=serde_json::from_value(serde_json::json!({
                "ids":{"low_bits":13,"base":0},"mpp_widths":row["mpp"],"extra_fields":false,"corruption_check":false,
                "position":row["position"],"keyframe_simulation_complete":row["simulation"]
            })).unwrap();
            let got = scan_equipment_state(&chunks, &registry, &encoding).unwrap();
            assert_eq!(
                serde_json::to_value(&got.samples).unwrap(),
                row["records"],
                "samples {folder}"
            );
            assert_eq!(
                serde_json::to_value(&got.stats).unwrap(),
                row["stats"],
                "stats {folder}"
            );
            total += got.samples.len();
            eprintln!(
                "{folder} automatic={} samples={}",
                row["automatic"],
                got.samples.len()
            );
        }
        assert_eq!(total, 29852);
    }
}
