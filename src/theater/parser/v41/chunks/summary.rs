use super::*;

impl V41SummaryChunkReader {
    pub(super) fn read(
        source: FilmChunk,
        source_position: usize,
        chunk_index: i32,
        data: Vec<u8>,
    ) -> SummaryChunk {
        let packets = packet_headers(&data, chunk_index, source.kind, source_position)
            .into_iter()
            .map(|header| {
                let payload =
                    &data[header.payload_offset..header.payload_offset + header.payload_size];
                let body = if header.packet_type == 9 {
                    crate::theater::parser::v41::summary::read_summary_packet(
                        payload,
                        header.chunk_index,
                        header.payload_offset,
                    )
                    .map(|(declared_events, events)| SummaryPacketBody::Events {
                        declared_events,
                        events,
                    })
                    .ok_or(PacketDecodeError::TruncatedSummaryCount {
                        packet_type: header.packet_type,
                    })
                } else {
                    Ok(SummaryPacketBody::Unknown)
                };
                SummaryPacket { header, body }
            })
            .collect();
        SummaryChunk {
            source,
            source_position,
            data,
            packets,
        }
    }
}
