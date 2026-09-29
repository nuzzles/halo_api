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
            Self::Replication(chunk) => chunk.packets.get(index).map(PacketRef::Replication),
            Self::Summary(chunk) => chunk.packets.get(index).map(PacketRef::Summary),
        }
    }
}

pub(super) fn chunk(film: &Film, position: usize) -> Option<FilmChunkRef<'_>> {
    if film.registry.source_position == position {
        return None;
    }
    if let Ok(i) = film
        .replication
        .chunks
        .binary_search_by_key(&position, |chunk| chunk.source_position)
    {
        return Some(FilmChunkRef::Replication(&film.replication.chunks[i]));
    }
    film.summaries
        .chunks
        .binary_search_by_key(&position, |chunk| chunk.source_position)
        .ok()
        .map(|i| FilmChunkRef::Summary(&film.summaries.chunks[i]))
}
