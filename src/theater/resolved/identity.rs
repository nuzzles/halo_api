//! Identity support for ResolvedFilm.
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

use super::*;

impl ResolvedFilm<'_> {
    /// Bootstrap identity by recorded film index. Dynamic roster evidence stays
    /// accessible through packet records; this does not guess entity ownership.
    pub fn player(&self, film_index: usize) -> Option<&PlayerTableSlot> {
        self.interpretations
            .player_table
            .as_ref()?
            .slots
            .get(*self.player_slot_by_film_index.get(&film_index)?)
    }
}

/// Recorded build and per-type versions, independent of external match metadata.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FilmIdentity {
    /// Header format version, also retained when publishing identity alone.
    #[serde(default)]
    pub format_version: u32,
    #[serde(default)]
    pub registry_blocks: usize,
    #[serde(default)]
    pub registry_fingerprint: u64,
    #[serde(default)]
    pub registry_named_slots: usize,
    pub version: String,
    pub build: String,
    pub flavor: String,
    pub build_id: u32,
    pub changelist: u32,
    pub type_versions: Vec<u32>,
    pub corruption_checks: bool,
    pub match_start_unix: u32,
    /// Byte location of the build string in the bootstrap chunk.
    pub build_offset: usize,
    /// First bit after the identification section; later sections are bit packed.
    pub body_bit: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PlayerTableReport {
    pub build: String,
    pub perso_bytes: usize,
    pub profile_delta_bits: i64,
    pub film_delta_bits: i64,
    pub film_delta_gaps: usize,
    pub calibration_agrees: bool,
    pub gaps_agree: usize,
    pub gaps_vacant: usize,
    pub gaps_hidden: usize,
    pub gaps_contradict: usize,
    pub occupied: usize,
    pub vacant: usize,
    pub head_vacant: usize,
    pub interleaved_vacant: bool,
    pub first_record_bit: usize,
    pub candidates_scanned: usize,
    pub candidates_real: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PlayerTableError {
    UnknownBuild,
    Truncated,
    NotFound,
}

/// A failed attempt retains diagnostics but never publishes partial slots.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PlayerTable {
    pub slots: Vec<PlayerTableSlot>,
    pub report: PlayerTableReport,
    pub error: Option<PlayerTableError>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum IdentityValue {
    /// Bits in wire order. No endian conversion or signed interpretation.
    Scalar(u64),
    /// Fixed-width byte field, including bytes after a string terminator.
    Bytes(Vec<u8>),
    /// Known extent, unknown interpretation; retained bootstrap holds the bytes.
    Opaque,
    /// Entire requested extent was not present. No zero padding is supplied.
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct IdentityField {
    pub name: String,
    /// Bootstrap-relative source coordinates, including the one-bit shift.
    pub bit: usize,
    pub bits: usize,
    pub value: IdentityValue,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct IdentityRead {
    /// Complete legacy projection only; partial reads live in fields.
    pub identity: Option<FilmIdentity>,
    pub error: Option<String>,
    pub source_bits: usize,
    /// Selected by the reference's bounded HI_ search, not a proven boundary.
    pub build_anchor_byte: Option<usize>,
    pub fields: Vec<IdentityField>,
}
