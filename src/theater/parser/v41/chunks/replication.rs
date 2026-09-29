use super::*;

impl V41ReplicationStreamChunkReader {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn read(
        source: FilmChunk,
        source_position: usize,
        chunk_index: i32,
        data: Vec<u8>,
        config: &FrameConfig,
        registry: &FilmRegistry,
        world: &mut FilmWorld,
    ) -> ReplicationStreamChunk {
        let packets = packet_headers(&data, chunk_index, source.kind, source_position)
            .into_iter()
            .map(|header| {
                let payload =
                    &data[header.payload_offset..header.payload_offset + header.payload_size];
                let mut context = packets::DecodeContext {
                    config,
                    registry,
                    world,
                    event_gate15: None,
                };
                ReplicationStreamPacket {
                    header,
                    body: packets::decode(header.packet_type, payload, &mut context),
                }
            })
            .collect();
        ReplicationStreamChunk {
            source,
            source_position,
            data,
            packets,
        }
    }
}
