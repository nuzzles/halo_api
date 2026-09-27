use serde::{Deserialize, Serialize};

/// Structural errors are separate from unsupported record forms in diagnostics.
#[derive(Debug, thiserror::Error)]
pub(crate) enum DecodeError {
    /// The film version has no checked decoder.
    #[error("unsupported Theater film major version {0}; supported: 41")]
    UnsupportedVersion(i32),
    /// A native signed option cannot be represented by this target's indexes.
    #[error(transparent)]
    KillOption(#[from] super::KillDecodeOptionError),
    /// A framed type-1 datum body failed native size validation.
    #[error(transparent)]
    Datums(#[from] super::DatumTableError),
    /// A supported capture's independent guards disagree.
    #[error("inconsistent Theater data: {0}")]
    Inconsistent(String),
}

/// Original location in a decompressed chunk; bits are MSB-first and half-open.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceSpan {
    /// Chunk index.
    pub chunk: i32,
    /// Packet payload byte offset in that chunk (zero for non-packet data).
    pub payload_byte: usize,
    /// First bit relative to the payload.
    pub bit: usize,
    /// First bit after the checked window; does not imply a complete record.
    pub end_bit: usize,
}

/// A summary event decoded independently from replication records.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    pub medal: Option<super::MedalAward>,
    /// Exact 60-byte event tail; intervening identity state is not decoded.
    #[serde(default)]
    pub source: Option<SourceSpan>,
    /// Recorded XUID field, independently of the roster and gamertag.
    #[serde(default)]
    pub identity_source: Option<SourceSpan>,
}
/// Summary kinds; no pairing of kills/deaths is implied.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

/// One checked byte-aligned replication packet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FilmPacket {
    pub chunk_index: i32,
    pub packet_type: u16,
    pub byte_2: u8,
    pub byte_3: u8,
    pub payload_offset: usize,
    pub payload_size: usize,
    pub timestamp_us: u64,
}
