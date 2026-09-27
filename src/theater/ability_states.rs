//! Grapple anchors and thruster impulse reads through the canonical ability grammar.
use super::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AbilityStateRead {
    pub source: FilmPacket,
    pub slot: u32,
    pub predicted: bool,
    /// Whether dispatch succeeded within source bounds. This does not imply that
    /// an optional ability body was enabled or read; consult its typed observation.
    pub complete: bool,
    pub component: DecodedComponent,
}
impl AbilityStateRead {
    fn value(&self, name: &str) -> Option<u64> {
        self.component
            .fields
            .iter()
            .find(|f| f.name == name)
            .map(|f| f.raw)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AbilityImpulse {
    /// Native packet ordinal; unknown for unframed caller-supplied input.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub packet_index: Option<usize>,
    pub source: FilmPacket,
    pub slot: u32,
    pub predicted: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrappleRead {
    /// Native packet ordinal; unknown for unframed caller-supplied input.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub packet_index: Option<usize>,
    pub source: FilmPacket,
    pub slot: u32,
    pub heavy: bool,
    pub position_quantized: [u32; 3],
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AbilityImpulseStats {
    pub records: usize,
    #[serde(rename = "WithI57")]
    pub with_predicted: usize,
    #[serde(rename = "WithI59")]
    pub with_non_predicted: usize,
    pub read: usize,
    pub unread: usize,
    #[serde(rename = "Tag1")]
    pub tag_one: usize,
    pub absent: bool,
    pub scanned: bool,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GrappleStats {
    pub records: usize,
    #[serde(rename = "WithI59")]
    pub with_component: usize,
    pub read: usize,
    pub unread: usize,
    #[serde(rename = "Tag3")]
    pub tag_three: usize,
    pub body_broken: usize,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BipedAbilityStates {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rejected_components: Vec<BipedChannelRejection>,
    /// Every attempted component in scan order, including intermediate fields and callbacks.
    /// Absence in old exports means this trace was unavailable.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub component_attempts: Vec<super::BipedComponentAttempt>,
    pub reads: Vec<AbilityStateRead>,
    pub impulses: Vec<AbilityImpulse>,
    pub grapple: Vec<GrappleRead>,
    pub impulse_stats: AbilityImpulseStats,
    /// None explicitly identifies an absent grapple component in the registry.
    pub grapple_stats: Option<GrappleStats>,
}
pub(crate) fn component_index(arch: &FilmArchetype, name: &str) -> Option<u8> {
    [format!("{name}-component"), name.into()]
        .iter()
        .find_map(|name| {
            arch.components
                .iter()
                .position(|n| n == name)
                .map(|i| i as u8)
        })
}

/// Scan all guarded position candidates, including those rejected by motion filters.
/// Broken known bodies retain their decoded prefix; truncated bodies are not published.
pub fn scan_biped_ability_states(
    chunks: &[crate::clients::hi::models::FilmChunkData],
    registry: &FilmRegistry,
    positions: &BipedPositionStream,
    encoding: &PositionEncoding,
) -> Result<BipedAbilityStates, DecodeError> {
    scan_biped_ability_states_impl(
        chunks,
        registry,
        positions,
        super::biped_channels::BipedComponentReader::Legacy(encoding),
    )
}

/// Native direct biped walk under a complete inherited reader context.
/// Traversal width overrides and corruption guards do not apply here.
pub fn scan_biped_ability_states_with_context(
    chunks: &[crate::clients::hi::models::FilmChunkData],
    registry: &FilmRegistry,
    positions: &BipedPositionStream,
    context: &super::FrameEncoding,
) -> Result<BipedAbilityStates, DecodeError> {
    scan_biped_ability_states_impl(
        chunks,
        registry,
        positions,
        super::biped_channels::BipedComponentReader::Context(context),
    )
}

fn scan_biped_ability_states_impl(
    chunks: &[crate::clients::hi::models::FilmChunkData],
    registry: &FilmRegistry,
    positions: &BipedPositionStream,
    reader: super::biped_channels::BipedComponentReader<'_>,
) -> Result<BipedAbilityStates, DecodeError> {
    let arch = registry
        .archetype(35)
        .ok_or(DecodeError::Missing("biped archetype"))?;
    let predicted = component_index(arch, "biped-spartan-ability");
    let non_predicted = component_index(arch, "biped-spartan-ability-non-predicted-state");
    let mut out = BipedAbilityStates {
        grapple_stats: non_predicted.map(|_| GrappleStats::default()),
        ..Default::default()
    };
    if predicted.is_none() && non_predicted.is_none() {
        out.impulse_stats.absent = true;
        out.impulse_stats.scanned = true;
        return Ok(out);
    }
    let packet_indices = super::fire_events::native_packet_indices(chunks);
    let bytes = super::fire_events::native_chunk_data(chunks);
    for candidate in &positions.candidates {
        // Native delta walking skips absent chunks before counting records.
        if !bytes.contains_key(&candidate.source.chunk_index) {
            continue;
        }
        out.impulse_stats.records += 1;
        if let Some(st) = &mut out.grapple_stats {
            st.records += 1;
        }
        let r = &candidate.record;
        let has57 = predicted.is_some_and(|id| r.component_indices.contains(&id));
        let has59 = non_predicted.is_some_and(|id| r.component_indices.contains(&id));
        out.impulse_stats.with_predicted += usize::from(has57);
        out.impulse_stats.with_non_predicted += usize::from(has59);
        if let Some(st) = &mut out.grapple_stats {
            st.with_component += usize::from(has59);
        }
        let target = predicted
            .filter(|_| has57)
            .into_iter()
            .chain(non_predicted.filter(|_| has59))
            .max();
        let Some(target) = target else { continue };
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
        let mut at = r.end_bit.saturating_add(2);
        let mut got57 = false;
        let mut got59 = false;
        for &id in r.component_indices.iter().skip(1) {
            let Some(name) = arch
                .components
                .get(usize::from(id))
                .filter(|n| !n.is_empty())
            else {
                break;
            };
            let level = arch.levels.get(usize::from(id)).copied().unwrap_or(0);
            let (raw_status, component) = reader.read_raw(data, at, name, level);
            let in_bounds =
                component.end_bit >= 0 && component.end_bit <= data.len().saturating_mul(8) as i64;
            let attempt = super::BipedComponentAttempt {
                component_index: id,
                status: raw_status,
                in_bounds,
                read: BipedChannelRead {
                    source,
                    packet_index: packet_indices
                        .get(&(source.chunk_index, source.payload_offset))
                        .copied(),
                    slot: r.slot,
                    record_start_bit: r.start_bit,
                    component: component.clone(),
                },
            };
            if raw_status != Some(true) || !in_bounds {
                out.rejected_components.push(attempt.clone());
            }
            out.component_attempts.push(attempt);
            let status = if in_bounds { raw_status } else { None };
            at = usize::try_from(component.end_bit).expect("in-bounds component endpoint");
            // Native scanners consume hook publications even when the following
            // walker bounds/dispatch check rejects the component. A partial read
            // without that hook is not a published ability state.
            let published = component
                .diagnostics
                .component_observations
                .iter()
                .any(|o| {
                    matches!(o, FilmComponentObservation::SpartanAbility { .. })
                        && Some(id) == predicted
                        || matches!(o, FilmComponentObservation::AbilityNonPredicted { .. })
                            && Some(id) == non_predicted
                });
            if published {
                let is_predicted = Some(id) == predicted;
                let read = AbilityStateRead {
                    source,
                    slot: r.slot,
                    predicted: is_predicted,
                    complete: status == Some(true),
                    component,
                };
                if is_predicted {
                    got57 = true
                } else {
                    got59 = true
                }
                out.impulse_stats.read += 1;
                if read.value("tag") == Some(1) {
                    out.impulse_stats.tag_one += 1;
                    out.impulses.push(AbilityImpulse {
                        packet_index: packet_indices
                            .get(&(source.chunk_index, source.payload_offset))
                            .copied(),
                        source,
                        slot: r.slot,
                        predicted: is_predicted,
                    });
                }
                if !is_predicted {
                    let st = out.grapple_stats.as_mut().unwrap();
                    st.read += 1;
                    let state = read
                        .component
                        .diagnostics
                        .component_observations
                        .iter()
                        .find_map(|o| {
                            if let FilmComponentObservation::AbilityNonPredicted { state } = o {
                                Some(state.as_ref())
                            } else {
                                None
                            }
                        });
                    if let Some(state) = state.filter(|state| state.tag == 3) {
                        st.tag_three += 1;
                        if !state.body_ok {
                            st.body_broken += 1;
                        } else if matches!(state.inner, Some(1 | 2)) {
                            out.grapple.push(GrappleRead {
                                packet_index: packet_indices
                                    .get(&(source.chunk_index, source.payload_offset))
                                    .copied(),
                                source,
                                slot: r.slot,
                                heavy: state.inner == Some(2),
                                position_quantized: state.position,
                            });
                        }
                    }
                }
                out.reads.push(read);
            }
            if status != Some(true) || id >= target {
                break;
            }
        }
        out.impulse_stats.unread += usize::from(has57 && !got57) + usize::from(has59 && !got59);
        if let Some(st) = &mut out.grapple_stats {
            st.unread += usize::from(has59 && !got59);
        }
    }
    out.impulses
        .sort_by_key(|e| (e.source.timestamp_us, e.slot, !e.predicted));
    out.impulse_stats.scanned = true;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clients::hi::models::{FilmChunk, FilmChunkData};
    use serde_json::{Value, json};
    use std::io::Read;
    #[test]
    fn ability_hooks_bodies_and_scanner_counters_match_go() {
        let mut text = String::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/biped-scan-levelup-v41.json.zlib")[..],
        )
        .read_to_string(&mut text)
        .unwrap();
        let oracle: Value = serde_json::from_str(&text).unwrap();
        let rows = oracle["ability_states"].as_array().unwrap();
        assert_eq!(rows.len(), 4096);
        for row in rows {
            let map: FilmMapBounds = serde_json::from_value(row["map"].clone()).unwrap();
            let mut encoding = map.position_encoding();
            encoding.full_precision = row["full_precision"].as_bool().unwrap();
            let mut archetypes: Vec<_> = (0..36)
                .map(|index| FilmArchetype {
                    index,
                    components: vec![],
                    levels: vec![],
                })
                .collect();
            archetypes[35].components =
                serde_json::from_value(oracle["ability_names"].clone()).unwrap();
            archetypes[35].levels = vec![0; 64];
            let level = row["level"].as_u64().unwrap() as u32;
            archetypes[35].levels[59] = level;
            let registry = FilmRegistry {
                archetypes,
                major_version: 41,
                format_version: 27,
                end_byte: 0,
                truncated: false,
            };
            let hex = row["hex"].as_str().unwrap();
            let bytes: Vec<_> = (0..hex.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                .collect();
            let at = row["at"].as_u64().unwrap() as usize;
            let (status, energy) = super::super::components::decode_component_attempt(
                &bytes,
                at,
                "biped-spartan-ability-energy-component",
                0,
                35,
                Some(&encoding),
            );
            assert_eq!(status, Some(true));
            assert_eq!(
                serde_json::json!(energy.end_bit),
                serde_json::json!(row["energy_end"].as_u64().unwrap() as usize)
            );
            let (status, mobility) = super::super::components::decode_component_attempt(
                &bytes,
                at,
                "biped-mobility-action-component",
                0,
                35,
                Some(&encoding),
            );
            assert_eq!(status, Some(true), "{row}");
            assert_eq!(
                serde_json::json!(mobility.end_bit),
                serde_json::json!(row["mobility_end"].as_u64().unwrap() as usize),
                "{row}"
            );
            for d in row["direct"].as_array().unwrap() {
                let predicted = d["predicted"].as_bool().unwrap();
                let name = &registry.archetypes[35].components[if predicted { 57 } else { 59 }];
                let (status, component) = super::super::components::decode_component_attempt(
                    &bytes,
                    at,
                    name,
                    level,
                    35,
                    Some(&encoding),
                );
                assert_eq!(status, Some(d["ok"].as_bool().unwrap()), "{d} {row}");
                assert_eq!(
                    serde_json::json!(component.end_bit),
                    serde_json::json!(d["end"].as_u64().unwrap() as usize),
                    "{d} {row}"
                );
                if status == Some(true) {
                    let short =
                        &bytes[..crate::theater::bits::native_address((component.end_bit - 1) / 8)];
                    assert_eq!(
                        super::super::components::decode_component_attempt(
                            short,
                            at,
                            name,
                            level,
                            35,
                            Some(&encoding)
                        )
                        .0,
                        None,
                        "truncated {d}"
                    );
                }
                let raw = |name: &str| {
                    component
                        .fields
                        .iter()
                        .find(|f| f.name == name)
                        .map(|f| f.raw)
                };
                let state = &d["state"];
                assert_eq!(
                    raw("tag"),
                    Some(if predicted {
                        d["tag"].as_u64().unwrap()
                    } else {
                        state["Tag"].as_u64().unwrap()
                    })
                );
                if predicted && d["has_ref"] == true {
                    assert_eq!(raw("sub"), d["sub"].as_u64());
                    assert_eq!(raw("reference"), d["ref"].as_u64());
                }
                if !predicted && state["BodyWalked"] == true {
                    assert_eq!(raw("inner"), state["Inner"].as_u64());
                    assert_eq!(raw("flags"), state["Zero3"].as_u64());
                    for i in 0..3 {
                        if let Some(q) = raw(&format!("position[{i}]")) {
                            assert_eq!(q, state["PosQ"][i].as_u64().unwrap());
                        }
                    }
                }
                let mut end = at;
                for f in &component.fields {
                    assert_eq!(serde_json::json!(f.bit), serde_json::json!(end));
                    end += usize::try_from(f.width).unwrap();
                    assert_eq!(
                        serde_json::json!(super::super::bits::Bits(&bytes).read(f.bit, f.width)),
                        serde_json::json!(Some(f.raw))
                    );
                }
                assert_eq!(serde_json::json!(end), serde_json::json!(component.end_bit));
            }
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
                        end_bit: at - 2,
                        slot: 512,
                        generation: 1,
                        component_indices: serde_json::from_value(row["mask"].clone()).unwrap(),
                        quantized: [0; 3],
                        world: [0.0; 3],
                        companions: Default::default(),
                    },
                    rejection: Some(BipedPositionRejection::Saturated),
                }],
            };
            let out = scan_biped_ability_states(
                std::slice::from_ref(&chunk),
                &registry,
                &positions,
                &encoding,
            )
            .unwrap();
            assert!(out.impulses.iter().all(|r| r.packet_index == Some(1)));
            assert!(out.grapple.iter().all(|r| r.packet_index == Some(1)));
            let restored: BipedAbilityStates =
                serde_json::from_value(serde_json::to_value(&out).unwrap()).unwrap();
            assert_eq!(restored, out);
            let mut charge_positions = positions.clone();
            charge_positions.candidates[0].record.component_indices =
                serde_json::from_value(row["charge_mask"].clone()).unwrap();
            let charges = scan_ability_charges(
                std::slice::from_ref(&chunk),
                &registry,
                &charge_positions,
                &encoding,
            )
            .unwrap();
            assert!(charges.records.iter().all(|r| r.packet_index == Some(1)));
            assert!(charges.components.iter().all(|r| r.packet_index == Some(1)));
            let restored: AbilityChargeStream =
                serde_json::from_value(serde_json::to_value(&charges).unwrap()).unwrap();
            assert_eq!(restored, charges);
            assert_eq!(json!(charges.stats), row["charge_stats"], "{row}");
            let published:Vec<_>=charges.records.iter().map(|a|json!({"Slot":a.slot,"Chunk":1,"PacketIndex":0,"TimestampUS":0,"Emplacement":a.emplacement,"Charges":a.charges,"Low":a.low})).collect();
            assert_eq!(
                published,
                row["charges"].as_array().cloned().unwrap_or_default(),
                "{row}"
            );
            for c in &charges.components {
                let mask = c.value("mask").unwrap();
                for i in 0..3 {
                    assert_eq!(
                        c.value(&format!("charge[{i}]")).is_some(),
                        mask & (1 << i) != 0
                    );
                }
            }
            assert_eq!(json!(out.impulse_stats), row["impulse_stats"], "{row}");
            assert_eq!(json!(out.grapple_stats), row["grapple_stats"], "{row}");
            let impulses:Vec<_>=out.impulses.iter().map(|a|json!({"Slot":a.slot,"Chunk":1,"PacketIndex":0,"TimestampUS":0,"Predicted":a.predicted})).collect();
            assert_eq!(
                impulses,
                row["impulses"].as_array().cloned().unwrap_or_default(),
                "{row}"
            );
            let grapple:Vec<_>=out.grapple.iter().map(|a|json!({"Slot":a.slot,"Chunk":1,"PacketIndex":0,"TimestampUS":0,"Heavy":a.heavy,"PosQ":a.position_quantized})).collect();
            assert_eq!(
                grapple,
                row["grapple"].as_array().cloned().unwrap_or_default(),
                "{row}"
            );
        }
    }
}
