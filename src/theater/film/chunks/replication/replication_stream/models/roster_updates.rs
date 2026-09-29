//! Reference data models.
use crate::theater::film::*;
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RosterEntry {
    pub player: PlayerTableSlot,
    pub identity: u64,
    /// -1 when the option gate is open or absent in the supplied format.
    pub option: i32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RosterReport {
    pub announced: u32,
    pub entries: usize,
    pub refused: usize,
    pub bits_read: usize,
    pub bits_available: usize,
    pub remaining: usize,
    /// Reference overflow verdict; a rejected list count is not necessarily overflow.
    pub overflow: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RosterUpdate {
    pub entries: Vec<RosterEntry>,
    pub report: RosterReport,
}
