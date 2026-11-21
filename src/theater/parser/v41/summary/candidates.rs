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
const MARKERS: [u16; 2] = [0x2dc0, 0x25c0];
#[cfg(any(test, target_arch = "aarch64"))]
const MIDDLE_BYTES: [u8; 12] = [
    0x2e, 0x4b, 0x5b, 0x5c, 0x6e, 0x70, 0x97, 0xb7, 0xb8, 0xc0, 0xdc, 0xe0,
];

// A marker may begin at any bit offset. Its middle byte must be one of these
// twelve values, regardless of the preceding and following bits.
const fn middle_byte_mask() -> [u8; 256] {
    let mut masks = [0; 256];
    let mut marker = 0;
    while marker < MARKERS.len() {
        let mut offset = 0;
        while offset < 8 {
            masks[((MARKERS[marker] >> offset) & 0xff) as usize] |= 1 << offset;
            offset += 1;
        }
        marker += 1;
    }
    masks
}

const MIDDLE_BYTE_MASK: [u8; 256] = middle_byte_mask();

#[cfg(target_arch = "aarch64")]
#[inline]
fn possible_middle_bytes(bytes: &[u8]) -> bool {
    use std::arch::aarch64::{vceqq_u8, vdupq_n_u8, vld1q_u8, vmaxvq_u8, vorrq_u8};

    debug_assert!(bytes.len() >= 16);
    // SAFETY: the caller provides at least sixteen bytes. NEON is baseline on
    // AArch64, including Apple Silicon.
    unsafe {
        let block = vld1q_u8(bytes.as_ptr());
        let mut matches = vdupq_n_u8(0);
        for byte in MIDDLE_BYTES {
            matches = vorrq_u8(matches, vceqq_u8(block, vdupq_n_u8(byte)));
        }
        vmaxvq_u8(matches) != 0
    }
}

fn for_each_marker(payload: &[u8], mut visit: impl FnMut(usize)) {
    let mut byte: usize = 12; // The identity preceding a marker requires 96 bits.
    #[cfg(target_arch = "aarch64")]
    while byte.saturating_add(18) <= payload.len() {
        if !possible_middle_bytes(&payload[byte + 1..]) {
            byte += 16;
            continue;
        }
        let end = byte + 16;
        while byte < end {
            visit_byte(payload, byte, &mut visit);
            byte += 1;
        }
    }
    while byte.saturating_add(2) < payload.len() {
        visit_byte(payload, byte, &mut visit);
        byte += 1;
    }
}

#[inline]
fn visit_byte(payload: &[u8], byte: usize, visit: &mut impl FnMut(usize)) {
    let mut mask = MIDDLE_BYTE_MASK[payload[byte + 1] as usize];
    if mask == 0 {
        return;
    }
    let window = (u32::from(payload[byte]) << 16)
        | (u32::from(payload[byte + 1]) << 8)
        | u32::from(payload[byte + 2]);
    while mask != 0 {
        let offset = mask.trailing_zeros() as usize;
        let marker = (window >> (8 - offset)) as u16;
        if MARKERS.contains(&marker) {
            visit(byte * 8 + offset);
        }
        mask &= mask - 1;
    }
}

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
    for_each_marker(payload, |bit| {
        if let Some(event) = read_event(bits, bit) {
            events.push(event);
        }
    });
    events.sort_by_key(|event| event.identity_source.start);
    events
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn byte_filter_matches_bit_windows() {
        for (byte, mask) in MIDDLE_BYTE_MASK.iter().enumerate() {
            assert_eq!(*mask != 0, MIDDLE_BYTES.contains(&(byte as u8)));
        }
        let mut random = vec![0u8; 16_384];
        let mut value = 0x7a53_4c2bu32;
        for byte in &mut random {
            value ^= value << 13;
            value ^= value >> 17;
            value ^= value << 5;
            *byte = value as u8;
        }
        let check = |payload: &[u8]| {
            let expected = Bits(payload)
                .windows()
                .filter(|(bit, word)| *bit >= 96 && matches!(word >> 48, 0x2dc0 | 0x25c0))
                .map(|(bit, _)| bit)
                .collect::<Vec<_>>();
            let mut actual = Vec::new();
            for_each_marker(payload, |bit| actual.push(bit));
            assert_eq!(actual, expected);
        };
        check(&random);
        for offset in 0..8 {
            let mut payload = random.clone();
            for (index, marker) in MARKERS.into_iter().enumerate() {
                let position = 512 + offset + index * 64;
                for shift in 0..16 {
                    let source = position + shift;
                    let mask = 1 << (7 - source % 8);
                    payload[source / 8] &= !mask;
                    payload[source / 8] |= (((marker >> (15 - shift)) as u8) & 1) * mask;
                }
            }
            check(&payload);
        }
        for len in 0..20 {
            check(&random[..len]);
        }
    }

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
