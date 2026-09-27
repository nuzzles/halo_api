//! Deterministic native chaining between independently established keyframe boundaries.
use super::{
    FrameEncoding, KeyframeRecord, KeyframeStop, decode_native_keyframe_record_contextual,
};
use crate::theater::{FilmReadDiagnostics, FilmRegistry, NativeReaderContext, native_bits_at};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyframeChainStop {
    #[serde(rename = "fin-du-payload")]
    End,
    #[serde(rename = "en-tete-invalide")]
    Header,
    #[serde(rename = "composant-non-porte")]
    Desync,
    #[serde(rename = "slot-non-croissant")]
    Slot,
    #[serde(rename = "budget-epuise")]
    Budget,
    InvalidEncoding,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyframeChainAttempt {
    pub start_bit: i64,
    pub end_bit: i64,
    pub id: u32,
    pub archetype: u32,
    /// Absent for the no-archetype entry, whose body is not read.
    pub record: Option<KeyframeRecord>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyframeChainResult {
    pub reached: bool,
    pub skipped: usize,
    pub skipped_without_archetype: usize,
    pub stop: KeyframeChainStop,
    pub attempts: Vec<KeyframeChainAttempt>,
    pub diagnostics: FilmReadDiagnostics,
}

/// Walk to exactly `want`, without scanning for headers. Like native, the loop
/// may consume 17 records but only checks the target before each attempt: reaching
/// it after record 17 reports Budget. Unlike table traversal, an ID sentinel is
/// a Header stop. Failed-body diagnostics survive without increasing skipped.
pub fn chain_keyframe_records(
    data: &[u8],
    registry: &FilmRegistry,
    from: i64,
    want: i64,
    previous_slot: i64,
    encoding: &FrameEncoding,
) -> KeyframeChainResult {
    chain_keyframe_records_contextual(data, registry, from, want, previous_slot, encoding, None)
}
pub(crate) fn chain_keyframe_records_contextual(
    data: &[u8],
    registry: &FilmRegistry,
    from: i64,
    want: i64,
    previous_slot: i64,
    encoding: &FrameEncoding,
    context: Option<&NativeReaderContext>,
) -> KeyframeChainResult {
    let mut out = KeyframeChainResult {
        reached: false,
        skipped: 0,
        skipped_without_archetype: 0,
        stop: KeyframeChainStop::End,
        attempts: Vec::new(),
        diagnostics: Default::default(),
    };
    let (mut pos, mut previous) = (from, previous_slot);
    let total = data.len().saturating_mul(8);
    while out.skipped <= 16 {
        if pos == want {
            out.reached = true;
            return out;
        }
        if pos > want || pos.wrapping_add(64) > total as i64 {
            return out;
        }
        let Some((id, archetype)) = native_header(data, pos) else {
            out.stop = KeyframeChainStop::Header;
            return out;
        };
        let slot = id & 0x3fff_ffff;
        if i64::from(slot) <= previous {
            out.stop = KeyframeChainStop::Slot;
            return out;
        }
        let Some(attempt) = read_attempt(data, registry, pos, id, archetype, encoding, context)
        else {
            out.stop = KeyframeChainStop::InvalidEncoding;
            return out;
        };
        let complete = attempt.complete();
        let end = attempt.end_bit;
        if let Some(record) = &attempt.record {
            out.diagnostics.merge(&record.diagnostics);
        }
        out.attempts.push(attempt);
        if !complete {
            out.stop = KeyframeChainStop::Desync;
            return out;
        }
        out.skipped += 1;
        out.skipped_without_archetype += usize::from(archetype == u32::MAX);
        pos = end;
        previous = i64::from(slot);
    }
    out.stop = KeyframeChainStop::Budget;
    out
}

fn native_header(data: &[u8], pos: i64) -> Option<(u32, u32)> {
    if pos < 0 || pos.wrapping_add(64) > (data.len() as i64 * 8) {
        return None;
    }
    let id = native_bits_at(data, pos, 32) as u32;
    if id == u32::MAX || id >> 30 == 0 || id & 0x3fff_ffff >= 8192 {
        return None;
    }
    let archetype = native_bits_at(data, pos.wrapping_add(32), 32) as u32;
    (archetype < 50 || archetype == u32::MAX).then_some((id, archetype))
}

impl KeyframeChainAttempt {
    fn complete(&self) -> bool {
        self.record
            .as_ref()
            .is_none_or(|record| record.stop == KeyframeStop::Complete)
    }
}

fn read_attempt(
    data: &[u8],
    registry: &FilmRegistry,
    start: i64,
    id: u32,
    archetype: u32,
    encoding: &FrameEncoding,
    context: Option<&NativeReaderContext>,
) -> Option<KeyframeChainAttempt> {
    if context.is_none() && !encoding.keyframe_layout.valid() {
        return None;
    }
    let (end_bit, record) = if archetype == u32::MAX {
        (
            start.wrapping_add(
                context.map_or(encoding.keyframe_layout.header_bits as i64, |c| {
                    c.profile.keyframe.header_bits
                }),
            ),
            None,
        )
    } else {
        let record =
            decode_native_keyframe_record_contextual(data, start, registry, encoding, context)?;
        (record.end_bit, Some(record))
    };
    Some(KeyframeChainAttempt {
        start_bit: start,
        end_bit,
        id,
        archetype,
        record,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeKeyframeTable {
    pub records: Vec<KeyframeChainAttempt>,
    pub stop: KeyframeChainStop,
    pub diagnostics: FilmReadDiagnostics,
}

/// Native sequential table traversal, starting after the one-bit prefix.
/// Header admission is bounded; body reads are zero-padded as in LevelUp.
/// Includes the failing body's fields and callbacks when traversal desynchronizes.
/// The bounded replication decoder remains available as `decode_keyframe_table`.
pub fn decode_native_keyframe_table(
    data: &[u8],
    registry: &FilmRegistry,
    encoding: &FrameEncoding,
) -> NativeKeyframeTable {
    decode_native_keyframe_table_contextual(data, registry, encoding, None)
}
pub(crate) fn decode_native_keyframe_table_contextual(
    data: &[u8],
    registry: &FilmRegistry,
    encoding: &FrameEncoding,
    context: Option<&NativeReaderContext>,
) -> NativeKeyframeTable {
    let mut out = NativeKeyframeTable {
        records: Vec::new(),
        stop: KeyframeChainStop::End,
        diagnostics: Default::default(),
    };
    let (mut pos, mut previous) = (1i64, None);

    loop {
        if out.records.len() >= 16_384 {
            out.stop = KeyframeChainStop::Budget;
            return out;
        }
        if pos.wrapping_add(64) > (data.len() as i64 * 8) {
            return out;
        }
        let Some((id, archetype)) = native_header(data, pos) else {
            // Native table traversal checks the sentinel even after a rejected
            // negative header. This read may panic, unlike chain traversal.
            if native_bits_at(data, pos, 32) != u32::MAX as u64 {
                out.stop = KeyframeChainStop::Header;
            }
            return out;
        };
        let slot = id & 0x3fff_ffff;
        if previous.is_some_and(|last| slot <= last) {
            out.stop = KeyframeChainStop::Slot;
            return out;
        }
        let Some(attempt) = read_attempt(data, registry, pos, id, archetype, encoding, context)
        else {
            out.stop = KeyframeChainStop::InvalidEncoding;
            return out;
        };
        let complete = attempt.complete();
        if let Some(record) = &attempt.record {
            out.diagnostics.merge(&record.diagnostics);
        }
        pos = attempt.end_bit;
        out.records.push(attempt);
        if !complete {
            out.stop = KeyframeChainStop::Desync;
            return out;
        }
        previous = Some(slot);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theater::{FilmArchetype, FilmComponentObservation, KeyframeLayout, RecordIdLayout};
    use std::io::Read;

    #[test]
    fn native_keyframe_chain() {
        #[derive(Deserialize)]
        struct Case {
            hex: String,
            from: i64,
            want: i64,
            previous: i64,
            names: Vec<String>,
            layout: KeyframeLayout,
            corruption: bool,
            reached: bool,
            skipped: usize,
            without: usize,
            stop: KeyframeChainStop,
            observations: Option<Vec<FilmComponentObservation>>,
        }
        let mut bytes = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("../fixtures/keyframe-chain-v41.json.zlib")[..],
        )
        .read_to_end(&mut bytes)
        .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(cases.len(), 512);
        let (mut reached, mut budget, mut header, mut slot, mut failed_callbacks, mut exact_budget) =
            (0, 0, 0, 0, 0, 0);
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
                archetypes: (0..6)
                    .map(|index| FilmArchetype {
                        index,
                        components: if index == 5 {
                            c.names.clone()
                        } else {
                            Vec::new()
                        },
                        levels: if index == 5 {
                            vec![0; c.names.len()]
                        } else {
                            Vec::new()
                        },
                    })
                    .collect(),
            };
            let encoding = FrameEncoding {
                keyframe_layout: c.layout,
                keyframe_simulation_complete: None,
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
            let result =
                chain_keyframe_records(&data, &registry, c.from, c.want, c.previous, &encoding);
            assert_eq!(
                (
                    result.reached,
                    result.skipped,
                    result.skipped_without_archetype,
                    result.stop
                ),
                (c.reached, c.skipped, c.without, c.stop),
                "result {i}"
            );
            assert_eq!(
                result.diagnostics.component_observations,
                c.observations.unwrap_or_default(),
                "observations {i}"
            );
            assert_eq!(
                serde_json::from_value::<KeyframeChainResult>(
                    serde_json::to_value(&result).unwrap()
                )
                .unwrap(),
                result
            );
            reached += usize::from(result.reached);
            header += usize::from(result.stop == KeyframeChainStop::Header);
            slot += usize::from(result.stop == KeyframeChainStop::Slot);
            if result.stop == KeyframeChainStop::Budget {
                budget += 1;
                assert_eq!(result.skipped, 17);
                exact_budget += usize::from(result.attempts.last().unwrap().end_bit == c.want);
            }
            failed_callbacks += usize::from(
                result.stop == KeyframeChainStop::Desync
                    && !result.diagnostics.component_observations.is_empty(),
            );
        }
        assert_eq!(
            (reached, budget, header, slot, failed_callbacks),
            (97, 7, 186, 46, 101)
        );
        assert!(exact_budget > 0);
    }
}

#[cfg(test)]
mod native_table_tests {
    use super::*;
    use crate::theater::{FilmArchetype, FilmComponentObservation, KeyframeLayout, RecordIdLayout};
    use std::io::Read;

    #[test]
    fn native_keyframe_table_padded_reads() {
        #[derive(Deserialize)]
        struct Expected {
            start: usize,
            end: usize,
            archetype: u32,
            slot: u32,
            generation: u32,
            desync: i64,
        }
        #[derive(Deserialize)]
        struct Case {
            hex: String,
            names: Vec<String>,
            layout: KeyframeLayout,
            corruption: bool,
            stop: KeyframeChainStop,
            records: Vec<Expected>,
            observations: Option<Vec<FilmComponentObservation>>,
        }
        let mut bytes = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("../fixtures/keyframe-native-table-v41.json.zlib")[..],
        )
        .read_to_end(&mut bytes)
        .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(cases.len(), 512);
        let (mut overruns, mut failed_callbacks, mut sentinel_entries) = (0, 0, 0);
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
                archetypes: (0..6)
                    .map(|index| FilmArchetype {
                        index,
                        components: if index == 5 {
                            c.names.clone()
                        } else {
                            Vec::new()
                        },
                        levels: if index == 5 {
                            vec![0; c.names.len()]
                        } else {
                            Vec::new()
                        },
                    })
                    .collect(),
            };
            let encoding = FrameEncoding {
                keyframe_layout: c.layout,
                keyframe_simulation_complete: None,
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
            let result = decode_native_keyframe_table(&data, &registry, &encoding);
            assert_eq!(result.stop, c.stop, "stop {i}");
            assert_eq!(result.records.len(), c.records.len(), "count {i}");
            for (actual, expected) in result.records.iter().zip(&c.records) {
                assert_eq!(
                    serde_json::json!((
                        actual.start_bit,
                        actual.end_bit,
                        actual.archetype,
                        actual.id & 0x3fff_ffff,
                        actual.id >> 30
                    )),
                    serde_json::json!((
                        expected.start,
                        expected.end,
                        expected.archetype,
                        expected.slot,
                        expected.generation
                    )),
                    "record {i}"
                );
                let desync = match actual.record.as_ref().map(|r| &r.stop) {
                    None | Some(KeyframeStop::Complete) => -1,
                    Some(KeyframeStop::UnsupportedComponent { index, .. }) => *index as i64,
                    other => panic!("unexpected stop {other:?} in {i}"),
                };
                assert_eq!(desync, expected.desync, "desync {i}");
                sentinel_entries += usize::from(actual.archetype == u32::MAX);
            }
            assert_eq!(
                result.diagnostics.component_observations,
                c.observations.unwrap_or_default(),
                "observations {i}"
            );
            assert_eq!(
                serde_json::from_value::<NativeKeyframeTable>(
                    serde_json::to_value(&result).unwrap()
                )
                .unwrap(),
                result
            );
            overruns += usize::from(
                result
                    .records
                    .iter()
                    .any(|r| r.end_bit > (data.len() * 8) as i64),
            );
            failed_callbacks += usize::from(
                result.stop == KeyframeChainStop::Desync
                    && !result.diagnostics.component_observations.is_empty(),
            );
        }
        assert_eq!(
            (overruns, failed_callbacks, sentinel_entries),
            (6, 172, 766)
        );
    }
}
