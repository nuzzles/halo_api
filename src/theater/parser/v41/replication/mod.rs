//! Sequential replication decoding: stream grammar, component grammar, and bindings.
use super::registry::ParsedRegistry;
use crate::theater::film::InflatedFilmChunk;
pub(crate) mod source;

use crate::theater::film::ParseError;
use crate::theater::parser::v41::context::V41DecodeConfig;
use crate::theater::parser::v41::packet::{PacketRead, PacketStream};
use crate::theater::parser::v41::replication::source::{
    ReplicationStreamPacket, ReplicationStreamPacketBody,
};
use crate::theater::parser::{
    DeltaRecordSite, FilmParseContext, ReplayTickSource, ReplayTickStatus, VehicleSnapshotSite,
};
use stream::DecodeContext;

pub(crate) mod bindings;
pub(crate) mod components;
mod event;
mod parsed;
pub(crate) mod state;
pub(crate) mod stream;
mod tag;

/// Read replication packet bodies with state retained across ordered chunks.
pub struct V41ReplicationStreamChunkReader;

/// Source-backed decoder result retained until admission and typed projection.
#[derive(Debug)]
pub(crate) struct ParsedReplicationStream {
    pub(crate) chunks: Vec<ParsedReplicationStreamChunk>,
    pub(crate) schema_kinds: std::collections::BTreeMap<
        crate::theater::film::EntitySchemaId,
        crate::theater::film::EntitySchemaKind,
    >,
}

impl std::ops::Deref for ParsedReplicationStream {
    type Target = [ParsedReplicationStreamChunk];

    fn deref(&self) -> &Self::Target {
        &self.chunks
    }
}

impl std::ops::DerefMut for ParsedReplicationStream {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.chunks
    }
}

impl<'a> IntoIterator for &'a ParsedReplicationStream {
    type Item = &'a ParsedReplicationStreamChunk;
    type IntoIter = std::slice::Iter<'a, ParsedReplicationStreamChunk>;

    fn into_iter(self) -> Self::IntoIter {
        self.chunks.iter()
    }
}

impl<'a> IntoIterator for &'a mut ParsedReplicationStream {
    type Item = &'a mut ParsedReplicationStreamChunk;
    type IntoIter = std::slice::IterMut<'a, ParsedReplicationStreamChunk>;

    fn into_iter(self) -> Self::IntoIter {
        self.chunks.iter_mut()
    }
}

#[derive(Debug)]
pub(crate) struct ParsedReplicationStreamChunk {
    pub(crate) index: usize,
    pub(crate) start_ms: i64,
    pub(crate) data: Vec<u8>,
    pub(crate) body: PacketStream<ReplicationStreamPacketBody>,
}

impl ParsedReplicationStreamChunk {
    pub(crate) fn payload(&self, packet: &ReplicationStreamPacket) -> Option<&[u8]> {
        packet.payload(&self.data)
    }
}

impl ParsedReplicationStream {
    pub(crate) fn parse(
        registry: &ParsedRegistry,
        inputs: impl IntoIterator<Item = InflatedFilmChunk>,
        parse_context: &FilmParseContext,
        max_packets: usize,
        max_control_records: usize,
    ) -> Result<Self, ParseError> {
        for &archetype in parse_context.delta_record_archetypes.values() {
            if archetype >= 50
                || registry
                    .body
                    .registry
                    .archetype(archetype as usize)
                    .is_none()
            {
                return Err(ParseError::InvalidDeltaContextSchema { schema: archetype });
            }
        }
        let mut config = V41DecodeConfig::from_registry(&registry.body);
        config.control_record_limit = max_control_records;
        config.event_kill_tail = parse_context.kill_event_tail;
        let recorded_script_prefix =
            crate::theater::parser::v41::context::events::recorded_script_prefix(&registry.body);
        if recorded_script_prefix
            .zip(parse_context.script_event_prefix)
            .is_some_and(|(recorded, supplied)| recorded != supplied)
        {
            return Err(ParseError::ContradictoryScriptPrefixContext);
        }
        config.event_script_prefix = parse_context
            .script_event_prefix
            .filter(|_| recorded_script_prefix.is_none());
        let mut state = state::ReplicationDecodeState::default();
        let mut context = DecodeContext {
            config: &config,
            registry: &registry.body.registry,
            state: &mut state,
            event_gate15: recorded_script_prefix.or(parse_context.script_event_prefix),
        };
        let mut chunks = Vec::new();
        for input in inputs {
            context.state.current_chunk = input.index;
            chunks.push(V41ReplicationStreamChunkReader::read(
                input,
                &mut context,
                parse_context,
                max_packets,
            )?);
        }
        if chunks.is_empty() {
            return Err(ParseError::MissingReplication);
        }
        Ok(Self {
            chunks,
            schema_kinds: registry
                .registry
                .schemas
                .iter()
                .map(|schema| (schema.index, schema.kind))
                .collect(),
        })
    }

    /// Source-backed tick admission, evaluated before dropping packet bytes.
    pub(crate) fn replay_tick_status(&self) -> ReplayTickStatus {
        let mut incomplete = false;
        for (chunk_index, chunk) in self.chunks.iter().enumerate() {
            incomplete |= !chunk.body.opaque.is_empty();
            for (packet_index, packet) in chunk.body.packets.iter().enumerate() {
                let Some(payload) = chunk.payload(packet) else {
                    incomplete = true;
                    continue;
                };
                if packet.header.packet_type == 0
                    && let Some((payload_bit, _)) =
                        crate::theater::parser::ticks::tick_counter(payload)
                {
                    return ReplayTickStatus::Found(ReplayTickSource {
                        chunk: chunk_index,
                        packet: packet_index,
                        payload_bit,
                    });
                }
            }
        }
        if incomplete {
            ReplayTickStatus::Undetermined
        } else {
            ReplayTickStatus::Absent
        }
    }
}

impl V41ReplicationStreamChunkReader {
    pub(crate) fn read(
        input: InflatedFilmChunk,
        context: &mut DecodeContext<'_>,
        parse_context: &FilmParseContext,
        max_packets: usize,
    ) -> Result<ParsedReplicationStreamChunk, crate::theater::film::ParseError> {
        let chunk_index = input.index;
        let body = input.read_packets(max_packets, |packet_index, header, payload| {
            let vehicle_gate = |id| {
                parse_context
                    .vehicle_snapshot_gates
                    .get(&VehicleSnapshotSite {
                        chunk_index,
                        packet_index,
                        id,
                    })
                    .copied()
            };
            let delta_schema = |start_bit, id| {
                parse_context
                    .delta_record_archetypes
                    .get(&DeltaRecordSite {
                        chunk_index,
                        packet_index,
                        start_bit,
                        id,
                    })
                    .copied()
            };
            match stream::decode_with_context(
                header.packet_type,
                payload,
                context,
                vehicle_gate,
                delta_schema,
            ) {
                Ok(body) => PacketRead::Decoded(body),
                Err(reason) => PacketRead::Opaque { reason },
            }
        });
        Ok(ParsedReplicationStreamChunk {
            index: input.index,
            start_ms: input.start_ms,
            data: input.data,
            body: body?,
        })
    }
}
