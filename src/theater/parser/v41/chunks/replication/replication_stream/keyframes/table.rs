//! Deterministic reference chaining between independently established keyframe boundaries.
use crate::theater::film::chunks::replication::replication_stream::models::keyframe_chain::{
    KeyframeChainAttempt, KeyframeChainStop, KeyframeTable,
};
use crate::theater::film::{FilmRegistry, KeyframeStop};
use crate::theater::parser::v41::chunks::replication::components::cursor::ComponentCursor;
use crate::theater::parser::v41::chunks::replication::replication_stream::keyframes::record::decode_reference_keyframe_record_contextual;
use crate::theater::parser::v41::context::layout::RecordLayout;
use crate::theater::parser::v41::context::profile::DecodeProfile;
use crate::theater::parser::v41::reference_bits::reference_bits_at;

fn reference_header(data: &[u8], pos: i64) -> Option<(u32, u32)> {
    if !ComponentCursor::signed(data, pos, None).fits_source(64) {
        return None;
    }
    let id = reference_bits_at(data, pos, 32) as u32;
    if id == u32::MAX || id >> 30 == 0 || id & 0x3fff_ffff >= 8192 {
        return None;
    }
    let archetype = reference_bits_at(data, pos.checked_add(32)?, 32) as u32;
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
    encoding: &RecordLayout,
    context: Option<&DecodeProfile>,
) -> Option<KeyframeChainAttempt> {
    if context.is_none() && !encoding.keyframe_layout.valid() {
        return None;
    }
    let (end_bit, record) = if archetype == u32::MAX {
        (
            start.checked_add(
                context.map_or(encoding.keyframe_layout.header_bits as i64, |c| {
                    c.keyframe.header_bits
                }),
            )?,
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
    encoding: &RecordLayout,
    context: Option<&DecodeProfile>,
) -> KeyframeTable {
    let mut out = KeyframeTable {
        records: Vec::new(),
        stop: KeyframeChainStop::End,
    };
    let (mut pos, mut previous) = (1i64, None);

    loop {
        if out.records.len() >= 16_384 {
            out.stop = KeyframeChainStop::Budget;
            return out;
        }
        if !ComponentCursor::signed(data, pos, None).fits_source(64) {
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
        if archetype == u32::MAX {
            let header_bits = context.map_or(encoding.keyframe_layout.header_bits as i64, |c| {
                c.keyframe.header_bits
            });
            if !ComponentCursor::signed(data, pos, None).fits_source(header_bits) {
                out.stop = KeyframeChainStop::Truncated;
                return out;
            }
        }
        let Some(attempt) = read_attempt(data, registry, pos, id, archetype, encoding, context)
        else {
            out.stop = KeyframeChainStop::InvalidEncoding;
            return out;
        };
        let complete = attempt.complete();
        pos = attempt.end_bit;
        out.records.push(attempt);
        if !complete {
            out.stop = KeyframeChainStop::Desync;
            return out;
        }
        previous = Some(slot);
    }
}
