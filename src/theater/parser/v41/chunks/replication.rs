use super::*;

impl V41ReplicationStreamChunkReader {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn read(
        source: FilmChunk,
        source_position: usize,
        data: Vec<u8>,
        transport: ChunkTransport,
        config: &FrameConfig,
        registry: &FilmRegistry,
        world: &mut FilmWorld,
    ) -> ReplicationStreamChunk {
        let (headers, opaque) = packet_headers(&data, source.kind, source_position);
        let packets = headers
            .into_iter()
            .map(|(packet_source, header)| {
                let payload = &data[packet_source.payload.start..packet_source.payload.end];
                let mut context = packets::DecodeContext {
                    config,
                    registry,
                    world,
                    event_gate15: None,
                };
                let body = match packets::decode(header.packet_type, payload, &mut context) {
                    Ok(value) => PacketRead::Decoded(value),
                    Err(reason) => PacketRead::Opaque { reason },
                };
                let packet = ReplicationStreamPacket {
                    source: packet_source,
                    header,
                    body,
                };
                debug_assert!(packet.payload(&data).is_some());
                packet
            })
            .collect();
        ReplicationStreamChunk {
            source,
            source_position,
            data,
            transport,
            body: PacketStream { packets, opaque },
        }
    }
}
