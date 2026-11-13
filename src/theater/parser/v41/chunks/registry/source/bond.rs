//! Lossless Bond Compact Binary v2 syntax. Numeric wire types are retained;
//! strings and floating-point values are never normalized into text or floats.
use crate::theater::parser::packet::BitRange;
use crate::theater::parser::v41::chunks::replication::source::components::field::RawBits;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BondBits {
    pub bit: usize,
    pub width: usize,
    /// None denotes the first unavailable read, not an implied zero.
    pub raw: Option<RawBits>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BondVarint {
    /// Original bytes, including continuation bits and redundant encodings.
    pub bytes: Vec<BondBits>,
    /// Unsigned encoded value, before any signed zigzag interpretation.
    pub value: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BondRead {
    pub root: BondValue,
    pub stop: BondStop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum BondStop {
    Complete,
    Truncated,
    /// A read would cross its declared enclosing structure, despite later bytes.
    StructureBoundary,
    InvalidType(u8),
    IntegerOverflow,
    InvalidLength,
    DepthLimit,
    ValueLimit,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BondValue {
    pub wire_type: u8,
    /// Successfully consumed bits. The first unavailable field may extend past it.
    pub source: BitRange,
    pub body: BondValueBody,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum BondValueBody {
    Fixed(BondBits),
    Integer(BondVarint),
    String {
        length: BondVarint,
        data: Option<BondBits>,
    },
    Struct {
        length: BondVarint,
        fields: Vec<BondField>,
    },
    Sequence {
        header: BondBits,
        count_encoding: Option<BondVarint>,
        count: Option<u32>,
        /// Fixed-width scalar arrays retain one exact contiguous payload.
        packed: Option<BondBits>,
        values: Vec<BondValue>,
    },
    Map {
        key_type: BondBits,
        value_type: Option<BondBits>,
        count: Option<BondVarint>,
        entries: Vec<BondMapEntry>,
    },
    /// No body grammar was admitted (unknown type or an explicit resource limit).
    Unread,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BondField {
    /// Header byte and any extended ID bytes, in recorded order.
    pub header: Vec<BondBits>,
    pub id: Option<u16>,
    pub wire_type: Option<u8>,
    /// Types 0 and 1 are retained terminators/base terminators, without a value.
    pub value: Option<BondValue>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BondMapEntry {
    pub key: BondValue,
    pub value: Option<BondValue>,
}
