//! Registry-driven biped channel reads from the shared guarded position anchors.
use super::{
    BipedPositionStream, DecodeError, DecodedComponent, FilmPacket, FilmRegistry, PositionEncoding,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BipedChannelStats {
    pub records: usize,
    pub with_component: usize,
    pub read: usize,
    pub unread: usize,
    /// Successful component reads that did not transmit the requested value.
    pub gated: usize,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BipedChannelRead {
    /// Native packet ordinal; unknown for unframed caller-supplied input.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub packet_index: Option<usize>,
    pub source: FilmPacket,
    pub slot: u32,
    pub record_start_bit: usize,
    pub component: DecodedComponent,
}
impl BipedChannelRead {
    pub fn value(&self, name: &str) -> Option<u64> {
        self.component
            .fields
            .iter()
            .find(|f| f.name == name)
            .map(|f| f.raw)
    }
}
/// An attempted component read. Native dispatch status and source
/// bounds are separate: a padded read can succeed beyond the packet payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BipedComponentAttempt {
    pub component_index: u8,
    pub status: Option<bool>,
    pub in_bounds: bool,
    pub read: BipedChannelRead,
}
/// Compatibility name for the rejected subset of component attempts.
pub type BipedChannelRejection = BipedComponentAttempt;
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BipedChannels {
    /// Failed intermediate/target attempts, including their raw fields and callbacks.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rejected_components: Vec<BipedChannelRejection>,
    /// Every attempted component in scan order, including intermediate fields and callbacks.
    /// Absence in old exports means this trace was unavailable.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub component_attempts: Vec<super::BipedComponentAttempt>,
    /// Independent native camo-scan failure; ability observations remain available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub camo_error: Option<String>,
    pub camo_stats: BipedChannelStats,
    pub ability_stats: BipedChannelStats,
    /// Includes successfully read components whose state channel was absent.
    pub camouflage: Vec<BipedChannelRead>,
    /// Includes the no-equipment gate, needed by the equipment-change channel.
    pub abilities: Vec<BipedChannelRead>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BipedCamoState {
    /// Native packet ordinal; unknown for unframed caller-supplied input.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub packet_index: Option<usize>,
    pub source: FilmPacket,
    pub slot: u32,
    /// Measured switch values are 0 and 4095; other values remain uninterpreted.
    pub quantum: u16,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BipedAbilityEmission {
    /// Native packet ordinal; unknown for unframed caller-supplied input.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub packet_index: Option<usize>,
    pub source: FilmPacket,
    pub slot: u32,
    pub counter: u8,
    /// None is an explicitly transmitted no-ability gate, not a failed read.
    pub rank: Option<u8>,
}
impl BipedChannels {
    pub fn camo_states(&self) -> impl Iterator<Item = BipedCamoState> + '_ {
        self.camouflage.iter().filter_map(|r| {
            Some(BipedCamoState {
                packet_index: r.packet_index,
                source: r.source,
                slot: r.slot,
                quantum: r.value("sub[1]")? as u16,
            })
        })
    }
    pub fn ability_emissions(&self) -> impl Iterator<Item = BipedAbilityEmission> + '_ {
        self.abilities.iter().filter_map(|r| {
            Some(BipedAbilityEmission {
                packet_index: r.packet_index,
                source: r.source,
                slot: r.slot,
                counter: r.value("counter")? as u8,
                rank: r.value("rank").map(|v| v as u8),
            })
        })
    }
    pub fn ability_ranks(&self) -> impl Iterator<Item = BipedAbilityEmission> + '_ {
        self.ability_emissions().filter(|r| r.rank.is_some())
    }
}
/// Traverse declared components once for the selected channels. Unlike position
/// tracks, gameplay channel scans include anchors rejected by movement filters.
/// Missing camo is reported in `BipedChannels::camo_error` without suppressing
/// ability data. Shared setup failures still return `Err`.
pub fn scan_biped_channels(
    chunks: &[crate::clients::hi::models::FilmChunkData],
    registry: &FilmRegistry,
    anchors: &BipedPositionStream,
    encoding: &PositionEncoding,
) -> Result<BipedChannels, DecodeError> {
    scan_biped_channels_impl(
        chunks,
        registry,
        anchors,
        super::biped_channels::BipedComponentReader::Legacy(encoding),
    )
}

/// Native direct biped walk under a complete inherited reader context.
/// Traversal width overrides and corruption guards do not apply here.
/// The camo channel can fail independently; inspect `BipedChannels::camo_error`.
pub fn scan_biped_channels_with_context(
    chunks: &[crate::clients::hi::models::FilmChunkData],
    registry: &FilmRegistry,
    anchors: &BipedPositionStream,
    context: &super::FrameEncoding,
) -> Result<BipedChannels, DecodeError> {
    scan_biped_channels_impl(
        chunks,
        registry,
        anchors,
        super::biped_channels::BipedComponentReader::Context(context),
    )
}

