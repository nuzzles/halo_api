//! Guarded v41 summary candidate search.
//!
//! The 3,667 events in the 32-film corpus share the same identity-to-tail offset.
//! This is a guarded captured layout, not a general decoder for the intervening state.

use super::models::SummaryEventRead;
use crate::theater::film::SummaryHighlightPayload;
use crate::theater::film::summary::medals::Medal;
use crate::theater::parser::v41::packet::BitRange;
use crate::theater::parser::wire::bits::Bits;

pub(super) fn candidate_payload(read: &SummaryEventRead) -> SummaryHighlightPayload {
    if read.medal_flag == 1 {
        return SummaryHighlightPayload::Medal(Medal::from_codes(read.type_code, read.metadata));
    }
    match read.type_code {
        10 => SummaryHighlightPayload::Mode,
        20 => SummaryHighlightPayload::Death,
        50 => SummaryHighlightPayload::Kill,
        _ => SummaryHighlightPayload::Unknown {
            type_code: read.type_code,
            medal_flag: read.medal_flag,
            metadata: read.metadata,
        },
    }
}

const IDENTITY_TO_TAIL_BITS: usize = 14_926;
const TAIL_BITS: usize = 60 * 8;
const END_MARKER: u64 = 0x0000_2ee0;

fn read_event(bits: Bits<'_>, marker_bit: usize) -> Option<SummaryEventRead> {
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
    let mut units = [0u16; 16];
    let mut name_units = Vec::with_capacity(16);
    let mut terminated = false;
    for (i, recorded) in units.iter_mut().enumerate() {
        let unit = (bits.read(tail + i * 16, 16)? as u16).swap_bytes();
        *recorded = unit;
        if unit == 0 {
            terminated = true;
        } else if terminated {
            return None;
        } else {
            name_units.push(unit);
        }
    }
    let name = String::from_utf16(&name_units).ok()?;
    if name.is_empty() || name.chars().any(char::is_control) {
        return None;
    }
    let type_code = bits.read(tail + 47 * 8, 8)? as u8;
    let metadata = bits.read(tail + 59 * 8, 8)? as u8;
    let timestamp_ms = bits.read(tail + 48 * 8, 32)? as u32;
    Some(SummaryEventRead {
        xuid,
        gamertag_utf16: units,
        timestamp_ms,
        metadata,
        medal_flag,
        type_code,
        source: BitRange {
            start: tail,
            end: tail + TAIL_BITS,
        },
        identity_source: BitRange {
            start: identity_bit,
            end: identity_bit + 64,
        },
    })
}

/// Candidate associations in source order. This search does not establish
/// sequential record boundaries or mutate the canonical summary packet.
pub(crate) fn read_summary_candidates(payload: &[u8]) -> Vec<SummaryEventRead> {
    let bits = Bits(payload);
    let mut events = Vec::new();
    for (bit, word) in bits.windows() {
        if bit < 32 + 64 || !matches!(word >> 48, 0x2dc0 | 0x25c0) {
            continue;
        }
        if let Some(event) = read_event(bits, bit) {
            events.push(event);
        }
    }
    events.sort_by_key(|event| event.identity_source.start);
    events
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flagged_unfamiliar_type_is_an_unknown_medal() {
        let mut read = SummaryEventRead {
            xuid: 1,
            gamertag_utf16: [0; 16],
            timestamp_ms: 0,
            metadata: 251,
            medal_flag: 1,
            type_code: 77,
            source: BitRange { start: 0, end: 0 },
            identity_source: BitRange { start: 0, end: 0 },
        };
        assert_eq!(
            candidate_payload(&read),
            SummaryHighlightPayload::Medal(Medal::Unknown {
                type_code: 77,
                medal_code: 251,
            })
        );

        read.medal_flag = 0;
        assert_eq!(
            candidate_payload(&read),
            SummaryHighlightPayload::Unknown {
                type_code: 77,
                medal_flag: 0,
                metadata: 251,
            }
        );
    }
}
