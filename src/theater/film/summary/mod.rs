//! Typed highlight feed retained after summary parsing.
use super::*;
pub mod medals;
use medals::Medal;

/// A semantic label for a guarded v41 summary candidate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SummaryHighlightKind {
    Mode,
    Death,
    Kill,
    Medal,
    Unknown,
}

/// Typed conclusion from a guarded event code. A kill/death identifies only
/// the recorded actor; no opponent or weapon is inferred from opaque bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SummaryHighlightPayload {
    Mode,
    Death,
    Kill,
    Medal(Medal),
    Unknown {
        type_code: u8,
        medal_flag: u8,
        metadata: u8,
    },
}

impl SummaryHighlightPayload {
    pub fn kind(self) -> SummaryHighlightKind {
        match self {
            Self::Mode => SummaryHighlightKind::Mode,
            Self::Death => SummaryHighlightKind::Death,
            Self::Kill => SummaryHighlightKind::Kill,
            Self::Medal(_) => SummaryHighlightKind::Medal,
            Self::Unknown { .. } => SummaryHighlightKind::Unknown,
        }
    }
}

/// A possible highlight. The marker guards identify fields but do not prove
/// sequential record boundaries or completeness against the declared count.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SummaryHighlight {
    /// Name decoded from the candidate's recorded UTF-16 field.
    pub gamertag: String,
    /// Recorded player identity, independent of roster linkage.
    pub xuid: crate::theater::film::Xuid,
    /// Film-relative candidate time in microseconds.
    pub timestamp_us: u64,
    /// Meaning inferred from the guarded type and medal fields.
    pub payload: SummaryHighlightPayload,
}

impl SummaryHighlight {
    /// Event category inferred from its typed payload.
    pub fn kind(&self) -> SummaryHighlightKind {
        self.payload.kind()
    }
}

/// The film's highlight feed. Each highlight is a labeled, guarded candidate;
/// the summary-record grammar does not yet prove completeness or boundaries.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Summary {
    /// Guarded candidates in recorded order. The list may be incomplete.
    pub highlights: Vec<SummaryHighlight>,
}