fn scan_biped_channels_impl(
    chunks: &[crate::clients::hi::models::FilmChunkData],
    registry: &FilmRegistry,
    anchors: &BipedPositionStream,
    reader: super::biped_channels::BipedComponentReader<'_>,
) -> Result<BipedChannels, DecodeError> {
    let arch = registry
        .archetype(35)
        .ok_or(DecodeError::Missing("biped archetype"))?;
    let camo = arch
        .components
        .iter()
        .position(|n| n == "unit-active-camo-state-component")
        .map(|index| index as u8);
    // The pinned ability scanner deliberately targets index 48; its hook still
    // requires that index to name the ability component.
    let ability = 48u8;
    let bytes = super::fire_events::native_chunk_data(chunks);
    let packet_indices = super::fire_events::native_packet_indices(chunks);
    let mut out = BipedChannels {
        camo_error: camo
            .is_none()
            .then(|| DecodeError::Missing("biped camouflage component").to_string()),
        ..Default::default()
    };
    for candidate in &anchors.candidates {
        let source = candidate.source;
        // Native walkDeltaBipedRecords skips absent chunks before emitting or
        // counting any records. Supplied anchors may outlive their chunk data.
        let Some(chunk) = bytes.get(&source.chunk_index) else {
            continue;
        };
        let r = &candidate.record;
        out.camo_stats.records += usize::from(camo.is_some());
        out.ability_stats.records += 1;
        let has_camo = camo.is_some_and(|id| r.component_indices.contains(&id));
        let has_ability = r.component_indices.contains(&ability);
        out.camo_stats.with_component += usize::from(has_camo);
        out.ability_stats.with_component += usize::from(has_ability);
        if !has_camo && !has_ability {
            continue;
        }
        let data = source
            .payload_offset
            .checked_add(source.payload_size)
            .and_then(|end| chunk.get(source.payload_offset..end))
            .ok_or(DecodeError::Truncated {
                chunk: source.chunk_index,
                offset: source.payload_offset,
            })?;
        let mut camo_read = false;
        let mut ability_read = false;
        let last = if has_ability {
            ability.max(if has_camo { camo.unwrap() } else { 0 })
        } else {
            camo.unwrap()
        };
        let walked = walk_biped_components_retaining_rejections(
            data,
            r,
            arch,
            reader,
            last,
            (
                source,
                packet_indices
                    .get(&(source.chunk_index, source.payload_offset))
                    .copied(),
            ),
            (&mut out.rejected_components, &mut out.component_attempts),
        );
        for (id, component) in walked {
            let name = &component.name;
            if Some(id) == camo {
                let read = BipedChannelRead {
                    packet_index: packet_indices
                        .get(&(source.chunk_index, source.payload_offset))
                        .copied(),
                    source,
                    slot: r.slot,
                    record_start_bit: r.start_bit,
                    component: component.clone(),
                };
                out.camo_stats.gated += usize::from(read.value("sub[1]").is_none());
                out.camouflage.push(read);
                camo_read = true;
            }
            if id == ability
                && matches!(
                    name.as_str(),
                    "biped-desired-ability-set" | "biped-desired-ability-set-component"
                )
            {
                let read = BipedChannelRead {
                    packet_index: packet_indices
                        .get(&(source.chunk_index, source.payload_offset))
                        .copied(),
                    source,
                    slot: r.slot,
                    record_start_bit: r.start_bit,
                    component,
                };
                out.ability_stats.gated += usize::from(read.value("rank").is_none());
                out.abilities.push(read);
                ability_read = true;
            }
            if (!has_camo || camo_read) && (!has_ability || ability_read) {
                break;
            }
        }
        out.camo_stats.read += usize::from(camo_read);
        out.camo_stats.unread += usize::from(has_camo && !camo_read);
        out.ability_stats.read += usize::from(ability_read);
        out.ability_stats.unread += usize::from(has_ability && !ability_read);
    }
    Ok(out)
}

/// Reader choice shared by biped scanners. Context dispatch uses the native padded
/// reader; publication still rejects any attempt ending beyond its source payload.
#[derive(Clone, Copy)]
pub(super) enum BipedComponentReader<'a> {
    Legacy(&'a PositionEncoding),
    Context(&'a super::FrameEncoding),
}
impl BipedComponentReader<'_> {
    pub(super) fn read_raw(
        self,
        data: &[u8],
        at: usize,
        name: &str,
        level: u32,
    ) -> (Option<bool>, DecodedComponent) {
        match self {
            Self::Legacy(position) => super::components::decode_component_attempt(
                data,
                at,
                name,
                level,
                35,
                Some(position),
            ),
            Self::Context(context) => {
                super::consume_component_at(data, at, name, 35, level, context)
            }
        }
    }
}

