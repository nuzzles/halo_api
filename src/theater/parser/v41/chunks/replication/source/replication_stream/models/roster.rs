//! Count-driven session and bot metadata, without inferred player ownership.
use crate::theater::parser::packet::BitRange;
use crate::theater::parser::v41::chunks::replication::source::components::field::RawBits;

/// Type 8 or 12 read. The enclosing packet retains the original timestamp/bytes.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PlayerMetadataRead {
    pub format_version: Option<u32>,
    pub build: Option<String>,
    pub personalization_bytes: Option<usize>,
    pub count: PlayerMetadataField,
    /// Includes the first partial entry; no successor search follows a stop.
    pub entries: Vec<PlayerMetadataEntry>,
    pub end_bit: usize,
    pub stop: PlayerMetadataStop,
    /// Exact remaining payload range. Type 8 does not validate its contents or
    /// length. Type 12 requires fewer than eight bits, of any value.
    pub tail: BitRange,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PlayerMetadataEntry {
    pub start_bit: usize,
    pub end_bit: usize,
    pub fields: Vec<PlayerMetadataField>,
    pub stop: PlayerMetadataStop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PlayerMetadataField {
    pub kind: PlayerMetadataFieldKind,
    pub bit: usize,
    pub width: usize,
    /// Exact MSB-first bits. None denotes the first unavailable field.
    pub raw: Option<RawBits>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PlayerMetadataFieldKind {
    Count,
    /// Type 8, format >=21: one means no following five-bit option.
    OptionAbsent,
    /// Role is not established; this is not a proven player slot.
    Option,
    /// Byte-copy representation: reverse bytes of the MSB word for the XUID.
    Identity,
    AbsoluteIndex,
    Slot,
    BotId,
    MaskLengthMinusOne,
    Mask,
    ByteCount,
    ByteList,
    WordCount,
    WordList,
    Block104,
    /// Original UTF-16BE units, including a consumed NUL, or all 16 units.
    GamertagUnit(u8),
    Block16,
    Representation,
    /// Byte-copy key at sub+0xcb8, retained even if it differs from Identity.
    JoinKey,
    /// Six raw words of widths 10,14,6,8,7,1. The six-bit native value is raw-1.
    Short(u8),
    Personalization,
    /// Kept byte by byte; internal layout, including team, depends on build.
    Block44Byte(u8),
}

impl PlayerMetadataFieldKind {
    pub(crate) fn opaque(self) -> bool {
        matches!(
            self,
            Self::Mask
                | Self::ByteList
                | Self::WordList
                | Self::Block104
                | Self::Block16
                | Self::Personalization
                | Self::Block44Byte(_)
        )
    }
}
