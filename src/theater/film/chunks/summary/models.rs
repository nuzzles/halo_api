use super::*;

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
    pub medal: Option<MedalAward>,
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
