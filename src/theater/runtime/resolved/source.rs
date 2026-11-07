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
    if position == film.summary.source_position {
        return Some(FilmChunkRef::Summary(&film.summary));
    }
    // The registry occupies position zero; the summary follows all replication.
    // Manifest indices may be sparse and are not array indices.
    let index = position.checked_sub(1)?;
    film.replication_stream
        .get(index)
        .filter(|chunk| chunk.source_position == position)
        .map(FilmChunkRef::Replication)
}
