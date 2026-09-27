//! Version-41 summary events, decoded using only the decompressed footer chunks.
//!
//! The 3,667 events in the 32-film corpus share the same identity-to-tail offset.
//! This is a guarded captured layout, not a general decoder for the intervening state.

use super::{MedalAward, SourceSpan, SummaryEvent, SummaryKind, bits::Bits};

const IDENTITY_TO_TAIL_BITS: usize = 14_926;
const TAIL_BITS: usize = 60 * 8;
const END_MARKER: u64 = 0x0000_2ee0;

fn source(chunk: i32, payload_byte: usize, bit: usize, length: usize) -> SourceSpan {
    SourceSpan {
        chunk,
        payload_byte,
        bit,
        end_bit: bit + length,
    }
}

fn read_event(
    bits: Bits<'_>,
    marker_bit: usize,
    chunk: i32,
    payload_byte: usize,
) -> Option<SummaryEvent> {
    let identity_bit = marker_bit.checked_sub(64)?;
    let xuid = bits.read(identity_bit, 64)?.swap_bytes();
    if xuid == 0 {
        return None;
    }
    let tail = identity_bit.checked_add(IDENTITY_TO_TAIL_BITS)?;
    let end = tail.checked_add(TAIL_BITS)?;
    if bits.read(end, 32)? != END_MARKER {
        return None;
    }
    // The two reserved three-byte fields and boolean medal flag independently guard the tail.
    if bits.read(tail + 52 * 8, 24)? != 0 || bits.read(tail + 56 * 8, 24)? != 0 {
        return None;
    }
    let medal_flag = bits.read(tail + 55 * 8, 8)? as u8;
    if medal_flag > 1 {
        return None;
    }
    let mut units = Vec::with_capacity(16);
    let mut terminated = false;
    for i in 0..16 {
        let unit = (bits.read(tail + i * 16, 16)? as u16).swap_bytes();
        if unit == 0 {
            terminated = true;
        } else if terminated {
            return None;
        } else {
            units.push(unit);
        }
    }
    let name = String::from_utf16(&units).ok()?;
    if name.is_empty() || name.chars().any(char::is_control) {
        return None;
    }
    let type_code = bits.read(tail + 47 * 8, 8)? as u8;
    let metadata = bits.read(tail + 59 * 8, 8)? as u8;
    let time_us = bits.read(tail + 48 * 8, 32)? * 1000;
    let kind = SummaryKind::from_fields(type_code, medal_flag);
    Some(SummaryEvent {
        xuid: xuid.to_string(),
        player: None,
        name,
        time_us,
        kind,
        metadata,
        medal_flag,
        type_code: Some(type_code),
        medal: (medal_flag == 1).then(|| MedalAward::from_film_id(metadata)),
        source: Some(source(chunk, payload_byte, tail, TAIL_BITS)),
        identity_source: Some(source(chunk, payload_byte, identity_bit, 64)),
    })
}

/// Guarded summary records in source order, without timestamp sorting. Kept
/// separate so recording-only callers do not need fabricated chunk metadata.
pub(crate) fn read_summary_packet(
    payload: &[u8],
    chunk: i32,
    payload_byte: usize,
) -> Option<(u32, Vec<SummaryEvent>)> {
    let declared = u32::from_be_bytes(payload.get(..4)?.try_into().ok()?);
    let bits = Bits(payload);
    let mut events = Vec::new();
    for (bit, word) in bits.windows() {
        if bit < 32 + 64 || !matches!(word >> 48, 0x2dc0 | 0x25c0) {
            continue;
        }
        if let Some(event) = read_event(bits, bit, chunk, payload_byte) {
            events.push(event);
        }
    }
    Some((declared, events))
}
