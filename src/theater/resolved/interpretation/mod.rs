//! Source-linked interpretation. These searches never modify the native Film.
use super::SourceRef;
use crate::theater::film::ChunkKind;
use crate::theater::{Film, parser::v41::*};

#[derive(Debug, Clone)]
pub struct ChunkInterpretation {
    pub source_position: usize,
    pub highlights: NativeHighlightScan,
}
#[derive(Debug, Clone)]
pub struct PacketInterpretation {
    pub source: SourceRef,
    pub bot_metadata: Option<NativeBotMetadataRead>,
    pub fire: Option<NativeFireRead>,
}
/// Heuristic evidence is deliberately separate from structurally decoded records.
/// A selected event gate or bootstrap anchor is not a recorded fact.
#[derive(Debug, Clone)]
pub struct Interpretations {
    pub identity: NativeIdentityRead,
    pub player_table: Option<PlayerTable>,
    pub player_slot_reads: Vec<NativePlayerSlotRead>,
    pub event_gate15: NativeEventGate15Selection,
    pub chunks: Vec<ChunkInterpretation>,
    pub packets: Vec<PacketInterpretation>,
}
impl Interpretations {
    pub(super) fn from_film(film: &Film) -> Self {
        let identity = read_native_identity(
            &film.registry.chunk.data,
            &film.registry.definition.registry,
        );
        let player_table = identity
            .identity
            .as_ref()
            .map(|id| decode_player_table(&film.registry.chunk.data, id));
        let player_slot_reads = player_table
            .as_ref()
            .map(|t| {
                t.slots
                    .iter()
                    .map(|s| {
                        read_native_player_slot(
                            &film.registry.chunk.data,
                            s.bit,
                            t.report.perso_bytes * 8,
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();
        let event_gate15 = select_native_event_gate15(
            film.replication.chunks.iter().flat_map(|c| {
                c.packets
                    .iter()
                    .filter(|p| p.header.packet_type == 0)
                    .filter_map(move |p| c.payload(p))
            }),
            NativeEventGate15Policy::ReferenceInference,
        );
        let mut chunks = Vec::new();
        let mut packets = Vec::new();
        for chunk in film.chunks() {
            if chunk.source.kind == ChunkKind::Summary {
                chunks.push(ChunkInterpretation {
                    source_position: chunk.source_position,
                    highlights: read_native_v41_highlights(&chunk.data),
                });
            }
            if chunk.source.kind != ChunkKind::Replication {
                continue;
            }
            for (index, packet) in chunk.packets.iter().enumerate() {
                let Some(payload) = chunk.payload(packet) else {
                    continue;
                };
                let bot_metadata =
                    (packet.header.packet_type == 12).then(|| read_native_bot_metadata(payload));
                let fire = (packet.header.packet_type == 0
                    && payload.first().is_some_and(|b| b & 0x7f == 0x52))
                .then(|| {
                    let mut read = read_native_fire_event(payload);
                    if let Some(event) = &mut read.event {
                        event.chunk = chunk.source_position as i64;
                        event.packet_index = index;
                        event.timestamp_us = packet.header.timestamp_us;
                    }
                    read
                });
                if bot_metadata.is_some() || fire.is_some() {
                    packets.push(PacketInterpretation {
                        source: SourceRef {
                            chunk: chunk.source_position,
                            packet: index,
                            record: super::RecordRef::Packet,
                        },
                        bot_metadata,
                        fire,
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
        }
    }
}

pub(crate) mod bootstrap;

pub(crate) mod bot_metadata;

pub(crate) mod fire_events;

pub(crate) mod head_observations;

pub(crate) mod highlight_events;

pub(crate) mod native_event_gate;

pub(crate) mod native_identity;

pub(crate) mod objective_extract;

pub(crate) mod player_table;

pub use bot_metadata::{FilmBotEntry, NativeBotCandidate, NativeBotMetadataRead};

pub use objective_extract::ObjectiveFooterEvent;

pub use native_event_gate::{NativeEventGate15Policy, NativeEventGate15Selection};

pub use fire_events::{
    FilmFireEvent, FireUnitReference, NativeFireAimAttempt, NativeFireAimMethod, NativeFireAimStop,
    NativeFireField, NativeFireHeaderStop, NativeFireRead,
};

pub use highlight_events::{
    NativeHighlightEvent, NativeHighlightIdentityRead, NativeHighlightScan, NativeHighlightTailRead,
};

use crate::theater::film::NativePlayerSlotRead;
use crate::theater::parser::v41::player_slot::read_native_player_slot;

pub use super::identity::{
    FilmIdentity, NativeIdentityField, NativeIdentityRead, NativeIdentityValue, PlayerTable,
    PlayerTableError, PlayerTableReport,
};
