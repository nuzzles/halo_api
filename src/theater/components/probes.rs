//! Legacy read-only grammar probes from LevelUp `probe_export.go`.
use super::Cursor;
use super::{ComponentField, FrameEncoding, Reader, defaults, widths};
use crate::theater::{FilmReadDiagnostics, FilmRegistry, NativeUnitReference};
use serde::{Deserialize, Serialize};

/// Native `ConsumeComponentAt`: one direct dispatch under a supplied reader
/// context. Traversal calibrated/stub widths and corruption guards do not apply.
/// Padded reads, fields and callbacks survive an unsupported result; `None`
/// status denotes invalid encoding or a primitive the Rust reader cannot perform.
/// An unspecified simulation-completion policy uses the native standalone
/// default (false); map-aware profile factories supply their explicit policy.
pub fn consume_component_at(
    data: &[u8],
    start: usize,
    name: &str,
    archetype: u32,
    level: u32,
    encoding: &FrameEncoding,
) -> (Option<bool>, super::DecodedComponent) {
    let map = encoding.position_capture.as_ref().map(|c| c.map());
    let mut reader = Reader {
        native_widths: None,
        width_error: None,
        live_observer: None,
        live_grammar: None,
        position_capture: encoding
            .position_capture
            .as_ref()
            .zip(map.as_ref())
            .map(|(c, map)| c.reader(map, 0)),
        position_start: 0,
        position_slot: 0,
        position_fallback: false,
        movement_slot: Some(0),
        references: Vec::new(),
        diagnostics: Default::default(),
        cursor: Cursor::new_padded(data, start),
        fields: Vec::new(),
        position_encoding: encoding.position.as_ref(),
    };
    // This dispatch never reads record IDs or a default-state MPP block.
    let status = if encoding
        .position
        .as_ref()
        .is_none_or(super::PositionEncoding::valid)
    {
        widths::read_component(
            &mut reader,
            name,
            level,
            archetype,
            None,
            (encoding.keyframe_simulation_complete.unwrap_or(false), None),
        )
    } else {
        None
    };
    (
        status,
        super::DecodedComponent {
            name: name.into(),
            start_bit: start as i64,
            end_bit: reader.cursor.position,
            fields: reader.fields,
            references: reader.references,
            diagnostics: reader.diagnostics,
        },
    )
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyframeBipedProbeComponent {
    /// Ordered retained fields, independent of overlapping source bit ranges.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub field_range: Option<[usize; 2]>,
    /// Ordered diagnostic observations emitted by this attempt. Absent in older serialized data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observation_range: Option<[usize; 2]>,
    pub index: usize,
    pub name: String,
    pub start_bit: i64,
    /// Includes the corruption guard for successfully consumed components.
    pub end_bit: i64,
    pub ported: bool,
    /// Native no-variant sentinel is u32::MAX. A calibrated skip instead leaves
    /// the native result's zero-initialized Variant field at zero.
    #[serde(default = "no_variant")]
    pub variant: u32,
}

fn no_variant() -> u32 {
    u32::MAX
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyframeBipedProbe {
    pub archetype: u32,
    pub start_bit: i64,
    pub end_bit: i64,
    pub mask: u64,
    /// Native trace end is zero for a missing archetype, even after consuming defaults.
    pub trace_end_bit: i64,
    pub desync_at: Option<usize>,
    pub components: Vec<KeyframeBipedProbeComponent>,
    pub fields: Vec<ComponentField>,
    pub references: Vec<NativeUnitReference>,
    pub diagnostics: FilmReadDiagnostics,
    /// Last directly decoded death component, including Mort=false. Calibrated
    /// skips do not replace an earlier observation.
    #[serde(default)]
    pub dead: Option<crate::theater::ObjectDeadState>,
}

/// Reproduce `TraverseKeyframeBipedAt` at a nonnegative state offset. Only TI35
/// consumes defaults. The following gate is read but does not suppress the delta
/// mask. This is a legacy probe, not the full-state keyframe record grammar.
/// Reads beyond the payload use the native zero-padding convention.
/// Native EntityTrace.DefaultBits and Gate stay at their zero values in this
/// probe; the actual default reads and gate are retained in `fields` instead.
/// Unspecified simulation completion uses the native standalone false default.
pub fn traverse_keyframe_biped_at(
    data: &[u8],
    state_bit: usize,
    registry: &FilmRegistry,
    archetype: u32,
    encoding: &FrameEncoding,
) -> Option<KeyframeBipedProbe> {
    // This probe has no record-ID header; only TI35 reads default-state MPP.
    if encoding.position.as_ref().is_some_and(|p| !p.valid())
        || (archetype == 35 && !encoding.mpp_widths.iter().all(|w| (1..=32).contains(w)))
    {
        return None;
    }
    let capture_map = encoding.position_capture.as_ref().map(|c| c.map());
    let mut r = Reader {
        native_widths: None,
        width_error: None,
        live_observer: None,
        live_grammar: None,
        position_capture: encoding
            .position_capture
            .as_ref()
            .zip(capture_map.as_ref())
            .map(|(c, map)| c.reader(map, 0)),
        position_start: 0,
        position_slot: 0,
        position_fallback: false,
        movement_slot: Some(0),
        references: Vec::new(),
        diagnostics: Default::default(),
        cursor: Cursor::new_padded(data, state_bit),
        fields: Vec::new(),
        position_encoding: encoding.position.as_ref(),
    };
    if archetype == 35 {
        defaults::state(&mut r, archetype, encoding.mpp_widths)?;
        // The pinned native biped default tail has a constant width of zero.
    }
    r.bit("keyframe.component_gate")?;
    let mut result = KeyframeBipedProbe {
        archetype,
        start_bit: state_bit as i64,
        end_bit: 0,
        trace_end_bit: 0,
        mask: 0,
        dead: None,
        desync_at: Some(0),
        components: Vec::new(),
        fields: Vec::new(),
        references: Vec::new(),
        diagnostics: Default::default(),
    };
    if let Some(arch) = registry.archetype(archetype as usize) {
        result.mask = r.mask()?;
        result.desync_at = None;
        for (index, name) in arch.components.iter().enumerate() {
            // Native traverseComponentLoop wraps the mask index at 64.
            if result.mask & (1 << (index & 63)) == 0 {
                continue;
            }
            let start_bit = r.cursor.position;
            let field_start = r.fields.len();
            let observation_start = r.diagnostics.component_observations.len();
            let ported = widths::read_component(
                &mut r,
                name,
                arch.levels.get(index).copied().unwrap_or(0),
                archetype,
                Some(&encoding.component_widths),
                (encoding.keyframe_simulation_complete.unwrap_or(false), None),
            )?;
            let fields = &r.fields[field_start..];
            let calibrated = encoding.component_widths.calibrated.contains_key(name);
            let variant = if calibrated {
                0
            } else if name == "weapon-state-type-info" {
                fields
                    .iter()
                    .find(|f| f.name == "variant")
                    .map_or(u32::MAX, |f| f.raw as u32)
            } else {
                u32::MAX
            };
            if !calibrated
                && name == "object-dead-state-component"
                && let Some(dead) = crate::theater::ObjectDeadState::from_fields(fields)
            {
                result.dead = Some(dead);
            }
            if ported && encoding.corruption_check {
                r.gate("component.corruption_check", 32, true)?;
            }
            result.components.push(KeyframeBipedProbeComponent {
                field_range: Some([field_start, r.fields.len()]),
                observation_range: Some([
                    observation_start,
                    r.diagnostics.component_observations.len(),
                ]),
                index,
                name: name.clone(),
                start_bit,
                end_bit: r.cursor.position,
                ported,
                variant,
            });
            if !ported {
                result.desync_at = Some(index);
                break;
            }
        }
        result.trace_end_bit = r.cursor.position;
    }
    result.end_bit = r.cursor.position;
    result.fields = r.fields;
    result.references = r.references;
    result.diagnostics = r.diagnostics;
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theater::{FilmArchetype, FilmComponentObservation, RecordIdLayout};
    use std::io::Read;

    #[test]
    fn native_keyframe_probe_trace_values() {
        #[derive(Deserialize)]
        struct Component {
            start: usize,
            index: usize,
            name: String,
            variant: u32,
            ported: bool,
        }
        #[derive(Deserialize)]
        struct Row {
            hex: String,
            start: usize,
            ti: u32,
            level: u32,
            corruption: bool,
            calibrated: std::collections::BTreeMap<String, usize>,
            stubs: std::collections::BTreeMap<String, usize>,
            components: Vec<Component>,
            end: usize,
            trace_end: usize,
            mask: u64,
            desync: i64,
            dead: Option<crate::theater::ObjectDeadState>,
            encoding: crate::theater::PositionEncoding,
            simulation: bool,
        }
        #[derive(Deserialize)]
        struct Oracle {
            names: Vec<String>,
            rows: Vec<Row>,
        }
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("../fixtures/probe-trace-values-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let oracle: Oracle = serde_json::from_slice(&raw).unwrap();
        assert_eq!(oracle.rows.len(), 1024);
        let (mut padded, mut failures, mut variants, mut skipped, mut alive) = (0, 0, 0, 0, 0);
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
                keyframe_simulation_complete: row.simulation.then_some(true),
                native_id_low_bits: None,
                component_widths: crate::theater::ComponentWidthOverrides {
                    calibrated: row.calibrated,
                    stubs: row.stubs,
                },
                new_record: Default::default(),
                position_capture: None,
                ids: RecordIdLayout {
                    low_bits: 255,
                    base: u32::MAX,
                },
                mpp_widths: if row.ti == 35 { [9, 5] } else { [0, 0] },
                position: Some(row.encoding),
                extra_fields: false,
                corruption_check: row.corruption,
            };
            let actual = traverse_keyframe_biped_at(
                &data,
                crate::theater::bits::native_address(row.start),
                &registry,
                row.ti,
                &encoding,
            )
            .unwrap();
            assert_eq!(
                serde_json::json!((
                    actual.end_bit,
                    actual.trace_end_bit,
                    actual.mask,
                    actual.desync_at.map_or(-1, |n| n as i64)
                )),
                serde_json::json!((row.end, row.trace_end, row.mask, row.desync)),
                "trace {i}"
            );
            assert_eq!(actual.dead, row.dead, "death {i}");
            assert_eq!(actual.components.len(), row.components.len());
            for (a, e) in actual.components.iter().zip(row.components) {
                assert_eq!(
                    serde_json::json!((a.index, a.name.as_str(), a.start_bit, a.variant, a.ported)),
                    serde_json::json!((e.index, e.name.as_str(), e.start, e.variant, e.ported)),
                    "component {i}/{}",
                    e.index
                );
                if encoding.component_widths.calibrated.contains_key(&a.name) {
                    skipped += 1;
                    assert_eq!(a.variant, 0);
                } else if a.variant != u32::MAX {
                    variants += 1;
                }
            }
            alive += usize::from(actual.dead.as_ref().is_some_and(|d| !d.mort));
            failures += usize::from(row.desync >= 0);
            padded += usize::from(row.end > data.len() * 8);
            let restored: KeyframeBipedProbe =
                serde_json::from_slice(&serde_json::to_vec(&actual).unwrap()).unwrap();
            assert_eq!(actual, restored);
        }
        assert!(padded > 0 && failures > 0 && variants > 0 && skipped > 0 && alive > 0);
    }

    #[test]
    fn native_standalone_component_probe() {
        #[derive(Deserialize)]
        struct Case {
            name: String,
            archetype: u32,
            level: u32,
            start: usize,
            hex: String,
            encoding: crate::theater::PositionEncoding,
            capture: crate::theater::PositionCaptureEncoding,
            simulation: bool,
            corruption: bool,
            end: usize,
            ported: bool,
            observations: Vec<FilmComponentObservation>,
        }
        let mut bytes = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("../fixtures/component-probe-v41.json.zlib")[..],
        )
        .read_to_end(&mut bytes)
        .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(cases.len(), 1024);
        let (mut padded, mut refused_simulation, mut observed) = (0, 0, 0);
        for (i, c) in cases.into_iter().enumerate() {
            let data: Vec<u8> = c
                .hex
                .as_bytes()
                .as_chunks::<2>()
                .0
                .iter()
                .map(|s| u8::from_str_radix(std::str::from_utf8(s).unwrap(), 16).unwrap())
                .collect();
            let encoding = FrameEncoding {
                keyframe_layout: Default::default(),
                native_id_low_bits: None,
                keyframe_simulation_complete: if !c.simulation && i % 3 == 1 {
                    None
                } else {
                    Some(c.simulation)
                },
                component_widths: crate::theater::ComponentWidthOverrides {
                    calibrated: [(c.name.clone(), 77)].into(),
                    stubs: [(c.name.clone(), 91)].into(),
                },
                new_record: Default::default(),
                position_capture: Some(c.capture),
                // Deliberately invalid for frame traversal: neither setting is
                // part of this standalone component's reader contract.
                ids: RecordIdLayout {
                    low_bits: 255,
                    base: u32::MAX,
                },
                mpp_widths: [0, 0],
                position: Some(c.encoding),
                extra_fields: false,
                corruption_check: c.corruption,
            };
            let (status, result) =
                consume_component_at(&data, c.start, &c.name, c.archetype, c.level, &encoding);
            assert_eq!(status, Some(c.ported), "status {i}");
            assert_eq!(
                serde_json::json!(result.end_bit),
                serde_json::json!(c.end),
                "end {i}"
            );
            assert_eq!(
                result.diagnostics.component_observations, c.observations,
                "callbacks {i}"
            );
            assert!(
                !result
                    .fields
                    .iter()
                    .any(|f| f.name.starts_with("component.calibrated")
                        || f.name.starts_with("component.stub")
                        || f.name == "component.corruption_check")
            );
            padded += usize::from(c.end > data.len() * 8);
            refused_simulation += usize::from(c.name == "simulation-state-component" && !c.ported);
            observed += c.observations.len();
        }
        assert!(padded > 0 && refused_simulation > 0 && observed > 0);
    }

    #[test]
    fn native_keyframe_biped_probe() {
        #[derive(Deserialize)]
        struct Case {
            hex: String,
            start: usize,
            archetype: u32,
            names: Vec<String>,
            corruption: bool,
            mask: u64,
            end: usize,
            trace_end: usize,
            desync: i64,
            components: Vec<KeyframeBipedProbeComponent>,
            observations: Option<Vec<FilmComponentObservation>>,
        }
        let mut bytes = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("../fixtures/keyframe-biped-probe-v41.json.zlib")[..],
        )
        .read_to_end(&mut bytes)
        .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(cases.len(), 512);
        let (mut padded, mut missing, mut failed, mut wrapped) = (0, 0, 0, 0);
        for (i, c) in cases.into_iter().enumerate() {
            let data: Vec<u8> = c
                .hex
                .as_bytes()
                .as_chunks::<2>()
                .0
                .iter()
                .map(|s| u8::from_str_radix(std::str::from_utf8(s).unwrap(), 16).unwrap())
                .collect();
            let registry = FilmRegistry {
                major_version: 41,
                format_version: 27,
                end_byte: 0,
                truncated: false,
                archetypes: (0..71)
                    .map(|index| FilmArchetype {
                        index,
                        components: if index == c.archetype as usize {
                            c.names.clone()
                        } else {
                            vec![]
                        },
                        levels: vec![0; c.names.len()],
                    })
                    .collect(),
            };
            let encoding = FrameEncoding {
                keyframe_layout: Default::default(),
                keyframe_simulation_complete: Some(true),
                native_id_low_bits: None,
                component_widths: Default::default(),
                new_record: Default::default(),
                position_capture: None,
                ids: RecordIdLayout {
                    low_bits: 11,
                    base: 0,
                },
                mpp_widths: [9, 5],
                position: None,
                extra_fields: false,
                corruption_check: c.corruption,
            };
            let actual =
                traverse_keyframe_biped_at(&data, c.start, &registry, c.archetype, &encoding)
                    .unwrap();
            assert_eq!(
                serde_json::json!((
                    actual.end_bit,
                    actual.trace_end_bit,
                    actual.mask,
                    actual.desync_at.map_or(-1, |n| n as i64)
                )),
                serde_json::json!((c.end, c.trace_end, c.mask, c.desync)),
                "trace {i}"
            );
            let mut native_components = actual.components.clone();
            let mut previous_fields = 0;
            let mut previous_observations = 0;
            for component in &mut native_components {
                let [start, end] = component.field_range.take().unwrap();
                assert!(previous_fields <= start && start <= end && end <= actual.fields.len());
                previous_fields = end;
                let [start, end] = component.observation_range.take().unwrap();
                assert!(
                    previous_observations <= start
                        && start <= end
                        && end <= actual.diagnostics.component_observations.len()
                );
                previous_observations = end;
            }
            // Native trace structs have no Rust provenance indices. Compare every
            // native field separately from the validated additional metadata.
            assert_eq!(native_components, c.components, "components {i}");
            assert_eq!(
                actual.diagnostics.component_observations,
                c.observations.unwrap_or_default(),
                "observations {i}"
            );
            assert_eq!(
                serde_json::from_value::<KeyframeBipedProbe>(
                    serde_json::to_value(&actual).unwrap()
                )
                .unwrap(),
                actual
            );
            padded += usize::from(actual.end_bit > (data.len() * 8) as i64);
            missing += usize::from(c.archetype == 99);
            failed += usize::from(actual.components.iter().any(|v| !v.ported));
            wrapped += usize::from(actual.components.iter().any(|v| v.index >= 64));
        }
        assert!(padded > 0 && missing > 0 && failed > 0 && wrapped > 0);
    }
}
