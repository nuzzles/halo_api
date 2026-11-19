//! Source-backed component fields.
use crate::theater::parser::wire::fields::RawBits;
/// One source-backed component field with its exact payload position and bits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentField {
    pub name: String,
    pub bit: usize,
    /// Zero is an explicit schema-selected, zero-bit read; the named field is
    /// retained with empty raw bytes and is distinct from an omitted field.
    pub width: usize,
    pub raw: RawBits,
}
