//! Counter-gated recovery of missing ability emissions.
use super::bits::Bits;
use super::equipment_changes::equipment_counter_step;
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EquipmentRecoveryWindow {
    pub slot: u32,
    pub from_counter: u8,
    pub to_counter: u8,
    pub missing: u8,
    pub min_time_us: u64,
    pub max_time_us: u64,
    pub min_chunk: i64,
    pub max_chunk: i64,
    pub head: bool,
    pub candidates: Vec<EquipmentEmission>,
}
pub fn equipment_recovery_windows(
    strict: &[EquipmentEmission],
    births: &BTreeMap<u32, u64>,
) -> Vec<EquipmentRecoveryWindow> {
    let mut slots = BTreeMap::<u32, Vec<&EquipmentEmission>>::new();
    for e in strict {
        slots.entry(e.ability.slot).or_default().push(e);
    }
    let mut out = Vec::new();
    for (slot, mut list) in slots {
        list.sort_by_key(|e| e.film_order());
        let first = &list[0].ability;
        if first.counter != 5
            && let Some(&birth) = births
                .get(&slot)
                .filter(|&&b| b < first.source.timestamp_us)
        {
            out.push(EquipmentRecoveryWindow {
                slot,
                from_counter: 4,
                to_counter: first.counter,
                missing: equipment_counter_step(5, first.counter),
                min_time_us: birth,
                max_time_us: first.source.timestamp_us,
                min_chunk: 1,
                max_chunk: list[0].native_chunk_number(),
                head: true,
                candidates: vec![],
            });
        }
        for pair in list.windows(2) {
            let a = &pair[0].ability;
            let b = &pair[1].ability;
            let step = equipment_counter_step(a.counter, b.counter);
            if step <= 1 {
                continue;
            }
            out.push(EquipmentRecoveryWindow {
                slot,
                from_counter: a.counter,
                to_counter: b.counter,
                missing: step - 1,
                min_time_us: a.source.timestamp_us,
                max_time_us: b.source.timestamp_us,
                min_chunk: pair[0].native_chunk_number(),
                max_chunk: pair[1].native_chunk_number(),
                head: false,
                candidates: vec![],
            });
        }
    }
    out.sort_by_key(|w| (w.min_time_us, w.slot));
    out
}
pub fn accept_equipment_recovery(window: &EquipmentRecoveryWindow) -> Vec<EquipmentEmission> {
    if window.missing == 0 {
        return vec![];
    }
    let mut predicted = [None; 8];
    for i in 0..window.missing {
        predicted[usize::from((window.from_counter + 1 + i) % 8)] = Some(i);
    }
    let mut seen = [false; 8];
    let mut kept = Vec::new();
    for c in &window.candidates {
        let counter = usize::from(c.ability.counter % 8);
        if predicted[counter].is_none() {
            continue;
        }
        if seen[counter] {
            return vec![];
        }
        seen[counter] = true;
        let mut c = c.clone();
        let Some(origin) = c.recovery.as_mut() else {
            continue;
        };
        origin.head = window.head;
        kept.push(c);
    }
    kept.sort_by_key(|c| {
        (
            c.ability.source.timestamp_us,
            c.recovery.as_ref().unwrap().bit_offset,
        )
    });
    let mut holes = 0;
    let mut prev = window.from_counter;
    for (i, c) in kept.iter().enumerate() {
        if i > 0
            && predicted[usize::from(c.ability.counter % 8)]
                <= predicted[usize::from(kept[i - 1].ability.counter % 8)]
        {
            return vec![];
        }
        holes += usize::from(equipment_counter_step(prev, c.ability.counter) != 1);
        prev = c.ability.counter;
    }
    holes += usize::from(equipment_counter_step(prev, window.to_counter) != 1);
    if holes > 1 { vec![] } else { kept }
}
#[cfg(test)]
fn recovery_at(
    data: &[u8],
    p: usize,
    arch: &FilmArchetype,
    map: &FilmMapBounds,
    position: &PositionEncoding,
) -> Option<(u8, Option<u8>)> {
    recovery_at_with_reader(
        data,
        p,
        arch,
        map,
        super::biped_channels::BipedComponentReader::Legacy(position),
    )
}
#[cfg(test)]
fn recovery_at_with_reader(
    data: &[u8],
    p: usize,
    arch: &FilmArchetype,
    map: &FilmMapBounds,
    reader: super::biped_channels::BipedComponentReader<'_>,
) -> Option<(u8, Option<u8>)> {
    recovery_at_observed(data, p, arch, map, reader, |_, _, _, _| {})
}
fn recovery_at_observed(
    data: &[u8],
    p: usize,
    arch: &FilmArchetype,
    map: &FilmMapBounds,
    reader: super::biped_channels::BipedComponentReader<'_>,
    observe: impl FnMut(u8, Option<bool>, &DecodedComponent, bool),
) -> Option<(u8, Option<u8>)> {
    let bits = Bits(data);
    let mut ids = Vec::new();
    let at;
    if bits.read(p + 17, 1)? == 0 {
        let n = bits.read(p + 18, 3)? as usize;
        if !(2..=7).contains(&n) {
            return None;
        }
        for i in 0..n {
            let id = bits.read(p + 21 + 6 * i, 6)? as u8;
            if ids.last().is_some_and(|&v| v >= id) {
                return None;
            }
            ids.push(id);
        }
        if ids[0] == 0 || !ids.contains(&48) {
            return None;
        }
        at = p + 21 + 6 * n;
    } else {
        let mask = bits.read(p + 18, 64)?;
        for i in 0..64 {
            if mask & (1u64 << i) != 0 {
                ids.push(i as u8);
            }
        }
        if !(2..=40).contains(&ids.len()) || ids[0] != 0 || !ids.contains(&48) {
            return None;
        }
        let i0 = p + 82;
        let region_bits = map.region_index_bits.max(1);
        if bits.read(i0, 4)? != 0 || bits.read(i0 + 4, region_bits)? != u64::from(map.region) {
            return None;
        }
        at = i0 + 4 + region_bits + map.axis_widths.iter().sum::<usize>() + 2;
        if at - 2 > bits.len() {
            return None;
        }
        ids.remove(0);
    }
    let components = super::biped_channels::walk_biped_components_at_observed(
        data, at, &ids, arch, reader, 48, observe,
    );
    let (_, c) = components.into_iter().find(|(id, _)| *id == 48)?;
    if !matches!(
        c.name.as_str(),
        "biped-desired-ability-set-component" | "biped-desired-ability-set"
    ) {
        return None;
    }
    let counter = c.fields.iter().find(|f| f.name == "counter")?.raw as u8;
    let rank = c
        .fields
        .iter()
        .find(|f| f.name == "rank")
        .map(|f| f.raw as u8);
    Some((counter, rank))
}
/// A component consumed while probing a speculative equipment recovery offset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EquipmentRecoveryComponentRead {
    pub component_index: u8,
    pub status: Option<bool>,
    pub in_bounds: bool,
    pub component: DecodedComponent,
}
/// Diagnostic evidence from a recovery probe, not an established native record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EquipmentRecoveryAttempt {
    pub source: FilmPacket,
    pub packet_index: Option<usize>,
    pub slot: u32,
    /// Candidate header offset within the packet payload.
    pub bit_offset: usize,
    /// The component walk reached the target and produced a recovery candidate.
    /// Counter/window validation may still reject that candidate.
    pub candidate: bool,
    pub components: Vec<EquipmentRecoveryComponentRead>,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EquipmentChangeStream {
    /// All component-consuming speculative probes, including failed walks.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub recovery_attempts: Vec<EquipmentRecoveryAttempt>,
    pub assembly: EquipmentChanges,
    pub walk: BipedChannelStats,
    pub recovery_windows: Vec<EquipmentRecoveryWindow>,
}
/// Complete strict/recovery assembly, with birth witnesses supplied by accepted positions.
pub fn scan_equipment_changes(
    chunks: &[crate::clients::hi::models::FilmChunkData],
    registry: &FilmRegistry,
    positions: &BipedPositionStream,
    channels: &BipedChannels,
    map: &FilmMapBounds,
) -> Result<EquipmentChangeStream, DecodeError> {
    scan_equipment_changes_with_position(
        chunks,
        registry,
        positions,
        channels,
        map,
        &map.position_encoding(),
    )
}
/// Use the same calibrated component precision as the strict ability scan.
pub fn scan_equipment_changes_with_position(
    chunks: &[crate::clients::hi::models::FilmChunkData],
    registry: &FilmRegistry,
    positions: &BipedPositionStream,
    channels: &BipedChannels,
    map: &FilmMapBounds,
    position: &PositionEncoding,
) -> Result<EquipmentChangeStream, DecodeError> {
    scan_equipment_changes_impl(
        chunks,
        registry,
        positions,
        channels,
        map,
        super::biped_channels::BipedComponentReader::Legacy(position),
    )
}
/// Use the strict scan's complete direct-reader context for recovery too.
pub fn scan_equipment_changes_with_context(
    chunks: &[crate::clients::hi::models::FilmChunkData],
    registry: &FilmRegistry,
    positions: &BipedPositionStream,
    channels: &BipedChannels,
    map: &FilmMapBounds,
    context: &FrameEncoding,
) -> Result<EquipmentChangeStream, DecodeError> {
    scan_equipment_changes_impl(
        chunks,
        registry,
        positions,
        channels,
        map,
        super::biped_channels::BipedComponentReader::Context(context),
    )
}
fn scan_equipment_changes_impl(
    chunks: &[crate::clients::hi::models::FilmChunkData],
    registry: &FilmRegistry,
    positions: &BipedPositionStream,
    channels: &BipedChannels,
    map: &FilmMapBounds,
    reader: super::biped_channels::BipedComponentReader<'_>,
) -> Result<EquipmentChangeStream, DecodeError> {
    let arch = registry
        .archetype(35)
        .ok_or(DecodeError::Missing("biped archetype"))?;
    let mut births = BTreeMap::new();
    for p in positions.accepted() {
        births
            .entry(p.record.slot)
            .and_modify(|t: &mut u64| *t = (*t).min(p.source.timestamp_us))
            .or_insert(p.source.timestamp_us);
    }
    let mut emissions: Vec<_> = channels
        .ability_emissions()
        .map(|ability| EquipmentEmission {
            chunk_number: None,
            ability,
            recovery: None,
        })
        .collect();
    let mut windows = equipment_recovery_windows(&emissions, &births);
    let mut recovery_attempts = Vec::new();
    // Native recovery uses FilmContext.ChunkAt for each numbered window chunk.
    // It selects the first matching metadata entry, regardless of chunk type,
    // and keeps the packet prefix before an End or malformed trailing packet.
    let mut selected = BTreeMap::new();
    for chunk in chunks {
        selected.entry(chunk.metadata.index).or_insert(chunk);
    }
    let packets: Vec<_> = selected
        .values()
        .flat_map(|chunk| {
            super::fire_events::native_chunk_packets(chunk)
                .into_iter()
                .enumerate()
        })
        .collect();
    let packet_indices: BTreeMap<_, _> = packets
        .iter()
        .map(|(index, packet)| ((packet.chunk_index, packet.payload_offset), *index))
        .collect();
    if !windows.is_empty() {
        for (_, source) in packets.into_iter().filter(|(_, p)| p.packet_type == 0) {
            let active: Vec<_> = windows
                .iter()
                .enumerate()
                .filter(|(_, w)| {
                    w.min_chunk <= i64::from(source.chunk_index)
                        && i64::from(source.chunk_index) <= w.max_chunk
                        && w.min_time_us <= source.timestamp_us
                        && source.timestamp_us <= w.max_time_us
                })
                .map(|(i, _)| i)
                .collect();
            if active.is_empty() {
                continue;
            }
            let data = &selected[&source.chunk_index].data
                [source.payload_offset..source.payload_offset + source.payload_size];
            let bits = Bits(data);
            for p in 0..bits.len().saturating_sub(26) {
                if bits.read(p, 1) != Some(1) {
                    continue;
                }
                let slot = bits.read(p + 1, 13).unwrap() as u32;
                let Some(&win) = active.iter().find(|&&i| windows[i].slot == slot) else {
                    continue;
                };
                if bits.read(p + 14, 2) != Some(1) || bits.read(p + 16, 1) != Some(0) {
                    continue;
                }
                let mut components = Vec::new();
                let recovered = recovery_at_observed(
                    data,
                    p,
                    arch,
                    map,
                    reader,
                    |id, status, component, in_bounds| {
                        components.push(EquipmentRecoveryComponentRead {
                            component_index: id,
                            status,
                            in_bounds,
                            component: component.clone(),
                        });
                    },
                );
                if !components.is_empty() {
                    recovery_attempts.push(EquipmentRecoveryAttempt {
                        source,
                        packet_index: packet_indices
                            .get(&(source.chunk_index, source.payload_offset))
                            .copied(),
                        slot,
                        bit_offset: p,
                        candidate: recovered.is_some(),
                        components,
                    });
                }
                if let Some((counter, rank)) = recovered {
                    windows[win].candidates.push(EquipmentEmission {
                        chunk_number: None,
                        ability: BipedAbilityEmission {
                            packet_index: packet_indices
                                .get(&(source.chunk_index, source.payload_offset))
                                .copied(),
                            source,
                            slot,
                            counter,
                            rank,
                        },
                        recovery: Some(EquipmentRecoveryOrigin {
                            bit_offset: p,
                            head: false,
                        }),
                    });
                }
            }
        }
        for w in &windows {
            emissions.extend(accept_equipment_recovery(w));
        }
    }
    let mut assembly = assemble_equipment_changes(&emissions, &births);
    for change in &mut assembly.records {
        change.packet_index = packet_indices
            .get(&(change.source.chunk_index, change.source.payload_offset))
            .copied();
    }
    Ok(EquipmentChangeStream {
        recovery_attempts,
        assembly,
        walk: channels.ability_stats.clone(),
        recovery_windows: windows,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
    use std::io::Read;

    #[test]
    fn native_equipment_recovery_source_boundaries() {
        use crate::clients::hi::models::{FilmChunk, FilmChunkData};
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/equipment-recovery-source-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(rows.len(), 24);
        let mut positive = 0;
        for (case, row) in rows.iter().enumerate() {
            let chunks: Vec<_> = row["inputs"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| {
                    let hex = v["hex"].as_str().unwrap();
                    let data: Vec<_> = (0..hex.len())
                        .step_by(2)
                        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                        .collect();
                    FilmChunkData {
                        metadata: FilmChunk {
                            index: v["index"].as_i64().unwrap() as i32,
                            chunk_type: v["chunk_type"].as_i64().unwrap() as i32,
                            start_time_offset_ms: 0,
                            duration_ms: 0,
                            size: data.len() as i64,
                            file_relative_path: String::new(),
                        },
                        data,
                    }
                })
                .collect();
            let registry = FilmRegistry {
                major_version: 41,
                format_version: 27,
                end_byte: 0,
                truncated: false,
                archetypes: (0..36)
                    .map(|index| FilmArchetype {
                        index,
                        components: if index == 35 {
                            serde_json::from_value(row["names"].clone()).unwrap()
                        } else {
                            vec![]
                        },
                        levels: vec![0; 64],
                    })
                    .collect(),
            };
            let map = FilmMapBounds {
                module: String::new(),
                min: [-1.; 3],
                max: [1.; 3],
                axis_widths: [6, 7, 8],
                region: 0,
                region_index_bits: 2,
            };
            let positions = BipedPositionStream {
                record_masks: vec![],
                slot_band: None,
                options: Default::default(),
                candidates: vec![],
            };
            let mut channels = BipedChannels::default();
            for (counter, chunk, timestamp_us) in [(5u8, 1, 100), (7, 3, 300)] {
                let ComponentDecode::Decoded(component) = decode_component(
                    &[counter << 5, 0x80],
                    0,
                    "biped-desired-ability-set-component",
                    0,
                    35,
                ) else {
                    panic!("ability input")
                };
                channels.abilities.push(BipedChannelRead {
                    packet_index: None,
                    source: FilmPacket {
                        chunk_index: chunk,
                        packet_type: 0,
                        byte_2: 0,
                        byte_3: 0,
                        payload_offset: 0,
                        payload_size: 2,
                        timestamp_us,
                    },
                    slot: 512,
                    record_start_bit: 0,
                    component,
                });
            }
            let result = scan_equipment_changes(&chunks, &registry, &positions, &channels, &map)
                .unwrap_or_else(|e| panic!("case {case}: {e}"));
            let recovered: Vec<_> = result
                .recovery_windows
                .iter()
                .flat_map(accept_equipment_recovery)
                .collect();
            let actual: Vec<_> = recovered.iter().map(|r| json!({"slot":r.ability.slot,"chunk":r.ability.source.chunk_index,
                "packet":r.ability.packet_index,"time":r.ability.source.timestamp_us,"counter":r.ability.counter,
                "rank":r.ability.rank,"bit":r.recovery.as_ref().unwrap().bit_offset,"head":r.recovery.as_ref().unwrap().head,"dense":false})).collect();
            assert_eq!(json!(actual), row["output"], "case {case}");
            positive += usize::from(!actual.is_empty());
            let published: Vec<_> = result
                .assembly
                .records
                .iter()
                .filter(|r| r.recovered)
                .collect();
            assert_eq!(published.len(), recovered.len(), "assembly case {case}");
            assert_eq!(result.assembly.stats.recovered, recovered.len());
            for (change, emission) in published.iter().zip(&recovered) {
                assert_eq!(change.packet_index, emission.ability.packet_index);
                assert_eq!(change.source, emission.ability.source);
                assert_eq!(change.counter, emission.ability.counter);
                assert_eq!(change.rank, emission.ability.rank);
                assert_eq!(change.slot, emission.ability.slot);
            }
            assert_eq!(
                serde_json::from_value::<EquipmentChangeStream>(json!(result)).unwrap(),
                result
            );
        }
        assert_eq!(positive, 12);
    }

    #[test]
    fn native_remaining_biped_contexts() {
        use crate::clients::hi::models::{FilmChunk, FilmChunkData};
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/biped-remaining-context-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(rows.len(), 512);
        assert_eq!(
            rows.iter().filter(|r| r["simulation"] == false).count(),
            256
        );
        for dense in [false, true] {
            for enabled in [false, true] {
                assert_eq!(
                    rows.iter()
                        .filter(|r| r["recovery"]["dense"] == dense && r["simulation"] == enabled)
                        .count(),
                    128
                );
            }
        }
        let bytes = |v: &Value| {
            let h = v.as_str().unwrap();
            (0..h.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap())
                .collect::<Vec<u8>>()
        };
        for (case, row) in rows.iter().enumerate() {
            let context:FrameEncoding=serde_json::from_value(json!({
                "ids":{"low_bits":13,"base":0},"mpp_widths":[9,5],"position":row["encoding"],
                "keyframe_simulation_complete":row["simulation"],"extra_fields":false,"corruption_check":false,
            })).unwrap();
            let mut archetypes: Vec<_> = (0..36)
                .map(|index| FilmArchetype {
                    index,
                    components: vec![],
                    levels: vec![],
                })
                .collect();
            archetypes[35] = FilmArchetype {
                index: 35,
                components: serde_json::from_value(row["names"].clone()).unwrap(),
                levels: vec![0; 64],
            };
            let registry = FilmRegistry {
                archetypes,
                major_version: 41,
                format_version: 27,
                end_byte: 0,
                truncated: false,
            };
            let data = bytes(&row["hex"]);
            let source = FilmPacket {
                chunk_index: 1,
                packet_type: 0,
                byte_2: 0,
                byte_3: 0,
                payload_offset: 0,
                payload_size: data.len(),
                timestamp_us: 1000,
            };
            let chunk = |data: Vec<u8>| FilmChunkData {
                metadata: FilmChunk {
                    index: 1,
                    chunk_type: 2,
                    start_time_offset_ms: 0,
                    duration_ms: 3,
                    size: data.len() as i64,
                    file_relative_path: String::new(),
                },
                data,
            };
            let anchors = BipedPositionStream {
                record_masks: vec![],
                slot_band: Some([512, 512]),
                options: Default::default(),
                candidates: vec![BipedPositionCandidate {
                    packet_index: None,
                    source,
                    rejection: None,
                    record: BipedPositionRecord {
                        start_bit: 0,
                        position_bit: 0,
                        end_bit: row["start"].as_u64().unwrap() as usize - 2,
                        slot: 512,
                        generation: 1,
                        component_indices: vec![0, 6, 43],
                        quantized: [0; 3],
                        world: [0.0; 3],
                        companions: Default::default(),
                    },
                }],
            };
            let weapons = scan_held_weapon_changes_with_context(
                &[chunk(data)],
                &registry,
                &anchors,
                &context,
                &[],
            )
            .unwrap();
            let expected: Vec<HeldWeaponChange> =
                serde_json::from_value(row["weapons"].clone()).unwrap();
            assert_eq!(weapons.records, expected, "weapons {case}");
            let attempts: Vec<_> = weapons.component_attempts.iter().map(|a| {
                assert_eq!(a.read.source, source);
                assert_eq!(a.read.slot, 512);
                json!({"id":a.component_index,"status":a.status,"end":a.read.component.end_bit,"in_bounds":a.in_bounds,"observations":a.read.component.diagnostics.component_observations})
            }).collect();
            assert_eq!(
                json!(attempts),
                row["weapon_attempts"],
                "all weapon attempts {case}"
            );
            let expected_rejections: Vec<_> = row["weapon_attempts"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|a| a["status"] == false || a["in_bounds"] == false)
                .cloned()
                .collect();
            let actual_rejections: Vec<_> = weapons.rejected_components.iter().map(|a| {
                assert_eq!(a.read.source, source);
                assert_eq!(a.read.slot, 512);
                json!({"id":a.component_index,"status":a.status,"end":a.read.component.end_bit,"in_bounds":a.in_bounds,"observations":a.read.component.diagnostics.component_observations})
            }).collect();
            assert_eq!(
                actual_rejections, expected_rejections,
                "weapon rejected {case}"
            );
            let restored: HeldWeaponChangeStream =
                serde_json::from_value(serde_json::to_value(&weapons).unwrap()).unwrap();
            assert_eq!(restored, weapons);

            assert_eq!(
                weapons.stats,
                HeldWeaponChangeStats {
                    records: 1,
                    with_component: 1,
                    emissions: expected.len(),
                    repeats: 0
                },
                "weapon counters {case}"
            );
            let map: FilmMapBounds = serde_json::from_value(row["map"].clone()).unwrap();
            let recovery = &row["recovery"];
            let payload = bytes(&recovery["hex"]);
            let expected = recovery["ok"].as_bool().unwrap().then(|| {
                (
                    recovery["counter"].as_u64().unwrap() as u8,
                    recovery["rank"]
                        .as_i64()
                        .filter(|r| *r >= 0)
                        .map(|r| r as u8),
                )
            });
            let mut attempts = Vec::new();
            let actual = recovery_at_observed(
                &payload,
                recovery["start"].as_u64().unwrap() as usize,
                &registry.archetypes[35],
                &map,
                super::super::biped_channels::BipedComponentReader::Context(&context),
                |id, status, component, in_bounds| {
                    attempts.push(EquipmentRecoveryComponentRead {
                        component_index: id,
                        status,
                        in_bounds,
                        component: component.clone(),
                    })
                },
            );
            let actual_attempts: Vec<_> = attempts.iter().map(|a| json!({
                "id": a.component_index, "start": a.component.start_bit, "end": a.component.end_bit,
                "status": a.status, "in_bounds": a.in_bounds,
                "observations": a.component.diagnostics.component_observations,
            })).collect();
            assert_eq!(
                json!(actual_attempts),
                recovery["attempts"],
                "recovery attempts {case}"
            );
            let observations: Vec<_> = attempts
                .iter()
                .flat_map(|a| a.component.diagnostics.component_observations.iter())
                .collect();
            assert_eq!(
                json!(observations),
                recovery["observations"],
                "recovery callbacks {case}"
            );
            assert_eq!(actual, expected, "recovery {case}");
            let mut packet = vec![0, 0, 0, 0];
            packet.extend_from_slice(&(payload.len() as u32).to_le_bytes());
            packet.extend_from_slice(&2000u64.to_le_bytes());
            packet.extend(payload);
            let channels = BipedChannels {
                abilities: [(1000, 1), (3000, 3)]
                    .into_iter()
                    .map(|(timestamp, counter)| BipedChannelRead {
                        packet_index: None,
                        source: FilmPacket {
                            timestamp_us: timestamp,
                            ..source
                        },
                        slot: 512,
                        record_start_bit: 0,
                        component: DecodedComponent {
                            name: "biped-desired-ability-set-component".into(),
                            start_bit: 0,
                            end_bit: 10,
                            references: vec![],
                            diagnostics: Default::default(),
                            fields: vec![
                                ComponentField {
                                    name: "counter".into(),
                                    bit: 0,
                                    width: 3,
                                    raw: counter,
                                },
                                ComponentField {
                                    name: "rank".into(),
                                    bit: 4,
                                    width: 6,
                                    raw: 7,
                                },
                            ],
                        },
                    })
                    .collect(),
                ..Default::default()
            };
            let recovered = scan_equipment_changes_with_context(
                &[chunk(packet)],
                &registry,
                &anchors,
                &channels,
                &map,
                &context,
            )
            .unwrap();
            assert_eq!(recovered.recovery_windows.len(), 1);
            let probe = recovered
                .recovery_attempts
                .iter()
                .find(|r| r.bit_offset == recovery["start"].as_u64().unwrap() as usize);
            assert_eq!(
                probe.is_some(),
                !attempts.is_empty(),
                "retained probe {case}"
            );
            if let Some(probe) = probe {
                assert_eq!(probe.components, attempts, "retained components {case}");
                assert_eq!(probe.candidate, expected.is_some());
                assert_eq!(probe.slot, 512);
                assert_eq!(probe.packet_index, Some(0));
                assert_eq!(probe.source.payload_offset, 16);
                assert_eq!(probe.source.timestamp_us, 2000);
            }
            let restored: EquipmentChangeStream =
                serde_json::from_value(serde_json::to_value(&recovered).unwrap()).unwrap();
            assert_eq!(restored, recovered);

            let candidates = &recovered.recovery_windows[0].candidates;
            assert_eq!(
                candidates.len(),
                usize::from(expected.is_some()),
                "recovery wrapper {case}"
            );
            if let Some((counter, rank)) = expected {
                assert_eq!(
                    (candidates[0].ability.counter, candidates[0].ability.rank),
                    (counter, rank)
                );
            }
        }
    }

    fn oracle() -> Value {
        let mut text = String::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/biped-scan-levelup-v41.json.zlib")[..],
        )
        .read_to_string(&mut text)
        .unwrap();
        serde_json::from_str(&text).unwrap()
    }
    fn emission(v: &Value, recovered: bool) -> EquipmentEmission {
        let e = &v["emission"];
        EquipmentEmission {
            chunk_number: None,
            ability: BipedAbilityEmission {
                packet_index: None,
                source: FilmPacket {
                    chunk_index: e["Chunk"].as_i64().unwrap() as i32,
                    packet_type: 0,
                    byte_2: 0,
                    byte_3: 0,
                    payload_offset: e["PacketIndex"].as_u64().unwrap() as usize,
                    payload_size: 0,
                    timestamp_us: e["TimestampUS"].as_u64().unwrap(),
                },
                slot: e["Slot"].as_u64().unwrap() as u32,
                counter: e["Counter"].as_u64().unwrap() as u8,
                rank: e["Rank"].as_u64().map(|v| v as u8),
            },
            recovery: recovered.then(|| EquipmentRecoveryOrigin {
                bit_offset: v["off"].as_u64().unwrap() as usize,
                head: v["head"].as_bool().unwrap(),
            }),
        }
    }
    #[test]
    fn windows_and_counter_acceptance_match_go() {
        let oracle = oracle();
        for case in oracle["equipment_cases"].as_array().unwrap() {
            let births = serde_json::from_value(case["births"].clone()).unwrap();
            let strict: Vec<_> = case["input"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|v| v["recovered"] == false)
                .map(|v| emission(v, false))
                .collect();
            let mut windows = equipment_recovery_windows(&strict, &births);
            let mut expected = case["windows"].as_array().unwrap().clone();
            // Native window sorting also leaves equal (time, slot) keys unordered.
            windows.sort_by_key(|w| {
                (
                    w.min_time_us,
                    w.slot,
                    w.max_time_us,
                    w.from_counter,
                    w.to_counter,
                )
            });
            expected.sort_by_key(|w| {
                (
                    w["min_time"].as_u64(),
                    w["slot"].as_u64(),
                    w["max_time"].as_u64(),
                    w["from"].as_u64(),
                    w["to"].as_u64(),
                )
            });
            assert_eq!(windows.len(), expected.len());
            for (mut w, e) in windows.into_iter().zip(&expected) {
                assert_eq!(
                    json!({"slot":w.slot,"from":w.from_counter,"to":w.to_counter,"missing":w.missing,"min_time":w.min_time_us,"max_time":w.max_time_us,"min_chunk":w.min_chunk,"max_chunk":w.max_chunk,"head":w.head}),
                    {
                        let mut e = e.clone();
                        e.as_object_mut().unwrap().remove("candidates");
                        e.as_object_mut().unwrap().remove("accepted");
                        e
                    }
                );
                w.candidates = e["candidates"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| emission(v, true))
                    .collect();
                let expected: Vec<_> = e["accepted"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| emission(v, true))
                    .collect();
                assert_eq!(accept_equipment_recovery(&w), expected, "{e}");
            }
        }
    }
    #[test]
    fn sparse_dense_and_truncated_recovery_walks_match_go() {
        let oracle = oracle();
        let arch = FilmArchetype {
            index: 35,
            components: serde_json::from_value(oracle["channel_names"].clone()).unwrap(),
            levels: vec![0; 64],
        };
        let rows = oracle["recovery_reads"].as_array().unwrap();
        assert_eq!(rows.len(), 2048);
        let mut accepted = 0;
        for row in rows {
            let map: FilmMapBounds = serde_json::from_value(row["map"].clone()).unwrap();
            let hex = row["hex"].as_str().unwrap();
            let bytes: Vec<_> = (0..hex.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                .collect();
            let at = row["at"].as_u64().unwrap() as usize;
            let actual = recovery_at(&bytes, at, &arch, &map, &map.position_encoding());
            let expected = row["ok"].as_bool().unwrap().then(|| {
                (
                    row["counter"].as_u64().unwrap() as u8,
                    row["rank"].as_u64().map(|v| v as u8),
                )
            });
            assert_eq!(actual, expected, "{row}");
            accepted += usize::from(actual.is_some());
        }
        assert!(accepted > 500);
    }
    #[test]
    fn recovery_inherits_calibrated_component_profile() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/equipment-profile-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<Value> = serde_json::from_slice(&raw).unwrap();
        let mut names = vec![String::new(); 64];
        names[4] = "object-position-dynamic-precision-component".into();
        names[48] = "biped-desired-ability-set-component".into();
        let arch = FilmArchetype {
            index: 35,
            components: names,
            levels: vec![0; 64],
        };
        let mut differs = 0;
        for (i, row) in rows.iter().enumerate() {
            let map: FilmMapBounds = serde_json::from_value(row["map"].clone()).unwrap();
            let mut position = map.position_encoding();
            position.delta_axis_bits = [row["width"].as_u64().unwrap() as usize; 3];
            position.full_precision = row["full"].as_bool().unwrap();
            let hex = row["hex"].as_str().unwrap();
            let bytes: Vec<_> = (0..hex.len())
                .step_by(2)
                .map(|p| u8::from_str_radix(&hex[p..p + 2], 16).unwrap())
                .collect();
            let at = row["at"].as_u64().unwrap() as usize;
            let actual = recovery_at(&bytes, at, &arch, &map, &position);
            let expected = row["ok"].as_bool().unwrap().then(|| {
                (
                    row["counter"].as_u64().unwrap() as u8,
                    row["rank"].as_u64().map(|r| r as u8),
                )
            });
            assert_eq!(actual, expected, "profile {i}");
            differs += usize::from(
                actual != recovery_at(&bytes, at, &arch, &map, &map.position_encoding()),
            );
        }
        assert!(
            differs > 100,
            "nondefault profiles must exercise behavior missed by map defaults"
        );
    }
}
