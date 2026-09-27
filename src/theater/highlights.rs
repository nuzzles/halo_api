//! Client highlight views adapted from the current summary decoder. No byte parsing.

use super::{
    DecodeError, FilmEventCounts, FilmMedal, SummaryEvent, SummaryKind, ValidatedSummaryEventReport,
};

/// Established name for the shared summary event kind.
pub type FilmEventKind = SummaryKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilmEvent {
    pub xuid: u64,
    pub gamertag: String,
    pub timestamp_ms: u32,
    pub kind: FilmEventKind,
    pub medal_flag: u8,
    pub metadata: u8,
}

/// A player's decoded summary-event counts compared with Halo's match record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilmPlayerEventValidation {
    pub xuid: u64,
    pub gamertag: Option<String>,
    pub decoded: FilmEventCounts,
    pub match_stats: FilmEventCounts,
}

impl FilmPlayerEventValidation {
    /// Returns whether this player's decoded counts agree with the match record.
    pub fn matches_stats(&self) -> bool {
        self.decoded == self.match_stats
    }
}

/// Per-player comparison between a Theater film's summary events and match stats.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FilmEventValidation {
    pub players: Vec<FilmPlayerEventValidation>,
}

impl FilmEventValidation {
    /// Returns whether every represented human player has matching event counts.
    pub fn matches_stats(&self) -> bool {
        self.players
            .iter()
            .all(FilmPlayerEventValidation::matches_stats)
    }
}

/// Decoded highlight events with their comparison to the match-statistics API.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilmEventReport {
    pub events: Vec<FilmEvent>,
    pub validation: FilmEventValidation,
}

impl FilmEvent {
    pub fn medal(&self) -> Option<FilmMedal> {
        if matches!(self.kind, FilmEventKind::Medal) {
            Some(FilmMedal::from_id(self.metadata))
        } else {
            None
        }
    }
}

impl TryFrom<SummaryEvent> for FilmEvent {
    type Error = DecodeError;

    fn try_from(event: SummaryEvent) -> Result<Self, Self::Error> {
        Ok(Self {
            xuid: event
                .xuid
                .parse()
                .map_err(|_| DecodeError::Inconsistent("invalid summary XUID".into()))?,
            gamertag: event.name,
            timestamp_ms: u32::try_from(event.time_us / 1000).map_err(|_| {
                DecodeError::Inconsistent("summary timestamp exceeds u32 milliseconds".into())
            })?,
            kind: event.kind,
            medal_flag: event.medal_flag,
            metadata: event.metadata,
        })
    }
}

impl TryFrom<ValidatedSummaryEventReport> for FilmEventReport {
    type Error = DecodeError;

    fn try_from(report: ValidatedSummaryEventReport) -> Result<Self, Self::Error> {
        Ok(Self {
            events: report
                .summary
                .events
                .into_iter()
                .map(FilmEvent::try_from)
                .collect::<Result<_, _>>()?,
            // This established API exposes aggregate counts. The full summary report
            // additionally retains declared-count and per-medal identity diagnostics.
            validation: FilmEventValidation {
                players: report
                    .validation
                    .players
                    .into_iter()
                    .map(|p| {
                        Ok(FilmPlayerEventValidation {
                            xuid: p.xuid.parse().map_err(|_| {
                                DecodeError::Inconsistent("invalid summary XUID".into())
                            })?,
                            gamertag: p.name,
                            decoded: p.decoded,
                            match_stats: p.match_stats,
                        })
                    })
                    .collect::<Result<_, DecodeError>>()?,
            },
        })
    }
}
