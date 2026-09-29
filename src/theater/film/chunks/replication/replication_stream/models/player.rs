//! Recorded player slot fields and bounded read results.
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerTableSlot {
    pub film_index: usize,
    pub xuid: u64,
    pub gamertag: String,
    /// Observed 48-bit token; session semantics remain unproven.
    pub session_token: u64,
    pub bit: usize,
    pub total_bits: usize,
    pub shorts: PlayerTableShorts,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerTableShorts {
    pub tete: u32,
    pub deux: u32,
    pub repr: u32,
    pub q64: u64,
    pub f10: u32,
    pub f14: u32,
    pub f6: i32,
    pub f8: u32,
    pub f7: u32,
    pub f1: u32,
}
/// One bounded native read. Opaque spans refer to the retained source bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeSlotField {
    pub name: String,
    pub bit: usize,
    pub bits: usize,
    pub value: NativeSlotValue,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativeSlotValue {
    Scalar(u64),
    Opaque,
    /// The requested range was unavailable; the cursor did not advance.
    Unavailable,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativePlayerSlotRead {
    pub start_bit: usize,
    pub end_bit: usize,
    pub source_bits: usize,
    pub truncated: bool,
    pub slot: Option<PlayerTableSlot>,
    pub fields: Vec<NativeSlotField>,
}
