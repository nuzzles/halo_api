//! Native data models.
use crate::theater::film::*;
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum KeyframeChainStop {
    #[serde(rename = "fin-du-payload")]
    End,
    #[serde(rename = "en-tete-invalide")]
    Header,
    #[serde(rename = "composant-non-porte")]
    Desync,
    #[serde(rename = "slot-non-croissant")]
    Slot,
    #[serde(rename = "budget-epuise")]
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
