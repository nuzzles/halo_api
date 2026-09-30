//! Chunk decompression and shared packet framing.
mod compression;
mod packet;
use crate::theater::film::{
    Chunk, ChunkTransport, FilmChunk, FilmPacketHeader, Packet, PacketBody, PacketRead,
    PacketStream,
};

/// One input inflated once, with both original and decompressed bytes retained.
pub(crate) struct PreparedChunk {
    pub source: FilmChunk,
    pub source_position: usize,
    pub data: Vec<u8>,
    pub transport: ChunkTransport,
}

impl PreparedChunk {
    pub(crate) fn new(source: FilmChunk, source_position: usize) -> Self {
        let (data, transport) = compression::inflate_film_chunk(&source.data);
        let data = data.into_owned();
        Self {
            source,
            source_position,
            data,
            transport,
        }
    }

    pub(crate) fn with_body<T>(self, body: T) -> Chunk<T> {
        Chunk {
            source: self.source,
            source_position: self.source_position,
            data: self.data,
            transport: self.transport,
            body,
        }
    }

    /// Frame packets once and retain any unwalked suffix independently of body decoding.
    pub(crate) fn read_packets<T: PacketBody>(
        &self,
        mut decode: impl FnMut(FilmPacketHeader, &[u8]) -> PacketRead<T>,
    ) -> PacketStream<T> {
        let headers = packet::read_packet_headers(&self.data);
        let end = headers.last().map_or(0, |(source, _)| source.payload.end);
        let opaque = if end < self.data.len() {
            tracing::warn!(kind = ?self.source.kind, source_position = self.source_position,
                byte_offset = end, remaining_bytes = self.data.len() - end,
                "stopped parsing film chunk before the end");
            vec![crate::theater::film::ByteRange {
                start: end,
                end: self.data.len(),
            }]
        } else {
            Vec::new()
        };
        let packets = headers
            .into_iter()
            .map(|(source, header)| {
                let body = decode(header, &self.data[source.payload.start..source.payload.end]);
                let packet = Packet {
                    source,
                    header,
                    body,
                };
                debug_assert!(packet.payload(&self.data).is_some());
                packet
            })
            .collect();
        PacketStream { packets, opaque }
    }
}