/// Keep failed target/intermediate reads without changing successful visits.
pub(super) fn walk_biped_components_retaining_rejections(
    data: &[u8],
    record: &super::BipedPositionRecord,
    arch: &super::FilmArchetype,
    reader: BipedComponentReader<'_>,
    last: u8,
    (source, packet_index): (FilmPacket, Option<usize>),
    (rejected, attempts): (
        &mut Vec<BipedChannelRejection>,
        &mut Vec<BipedComponentAttempt>,
    ),
) -> Vec<(u8, DecodedComponent)> {
    walk_biped_components_at_observed(
        data,
        record.end_bit.saturating_add(2),
        &record.component_indices[1..],
        arch,
        reader,
        last,
        |id, status, component, in_bounds| {
            let attempt = BipedComponentAttempt {
                component_index: id,
                status,
                in_bounds,
                read: BipedChannelRead {
                    packet_index,
                    source,
                    slot: record.slot,
                    record_start_bit: record.start_bit,
                    component: component.clone(),
                },
            };
            if status != Some(true) || !in_bounds {
                rejected.push(attempt.clone());
            }
            attempts.push(attempt);
        },
    )
}

pub(super) fn walk_biped_components_at_observed(
    data: &[u8],
    mut at: usize,
    indices: &[u8],
    arch: &super::FilmArchetype,
    reader: BipedComponentReader<'_>,
    last: u8,
    mut observe: impl FnMut(u8, Option<bool>, &DecodedComponent, bool),
) -> Vec<(u8, DecodedComponent)> {
    let mut out = Vec::new();
    for &id in indices {
        let Some(name) = arch
            .components
            .get(usize::from(id))
            .filter(|n| !n.is_empty())
        else {
            break;
        };
        let level = arch.levels.get(usize::from(id)).copied().unwrap_or(0);
        let (status, component) = reader.read_raw(data, at, name, level);
        let in_bounds =
            component.end_bit >= 0 && component.end_bit <= data.len().saturating_mul(8) as i64;
        observe(id, status, &component, in_bounds);
        if status != Some(true) || !in_bounds {
            break;
        }
        at = usize::try_from(component.end_bit).expect("in-bounds component endpoint");
        out.push((id, component));
        if id >= last {
            break;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clients::hi::models::{FilmChunk, FilmChunkData};
    use crate::theater::{
        BipedPositionCandidate, BipedPositionRecord, BipedPositionRejection, FilmArchetype,
        FilmMapBounds,
    };
    use std::io::Read;
    #[test]
    fn native_independent_camo_and_ability_channels() {
        compare_native_channels(
            include_bytes!("fixtures/channel-independence-v41.json.zlib"),
            1024,
            192,
        );
    }
    #[test]
    fn native_biped_source_availability() {
        compare_native_channels(
            include_bytes!("fixtures/biped-source-availability-v41.json.zlib"),
            512,
            0,
        );
    }
    #[test]
    fn native_biped_duplicate_sources() {
        compare_native_channels(
            include_bytes!("fixtures/biped-duplicate-sources-v41.json.zlib"),
            512,
            0,
        );
    }
    #[test]
    fn native_inventory_duplicate_sources() {
        compare_native_channels(
            include_bytes!("fixtures/inventory-duplicate-sources-v41.json.zlib"),
            512,
            0,
        );
    }
    #[test]
    fn native_inventory_source_availability() {
        compare_native_channels(
            include_bytes!("fixtures/inventory-source-availability-v41.json.zlib"),
            512,
            0,
        );
    }
    fn compare_native_channels(fixture: &[u8], count: usize, retained: usize) {
        use crate::theater::*;
        use serde_json::json;
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(fixture)
            .read_to_end(&mut raw)
            .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(rows.len(), count);
        let mut retained_without_camo = 0;
        for (case, row) in rows.iter().enumerate() {
            let hex = row["hex"].as_str().unwrap();
            let data: Vec<u8> = (0..hex.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                .collect();
            let source = FilmPacket {
                chunk_index: 1,
                packet_type: 0,
                byte_2: 0,
                byte_3: 0,
                payload_offset: 16,
                payload_size: data.len() - 16,
                timestamp_us: 1000 + case as u64,
            };
            let mut chunks = vec![FilmChunkData {
                metadata: FilmChunk {
                    index: 1,
                    chunk_type: 2,
                    start_time_offset_ms: 0,
                    duration_ms: 1,
                    size: data.len() as i64,
                    file_relative_path: String::new(),
                },
                data,
            }];
            if let Some(hex) = row["duplicate"].as_str() {
                let mut duplicate = chunks[0].clone();
                duplicate.data = (0..hex.len())
                    .step_by(2)
                    .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                    .collect();
                duplicate.metadata.size = duplicate.data.len() as i64;
                chunks.push(duplicate);
            }
            let mut archetypes: Vec<_> = (0..36)
                .map(|index| FilmArchetype {
                    index,
                    components: vec![],
                    levels: vec![],
                })
                .collect();
            archetypes[35].components = serde_json::from_value(row["names"].clone()).unwrap();
            archetypes[35].levels = vec![0; archetypes[35].components.len()];
            let registry = FilmRegistry {
                archetypes,
                major_version: 41,
                format_version: 27,
                end_byte: 0,
                truncated: false,
            };
            let mut anchors = BipedPositionStream {
                record_masks: vec![],
                slot_band: Some([512, 512]),
                options: Default::default(),
                candidates: row["anchors"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|a| BipedPositionCandidate {
                        packet_index: None,
                        source,
                        rejection: None,
                        record: BipedPositionRecord {
                            start_bit: case % 8,
                            position_bit: a["i0"].as_u64().unwrap() as usize,
                            end_bit: a["i0"].as_u64().unwrap() as usize + 45,
                            slot: a["slot"].as_u64().unwrap() as u32,
                            generation: 1,
                            component_indices: serde_json::from_value(a["mask"].clone()).unwrap(),
                            quantized: [1; 3],
                            world: [0.; 3],
                            companions: Default::default(),
                        },
                    })
                    .collect(),
            };
            let base = std::mem::take(&mut anchors.candidates);
            anchors.candidates = row["requested"]
                .as_array()
                .unwrap()
                .iter()
                .flat_map(|id| {
                    base.iter().cloned().map(move |mut candidate| {
                        candidate.source.chunk_index = id.as_u64().unwrap() as i32;
                        candidate
                    })
                })
                .collect();
            let context = serde_json::from_value(json!({"ids":{"low_bits":13,"base":0},"mpp_widths":[9,5],"position":row.get("encoding").cloned().unwrap_or(json!(null)),"extra_fields":false,"corruption_check":false})).unwrap();
            let channels =
                scan_biped_channels_with_context(&chunks, &registry, &anchors, &context).unwrap();
            if case == 0 && count == 1024 {
                // These forged anchors cannot be produced by native packet
                // discovery; reject them explicitly instead of indexing/panicking.
                for (offset, size) in [
                    (usize::MAX, 1),
                    (16, usize::MAX),
                    (0, chunks[0].data.len() + 1),
                ] {
                    let mut malformed = anchors.clone();
                    malformed.candidates[0].source.payload_offset = offset;
                    malformed.candidates[0].source.payload_size = size;
                    assert!(matches!(
                        scan_biped_channels_with_context(&chunks, &registry, &malformed, &context),
                        Err(DecodeError::Truncated { chunk: 1, .. })
                    ));
                }
            }
            assert_eq!(
                channels.camo_error.is_some(),
                row["camoError"].as_bool().unwrap(),
                "error {case}"
            );
            for (stats, expected, with, gated) in [
                (
                    &channels.ability_stats,
                    &row["abilityStats"],
                    "WithI48",
                    "Gated",
                ),
                (
                    &channels.camo_stats,
                    &row["camoStats"],
                    "WithI28",
                    "NoChannel",
                ),
            ] {
                assert_eq!(
                    json!({"Records":stats.records,with:stats.with_component,"Read":stats.read,"Unread":stats.unread,gated:stats.gated}),
                    *expected,
                    "stats {case}"
                );
            }
            let project = |e: BipedAbilityEmission| json!({"Slot":e.slot,"Chunk":e.source.chunk_index,"PacketIndex":e.packet_index,"TimestampUS":e.source.timestamp_us,"Counter":e.counter,"Rank":e.rank.map(i32::from).unwrap_or(-1)});
            assert_eq!(
                json!(
                    channels
                        .ability_emissions()
                        .map(project)
                        .collect::<Vec<_>>()
                ),
                row["emissions"],
                "emissions {case}"
            );
            assert_eq!(
                json!(channels.ability_ranks().map(project).collect::<Vec<_>>()),
                if row["ranks"].is_null() {
                    json!([])
                } else {
                    row["ranks"].clone()
                },
                "ranks {case}"
            );
            let camo: Vec<_> = channels.camo_states().map(|c| json!({"Slot":c.slot,"Chunk":c.source.chunk_index,"PacketIndex":c.packet_index,"TimestampUS":c.source.timestamp_us,"Q":c.quantum})).collect();
            assert_eq!(
                json!(camo),
                if row["camo"].is_null() {
                    json!([])
                } else {
                    row["camo"].clone()
                },
                "camo {case}"
            );
            if channels.camo_error.is_some() {
                retained_without_camo += channels.abilities.len();
            }
            if row.get("chargeStats").is_some() {
                let normalize = |value: &serde_json::Value| {
                    if value.is_null() {
                        json!([])
                    } else {
                        value.clone()
                    }
                };
                let states =
                    scan_biped_ability_states_with_context(&chunks, &registry, &anchors, &context)
                        .unwrap();
                let charges =
                    scan_ability_charges_with_context(&chunks, &registry, &anchors, &context)
                        .unwrap();
                let equipment =
                    scan_unit_equipment_with_context(&chunks, &registry, &anchors, &context)
                        .unwrap();
                assert_eq!(
                    json!(charges.stats),
                    row["chargeStats"],
                    "charge stats {case}"
                );
                assert_eq!(
                    json!(states.impulse_stats),
                    row["impulseStats"],
                    "impulse stats {case}"
                );
                assert_eq!(
                    json!(states.grapple_stats),
                    row["grappleStats"],
                    "grapple stats {case}"
                );
                let ch: Vec<_> = charges.records.iter().map(|r| json!({"Slot":r.slot,"Chunk":r.source.chunk_index,"PacketIndex":r.packet_index,"TimestampUS":r.source.timestamp_us,"Emplacement":r.emplacement,"Charges":r.charges,"Low":r.low})).collect();
                assert_eq!(json!(ch), normalize(&row["charges"]), "charges {case}");
                let im: Vec<_> = states.impulses.iter().map(|r| json!({"Slot":r.slot,"Chunk":r.source.chunk_index,"PacketIndex":r.packet_index,"TimestampUS":r.source.timestamp_us,"Predicted":r.predicted})).collect();
                assert_eq!(json!(im), normalize(&row["impulses"]), "impulses {case}");
                let gr: Vec<_> = states.grapple.iter().map(|r| json!({"Slot":r.slot,"Chunk":r.source.chunk_index,"PacketIndex":r.packet_index,"TimestampUS":r.source.timestamp_us,"Heavy":r.heavy,"PosQ":r.position_quantized})).collect();
                assert_eq!(json!(gr), normalize(&row["grapple"]), "grapple {case}");
                let eq: Vec<_> = equipment.records.iter().map(|r| json!({"Slot":r.slot,"TimestampUS":r.source.timestamp_us,"Read":{"Head":r.read.head,"Entries":r.read.entries.iter().map(|e|json!({"Val":e.value,"Tail":e.tail,"Present":e.present})).collect::<Vec<_>>()}})).collect();
                let mut expected = normalize(&row["equipment"]);
                for r in expected.as_array_mut().unwrap() {
                    if r["Read"]["Entries"].is_null() {
                        r["Read"]["Entries"] = json!([]);
                    }
                }
                assert_eq!(json!(eq), expected, "equipment {case}");
            }
            if row.get("inventoryStats").is_some() {
                let inventory =
                    scan_inventory_deltas_with_context(&chunks, &registry, &anchors, &context)
                        .unwrap();
                let held = scan_held_weapon_changes_with_context(
                    &chunks,
                    &registry,
                    &anchors,
                    &context,
                    &[],
                )
                .unwrap();
                assert_eq!(
                    json!(inventory.stats),
                    row["inventoryStats"],
                    "inventory stats {case}"
                );
                assert_eq!(json!(held.stats), row["heldStats"], "held stats {case}");
                assert_eq!(
                    json!(held.records),
                    if row["held"].is_null() {
                        json!([])
                    } else {
                        row["held"].clone()
                    },
                    "held {case}"
                );
                let records: Vec<_> = inventory.records.iter().map(|r| {
                    let ammo: Vec<_> = r.ammo.iter().map(|a| json!({"WeaponSlot":a.weapon_slot,"Mag":a.magazine,"FracQ":a.fraction_quantum,"Res":a.reserve})).collect();
                    json!({"Slot":r.slot,"Chunk":r.source.chunk_index,"PacketIndex":r.packet_index,"TimestampUS":r.source.timestamp_us,"Grenades":r.grenades,"SelRead":r.selection.is_some(),"Sel":r.selection.as_ref().map_or(0,|s|s.rank.map_or(-1,i32::from)),"Mask":r.selection.as_ref().map_or(0,|s|s.mask),"Ammo":if ammo.is_empty(){serde_json::Value::Null}else{json!(ammo)}})
                }).collect();
                assert_eq!(
                    json!(records),
                    if row["inventory"].is_null() {
                        json!([])
                    } else {
                        row["inventory"].clone()
                    },
                    "inventory {case}"
                );
                if case == 0 || case == 2 {
                    for (offset, size) in [
                        (usize::MAX, 1),
                        (16, usize::MAX),
                        (0, chunks[0].data.len() + 1),
                    ] {
                        let mut invalid = anchors.clone();
                        invalid.candidates[0].source.payload_offset = offset;
                        invalid.candidates[0].source.payload_size = size;
                        let error = if case == 0 {
                            scan_inventory_deltas_with_context(
                                &chunks, &registry, &invalid, &context,
                            )
                            .unwrap_err()
                        } else {
                            scan_held_weapon_changes_with_context(
                                &chunks,
                                &registry,
                                &invalid,
                                &context,
                                &[],
                            )
                            .unwrap_err()
                        };
                        assert!(matches!(error, DecodeError::Truncated { chunk: 1, .. }));
                    }
                }
            }
            let restored: BipedChannels =
                serde_json::from_value(serde_json::to_value(&channels).unwrap()).unwrap();
            assert_eq!(restored, channels);
            let mut legacy = serde_json::to_value(&channels).unwrap();
            legacy.as_object_mut().unwrap().remove("component_attempts");
            let legacy: BipedChannels = serde_json::from_value(legacy).unwrap();
            assert!(legacy.component_attempts.is_empty());
            assert_eq!(legacy.rejected_components, channels.rejected_components);
        }
        assert_eq!(retained_without_camo, retained);
    }
    #[test]
    fn native_biped_context_walk_and_scanner_gates() {
        use crate::theater::*;
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/biped-context-v41.json.zlib")[..])
            .read_to_end(&mut raw)
            .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(rows.len(), 512);
        let mut rejected_callbacks = 0;
        let mut rejected_count = 0;
        for (case, row) in rows.iter().enumerate() {
            let hex = row["hex"].as_str().unwrap();
            let data: Vec<u8> = (0..hex.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                .collect();
            let start = row["start"].as_u64().unwrap() as usize;
            let ids: Vec<u8> = serde_json::from_value(row["ids"].clone()).unwrap();
            let arch = FilmArchetype {
                index: 35,
                components: serde_json::from_value(row["names"].clone()).unwrap(),
                levels: vec![row["level"].as_u64().unwrap() as u32; 64],
            };
            let context: FrameEncoding = serde_json::from_value(serde_json::json!({
                "ids":{"low_bits":13,"base":0}, "mpp_widths":[9,5],
                "position":row["encoding"], "extra_fields":false,"corruption_check":false,
                "keyframe_simulation_complete":row["simulation"],
            }))
            .unwrap();
            let mut attempted = Vec::new();
            let mut walk_observations = Vec::new();
            let decoded = walk_biped_components_at_observed(
                &data,
                start,
                &ids,
                &arch,
                BipedComponentReader::Context(&context),
                59,
                |id, status, component, in_bounds| {
                    attempted.push(serde_json::json!({"id":id,"status":status,"end":component.end_bit,"in_bounds":in_bounds,"observations":component.diagnostics.component_observations}));
                    walk_observations.extend(component.diagnostics.component_observations.clone());
                },
            );
            assert_eq!(
                serde_json::json!(attempted),
                row["attempts"],
                "attempts {case}"
            );
            assert_eq!(
                serde_json::json!(walk_observations),
                row["walk_observations"],
                "walk callbacks {case}"
            );
            let expected_ids: Vec<u8> = serde_json::from_value(row["visited"].clone()).unwrap();
            assert_eq!(
                decoded.iter().map(|x| x.0).collect::<Vec<_>>(),
                expected_ids,
                "walk {case}"
            );
            assert_eq!(decoded.len(), row["steps"].as_array().unwrap().len());
            for ((id, component), expected) in decoded.iter().zip(row["steps"].as_array().unwrap())
            {
                assert_eq!(u64::from(*id), expected["id"].as_u64().unwrap());
                assert_eq!(
                    component.end_bit as u64,
                    expected["end"].as_u64().unwrap(),
                    "end {case}/{id}"
                );
                let observations: Vec<FilmComponentObservation> =
                    serde_json::from_value(expected["observations"].clone()).unwrap();
                assert_eq!(
                    component.diagnostics.component_observations, observations,
                    "hooks {case}/{id}"
                );
            }
            let source = FilmPacket {
                chunk_index: 1,
                packet_type: 0,
                byte_2: 0,
                byte_3: 0,
                payload_offset: 0,
                payload_size: data.len(),
                timestamp_us: 1000,
            };
            let chunks = vec![FilmChunkData {
                metadata: FilmChunk {
                    index: 1,
                    chunk_type: 2,
                    start_time_offset_ms: 0,
                    duration_ms: 1,
                    size: data.len() as i64,
                    file_relative_path: String::new(),
                },
                data,
            }];
            let mut archetypes: Vec<_> = (0..36)
                .map(|index| FilmArchetype {
                    index,
                    components: vec![],
                    levels: vec![],
                })
                .collect();
            archetypes[35] = arch;
            let registry = FilmRegistry {
                archetypes,
                major_version: 41,
                format_version: 27,
                end_byte: 0,
                truncated: false,
            };
            let anchors = BipedPositionStream {
                record_masks: vec![],
                slot_band: Some([512, 512]),
                options: Default::default(),
                candidates: vec![BipedPositionCandidate {
                    packet_index: None,
                    source,
                    rejection: Some(BipedPositionRejection::Saturated),
                    record: BipedPositionRecord {
                        start_bit: 0,
                        position_bit: 0,
                        end_bit: start - 2,
                        slot: 512,
                        generation: 1,
                        component_indices: std::iter::once(0).chain(ids.iter().copied()).collect(),
                        quantized: [0; 3],
                        world: [0.0; 3],
                        companions: Default::default(),
                    },
                }],
            };
            let channels =
                scan_biped_channels_with_context(&chunks, &registry, &anchors, &context).unwrap();
            let expected_rejections: Vec<_> = row["attempts"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|a| {
                    a["id"].as_u64().unwrap() <= 48
                        && (a["status"] == false || a["in_bounds"] == false)
                })
                .cloned()
                .collect();
            let actual_rejections: Vec<_> = channels.rejected_components.iter().map(|a| {
                assert_eq!(a.read.source, source);
                assert_eq!(a.read.slot, 512);
                assert_eq!(a.read.record_start_bit, 0);
                rejected_callbacks += a.read.component.diagnostics.component_observations.len();
                serde_json::json!({"id":a.component_index,"status":a.status,"end":a.read.component.end_bit,"in_bounds":a.in_bounds,"observations":a.read.component.diagnostics.component_observations})
            }).collect();
            rejected_count += actual_rejections.len();
            assert_eq!(
                actual_rejections, expected_rejections,
                "retained rejections {case}"
            );
            let restored: BipedChannels =
                serde_json::from_value(serde_json::to_value(&channels).unwrap()).unwrap();
            assert_eq!(restored, channels);
            assert_eq!(
                channels.camo_stats.read,
                usize::from(expected_ids.contains(&28)),
                "camo {case}"
            );
            assert_eq!(
                channels.ability_stats.read,
                usize::from(expected_ids.contains(&48)),
                "ability {case}"
            );
            let charges =
                scan_ability_charges_with_context(&chunks, &registry, &anchors, &context).unwrap();
            assert_eq!(
                charges.stats.read,
                usize::from(expected_ids.contains(&56)),
                "charges {case}"
            );
            let inventory =
                scan_inventory_deltas_with_context(&chunks, &registry, &anchors, &context).unwrap();
            assert_eq!(
                inventory.stats.i22_read,
                usize::from(expected_ids.contains(&22)),
                "inventory {case}"
            );
            let equipment =
                scan_unit_equipment_with_context(&chunks, &registry, &anchors, &context).unwrap();
            assert_eq!(
                equipment.stats.read,
                usize::from(expected_ids.contains(&26)),
                "equipment {case}"
            );
            for (last, rejected) in [
                (22, &inventory.rejected_components),
                (26, &equipment.rejected_components),
                (56, &charges.rejected_components),
            ] {
                let expected: Vec<_> = row["attempts"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|a| {
                        a["id"].as_u64().unwrap() <= last
                            && (a["status"] == false || a["in_bounds"] == false)
                    })
                    .cloned()
                    .collect();
                let actual: Vec<_> = rejected.iter().map(|a| {
                    assert_eq!(a.read.source, source);
                    assert_eq!(a.read.slot, 512);
                    assert_eq!(a.read.record_start_bit, 0);
                    serde_json::json!({"id":a.component_index,"status":a.status,"end":a.read.component.end_bit,"in_bounds":a.in_bounds,"observations":a.read.component.diagnostics.component_observations})
                }).collect();
                assert_eq!(actual, expected, "scanner ending {last}, case {case}");
            }
            let restored: InventoryDeltaStream =
                serde_json::from_value(serde_json::to_value(&inventory).unwrap()).unwrap();
            assert_eq!(restored, inventory);
            let restored: UnitEquipmentStream =
                serde_json::from_value(serde_json::to_value(&equipment).unwrap()).unwrap();
            assert_eq!(restored, equipment);
            let restored: AbilityChargeStream =
                serde_json::from_value(serde_json::to_value(&charges).unwrap()).unwrap();
            assert_eq!(restored, charges);
            let states =
                scan_biped_ability_states_with_context(&chunks, &registry, &anchors, &context)
                    .unwrap();
            for (last, attempts) in [
                (48, &channels.component_attempts),
                (22, &inventory.component_attempts),
                (26, &equipment.component_attempts),
                (56, &charges.component_attempts),
                (u64::MAX, &states.component_attempts),
            ] {
                let expected: Vec<_> = row["attempts"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|a| a["id"].as_u64().unwrap() <= last)
                    .cloned()
                    .collect();
                let actual: Vec<_> = attempts.iter().map(|a| {
                    assert_eq!(a.read.source, source);
                    assert_eq!(a.read.slot, 512);
                    assert_eq!(a.read.record_start_bit, 0);
                    serde_json::json!({"id":a.component_index,"status":a.status,"end":a.read.component.end_bit,"in_bounds":a.in_bounds,"observations":a.read.component.diagnostics.component_observations})
                }).collect();
                assert_eq!(actual, expected, "all attempts through {last}, case {case}");
            }
            let expected: Vec<_> = row["attempts"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|a| a["status"] == false || a["in_bounds"] == false)
                .cloned()
                .collect();
            let actual: Vec<_> = states.rejected_components.iter().map(|a| {
                assert_eq!(a.read.source, source);
                assert_eq!(a.read.slot, 512);
                serde_json::json!({"id":a.component_index,"status":a.status,"end":a.read.component.end_bit,"in_bounds":a.in_bounds,"observations":a.read.component.diagnostics.component_observations})
            }).collect();
            assert_eq!(actual, expected, "ability rejected {case}");
            assert_eq!(
                serde_json::json!(states.grapple_stats),
                row["grapple_stats"],
                "grapple stats {case}"
            );
            let grapple: Vec<_> = states.grapple.iter().map(|r| serde_json::json!({
                "Slot": r.slot, "Chunk": r.source.chunk_index, "PacketIndex": 0,
                "TimestampUS": r.source.timestamp_us, "Heavy": r.heavy, "PosQ": r.position_quantized
            })).collect();
            let expected_grapple = row["grapple"].as_array().cloned().unwrap_or_default();
            assert_eq!(grapple, expected_grapple, "grapple publications {case}");

            let restored: BipedAbilityStates =
                serde_json::from_value(serde_json::to_value(&states).unwrap()).unwrap();
            assert_eq!(restored, states);
            assert_eq!(
                serde_json::json!(states.impulse_stats),
                row["impulse_stats"],
                "impulse stats {case}"
            );
            let impulses: Vec<_> = states
                .impulses
                .iter()
                .map(|r| {
                    serde_json::json!({
                        "Slot": r.slot, "Chunk": r.source.chunk_index, "PacketIndex": 0,
                        "TimestampUS": r.source.timestamp_us, "Predicted": r.predicted
                    })
                })
                .collect();
            assert_eq!(
                impulses,
                row["impulses"].as_array().cloned().unwrap_or_default(),
                "impulse publications {case}"
            );
        }
        assert_eq!(rejected_count, 235);
        assert_eq!(rejected_callbacks, 29);
    }

    #[test]
    fn channel_walk_handles_missing_and_truncated_intermediate_components_like_go() {
        let mut text = String::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/biped-scan-levelup-v41.json.zlib")[..],
        )
        .read_to_string(&mut text)
        .unwrap();
        let oracle: serde_json::Value = serde_json::from_str(&text).unwrap();
        let map: FilmMapBounds = serde_json::from_value(oracle["scans"][0]["map"].clone()).unwrap();
        let mut archetypes: Vec<_> = (0..36)
            .map(|index| FilmArchetype {
                index,
                components: vec![],
                levels: vec![],
            })
            .collect();
        archetypes[35].components =
            serde_json::from_value(oracle["channel_names"].clone()).unwrap();
        archetypes[35].levels = vec![0; 64];
        let registry = FilmRegistry {
            archetypes,
            major_version: 41,
            format_version: 27,
            end_byte: 0,
            truncated: false,
        };
        let rows = oracle["channels"].as_array().unwrap();
        assert_eq!(rows.len(), 1536);
        for row in rows {
            let hex = row["hex"].as_str().unwrap();
            let bytes: Vec<u8> = (0..hex.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                .collect();
            let source = FilmPacket {
                chunk_index: 1,
                packet_type: 0,
                byte_2: 0,
                byte_3: 0,
                payload_offset: 33,
                payload_size: bytes.len(),
                timestamp_us: 0,
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
                data: super::super::fire_events::test_payload_after_keyframe(&bytes, 0),
            };
            let mask: Vec<u8> = serde_json::from_value(row["mask"].clone()).unwrap();
            let anchors = BipedPositionStream {
                record_masks: vec![],
                slot_band: Some([512, 512]),
                options: Default::default(),
                candidates: vec![BipedPositionCandidate {
                    packet_index: None,
                    source,
                    record: BipedPositionRecord {
                        start_bit: 0,
                        position_bit: 0,
                        end_bit: row["at"].as_u64().unwrap() as usize - 2,
                        slot: 512,
                        generation: 1,
                        component_indices: mask.clone(),
                        quantized: [0; 3],
                        world: [0.0; 3],
                        companions: Default::default(),
                    },
                    rejection: Some(BipedPositionRejection::Saturated),
                }],
            };
            let out = scan_biped_channels(&[chunk], &registry, &anchors, &map.position_encoding())
                .unwrap();
            assert!(
                out.camouflage
                    .iter()
                    .chain(&out.abilities)
                    .all(|r| r.packet_index == Some(1))
            );
            assert!(out.camo_states().all(|r| r.packet_index == Some(1)));
            assert!(out.ability_emissions().all(|r| r.packet_index == Some(1)));
            let restored: BipedChannels =
                serde_json::from_value(serde_json::to_value(&out).unwrap()).unwrap();
            assert_eq!(restored, out);
            for (id, key, stats) in [
                (28, "camo_read", &out.camo_stats),
                (48, "ability_read", &out.ability_stats),
            ] {
                let read = row[key].as_bool().unwrap();
                assert_eq!(stats.records, 1);
                assert_eq!(stats.with_component, usize::from(mask.contains(&id)));
                assert_eq!(stats.read, usize::from(read));
                assert_eq!(stats.unread, usize::from(mask.contains(&id) && !read));
            }
            if let Some(r) = out.camouflage.first() {
                let expected = row["camo_present"]
                    .as_bool()
                    .unwrap()
                    .then(|| row["camo_q"].as_u64().unwrap());
                assert_eq!(r.value("sub[1]"), expected);
                assert_eq!(out.camo_stats.gated, usize::from(expected.is_none()));
            }
            if let Some(r) = out.abilities.first() {
                assert_eq!(r.value("counter"), row["counter"].as_u64());
                assert_eq!(r.value("rank"), row["rank"].as_u64());
                assert_eq!(
                    out.ability_stats.gated,
                    usize::from(row["rank"].as_i64().unwrap() < 0)
                );
            }
        }
    }
}
