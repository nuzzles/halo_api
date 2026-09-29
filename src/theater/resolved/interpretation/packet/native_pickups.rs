//! Native data models.
use super::*;
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum NativePickupOutcome {
    EmptyList,
    BoardVehicle,
    OtherType,
    NoReference,
    NoCatalog,
    Accepted,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NativePickupRead {
    pub config: bool,
    pub more: bool,
    pub kind: u8,
    pub references: Vec<EventReference>,
    pub class: Option<u8>,
    pub catalog_id: Option<u32>,
    pub more_events: Option<bool>,
    pub source_bits: usize,
    pub end_bit: usize,
    pub padded_bits: usize,
    pub outcome: NativePickupOutcome,
}
