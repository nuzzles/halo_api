//! Sequential replication decoding: stream grammar, component grammar, and bindings.
use crate::theater::film::{PacketRead, ReplicationStreamChunk};
use crate::theater::parser::transport::PreparedChunk;
use replication_stream::DecodeContext;

pub(crate) mod bindings;
pub(crate) mod components;
pub(crate) mod replication_stream;
pub(crate) mod state;

/// Read replication packet bodies with state retained across ordered chunks.
pub struct V41ReplicationStreamChunkReader;

impl V41ReplicationStreamChunkReader {
    pub(crate) fn read(
        input: PreparedChunk,
        context: &mut DecodeContext<'_>,
    ) -> ReplicationStreamChunk {
        let body = input.read_packets(|header, payload| {
            match replication_stream::decode(header.packet_type, payload, context) {
                Ok(body) => PacketRead::Decoded(body),
                Err(reason) => PacketRead::Opaque { reason },
            }
        });
        input.with_body(body)
    }
}
