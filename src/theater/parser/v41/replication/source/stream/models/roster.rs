//! Count-driven session and bot metadata, without inferred player ownership.
use crate::theater::parser::v41::packet::{BitRange, SourceRegion};

/// Type 8 or 12 read. The enclosing packet retains the original timestamp/bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerMetadataRead {
    pub build: Option<String>,
    /// Includes the first partial entry; no successor search follows a stop.
    pub entries: Vec<PlayerMetadataEntry>,
    /// Coalesced source coverage, without copies of uninterpreted values.
    pub regions: Vec<SourceRegion>,
    pub end_bit: usize,
    pub stop: PlayerMetadataStop,
    /// Type 8 does not validate its tail. Type 12 requires fewer than eight bits.
    pub tail: BitRange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerMetadataEntry {
    pub start_bit: usize,
    pub end_bit: usize,
    pub values: Option<(PlayerMetadataIdentity, PlayerMetadataBody)>,
    pub stop: PlayerMetadataStop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerMetadataIdentity {
    /// Byte-copy representation; reverse bytes of the MSB word for the XUID.
    Session(u64),
    Bot {
        absolute_index: u32,
        slot: u32,
        id: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerMetadataBody {
    pub gamertag: [u16; 16],
    pub gamertag_len: usize,
    /// Payload-relative bits. Decoded only after the entire packet is admitted.
    pub personalization: BitRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerMetadataStop {
    Complete,
    Truncated,
    UnsupportedFormat,
    UnknownPersonalization,
    /// A reader admission limit, not a claim about the full native domain.
    CountLimit {
        kind: PlayerMetadataFieldKind,
        count: u64,
        limit: u64,
    },
    TrailingData,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerMetadataFieldKind {
    Count,
    ByteCount,
    WordCount,
}
