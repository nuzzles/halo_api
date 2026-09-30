//! Guarded v41 summary candidate search, separate from canonical decoding.
//!
//! The 3,667 events in the 32-film corpus share the same identity-to-tail offset.
//! This is a guarded captured layout, not a general decoder for the intervening state.

use crate::theater::{film::BitRange, parser::bits::Bits};

/// Bounded field reads from a guarded summary candidate.
/// Marker matching and identity-to-tail association are derived, not a proven record boundary.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SummaryEventRead {
    pub xuid: u64,
    /// All 16 recorded UTF-16 code units, including the terminator and padding.
    /// String interpretation belongs to the resolved model.
    pub gamertag_utf16: [u16; 16],
    /// Recorded film-relative timestamp in milliseconds.
    pub timestamp_ms: u32,
    /// Raw metadata byte (medal code for medal events).
    pub metadata: u8,
    /// Raw medal flag.
    pub medal_flag: u8,
    pub type_code: u8,
    /// Exact 60-byte event tail; intervening identity state is not decoded.
    pub source: BitRange,
    /// Recorded XUID field, independently of the roster and gamertag.
    pub identity_source: BitRange,
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
    fn captured_layout_read_preserves_the_full_utf16_slot() {
        fn put_bytes(output: &mut [u8], start_bit: usize, bytes: &[u8]) {
            for (index, byte) in bytes.iter().enumerate() {
                for shift in 0..8 {
                    if byte & (1 << (7 - shift)) != 0 {
                        let bit = start_bit + index * 8 + shift;
                        output[bit / 8] |= 1 << (7 - bit % 8);
                    }
                }
            }
        }
        // Intentionally unaligned identity and tail fields exercise bit ranges.
        let identity = 37;
        let tail = identity + IDENTITY_TO_TAIL_BITS;
        let end = tail + TAIL_BITS;
        let mut payload = vec![0; (end + 32).div_ceil(8)];
        put_bytes(&mut payload, identity, &42u64.to_le_bytes());
        put_bytes(&mut payload, identity + 64, &[0x2d, 0xc0]);
        let mut units = [0u16; 16];
        for (slot, unit) in units.iter_mut().zip("Film".encode_utf16()) {
            *slot = unit;
        }
        for (index, unit) in units.iter().enumerate() {
            put_bytes(&mut payload, tail + index * 16, &unit.to_le_bytes());
        }
        put_bytes(&mut payload, tail + 47 * 8, &[50]);
        put_bytes(&mut payload, tail + 48 * 8, &1234u32.to_be_bytes());
        put_bytes(&mut payload, end, &(END_MARKER as u32).to_be_bytes());
        let read = read_event(Bits(&payload), identity + 64).unwrap();
        assert_eq!(read.gamertag_utf16, units);
        assert_eq!(read.gamertag_utf16.len(), 16);
        assert_eq!(read.timestamp_ms, 1234);
        assert_eq!(
            read.identity_source,
            BitRange {
                start: identity,
                end: identity + 64
            }
        );
        assert_eq!(read.source, BitRange { start: tail, end });
        payload[end / 8] ^= 1 << (7 - end % 8);
        assert!(read_event(Bits(&payload), identity + 64).is_none());
    }
}
