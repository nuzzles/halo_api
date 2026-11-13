//! Source-backed replay tick admission evidence.

/// Location of the first tick-message signature in the supplied replay data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReplayTickSource {
    /// Position in `Film::replication_stream`, not the manifest chunk index.
    pub chunk: usize,
    /// Position in the chunk's framed packet list.
    pub packet: usize,
    /// Bit offset from the beginning of the packet payload.
    pub payload_bit: usize,
}

/// Whether the retained replay data contains the high-frequency tick marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplayTickStatus {
    /// A framed type-0 payload contains the tick update's 27-bit signature,
    /// followed by its eight-bit counter. Marker presence alone does not prove
    /// that the enclosing entity view was decoded successfully.
    Found(ReplayTickSource),
    /// Every replay chunk was completely inflated and framed, and no framed
    /// type-0 payload contained the marker.
    Absent,
    /// Some source bytes were not inflated or framed, so absence is unproven.
    Undetermined,
}

// Update op 3; entity 123, generation 1; no baseline; component 0.
pub(crate) const TICK_SIGNATURE: u32 = 0b100_0000_1111_0110_1000_0100_0000;
const SIGNATURE_BITS: usize = 27;
const COUNTER_BITS: usize = 8;

/// First complete tick signature and its following eight-bit source counter.
pub(crate) fn tick_counter(payload: &[u8]) -> Option<(usize, u8)> {
    let bits = payload.len().checked_mul(8)?;
    if bits < SIGNATURE_BITS + COUNTER_BITS {
        return None;
    }
    let mut window = 0u32;
    for bit in 0..bits - COUNTER_BITS {
        let value = (payload[bit / 8] >> (7 - bit % 8)) & 1;
        window = ((window << 1) | u32::from(value)) & ((1 << SIGNATURE_BITS) - 1);
        if bit + 1 >= SIGNATURE_BITS && window == TICK_SIGNATURE {
            let start = bit + 1 - SIGNATURE_BITS;
            let counter_start = bit + 1;
            let mut counter = 0u8;
            for at in counter_start..counter_start + COUNTER_BITS {
                counter = (counter << 1) | ((payload[at / 8] >> (7 - at % 8)) & 1);
            }
            return Some((start, counter));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bits(s: &str) -> Vec<u8> {
        let mut out = vec![0u8; s.len().div_ceil(8)];
        for (i, bit) in s.bytes().enumerate() {
            if bit == b'1' {
                out[i / 8] |= 1 << (7 - i % 8);
            }
        }
        out
    }

    #[test]
    fn finds_source_marker_at_unaligned_bit_and_reads_counter() {
        let signature = format!("{TICK_SIGNATURE:027b}");
        let payload = bits(&format!("00{signature}00101010{}", "0".repeat(5)));
        assert_eq!(tick_counter(&payload), Some((2, 0b0010_1010)));
    }

    #[test]
    fn rejects_incomplete_signature_or_counter() {
        let signature = format!("{TICK_SIGNATURE:027b}");
        assert_eq!(tick_counter(&[0; 8]), None);
        assert_eq!(tick_counter(&bits(&signature)), None);
    }
}
