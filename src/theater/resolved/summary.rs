//! Semantic views over recorded type-3 summary entries.

use super::interpretation::SummaryPacketInterpretation;
use super::{RecordRef, SourceRef};

/// Meaning assigned to the recorded v41 summary type and medal flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SummaryKind {
    Mode,
    Death,
    Kill,
    Medal,
    Unknown { type_code: u8, medal_flag: u8 },
}

/// Query-ready interpretation of one guarded summary candidate.
/// The association and event category are derived; individual fields are bounded source reads.
///
/// Raw code units, flags, and bit ranges remain available through `source`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedSummary {
    pub derivation: SummaryDerivation,
    pub timestamp_us: u64,
    pub xuid: u64,
    pub gamertag: String,
    pub kind: SummaryKind,
    pub metadata: u8,
    pub type_code: u8,
    pub medal_flag: u8,
    pub player_index: Option<usize>,
    pub source: SourceRef,
}

/// How summary fields were associated into a higher-level event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SummaryDerivation {
    /// Marker guards and the captured v41 identity-to-tail offset.
    GuardedV41Layout,
}

pub(super) fn resolve_summaries(
    packets: &[SummaryPacketInterpretation],
    players: Option<&super::identity::PlayerTable>,
) -> Vec<ResolvedSummary> {
    let mut output = Vec::new();
    for packet in packets {
        for (record_index, event) in packet.events.iter().enumerate() {
            let player_index = players.and_then(|players| {
                let mut matches = players.slots.iter().filter(|slot| slot.xuid == event.xuid);
                let slot = matches.next()?;
                matches.next().is_none().then_some(slot.film_index)
            });
            let kind = if event.medal_flag != 0 {
                SummaryKind::Medal
            } else {
                match event.type_code {
                    10 => SummaryKind::Mode,
                    20 => SummaryKind::Death,
                    50 => SummaryKind::Kill,
                    type_code => SummaryKind::Unknown {
                        type_code,
                        medal_flag: event.medal_flag,
                    },
                }
            };
            output.push(ResolvedSummary {
                derivation: SummaryDerivation::GuardedV41Layout,
                timestamp_us: u64::from(event.timestamp_ms) * 1_000,
                xuid: event.xuid,
                gamertag: String::from_utf16_lossy(
                    &event.gamertag_utf16[..event
                        .gamertag_utf16
                        .iter()
                        .position(|unit| *unit == 0)
                        .unwrap_or(event.gamertag_utf16.len())],
                ),
                kind,
                metadata: event.metadata,
                type_code: event.type_code,
                medal_flag: event.medal_flag,
                player_index,
                source: SourceRef {
                    chunk: packet.source.chunk,
                    packet: packet.source.packet,
                    record: RecordRef::Summary(record_index),
                },
            });
        }
    }
    output
}
