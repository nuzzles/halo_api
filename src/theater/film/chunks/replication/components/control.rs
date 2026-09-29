//! Native data models.
/// Recorded action-block fields. Masks number entries from the first wire bit.
/// Weapon indices: -2 means absent; -1 is the recorded sentinel; 0..3 are indices.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NativeActionBlock {
    pub start_bit: i64,
    pub end_bit: i64,
    pub present: bool,
    pub triggers: [u8; 2],
    pub barrels: [u8; 2],
    pub weapons: [i8; 2],
}
