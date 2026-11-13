//! Parser resource policy, independent of the recorded film format.

/// Internal decoding limits used by `Film::parse`.
/// These are policies, not wire maxima.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParseLimits {
    /// Maximum packet envelopes admitted in each replication or summary chunk.
    pub max_packets_per_chunk: usize,
    /// Maximum control records admitted in one frame. A following zero
    /// terminator can close an exactly-full list; this is not a wire maximum.
    pub max_control_records: usize,
}

impl Default for ParseLimits {
    fn default() -> Self {
        Self {
            max_packets_per_chunk: 1 << 16,
            max_control_records: 1 << 16,
        }
    }
}
