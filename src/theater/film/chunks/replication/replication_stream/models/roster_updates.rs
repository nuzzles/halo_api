//! Native data models.
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
    /// Native overflow verdict; a rejected list count is not necessarily overflow.
    pub overflow: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RosterUpdate {
    pub entries: Vec<RosterEntry>,
    pub report: RosterReport,
}

/// Native roster fields include reads in the first rejected entry.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NativeRosterRead {
    pub roster: RosterUpdate,
    pub fields: Vec<crate::theater::film::NativeSlotField>,
    /// Index into fields for each attempted entry, including a rejected entry.
    pub entry_field_starts: Vec<usize>,
}
