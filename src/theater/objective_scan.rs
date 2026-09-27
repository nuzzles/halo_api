//! Native managed-objective scan diagnostics (ti=11).
//! The reference does not validate these candidate readings as gameplay progress.
//! Delta and keyframe provenance and their acceptance counters remain separate.
use super::*;
use crate::clients::hi::models::FilmChunkData;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ObjectiveRead {
    pub slot: u32,
    #[serde(rename = "TimestampUS")]
    pub timestamp_us: u64,
    /// Registry-name form of the native six-valued ObjectiveField enum.
    pub field: NativeObjectiveField,
    pub value: u64,
    pub value_b: u64,
    pub has_b: bool,
    pub chained: bool,
    pub from_keyframe: bool,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ObjectiveScan {
    /// Ordered scan attempts; these are diagnostic evidence, not accepted objective facts.
    /// Old exports without a trace cannot prove that no reads occurred.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attempts: Vec<ObjectiveScanAttempt>,
    pub reads: Vec<ObjectiveRead>,
    pub slots: usize,
    pub records: usize,
    pub walked: usize,
    pub broken: usize,
    pub chained: usize,
    pub key_records: usize,
    pub key_walked: usize,
    pub key_broken: usize,
    pub key_chained: usize,
    pub key_closed: usize,
    pub key_bounded: usize,
}
/// One source-attributed delta component or full-state keyframe attempt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjectiveScanAttempt {
    pub source: FilmPacket,
    pub packet_index: usize,
    pub record_start_bit: usize,
    pub slot: u32,
    pub data: ObjectiveAttemptData,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ObjectiveAttemptData {
    Delta {
        component_index: usize,
        status: Option<bool>,
        in_bounds: bool,
        component: DecodedComponent,
    },
    Keyframe {
        /// None means the low-level decoder could not return a record.
        record: Option<KeyframeRecord>,
    },
}
/// Native scans return their accumulated counters even on setup failure.
#[derive(Debug, thiserror::Error)]
#[error("{error}")]
pub struct ObjectiveScanFailure {
    pub scan: Box<ObjectiveScan>,
    #[source]
    pub error: DecodeError,
}
impl NativeObjectiveField {
    /// Ordinal used by the pinned native ObjectiveField enum.
    pub fn native_code(self) -> u8 {
        match self {
            Self::Timers => 0,
            Self::ObjectReference => 1,
            Self::Type => 2,
            Self::Progress => 3,
            Self::RequiredProgress => 4,
            Self::State => 5,
        }
    }
}
fn observation(
    o: &FilmComponentObservation,
    slot: u32,
    timestamp_us: u64,
    from_keyframe: bool,
) -> Option<ObjectiveRead> {
    let FilmComponentObservation::Objective { field, values } = o else {
        return None;
    };
    Some(ObjectiveRead {
        slot,
        timestamp_us,
        field: *field,
        value: *values.first()?,
        value_b: values.get(1).copied().unwrap_or(0),
        has_b: values.len() > 1,
        chained: false,
        from_keyframe,
    })
}
impl ObjectiveScan {
    fn delta(
        &mut self,
        pay: &[u8],
        band: &BTreeSet<u32>,
        (source, packet_index): (FilmPacket, usize),
        arch: &FilmArchetype,
        encoding: &FrameEncoding,
    ) {
        use super::navpoint_radial_scan::{header_at, indices, read};
        let ts = source.timestamp_us;
        let total = pay.len() * 8;
        let Some(limit) = total.checked_sub(27) else {
            return;
        };
        let mut p = 0;
        while p <= limit {
            let start = p;
            p += 1;
            if read(pay, start, 1) != 1 || read(pay, start + 16, 2) != 0 {
                continue;
            }
            let slot = read(pay, start + 1, 13) as u32;
            if !band.contains(&slot) {
                continue;
            }
            let count = read(pay, start + 18, 3) as usize;
            if count == 0 {
                continue;
            }
            let Some(ids) = indices(pay, start + 21, count) else {
                continue;
            };
            if ids.iter().any(|&i| i >= arch.components.len()) {
                continue;
            }
            let after = start + 21 + 6 * count;
            self.records += 1;
            let first = self.reads.len();
            let mut at = after;
            let mut done = true;
            for id in ids {
                let name = &arch.components[id];
                if name.is_empty() || at > total {
                    done = false;
                    break;
                }
                let (ported, component) = consume_component_at(
                    pay,
                    at,
                    name,
                    11,
                    arch.levels.get(id).copied().unwrap_or(0),
                    encoding,
                );
                self.attempts.push(ObjectiveScanAttempt {
                    source,
                    packet_index,
                    record_start_bit: start,
                    slot,
                    data: ObjectiveAttemptData::Delta {
                        component_index: id,
                        status: ported,
                        in_bounds: component.end_bit >= 0 && component.end_bit <= total as i64,
                        component: component.clone(),
                    },
                });
                if ported != Some(true) || component.end_bit < 0 || component.end_bit > total as i64
                {
                    done = false;
                    break;
                }
                at = usize::try_from(component.end_bit).expect("in-bounds component endpoint");
                // The native direct walk retains only the last callback of a component.
                if let Some(r) = component
                    .diagnostics
                    .component_observations
                    .iter()
                    .rev()
                    .find_map(|o| observation(o, slot, ts, false))
                {
                    self.reads.push(r)
                }
            }
            if done {
                self.walked += 1;
                if header_at(pay, at) {
                    self.chained += 1;
                    for r in &mut self.reads[first..] {
                        r.chained = true;
                    }
                }
            } else {
                self.broken += 1;
            }
            // Earlier component publications survive a later delta failure.
            p = after + 1;
        }
    }
    fn keyframe(
        &mut self,
        pay: &[u8],
        (source, packet_index): (FilmPacket, usize),
        registry: &FilmRegistry,
        encoding: &FrameEncoding,
    ) {
        let ts = source.timestamp_us;
        let mut anchors = recover_keyframe_anchors(pay);
        super::native_sort::sort_by(&mut anchors, |a, b| a.bit.cmp(&b.bit));
        for (i, a) in anchors.iter().enumerate() {
            if a.archetype != 11 {
                continue;
            }
            self.key_records += 1;
            let want = anchors.get(i + 1).map(|a| a.bit);
            self.key_bounded += usize::from(want.is_some());
            let record = decode_native_keyframe_record(pay, a.bit, registry, encoding);
            self.attempts.push(ObjectiveScanAttempt {
                source,
                packet_index,
                record_start_bit: a.bit,
                slot: a.id & 0x3fff_ffff,
                data: ObjectiveAttemptData::Keyframe {
                    record: record.clone(),
                },
            });
            let Some(record) = record else {
                self.key_broken += 1;
                continue;
            };
            // Unlike deltas, a broken keyframe publishes none of its readings.
            if record.stop != KeyframeStop::Complete
                || record.end_bit < 0
                || record.end_bit > (pay.len() * 8) as i64
            {
                self.key_broken += 1;
                continue;
            }
            self.key_walked += 1;
            self.key_closed += usize::from(want.map(|bit| bit as i64) == Some(record.end_bit));
            let chained = super::recovery::anchor(
                super::bits::Bits(pay),
                usize::try_from(record.end_bit).expect("in-bounds keyframe endpoint"),
                None,
            )
            .is_some();
            self.key_chained += usize::from(chained);
            for mut r in record
                .diagnostics
                .component_observations
                .iter()
                .filter_map(|o| observation(o, a.id & 0x3fff_ffff, ts, true))
            {
                r.chained = chained;
                self.reads.push(r);
            }
        }
    }
}
/// Retain native objective-field candidates and every scan denominator in chunk,
/// packet and publication order. This is an explicit diagnostic scan; its values
/// must not be presented as validated objective captures or gameplay events.
/// The native field enum is represented by its stable registry names.
pub fn scan_objectives(
    chunks: &[FilmChunkData],
    registry: &FilmRegistry,
    encoding: &FrameEncoding,
) -> Result<ObjectiveScan, ObjectiveScanFailure> {
    if registry.major_version != 41 {
        return Err(ObjectiveScanFailure {
            scan: Box::default(),
            error: DecodeError::UnsupportedVersion(registry.major_version as i32),
        });
    }
    let prefix =
        super::fire_events::native_chunk_prefix(chunks).map_err(|error| ObjectiveScanFailure {
            scan: Box::default(),
            error,
        })?;
    let census = scan_world_object_keyframes(chunks, 11);
    let band: BTreeSet<_> = census
        .seen_us
        .iter()
        .map(|s| s.slot)
        .filter(|s| census.band.contains(s))
        .collect();
    if band.is_empty() {
        return Err(ObjectiveScanFailure {
            scan: Box::default(),
            error: DecodeError::Missing("observed managed-objective slots"),
        });
    }
    let mut out = ObjectiveScan {
        slots: band.len(),
        ..Default::default()
    };
    let arch = registry.archetype(11).ok_or_else(|| ObjectiveScanFailure {
        scan: Box::new(out.clone()),
        error: DecodeError::Missing("managed-objective archetype 11"),
    })?;
    for chunk in prefix {
        for (index, p) in super::fire_events::native_chunk_packets(chunk)
            .into_iter()
            .enumerate()
        {
            let pay = &chunk.data[p.payload_offset..p.payload_offset + p.payload_size];
            match p.packet_type {
                0 => out.delta(pay, &band, (p, index), arch, encoding),
                2 => out.keyframe(pay, (p, index), registry, encoding),
                _ => {}
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clients::hi::models::FilmChunk;
    use std::io::Read;
    #[test]
    fn native_objective_scan_reports() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/objective-scan-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(rows.len(), 256);
        for (i, row) in rows.into_iter().enumerate() {
            let names: Vec<String> = serde_json::from_value(row["names"].clone()).unwrap();
            let n = if row["missing_arch"].as_bool().unwrap() {
                11
            } else {
                13
            };
            let registry = FilmRegistry {
                archetypes: (0..n)
                    .map(|index| FilmArchetype {
                        index,
                        components: if index == 11 { names.clone() } else { vec![] },
                        levels: if index == 11 {
                            vec![0; names.len()]
                        } else {
                            vec![]
                        },
                    })
                    .collect(),
                major_version: 41,
                format_version: 27,
                end_byte: 0,
                truncated: false,
            };
            let encoding = FrameEncoding {
                keyframe_layout: Default::default(),
                keyframe_simulation_complete: Some(row["simulation"].as_bool().unwrap()),
                native_id_low_bits: None,
                component_widths: serde_json::from_value(row["widths"].clone()).unwrap(),
                new_record: Default::default(),
                position_capture: None,
                ids: RecordIdLayout {
                    low_bits: 13,
                    base: 0,
                },
                mpp_widths: [9, 5],
                position: None,
                extra_fields: false,
                corruption_check: row["check"].as_bool().unwrap(),
            };
            let chunks: Vec<_> = row["chunks"]
                .as_array()
                .unwrap()
                .iter()
                .map(|c| {
                    let hex = c["hex"].as_str().unwrap();
                    let data: Vec<_> = (0..hex.len())
                        .step_by(2)
                        .map(|k| u8::from_str_radix(&hex[k..k + 2], 16).unwrap())
                        .collect();
                    FilmChunkData {
                        metadata: FilmChunk {
                            index: c["index"].as_i64().unwrap() as i32,
                            chunk_type: 0,
                            start_time_offset_ms: 0,
                            duration_ms: 0,
                            size: data.len() as i64,
                            file_relative_path: String::new(),
                        },
                        data,
                    }
                })
                .collect();
            let result = scan_objectives(&chunks, &registry, &encoding);
            assert_eq!(
                result.is_err(),
                row["error"].as_bool().unwrap(),
                "status {i}"
            );
            let out = match result {
                Ok(out) => out,
                Err(failure) => *failure.scan,
            };
            let expected: ObjectiveScan = serde_json::from_value(row["output"].clone()).unwrap();
            let mut published = out.clone();
            published.attempts.clear();
            assert_eq!(published, expected, "case {i}");
            let mut emissions = Vec::new();
            for a in &out.attempts {
                let (keyframe, diagnostics) = match &a.data {
                    ObjectiveAttemptData::Delta {
                        component,
                        in_bounds,
                        ..
                    } => {
                        assert_eq!(
                            *in_bounds,
                            component.end_bit <= (a.source.payload_size * 8) as i64
                        );
                        assert!(component.start_bit >= a.record_start_bit as i64);
                        (false, &component.diagnostics)
                    }
                    ObjectiveAttemptData::Keyframe {
                        record: Some(record),
                    } => {
                        assert_eq!(
                            serde_json::json!(record.start_bit),
                            serde_json::json!(a.record_start_bit)
                        );
                        (true, &record.diagnostics)
                    }
                    ObjectiveAttemptData::Keyframe { record: None } => continue,
                };
                let chunk = chunks
                    .iter()
                    .find(|c| c.metadata.index == a.source.chunk_index)
                    .unwrap();
                assert_eq!(
                    super::super::fire_events::native_chunk_packets(chunk)[a.packet_index],
                    a.source
                );
                for o in &diagnostics.component_observations {
                    if let FilmComponentObservation::Objective { field, values } = o {
                        emissions.push(serde_json::json!({"chunk":a.source.chunk_index,"packet":a.packet_index,"keyframe":keyframe,"field":field.native_code(),"values":values}));
                    }
                }
            }
            assert_eq!(
                serde_json::json!(emissions),
                row["emissions"],
                "all objective callbacks {i}"
            );
            for (read, native) in out
                .reads
                .iter()
                .zip(row["output"]["Reads"].as_array().unwrap())
            {
                assert_eq!(
                    u64::from(read.field.native_code()),
                    native["FieldCode"].as_u64().unwrap(),
                    "native field {i}"
                );
            }
            assert_eq!(
                serde_json::from_value::<ObjectiveScan>(serde_json::to_value(&out).unwrap())
                    .unwrap(),
                out
            );
        }
    }
}
