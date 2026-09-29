//! Native data models.
use super::*;
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NativeZoomRead {
    pub head: DecodedHeadEvent,
    pub source_bits: usize,
    pub padded_bits: usize,
    /// Native domain-four slot; check unit_reference_recorded before treating
    /// every reference bit as recorded rather than padded.
    pub slot: u32,
    /// Native level; check level_recorded before interpreting it as recorded.
    pub level: u8,
}
