//! Native data models.
use super::{FilmReadDiagnostics, KeyframeRecord};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyframeChainStop {
    End,
    Header,
    Desync,
    Slot,
    Budget,
    InvalidEncoding,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct KeyframeChainAttempt {
    pub start_bit: i64,
    pub end_bit: i64,
    pub id: u32,
    pub archetype: u32,
    /// Absent for the no-archetype entry, whose body is not read.
    pub record: Option<KeyframeRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NativeKeyframeTable {
    pub records: Vec<KeyframeChainAttempt>,
    pub stop: KeyframeChainStop,
    pub diagnostics: FilmReadDiagnostics,
}
