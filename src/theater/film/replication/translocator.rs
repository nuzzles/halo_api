//! Native data models.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TranslocatorStop {
    PositionsRead,
    Truncated,
    UnsupportedReferences,
    MissingMap,
    InvalidMap,
    UnknownRegion(u32),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TeleportPosition {
    pub start_bit: usize,
    pub end_bit: usize,
    /// None is the engine's default box (22 bits, +/-20000), not an inferred map.
    pub region: Option<u32>,
    pub axis_bits: [usize; 3],
    pub quantized: [u32; 3],
    pub world: [f32; 3],
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TranslocatorEvent {
    pub config: bool,
    pub slot: u32,
    pub generation: u8,
    pub other_reference_gates: Option<[bool; 2]>,
    pub effect_present: Option<bool>,
    pub effect: Option<u32>,
    pub from: Option<TeleportPosition>,
    pub to: Option<TeleportPosition>,
    pub end_bit: usize,
    pub stop: TranslocatorStop,
}

/// Native decoder outcome, including its synthetic zero-tail convention.
/// Slot/reference values crossing source_bits are not fully recorded identities.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NativeTranslocatorEvent {
    pub event: TranslocatorEvent,
    pub source_bits: usize,
    /// Logical reader extent beyond the payload, including skipped tail fields.
    pub padded_bits: usize,
}
