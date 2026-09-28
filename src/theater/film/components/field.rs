//! Native data models.
/// One scalar from a component, with exact positions in the supplied payload.
/// For native widths above 64, raw is the low 64 bits, matching ReadBits.
/// Additional `<name>.discarded[N]` fields retain all source bits from the
/// discarded prefix. Synthetic zero padding is not expanded into prefix fields.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ComponentField {
    pub name: String,
    pub bit: i64,
    pub width: u64,
    pub raw: u64,
}
