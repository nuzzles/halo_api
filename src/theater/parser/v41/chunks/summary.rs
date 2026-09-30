use super::*;

impl V41SummaryChunkReader {
    pub(super) fn read(
        source: FilmChunk,
        source_position: usize,
        data: Vec<u8>,
        transport: ChunkTransport,
    ) -> SummaryChunk {
        let (headers, opaque) = packet_headers(&data, source.kind, source_position);
        let packets = headers
            .into_iter()
            .map(|(packet_source, header)| {
                let payload = &data[packet_source.payload.start..packet_source.payload.end];
                let body = if header.packet_type == 9 {
                    crate::theater::parser::v41::summary::read_summary_packet(payload)
                        .map(|(declared_events, records)| {
                            PacketRead::Decoded(SummaryPacketBody::Events {
                                declared_events,
                                records,
                            })
                        })
                        .unwrap_or(PacketRead::Opaque {
                            reason: PacketDecodeError::TruncatedSummaryCount {
                                packet_type: header.packet_type,
                            },
                        })
                } else {
                    PacketRead::Opaque {
                        reason: PacketDecodeError::UnsupportedLayout {
                            packet_type: header.packet_type,
                        },
                    }
                };
                let packet = SummaryPacket {
                    source: packet_source,
                    header,
                    body,
                };
                debug_assert!(packet.payload(&data).is_some());
                packet
            })
            .collect();
        SummaryChunk {
            source,
            source_position,
            data,
            transport,
            body: PacketStream { packets, opaque },
        }
    }
}
