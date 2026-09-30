//! Typed summary events derived from bounded v41 candidate fields.
use super::interpretation::SummaryPacketInterpretation;
use super::{RecordRef, SourceRef};
use crate::theater::runtime::medals::Medal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SummaryKind {
    Mode,
    Death,
    Kill,
    Medal,
    Unknown,
}

/// Supported meaning of a summary. Kill/death events identify the recorded actor;
/// they do not invent an opposing player, weapon, or an entity ownership link.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SummaryPayload {
    Mode,
    Death,
    Kill,
    Medal(Medal),
    Unknown,
}
impl SummaryPayload {
    pub fn kind(self) -> SummaryKind {
        match self {
            Self::Mode => SummaryKind::Mode,
            Self::Death => SummaryKind::Death,
            Self::Kill => SummaryKind::Kill,
            Self::Medal(_) => SummaryKind::Medal,
            Self::Unknown => SummaryKind::Unknown,
        }
    }
}

/// The exact summary codes used by semantic interpretation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SummaryCodes {
    pub type_code: u8,
    pub metadata: u8,
    pub medal_flag: u8,
}

/// Bootstrap roster linkage is independent of the identity read in the summary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerLink {
    Unique {
        film_index: usize,
        gamertag_matches: bool,
    },
    Missing,
    Ambiguous,
}
impl PlayerLink {
    pub fn film_index(self) -> Option<usize> {
        match self {
            Self::Unique { film_index, .. } => Some(film_index),
            _ => None,
        }
    }
}

/// Identity read at the summary candidate. A roster name never replaces its name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SummaryActor {
    pub xuid: u64,
    pub gamertag: String,
    pub roster_link: PlayerLink,
}

/// Chronological, query-ready summary candidate with explicit interpretation provenance.
/// Raw UTF-16 units, flags, and ranges are available through `TheaterRuntime::record`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SummaryEvent {
    pub derivation: SummaryDerivation,
    /// Milliseconds read from the candidate tail, converted exactly to microseconds.
    /// This is not a video-observed physical-action time or the packet timestamp.
    pub timestamp_us: u64,
    /// Stable order in the runtime's complete event stream at this timestamp.
    pub order: usize,
    pub actor: SummaryActor,
    pub payload: SummaryPayload,
    pub codes: SummaryCodes,
    pub source: SourceRef,
}
impl SummaryEvent {
    pub fn kind(&self) -> SummaryKind {
        self.payload.kind()
    }
}
impl std::fmt::Display for SummaryEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({}) ", self.actor.gamertag, self.actor.xuid)?;
        match self.payload {
            SummaryPayload::Medal(medal) => write!(f, "received {medal}")?,
            SummaryPayload::Kill => f.write_str("recorded a kill")?,
            SummaryPayload::Death => f.write_str("died")?,
            SummaryPayload::Mode => f.write_str("had a mode event")?,
            SummaryPayload::Unknown => write!(
                f,
                "had an unknown summary ({}, {}, {})",
                self.codes.type_code, self.codes.metadata, self.codes.medal_flag
            )?,
        }
        let ms = self.timestamp_us / 1000;
        write!(
            f,
            " at {}:{:02}.{:03}",
            ms / 60_000,
            (ms / 1000) % 60,
            ms % 1000
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SummaryDerivation {
    /// Marker guards and the captured v41 identity-to-tail offset. Neither the
    /// candidate association nor its semantic label is a canonical decoded record.
    GuardedV41Layout,
}

/// A count comparison is evidence of coverage, not proof of a complete grammar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SummaryReadReport {
    pub source: SourceRef,
    pub declared_events: Option<u32>,
    pub candidate_events: usize,
}
impl SummaryReadReport {
    pub fn count_matches(&self) -> Option<bool> {
        self.declared_events
            .map(|count| usize::try_from(count).ok() == Some(self.candidate_events))
    }
}

fn payload(codes: SummaryCodes) -> SummaryPayload {
    // Pinned LevelUp/SPNKr medal sorting weights. A metadata byte alone cannot
    // distinguish a medal from a kill or a mode event.
    if codes.medal_flag == 1
        && matches!(
            codes.type_code,
            50 | 51
                | 52
                | 100
                | 101
                | 150
                | 200
                | 205
                | 210
                | 220
                | 225
                | 230
                | 235
                | 240
                | 245
                | 250
        )
    {
        return SummaryPayload::Medal(Medal::from_codes(codes.type_code, codes.metadata));
    }
    if codes.medal_flag > 1 {
        return SummaryPayload::Unknown;
    }
    match codes.type_code {
        10 => SummaryPayload::Mode,
        20 => SummaryPayload::Death,
        50 => SummaryPayload::Kill,
        _ => SummaryPayload::Unknown,
    }
}

pub(super) fn resolve_summaries(
    packets: &[SummaryPacketInterpretation],
    players: Option<&super::identity::PlayerTable>,
) -> Vec<SummaryEvent> {
    let mut output = Vec::new();
    for packet in packets {
        for (record_index, event) in packet.events.iter().enumerate() {
            let gamertag = String::from_utf16_lossy(
                &event.gamertag_utf16[..event
                    .gamertag_utf16
                    .iter()
                    .position(|unit| *unit == 0)
                    .unwrap_or(event.gamertag_utf16.len())],
            );
            let roster_link = players.map_or(PlayerLink::Missing, |players| {
                let mut matches = players.slots.iter().filter(|slot| slot.xuid == event.xuid);
                let Some(slot) = matches.next() else {
                    return PlayerLink::Missing;
                };
                if matches.next().is_some() {
                    return PlayerLink::Ambiguous;
                }
                if players
                    .slots
                    .iter()
                    .filter(|other| other.film_index == slot.film_index)
                    .count()
                    != 1
                {
                    return PlayerLink::Ambiguous;
                }
                PlayerLink::Unique {
                    film_index: slot.film_index,
                    gamertag_matches: gamertag == slot.gamertag,
                }
            });
            let codes = SummaryCodes {
                type_code: event.type_code,
                metadata: event.metadata,
                medal_flag: event.medal_flag,
            };
            output.push(SummaryEvent {
                derivation: SummaryDerivation::GuardedV41Layout,
                timestamp_us: u64::from(event.timestamp_ms) * 1_000,
                order: 0,
                actor: SummaryActor {
                    xuid: event.xuid,
                    gamertag,
                    roster_link,
                },
                payload: payload(codes),
                codes,
                source: SourceRef {
                    chunk: packet.source.chunk,
                    packet: packet.source.packet,
                    record: RecordRef::Summary(record_index),
                },
            });
        }
    }
    output.sort_by_key(|event| (event.timestamp_us, event.source.chunk, event.source.packet));
    output
}

/// Inclusive time bounds. XUID filtering works even without a bootstrap roster.
#[derive(Debug, Clone, Copy, Default)]
pub struct SummaryFilter {
    pub start_us: Option<u64>,
    pub end_us: Option<u64>,
    pub kind: Option<SummaryKind>,
    pub xuid: Option<u64>,
    pub medal: Option<Medal>,
}
