//! Native type-2 table: 8,191 entries, including vacant slots. A vacant
//! archetype skips the body; no identifier word terminates the table.
use super::record::decode_keyframe_record_contextual;
use crate::theater::parser::bits::Cursor;
use crate::theater::parser::packet::*;
use crate::theater::parser::v41::chunks::registry::source::*;
use crate::theater::parser::v41::chunks::replication::source::*;
use crate::theater::parser::v41::context::V41DecodeConfig;

const SLOT_COUNT: usize = 0x1fff;
const HEADER_BITS: usize = 108;

pub(crate) fn decode(
    data: &[u8],
    registry: &FilmRegistry,
    config: &V41DecodeConfig,
    vehicle_gate: impl Fn(u32) -> Option<bool>,
) -> KeyframeTable {
    let mut r = Cursor::new(data, 0).unwrap();
    let configuration = r.read(1).map(|v| v != 0);
    let mut layout = config.record_layout();
    layout.category_references = configuration.unwrap_or(false);
    let mut slots = KeyframeSlotsRead {
        configuration,
        headers: Vec::new(),
        end_bit: r.position,
        tail: BitRange {
            start: r.position,
            end: data.len() * 8,
        },
    };
    let mut out = KeyframeTable {
        records: Vec::new(),
        stop: KeyframeChainStop::Truncated,
        slots: None,
    };
    if config.profile.keyframe.header_bits != HEADER_BITS as i64
        || config.profile.keyframe.size_word_bits != 32
    {
        out.stop = KeyframeChainStop::InvalidEncoding;
    } else if configuration.is_some() {
        for _ in 0..SLOT_COUNT {
            let start = r.position;
            if data.len() * 8 - start < HEADER_BITS {
                break;
            }
            let header = KeyframeSlotHeader {
                start_bit: start,
                id: r.read(32).unwrap() as u32,
                archetype: r.read(32).unwrap() as u32,
                word: r.read(32).unwrap() as u32,
                flags: r.read(4).unwrap() as u8,
                byte: r.read(8).unwrap() as u8,
            };
            slots.headers.push(header);
            if header.archetype == u32::MAX {
                continue;
            }
            if header.archetype >= 50 || registry.archetype(header.archetype as usize).is_none() {
                out.stop = KeyframeChainStop::Header;
                break;
            }
            let mut record_layout = std::borrow::Cow::Borrowed(&layout);
            if header.archetype == 40
                && let Some(gate) = vehicle_gate(header.id)
            {
                record_layout.to_mut().snapshot_vehicle_type_gate = Some(gate);
            }
            let Some(record) = decode_keyframe_record_contextual(
                data,
                start,
                registry,
                &record_layout,
                Some(&config.profile),
            ) else {
                out.stop = KeyframeChainStop::InvalidEncoding;
                break;
            };
            let complete = record.stop == KeyframeStop::Complete;
            r.position = record.end_bit as usize;
            out.records.push(KeyframeChainAttempt {
                start_bit: start as i64,
                end_bit: record.end_bit,
                id: header.id,
                archetype: header.archetype,
                record: Some(record),
            });
            if !complete {
                out.stop = KeyframeChainStop::Desync;
                break;
            }
        }
        if slots.headers.len() == SLOT_COUNT
            && out.records.iter().all(|a| {
                a.record
                    .as_ref()
                    .is_none_or(|r| r.stop == KeyframeStop::Complete)
            })
            && out.stop == KeyframeChainStop::Truncated
        {
            out.stop = if data.len() * 8 - r.position < 8 {
                KeyframeChainStop::End
            } else {
                KeyframeChainStop::TrailingData
            };
        }
    }
    slots.end_bit = r.position;
    slots.tail.start = r.position;
    out.slots = Some(slots);
    out
}

#[cfg(test)]
mod tests;
