//! Native data models.
use super::*;
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NativeWeaponDamageField {
    pub field: ComponentField,
    /// The reference skips these bits; retaining their raw value does not
    /// assign semantics to the field. Offset names follow the native reader.
    pub opaque: bool,
    /// Synthetic zero tail bits included in this field's raw value.
    pub padded_bits: usize,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NativeWeaponDamageRead {
    pub source_bits: usize,
    pub end_bit: usize,
    pub padding_bits: usize,
    /// None retains a refused fixed-prefix/type read, not an absent action.
    pub read: Option<WeaponDamageRead>,
    pub fields: Vec<NativeWeaponDamageField>,
}
