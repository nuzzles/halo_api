//! Deterministic reference chaining between independently established keyframe boundaries.
use super::{FrameEncoding, KeyframeStop, decode_reference_keyframe_record_contextual};
pub(crate) use crate::theater::film::chunks::replication::replication_stream::models::keyframe_chain::{
    KeyframeChainAttempt, KeyframeChainStop, KeyframeTable,
};
use crate::theater::parser::v41::{FilmRegistry, ReaderContext, reference_bits_at};

fn reference_header(data: &[u8], pos: i64) -> Option<(u32, u32)> {
    if pos < 0 || pos.wrapping_add(64) > (data.len() as i64 * 8) {
        return None;
    }
    let id = reference_bits_at(data, pos, 32) as u32;
    if id == u32::MAX || id >> 30 == 0 || id & 0x3fff_ffff >= 8192 {
        return None;
    }
    let archetype = reference_bits_at(data, pos.wrapping_add(32), 32) as u32;
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
    context: Option<&ReaderContext>,
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
            decode_reference_keyframe_record_contextual(data, start, registry, encoding, context)?;
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

pub(crate) fn decode_reference_keyframe_table_contextual(
    data: &[u8],
    registry: &FilmRegistry,
    encoding: &FrameEncoding,
    context: Option<&ReaderContext>,
) -> KeyframeTable {
    let mut out = KeyframeTable {
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
        let Some((id, archetype)) = reference_header(data, pos) else {
            // Reference table traversal checks the sentinel even after a rejected
            // negative header. This read may panic, unlike chain traversal.
            if reference_bits_at(data, pos, 32) != u32::MAX as u64 {
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
