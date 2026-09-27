//! Typed projections of the six native `capture.go` payloads. No bytes are reread.
use super::{ComponentField, DecodedComponent, EntityRecord, KeyframeBipedProbe, KeyframeRecord};
use crate::theater::{
    DecodedBodyVitality, DecodedShieldVitality, FilmComponentObservation, NativeObjectParentState,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum CapturedComponentPayload {
    Body(DecodedBodyVitality),
    Shield(DecodedShieldVitality),
    /// Uses the retained observer's actual end bit. The native payload leaves
    /// EndBit zero when its parent observer is absent; Rust retains that observer.
    Parent(NativeObjectParentState),
    Dissolver(DecodedObjectDissolver),
    Respawn(DecodedRespawnTimer),
    RoundTimer(DecodedRoundTimer),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecodedObjectDissolver {
    pub state: u32,
    /// Native discards these 96 bits; retain them rather than only its body flag.
    pub body: Option<[u32; 3]>,
    pub duration_quantum: u32,
    pub flag: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecodedRespawnTimer {
    pub active: bool,
    /// Raw ten-bit words. Their time unit is not established by the recording.
    pub timers: [u16; 2],
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecodedRoundTimer {
    pub quanta: [u16; 2],
    /// Native endpoint dequantization, in seconds; no elapsed/remaining attribution.
    pub seconds: [f32; 2],
    pub tail: u8,
}

fn round_seconds(q: u16) -> f32 {
    crate::theater::dequantize_native_endpoint(u64::from(q), 0., 36000., 16, false, true)
}

fn project(
    name: &str,
    fields: &[ComponentField],
    observations: &[FilmComponentObservation],
    start: i64,
) -> Option<CapturedComponentPayload> {
    let raw = |name: &str| fields.iter().find(|f| f.name == name).map(|f| f.raw);
    let bit = |name: &str| raw(name).map(|v| v != 0);
    use CapturedComponentPayload as P;
    Some(match name {
        "object-body-vitality-component" => {
            let quantum = raw("health")? as u8;
            P::Body(DecodedBodyVitality {
                quantum,
                health: crate::theater::vitality::vitality_value(quantum, true),
                flags: [bit("flags[0]")?, bit("flags[1]")?, bit("flags[2]")?],
            })
        }
        "object-shield-vitality-component" => {
            let quantum = raw("shield")? as u8;
            let regen_present = bit("regeneration")?;
            let mut regen = [None; 2];
            if regen_present {
                for (i, name) in ["regeneration_a", "regeneration_b"].iter().enumerate() {
                    if bit(&format!("{name}.gate"))? {
                        regen[i] = Some(raw(name)? as u16);
                    }
                }
            }
            P::Shield(DecodedShieldVitality {
                quantum,
                shield: crate::theater::vitality::vitality_value(quantum, false),
                regen_present,
                regen,
                block_64: raw("block64")? as u16,
                flags: [
                    bit("flags[0]")?,
                    bit("flags[1]")?,
                    bit("flags[2]")?,
                    bit("flags[3]")?,
                ],
            })
        }
        "object-parent-state-component" => {
            P::Parent(observations.iter().find_map(|o| match o {
                FilmComponentObservation::ObjectParent { state } if state.start_bit == start => {
                    Some((**state).clone())
                }
                _ => None,
            })?)
        }
        "object-dissolver-component" => {
            let state = raw("state")? as u32;
            let body = if state == 13 {
                None
            } else {
                Some([
                    raw("body_words[0]")? as u32,
                    raw("body_words[1]")? as u32,
                    raw("body_words[2]")? as u32,
                ])
            };
            P::Dissolver(DecodedObjectDissolver {
                state,
                body,
                duration_quantum: if body.is_some() {
                    raw("duration")? as u32
                } else {
                    0
                },
                flag: body.is_some() && bit("flag")?,
            })
        }
        "player-respawn-timer-component" => P::Respawn(DecodedRespawnTimer {
            active: bit("active")?,
            timers: [raw("timers[0]")? as u16, raw("timers[1]")? as u16],
        }),
        "game-engine-round-timer-component" => {
            let quanta = [raw("timers[0]")? as u16, raw("timers[1]")? as u16];
            P::RoundTimer(DecodedRoundTimer {
                quanta,
                seconds: quanta.map(round_seconds),
                tail: raw("state")? as u8,
            })
        }
        _ => return None,
    })
}

fn observation_slice(
    observations: &[FilmComponentObservation],
    range: Option<[usize; 2]>,
) -> Option<&[FilmComponentObservation]> {
    let Some([start, end]) = range else {
        return Some(&[]);
    };
    observations.get(start..end)
}

impl DecodedComponent {
    /// Native typed value, when this named payload was fully read. Unsupported
    /// names, skipped widths and incomplete field sets return None.
    pub fn captured_payload(&self) -> Option<CapturedComponentPayload> {
        project(
            &self.name,
            &self.fields,
            &self.diagnostics.component_observations,
            self.start_bit,
        )
    }
}
impl EntityRecord {
    /// Project one attempt by its ordered attempt index, not its mask index.
    pub fn captured_payload(&self, attempt: usize) -> Option<CapturedComponentPayload> {
        let a = self.attempts.get(attempt)?;
        if a.status != Some(true) {
            return None;
        }
        project(
            &a.span.name,
            self.fields.get(a.field_start..a.field_end)?,
            observation_slice(
                &self.diagnostics.component_observations,
                a.observation_range,
            )?,
            a.span.start_bit,
        )
    }
}
impl KeyframeBipedProbe {
    /// Project one component by its position in the native trace's component list.
    pub fn captured_payload(&self, component: usize) -> Option<CapturedComponentPayload> {
        let c = self.components.get(component)?;
        if !c.ported {
            return None;
        }
        let [start, end] = c.field_range?;
        project(
            &c.name,
            self.fields.get(start..end)?,
            observation_slice(
                &self.diagnostics.component_observations,
                c.observation_range,
            )?,
            c.start_bit,
        )
    }
}

impl KeyframeRecord {
    /// Project a successfully read full-state component by its ordered span.
    /// This uses retained fields and observations; it never rereads source bytes
    /// or projects the unsupported component that stopped the record.
    pub fn captured_payload(&self, component: usize) -> Option<CapturedComponentPayload> {
        let c = self.components.get(component)?;
        let [start, end] = c.field_range?;
        project(
            &c.name,
            self.fields.get(start..end)?,
            observation_slice(
                &self.diagnostics.component_observations,
                c.observation_range,
            )?,
            c.start_bit,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theater::*;
    use std::{collections::BTreeMap, io::Read};

    #[test]
    fn native_captured_component_payloads() {
        #[derive(Deserialize)]
        struct Component {
            start: i64,
            payload: Option<CapturedComponentPayload>,
        }
        #[derive(Deserialize)]
        struct Row {
            hex: String,
            start: i64,
            ti: u32,
            level: u32,
            corruption: bool,
            calibrated: BTreeMap<String, usize>,
            components: Vec<Component>,
            end: usize,
        }
        #[derive(Deserialize)]
        struct Oracle {
            names: Vec<String>,
            rows: Vec<Row>,
            timer: Vec<u32>,
        }
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("../fixtures/captured-payloads-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let oracle: Oracle = serde_json::from_slice(&raw).unwrap();
        assert_eq!(oracle.rows.len(), 1024);
        assert_eq!(oracle.timer.len(), 65536);
        for (q, bits) in oracle.timer.into_iter().enumerate() {
            assert_eq!(round_seconds(q as u16).to_bits(), bits, "timer quantum {q}");
        }
        let (mut values, mut absent, mut padded) = (0, 0, 0);
        for (i, row) in oracle.rows.into_iter().enumerate() {
            let data: Vec<u8> = row
                .hex
                .as_bytes()
                .as_chunks::<2>()
                .0
                .iter()
                .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap())
                .collect();
            let registry = FilmRegistry {
                major_version: 41,
                format_version: 27,
                end_byte: 0,
                truncated: false,
                archetypes: (0..41)
                    .map(|index| FilmArchetype {
                        index,
                        components: if index == row.ti as usize {
                            oracle.names.clone()
                        } else {
                            vec![]
                        },
                        levels: vec![row.level; oracle.names.len()],
                    })
                    .collect(),
            };
            let encoding = FrameEncoding {
                keyframe_layout: Default::default(),
                keyframe_simulation_complete: None,
                native_id_low_bits: None,
                component_widths: ComponentWidthOverrides {
                    calibrated: row.calibrated,
                    stubs: Default::default(),
                },
                new_record: Default::default(),
                position_capture: None,
                ids: RecordIdLayout {
                    low_bits: 11,
                    base: 0,
                },
                mpp_widths: [9, 5],
                position: None,
                extra_fields: false,
                corruption_check: row.corruption,
            };
            let probe = traverse_keyframe_biped_at(
                &data,
                crate::theater::bits::native_address(row.start),
                &registry,
                row.ti,
                &encoding,
            )
            .unwrap();
            assert_eq!(
                serde_json::json!(probe.end_bit),
                serde_json::json!(row.end),
                "end {i}"
            );
            assert_eq!(
                probe.components.len(),
                row.components.len(),
                "components {i}"
            );
            padded += usize::from(row.end > data.len() * 8);
            for (j, c) in row.components.into_iter().enumerate() {
                assert_eq!(probe.components[j].start_bit, c.start, "start {i}/{j}");
                let payload = probe.captured_payload(j);
                assert_eq!(payload, c.payload, "payload {i}/{j}");
                if let Some(expected) = c.payload {
                    values += 1;
                    let (status, direct) = consume_component_at(
                        &data,
                        crate::theater::bits::native_address(c.start),
                        &oracle.names[j],
                        row.ti,
                        row.level,
                        &encoding,
                    );
                    assert_eq!(status, Some(true));
                    assert_eq!(direct.captured_payload(), Some(expected), "direct {i}/{j}");
                    let restored: CapturedComponentPayload =
                        serde_json::from_slice(&serde_json::to_vec(&payload.unwrap()).unwrap())
                            .unwrap();
                    assert_eq!(Some(restored), probe.captured_payload(j));
                } else {
                    absent += 1;
                }
            }
            assert_eq!(probe.captured_payload(probe.components.len()), None);
        }
        assert_eq!((values, absent), (5997, 147));
        assert!(padded > 0);
    }
}
