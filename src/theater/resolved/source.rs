//! Internal borrowed navigation across the canonical chunk variants.
use crate::theater::film::*;

#[derive(Debug, Clone, Copy)]
pub(super) enum FilmChunkRef<'a> {
    Replication(&'a ReplicationStreamChunk),
    Summary(&'a SummaryChunk),
}

#[derive(Debug, Clone, Copy)]
pub(super) enum PacketRef<'a> {
    Replication(&'a ReplicationStreamPacket),
    Summary(&'a SummaryPacket),
}

impl PacketRef<'_> {
    pub(super) fn header(self) -> FilmPacketHeader {
        match self {
            Self::Replication(packet) => packet.header,
            Self::Summary(packet) => packet.header,
        }
    }
}

impl<'a> FilmChunkRef<'a> {
    pub(super) fn packet(self, index: usize) -> Option<PacketRef<'a>> {
        match self {
            Self::Replication(chunk) => chunk.body.packets.get(index).map(PacketRef::Replication),
            Self::Summary(chunk) => chunk.body.packets.get(index).map(PacketRef::Summary),
        }
    }
}

pub(super) fn chunk(film: &Film, position: usize) -> Option<FilmChunkRef<'_>> {
    if position == 0 {
        return None;
    }
    match film.chunks.get(position - 1)? {
        FilmDataChunk::Replication(chunk) => Some(FilmChunkRef::Replication(chunk)),
        FilmDataChunk::Summary(chunk) => Some(FilmChunkRef::Summary(chunk)),
        FilmDataChunk::Unknown(_) => None,
    }
}
