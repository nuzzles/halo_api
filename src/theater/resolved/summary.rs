//! Semantic views over recorded type-3 summary entries.

use super::{PacketRead, RecordRef, SourceRef, SummaryPacketBody, SummarySegment};
use crate::theater::film::Film;

/// Meaning assigned to the recorded v41 summary type and medal flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SummaryKind {
    Mode,
    Death,
    Kill,
    Medal,
    Unknown { type_code: u8, medal_flag: u8 },
}

/// Query-ready interpretation of one canonical summary entry.
///
/// Raw code units, flags, and bit ranges remain available through `source`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedSummary {
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

pub(super) fn resolve_summaries(
    film: &Film,
    players: Option<&super::identity::PlayerTable>,
) -> Vec<ResolvedSummary> {
    let mut output = Vec::new();
    for chunk in film.summary_chunks() {
        for (packet_index, packet) in chunk.body.packets.iter().enumerate() {
            let PacketRead::Complete(SummaryPacketBody::Events { segments, .. }) = &packet.body
            else {
                continue;
            };
            for (record_index, event) in segments
                .iter()
                .filter_map(|segment| match segment {
                    SummarySegment::Event(event) => Some(event),
                    SummarySegment::Opaque { .. } => None,
                })
                .enumerate()
            {
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
                    timestamp_us: u64::from(event.timestamp_ms) * 1_000,
                    xuid: event.xuid,
                    gamertag: String::from_utf16_lossy(&event.gamertag_utf16),
                    kind,
                    metadata: event.metadata,
                    type_code: event.type_code,
                    medal_flag: event.medal_flag,
                    player_index,
                    source: SourceRef {
                        chunk: chunk.source_position,
                        packet: packet_index,
                        record: RecordRef::Summary(record_index),
                    },
                });
            }
        }
    }
    output
}
