//! Source-linked interpretation. These searches never modify the reference Film.
use super::SourceRef;
use crate::theater::Film;
use crate::theater::film::*;
use crate::theater::parser::{
    bits,
    v41::{DecodeError, scan_kill_event_chains},
};
use bot_metadata::read_bot_metadata;
use event_gate::select_event_gate15;
use fire_events::read_fire_event;
use highlight_events::read_v41_highlights;
use identity::read_identity;
use packet::{packet_heads::read_packet_head, weapon_damage::read_weapon_damage};
use player_table::decode_player_table;

#[derive(Debug, Clone)]
pub struct ChunkInterpretation {
    pub source_position: usize,
    pub highlights: HighlightScan,
}
#[derive(Debug, Clone)]
pub struct PacketInterpretation {
    pub source: SourceRef,
    pub bot_metadata: Option<BotMetadataRead>,
    pub fire: Option<FireRead>,
    pub packet_head: Option<PacketHeadRead>,
    pub damage: Option<WeaponDamageTrace>,
}
/// One summary packet's guarded candidate reads, never canonical records.
#[derive(Debug, Clone)]
pub struct SummaryPacketInterpretation {
    pub source: SourceRef,
    pub events: Vec<SummaryEventRead>,
}

/// Heuristic evidence is deliberately separate from structurally decoded records.
/// A selected event gate or bootstrap anchor is not a recorded fact.
#[derive(Debug, Clone)]
pub struct Interpretations {
    pub identity: IdentityRead,
    pub player_table: Option<PlayerTable>,
    pub player_slot_reads: Vec<PlayerSlotRead>,
    pub event_gate15: EventGate15Selection,
    pub chunks: Vec<ChunkInterpretation>,
    pub packets: Vec<PacketInterpretation>,
    pub summary_packets: Vec<SummaryPacketInterpretation>,
}
impl Interpretations {
    pub(super) fn from_film(film: &Film) -> Self {
        let identity = read_identity(&film.registry.data, &film.registry.body);
        let player_table = identity
            .identity
            .as_ref()
            .map(|id| decode_player_table(&film.registry.data, id));
        let player_slot_reads = player_table
            .as_ref()
            .map(|t| {
                t.slots
                    .iter()
                    .map(|s| read_player_slot(&film.registry.data, s.bit, t.report.perso_bytes * 8))
                    .collect()
            })
            .unwrap_or_default();
        let event_gate15 = select_event_gate15(
            film.replication_chunks().flat_map(|c| {
                c.body
                    .packets
                    .iter()
                    .filter(|p| p.header.packet_type == 0)
                    .filter_map(move |p| c.payload(p))
            }),
            EventGate15Policy::CandidateCountInference,
        );
        let chunks = film
            .summary_chunks()
            .map(|chunk| ChunkInterpretation {
                source_position: chunk.source_position,
                highlights: read_v41_highlights(&chunk.data),
            })
            .collect();
        let summary_packets = film
            .summary_chunks()
            .flat_map(|chunk| {
                chunk
                    .body
                    .packets
                    .iter()
                    .enumerate()
                    .filter_map(move |(index, packet)| {
                        packet.body.decoded()?;
                        let payload = chunk.payload(packet)?;
                        Some(SummaryPacketInterpretation {
                            source: SourceRef {
                                chunk: chunk.source_position,
                                packet: index,
                                record: super::RecordRef::Packet,
                            },
                            events: summary::read_summary_candidates(payload),
                        })
                    })
            })
            .collect();
        let mut packets = Vec::new();
        for chunk in film.replication_chunks() {
            for (index, packet) in chunk.body.packets.iter().enumerate() {
                let Some(payload) = chunk.payload(packet) else {
                    continue;
                };
                let bot_metadata =
                    (packet.header.packet_type == 12).then(|| read_bot_metadata(payload));
                let fire = (packet.header.packet_type == 0
                    && payload.first().is_some_and(|b| b & 0x7f == 0x52))
                .then(|| {
                    let mut read = read_fire_event(payload);
                    if let Some(event) = &mut read.event {
                        event.chunk = chunk.source_position as i64;
                        event.packet_index = index;
                        event.timestamp_us = packet.header.timestamp_us;
                    }
                    read
                });
                let packet_head = read_packet_head(packet.header.packet_type, payload, None);
                let damage =
                    (packet.header.packet_type == 0 && payload.len() >= 2 && payload[0] == 0xc0)
                        .then(|| {
                            let mut read = read_weapon_damage(payload, packet.header.timestamp_us);
                            if let Some(damage) = &mut read.read {
                                damage.source = Some(packet.header);
                                damage.packet_index = Some(index);
                            }
                            read
                        });
                if bot_metadata.is_some()
                    || fire.is_some()
                    || packet_head.is_some()
                    || damage.is_some()
                {
                    packets.push(PacketInterpretation {
                        source: SourceRef {
                            chunk: chunk.source_position,
                            packet: index,
                            record: super::RecordRef::Packet,
                        },
                        bot_metadata,
                        fire,
                        packet_head,
                        damage,
                    });
                }
            }
        }
        Self {
            identity,
            player_table,
            player_slot_reads,
            event_gate15,
            chunks,
            packets,
            summary_packets,
        }
    }
}

pub(crate) mod bootstrap;

pub(crate) mod bot_metadata;

pub(crate) mod fire_events;

pub(crate) mod head_observations;

pub(crate) mod highlight_events;

pub(crate) mod event_gate;

pub(crate) mod identity;

pub(crate) mod objective_extract;

pub(crate) mod player_table;

pub mod player_slot;
pub use player_slot::{PlayerSlotRead, SlotField, SlotValue};

pub mod packet;
pub use packet::*;

pub use bot_metadata::{BotCandidate, BotMetadataRead, FilmBotEntry};

pub use objective_extract::ObjectiveFooterEvent;

pub use event_gate::{EventGate15Policy, EventGate15Selection};

pub use fire_events::{
    FilmFireEvent, FireAimAttempt, FireAimMethod, FireAimStop, FireField, FireHeaderStop, FireRead,
    FireUnitReference,
};

pub use highlight_events::{
    HighlightEvent, HighlightIdentityRead, HighlightScan, HighlightTailRead,
};

use player_slot::read_player_slot;

/// One scalar produced by an interpretation reader. `source_bits` records how
/// much of the requested width came from the packet; the remainder is reader
/// padding and is never represented as a canonical component field.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ProjectedField {
    pub name: String,
    pub bit: usize,
    pub width: usize,
    pub value: u64,
    pub source_bits: usize,
}

pub use super::identity::{
    FilmIdentity, IdentityField, IdentityRead, IdentityValue, PlayerTable, PlayerTableError,
    PlayerTableReport,
};

/// Bounded field reads and guarded associations from summary packet payloads.
pub mod summary;
pub use summary::SummaryEventRead;
