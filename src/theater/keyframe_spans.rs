//! Native keyframe span and vehicle extra-block measurements.
//! Spans can cover skipped records; an extra block is not proof of occupancy.
use super::{bits::Bits, native_sort, recover_keyframe_anchors};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
pub const VEHICLE_KEYFRAME_BLOCK_BITS: i64 = 89;
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct KeyframeRecordSpan {
    pub slot: i64,
    #[serde(rename = "TI")]
    pub archetype: i64,
    #[serde(rename = "Gen")]
    pub generation: i64,
    pub bit_start: i64,
    pub bit_end: i64,
    pub length_bits: i64,
    pub slot_gap: i64,
    #[serde(rename = "TimestampUS")]
    pub timestamp_us: u64,
    pub chunk: i64,
    pub packet_index: i64,
}
pub fn keyframe_record_spans(payload: &[u8]) -> Vec<KeyframeRecordSpan> {
    spans_from_anchors(recover_keyframe_anchors(payload), payload.len() * 8)
}

pub(super) fn spans_from_anchors(
    mut anchors: Vec<super::RecoveredKeyframeAnchor>,
    payload_bits: usize,
) -> Vec<KeyframeRecordSpan> {
    native_sort::sort_by(&mut anchors, |a, b| a.bit.cmp(&b.bit));
    anchors
        .iter()
        .enumerate()
        .map(|(i, a)| {
            let end = anchors.get(i + 1).map_or(payload_bits, |a| a.bit);
            let slot = i64::from(a.id & 0x3fff_ffff);
            KeyframeRecordSpan {
                slot,
                archetype: i64::from(a.archetype),
                generation: i64::from(a.id >> 30),
                bit_start: a.bit as i64,
                bit_end: end as i64,
                length_bits: (end - a.bit) as i64,
                slot_gap: anchors
                    .get(i + 1)
                    .map_or(0, |a| i64::from(a.id & 0x3fff_ffff) - slot),
                ..Default::default()
            }
        })
        .collect()
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct VehicleKeyframeState {
    pub slot: i64,
    #[serde(rename = "TimestampUS")]
    pub timestamp_us: u64,
    pub chunk: i64,
    pub packet_index: i64,
    pub length_bits: i64,
    pub slot_gap: i64,
    pub measurable: bool,
    pub baseline_bits: i64,
    pub excess_bits: i64,
    pub extra_block: bool,
}
pub fn vehicle_keyframe_states(spans: &[KeyframeRecordSpan]) -> Vec<VehicleKeyframeState> {
    let mut base = BTreeMap::<i64, i64>::new();
    for s in spans
        .iter()
        .filter(|s| s.archetype == 40 && s.slot_gap == 1)
    {
        base.entry(s.slot)
            .and_modify(|b| *b = (*b).min(s.length_bits))
            .or_insert(s.length_bits);
    }
    let mut out: Vec<_> = spans
        .iter()
        .filter(|s| s.archetype == 40)
        .map(|s| {
            let mut state = VehicleKeyframeState {
                slot: s.slot,
                timestamp_us: s.timestamp_us,
                chunk: s.chunk,
                packet_index: s.packet_index,
                length_bits: s.length_bits,
                slot_gap: s.slot_gap,
                measurable: s.slot_gap == 1,
                ..Default::default()
            };
            if state.measurable
                && let Some(&b) = base.get(&s.slot)
            {
                state.baseline_bits = b;
                state.excess_bits = s.length_bits.wrapping_sub(b);
                state.extra_block = state.excess_bits >= VEHICLE_KEYFRAME_BLOCK_BITS;
            }
            state
        })
        .collect();
    out.sort_by_key(|s| (s.timestamp_us, s.slot));
    out
}
#[derive(Debug, Clone, Copy)]
pub struct KeyframeRecordBits<'a> {
    pub payload: &'a [u8],
    pub start_bit: usize,
    pub end_bit: usize,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct KeyframeBlockInsertion {
    pub insert_bit: usize,
    pub block_bits: usize,
    pub agree: usize,
    pub compared: usize,
    pub agree_head: usize,
    pub agree_tail: usize,
    pub valid: bool,
}
/// Find the first best insertion offset. Out-of-buffer bits use native zero-tail
/// reads. Records with nonpositive lengths or no positive insertion return invalid.
pub fn find_keyframe_block_insertion(
    long: KeyframeRecordBits<'_>,
    short: KeyframeRecordBits<'_>,
) -> KeyframeBlockInsertion {
    let lo = long.end_bit.saturating_sub(long.start_bit);
    let lf = short.end_bit.saturating_sub(short.start_bit);
    if lf == 0 || lo <= lf {
        return KeyframeBlockInsertion::default();
    }
    let bit = |r: KeyframeRecordBits<'_>, at: usize| Bits(r.payload).read(at, 1).unwrap_or(0);
    let mut tail = (0..lf)
        .filter(|&i| bit(long, long.end_bit - lf + i) == bit(short, short.start_bit + i))
        .count();
    let mut out = KeyframeBlockInsertion {
        block_bits: lo - lf,
        compared: lf,
        agree: tail,
        agree_head: tail,
        valid: true,
        ..Default::default()
    };
    let mut head = 0;
    for i in 0..lf {
        head += usize::from(bit(long, long.start_bit + i) == bit(short, short.start_bit + i));
        tail -= usize::from(bit(long, long.end_bit - lf + i) == bit(short, short.start_bit + i));
        if head + tail > out.agree {
            out.agree = head + tail;
            out.insert_bit = i + 1;
        }
    }
    out.agree_tail = head;
    out
}

impl super::LegacyFilm {
    /// Retain span measurements from the existing recovery pass. Payload size
    /// bounds the final measurement only; it does not delimit a decoded record.
    pub(super) fn retain_keyframe_measurements(&mut self) {
        let Some(stream) = &self.replication else {
            return;
        };
        let mut indices = BTreeMap::<i32, i64>::new();
        let mut spans = Vec::new();
        for packet in &stream.packets {
            let index = indices.entry(packet.source.chunk_index).or_default();
            if let Some(recovery) = &packet.keyframe_recovery {
                let mut measured = spans_from_anchors(
                    recovery.records.iter().map(|r| r.anchor.clone()).collect(),
                    packet.source.payload_size * 8,
                );
                for span in &mut measured {
                    span.timestamp_us = packet.source.timestamp_us;
                    span.chunk = i64::from(packet.source.chunk_index);
                    span.packet_index = *index;
                }
                spans.extend(measured);
            }
            *index += 1;
        }
        self.vehicle_keyframe_states = Some(vehicle_keyframe_states(&spans));
        self.keyframe_record_spans = Some(spans);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn native_keyframe_span_and_vehicle_measurements() {
        #[derive(Deserialize)]
        struct Case {
            hex: String,
            spans: Vec<KeyframeRecordSpan>,
            input: Vec<KeyframeRecordSpan>,
            states: Vec<VehicleKeyframeState>,
            long: String,
            short: String,
            ranges: [usize; 4],
            insertion: KeyframeBlockInsertion,
        }
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/keyframe-spans-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(cases.len(), 512);
        let bytes = |s: &str| {
            (0..s.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
                .collect::<Vec<_>>()
        };
        for (i, c) in cases.into_iter().enumerate() {
            assert_eq!(keyframe_record_spans(&bytes(&c.hex)), c.spans, "spans {i}");
            assert_eq!(vehicle_keyframe_states(&c.input), c.states, "states {i}");
            let long = bytes(&c.long);
            let short = bytes(&c.short);
            let [ls, le, ss, se] = c.ranges;
            assert_eq!(
                find_keyframe_block_insertion(
                    KeyframeRecordBits {
                        payload: &long,
                        start_bit: ls,
                        end_bit: le
                    },
                    KeyframeRecordBits {
                        payload: &short,
                        start_bit: ss,
                        end_bit: se
                    }
                ),
                c.insertion,
                "insertion {i}"
            );
        }
    }
}
