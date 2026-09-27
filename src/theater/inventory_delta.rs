//! Native grenade and ammunition delta observations and whole-film rejection rules.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InventoryDeltaStats {
    #[serde(rename = "Records")]
    pub records: usize,
    #[serde(rename = "WithI22")]
    pub with_i22: usize,
    #[serde(rename = "WithI47")]
    pub with_i47: usize,
    #[serde(rename = "I22Read")]
    pub i22_read: usize,
    #[serde(rename = "I22Unread")]
    pub i22_unread: usize,
    #[serde(rename = "I47Read")]
    pub i47_read: usize,
    #[serde(rename = "I47Unread")]
    pub i47_unread: usize,
    #[serde(rename = "Implausible")]
    pub implausible: usize,
    #[serde(rename = "NoSelection")]
    pub no_selection: usize,
    #[serde(rename = "MaskEmpty")]
    pub mask_empty: usize,
    #[serde(rename = "SelOutsideMask")]
    pub sel_outside_mask: usize,
    #[serde(rename = "WithAmmo")]
    pub with_ammo: usize,
    #[serde(rename = "WithRounds")]
    pub with_rounds: usize,
    #[serde(rename = "AmmoRead")]
    pub ammo_read: usize,
    #[serde(rename = "RoundsRead")]
    pub rounds_read: usize,
    #[serde(rename = "MagRead")]
    pub mag_read: usize,
    #[serde(rename = "MagOutOfEnvelope")]
    pub mag_out_of_envelope: usize,
    #[serde(rename = "ResOutOfEnvelope")]
    pub res_out_of_envelope: usize,
    #[serde(rename = "MagCorroborated")]
    pub mag_corroborated: usize,
    #[serde(rename = "MagOutOfEnvelopeCorroborated")]
    pub mag_out_of_envelope_corroborated: usize,
    #[serde(rename = "AccordChecked")]
    pub accord_checked: usize,
    #[serde(rename = "Accord")]
    pub accord: usize,
    #[serde(rename = "Emitted")]
    pub emitted: usize,
    #[serde(rename = "AmmoRefused")]
    pub ammo_refused: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InventoryAmmo {
    pub weapon_slot: u8,
    pub magazine: Option<u32>,
    pub fraction_quantum: Option<u32>,
    pub reserve: Option<u32>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InventoryGrenadeSelection {
    pub mask: u32,
    pub rank: Option<u8>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InventoryDeltaRead {
    /// Requested native file number; distinct from the loaded buffer identity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chunk_number: Option<i64>,
    pub source: FilmPacket,
    /// Zero-based native packet ordinal, counting every framed packet type.
    /// Unknown when callers supply anchors over unframed component payloads.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub packet_index: Option<usize>,
    pub slot: u32,
    pub grenades: Option<[u32; 4]>,
    pub selection: Option<InventoryGrenadeSelection>,
    pub ammo: Vec<InventoryAmmo>,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InventoryDeltaStream {
    /// Failed intermediate/target reads with raw fields, source and callbacks.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rejected_components: Vec<BipedChannelRejection>,
    /// Every attempted component in scan order, including intermediate fields and callbacks.
    /// Absence in old exports means this trace was unavailable.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub component_attempts: Vec<super::BipedComponentAttempt>,
    pub stats: InventoryDeltaStats,
    pub records: Vec<InventoryDeltaRead>,
    /// Includes implausible or globally refused values with their original bit ranges.
    pub components: Vec<BipedChannelRead>,
}
#[derive(Clone, Copy)]
pub(super) enum Role {
    Grenades,
    Selection,
    Ammo(usize),
    Rounds(usize),
}
pub(super) fn roles(arch: &FilmArchetype) -> BTreeMap<u8, Role> {
    let mut out = BTreeMap::new();
    for (names, role) in [
        (&["unit-grenade-counts-component"][..], Role::Grenades),
        (
            &[
                "biped-desired-grenade-set-component",
                "biped-desired-grenade-set",
            ][..],
            Role::Selection,
        ),
    ] {
        if let Some(i) = names
            .iter()
            .find_map(|n| arch.components.iter().position(|v| v == n))
        {
            out.insert(i as u8, role);
        }
    }
    for (name, ammo) in [
        ("weapon-state-ammo", true),
        ("weapon-state-rounds-inventory", false),
    ] {
        for (slot, (i, _)) in arch
            .components
            .iter()
            .enumerate()
            .filter(|(_, n)| n.as_str() == name)
            .take(4)
            .enumerate()
        {
            out.insert(
                i as u8,
                if ammo {
                    Role::Ammo(slot)
                } else {
                    Role::Rounds(slot)
                },
            );
        }
    }
    out
}
/// Read all announced inventory components with the same walker as ability/camouflage.
pub fn scan_inventory_deltas(
    chunks: &[crate::clients::hi::models::FilmChunkData],
    registry: &FilmRegistry,
    anchors: &BipedPositionStream,
    encoding: &PositionEncoding,
) -> Result<InventoryDeltaStream, DecodeError> {
    scan_inventory_deltas_impl(
        chunks,
        registry,
        anchors,
        super::biped_channels::BipedComponentReader::Legacy(encoding),
    )
}

/// Native direct biped walk under a complete inherited reader context.
/// Traversal width overrides and corruption guards do not apply here.
pub fn scan_inventory_deltas_with_context(
    chunks: &[crate::clients::hi::models::FilmChunkData],
    registry: &FilmRegistry,
    anchors: &BipedPositionStream,
    context: &super::FrameEncoding,
) -> Result<InventoryDeltaStream, DecodeError> {
    scan_inventory_deltas_impl(
        chunks,
        registry,
        anchors,
        super::biped_channels::BipedComponentReader::Context(context),
    )
}

fn scan_inventory_deltas_impl(
    chunks: &[crate::clients::hi::models::FilmChunkData],
    registry: &FilmRegistry,
    anchors: &BipedPositionStream,
    reader: super::biped_channels::BipedComponentReader<'_>,
) -> Result<InventoryDeltaStream, DecodeError> {
    let arch = registry
        .archetype(35)
        .ok_or(DecodeError::Missing("biped archetype"))?;
    let roles = roles(arch);
    if roles.is_empty() {
        return Err(DecodeError::Missing("biped inventory components"));
    }
    let bytes = super::fire_events::native_chunk_data(chunks);
    let packet_indices = super::fire_events::native_packet_indices(chunks);
    let mut out = InventoryDeltaStream::default();
    for candidate in &anchors.candidates {
        let source = candidate.source;
        // Match native delta walking: missing chunks have no record callbacks.
        let Some(chunk) = bytes.get(&source.chunk_index) else {
            continue;
        };
        out.stats.records += 1;
        let r = &candidate.record;
        let wanted: Vec<_> = r
            .component_indices
            .iter()
            .skip(1)
            .filter_map(|id| roles.get(id).map(|role| (*id, *role)))
            .collect();
        let Some(&(last, _)) = wanted.last() else {
            continue;
        };
        for (_, role) in &wanted {
            match role {
                Role::Grenades => out.stats.with_i22 += 1,
                Role::Selection => out.stats.with_i47 += 1,
                Role::Ammo(_) => out.stats.with_ammo += 1,
                Role::Rounds(_) => out.stats.with_rounds += 1,
            }
        }
        let data = source
            .payload_offset
            .checked_add(source.payload_size)
            .and_then(|end| chunk.get(source.payload_offset..end))
            .ok_or(DecodeError::Truncated {
                chunk: source.chunk_index,
                offset: source.payload_offset,
            })?;
        let mut counts = None;
        let mut selection = None;
        let mut ammo: [Option<(Option<u32>, Option<u32>)>; 4] = [None; 4];
        let mut rounds = [None; 4];
        for (id, component) in super::biped_channels::walk_biped_components_retaining_rejections(
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
        ) {
            let Some(role) = roles.get(&id) else { continue };
            let read = BipedChannelRead {
                packet_index: packet_indices
                    .get(&(source.chunk_index, source.payload_offset))
                    .copied(),
                source,
                slot: r.slot,
                record_start_bit: r.start_bit,
                component,
            };
            match role {
                Role::Grenades => {
                    let count = read.value("count").unwrap();
                    let values: Vec<_> = (0..count)
                        .map(|i| read.value(&format!("grenades[{i}]")).unwrap() as u32)
                        .collect();
                    counts = Some((count, values));
                }
                Role::Selection => {
                    selection = Some((
                        read.value("mask").unwrap() as u32,
                        read.value("selection").unwrap() as u8,
                    ))
                }
                Role::Ammo(i) => {
                    ammo[*i] = Some((
                        read.value("magazine").map(|v| v as u32),
                        read.value("fraction").map(|v| v as u32),
                    ))
                }
                Role::Rounds(i) => rounds[*i] = read.value("rounds").map(|v| v as u32),
            }
            out.components.push(read);
        }
        publish_inventory_record(
            &mut out,
            InventoryDeltaRead {
                chunk_number: Some(i64::from(source.chunk_index)),
                source,
                packet_index: packet_indices
                    .get(&(source.chunk_index, source.payload_offset))
                    .copied(),
                slot: r.slot,
                grenades: None,
                selection: None,
                ammo: vec![],
            },
            InventoryRecordValues {
                counts,
                selection,
                ammo,
                rounds,
            },
        );
    }
    refuse_inventory_ammo(&mut out);
    Ok(out)
}

#[derive(Default)]
pub(super) struct InventoryRecordValues {
    pub counts: Option<(u64, Vec<u32>)>,
    pub selection: Option<(u32, u8)>,
    pub ammo: [Option<(Option<u32>, Option<u32>)>; 4],
    pub rounds: [Option<u32>; 4],
}
pub(super) fn publish_inventory_record(
    out: &mut InventoryDeltaStream,
    mut rec: InventoryDeltaRead,
    values: InventoryRecordValues,
) {
    let InventoryRecordValues {
        counts,
        selection,
        ammo,
        rounds,
    } = values;
    let st = &mut out.stats;
    if let Some((count, values)) = counts {
        st.i22_read += 1;
        if count == 4 && values.len() == 4 && values.iter().all(|&v| v <= 2) {
            rec.grenades = Some(values.try_into().unwrap());
        } else {
            st.implausible += 1;
        }
    } else {
        st.i22_unread += 1;
    }
    if let Some((mask, sel)) = selection {
        st.i47_read += 1;
        let mut rank = None;
        if mask == 0 {
            st.mask_empty += 1;
        } else if sel == 0 {
            st.no_selection += 1;
        } else if sel > 4 || mask & (1 << (sel - 1)) == 0 {
            st.sel_outside_mask += 1;
        } else {
            rank = Some(sel - 1);
        }
        rec.selection = Some(InventoryGrenadeSelection { mask, rank });
    } else {
        st.i47_unread += 1;
    }
    for i in 0..4 {
        let mut value = InventoryAmmo {
            weapon_slot: i as u8,
            magazine: None,
            fraction_quantum: None,
            reserve: None,
        };
        if let Some((mag, frac)) = ammo[i] {
            st.ammo_read += 1;
            value.fraction_quantum = frac;
            if let Some(mag) = mag {
                st.mag_read += 1;
                let corroborated = rec.grenades.is_some();
                st.mag_corroborated += usize::from(corroborated);
                if mag > 120 {
                    st.mag_out_of_envelope += 1;
                    st.mag_out_of_envelope_corroborated += usize::from(corroborated);
                } else {
                    value.magazine = Some(mag);
                }
            }
        }
        if let Some(res) = rounds[i] {
            st.rounds_read += 1;
            if res > 400 {
                st.res_out_of_envelope += 1;
            } else {
                value.reserve = Some(res);
            }
        }
        if value.magazine.is_some() || value.fraction_quantum.is_some() || value.reserve.is_some() {
            rec.ammo.push(value);
        }
    }
    if let (Some(grenades), Some(sel)) = (&rec.grenades, &rec.selection) {
        st.accord_checked += 1;
        let bitmap = grenades
            .iter()
            .enumerate()
            .fold(0, |m, (i, &n)| m | if n > 0 { 1 << i } else { 0 });
        st.accord += usize::from(bitmap == sel.mask);
    }
    if rec.grenades.is_some() || rec.selection.is_some() || !rec.ammo.is_empty() {
        out.records.push(rec);
        st.emitted += 1;
    }
}
pub(super) fn refuse_inventory_ammo(out: &mut InventoryDeltaStream) {
    if out.stats.mag_read >= 200
        && out.stats.mag_out_of_envelope as f64 / out.stats.mag_read as f64 >= 0.01
    {
        out.stats.ammo_refused = true;
        for r in &mut out.records {
            r.ammo.clear();
        }
        out.records
            .retain(|r| r.grenades.is_some() || r.selection.is_some());
        out.stats.emitted = out.records.len();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clients::hi::models::{FilmChunk, FilmChunkData};
    use std::io::Read;
    #[test]
    fn inventory_publication_and_whole_film_ammo_refusal_match_go() {
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
            serde_json::from_value(oracle["inventory_names"].clone()).unwrap();
        archetypes[35].levels = vec![0; 64];
        let registry = FilmRegistry {
            archetypes,
            major_version: 41,
            format_version: 27,
            end_byte: 0,
            truncated: false,
        };
        let cases = oracle["inventory_cases"].as_array().unwrap();
        assert_eq!(cases.len(), 16);
        let mut refused = 0;
        for case in cases {
            let mut chunks = Vec::new();
            let mut anchors = BipedPositionStream {
                record_masks: vec![],
                slot_band: Some([512, 512]),
                options: Default::default(),
                candidates: vec![],
            };
            for (i, row) in case["input"].as_array().unwrap().iter().enumerate() {
                let hex = row["hex"].as_str().unwrap();
                let bytes: Vec<u8> = (0..hex.len())
                    .step_by(2)
                    .map(|j| u8::from_str_radix(&hex[j..j + 2], 16).unwrap())
                    .collect();
                let source = FilmPacket {
                    chunk_index: i as i32 + 1,
                    packet_type: 0,
                    byte_2: 0,
                    byte_3: 0,
                    payload_offset: 0,
                    payload_size: bytes.len(),
                    timestamp_us: i as u64,
                };
                chunks.push(FilmChunkData {
                    metadata: FilmChunk {
                        index: source.chunk_index,
                        chunk_type: 2,
                        start_time_offset_ms: 0,
                        duration_ms: 0,
                        size: bytes.len() as i64,
                        file_relative_path: String::new(),
                    },
                    data: bytes,
                });
                let mask = serde_json::from_value(row["mask"].clone()).unwrap();
                anchors.candidates.push(BipedPositionCandidate {
                    packet_index: None,
                    source,
                    record: BipedPositionRecord {
                        start_bit: 0,
                        position_bit: 0,
                        end_bit: row["at"].as_u64().unwrap() as usize - 2,
                        slot: 512,
                        generation: 1,
                        component_indices: mask,
                        quantized: [0; 3],
                        world: [0.0; 3],
                        companions: Default::default(),
                    },
                    rejection: Some(BipedPositionRejection::Saturated),
                });
            }
            let out = scan_inventory_deltas(&chunks, &registry, &anchors, &map.position_encoding())
                .unwrap();
            assert_eq!(serde_json::json!(out.stats), case["stats"]);
            refused += usize::from(out.stats.ammo_refused);
            let expected = case["records"].as_array().cloned().unwrap_or_default();
            assert_eq!(out.records.len(), expected.len());
            for (a, e) in out.records.iter().zip(&expected) {
                let ammo:Vec<_>=a.ammo.iter().map(|v|serde_json::json!({"WeaponSlot":v.weapon_slot,"Mag":v.magazine,"FracQ":v.fraction_quantum,"Res":v.reserve})).collect();
                let ammo = if ammo.is_empty() {
                    serde_json::Value::Null
                } else {
                    serde_json::json!(ammo)
                };
                assert_eq!(
                    serde_json::json!({"Slot":a.slot,"Chunk":a.source.chunk_index,"PacketIndex":0,"TimestampUS":a.source.timestamp_us,"Grenades":a.grenades,"SelRead":a.selection.is_some(),"Sel":a.selection.as_ref().map_or(0,|s|s.rank.map_or(-1,i32::from)),"Mask":a.selection.as_ref().map_or(0,|s|s.mask),"Ammo":ammo}),
                    *e
                );
            }
            assert!(!out.components.is_empty());
            // The oracle supplies unframed payloads. Exercise the same positive
            // component bytes in real envelopes with a preceding keyframe.
            assert!(out.records.iter().all(|r| r.packet_index.is_none()));
            for (chunk, anchor) in chunks.iter_mut().zip(&mut anchors.candidates) {
                let payload = std::mem::take(&mut chunk.data);
                let mut framed = Vec::new();
                for (kind, bytes) in [(2u16, &[0x40][..]), (0, payload.as_slice())] {
                    framed.extend_from_slice(&kind.to_le_bytes());
                    framed.extend_from_slice(&[0, 0]);
                    framed.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
                    framed.extend_from_slice(&anchor.source.timestamp_us.to_le_bytes());
                    framed.extend_from_slice(bytes);
                }
                chunk.data = framed;
                chunk.metadata.size = chunk.data.len() as i64;
                anchor.source.payload_offset = 33;
            }
            let framed =
                scan_inventory_deltas(&chunks, &registry, &anchors, &map.position_encoding())
                    .unwrap();
            assert_eq!(framed.stats, out.stats);
            assert_eq!(framed.records.len(), out.records.len());
            for (actual, expected) in framed.records.iter().zip(&out.records) {
                assert_eq!(actual.packet_index, Some(1));
                assert_eq!(actual.source.payload_offset, 33);
                let mut normalized = actual.clone();
                normalized.packet_index = None;
                normalized.source.payload_offset = 0;
                assert_eq!(&normalized, expected);
            }
            let restored: InventoryDeltaStream =
                serde_json::from_value(serde_json::to_value(&framed).unwrap()).unwrap();
            assert_eq!(restored, framed);
        }
        assert!(refused > 0 && refused < 16);
    }
}
