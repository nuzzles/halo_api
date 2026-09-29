//! Recorded summary data.
use super::*;
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SummaryEvents {
    pub chunks: Vec<SummaryChunk>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SummaryChunk {
    pub source: FilmChunk,
    pub source_position: usize,
    pub data: Vec<u8>,
    pub packets: Vec<NativeFilmPacket>,
    /// Start of the unwalked suffix. Nested packet bit offsets address `data`.
    pub packet_walk_end_byte: usize,
}
impl SummaryChunk {
    pub fn payload(&self, packet: &NativeFilmPacket) -> Option<&[u8]> {
        let header = packet.header;
        self.data
            .get(header.payload_offset..header.payload_offset.checked_add(header.payload_size)?)
    }
}
impl SummaryEvents {
    /// Recorded summary entries in source order, without inferred player linkage.
    pub fn events(&self) -> impl Iterator<Item = &SummaryEvent> {
        self.chunks
            .iter()
            .flat_map(|c| &c.packets)
            .flat_map(|p| match &p.body {
                NativeFilmPacketBody::Summary { events, .. } => events.as_slice(),
                _ => &[],
            })
    }
}

pub mod medals;
pub use medals::{FilmMedalDefinition, MedalAward};

/// A summary event decoded independently from replication records.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SummaryEvent {
    /// Recorded XUID as decimal text.
    pub xuid: String,
    /// Bound roster index, if available.
    pub player: Option<u8>,
    /// Recorded gamertag.
    pub name: String,
    /// Film-relative time in microseconds.
    pub time_us: u64,
    /// Summary kind, including unrecognized numeric codes.
    pub kind: SummaryKind,
    /// Raw metadata byte (medal code for medal events).
    pub metadata: u8,
    /// Raw medal flag.
    pub medal_flag: u8,
    /// Original type byte; for medals this is the recorded sorting weight.
    #[serde(default)]
    pub type_code: Option<u8>,
    /// Named medal and its distinct stats API identifier, when known.
    #[serde(default)]
    pub medal: Option<crate::theater::film::MedalAward>,
    /// Exact 60-byte event tail; intervening identity state is not decoded.
    #[serde(default)]
    pub source: Option<SourceSpan>,
    /// Recorded XUID field, independently of the roster and gamertag.
    #[serde(default)]
    pub identity_source: Option<SourceSpan>,
}

/// Summary kinds; no pairing of kills/deaths is implied.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SummaryKind {
    /// Mode marker.
    Mode,
    /// Death.
    Death,
    /// Kill.
    Kill,
    /// Medal; metadata retains its film code.
    Medal,
    /// Unrecognized code.
    Other(u8),
}

impl SummaryKind {
    /// Interpret the event-type byte and medal flag. Byte decoders separately
    /// validate which flag values their supported recording layout accepts.
    pub const fn from_fields(code: u8, medal_flag: u8) -> Self {
        if medal_flag != 0 {
            return Self::Medal;
        }
        match code {
            10 => Self::Mode,
            20 => Self::Death,
            50 => Self::Kill,
            other => Self::Other(other),
        }
    }
}
