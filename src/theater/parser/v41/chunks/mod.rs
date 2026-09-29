//! Version-specific readers for the three film chunk kinds.

use super::*;

mod registry;
mod replication;
mod summary;

impl V41ChunkReader {
    pub(crate) fn read(
        self,
        registry_source: FilmChunk,
        registry_data: Vec<u8>,
        registry_transport: ChunkTransport,
        inputs: impl IntoIterator<Item = FilmChunk>,
    ) -> Result<Film, ParseError> {
        let registry =
            V41RegistryChunkReader::read(registry_source, registry_data, registry_transport)?;
        let config = FrameConfig::default();
        let mut world = FilmWorld::default();
        let mut chunks = Vec::new();

        for (index, input) in inputs.into_iter().enumerate() {
            let source_position = index + 1;
            let chunk_index =
                i32::try_from(source_position).map_err(|_| ParseError::TooManyChunks)?;
            let (data, transport) = transport::inflate_film_chunk(&input.data);
            let data = data.into_owned();
            match input.kind {
                ChunkKind::Registry => return Err(ParseError::MultipleRegistries),
                ChunkKind::Replication => {
                    world.current_chunk = input.index.unwrap_or(source_position as i64);
                    chunks.push(FilmDataChunk::Replication(
                        V41ReplicationStreamChunkReader::read(
                            input,
                            source_position,
                            data,
                            transport,
                            &config,
                            &registry.body.registry,
                            &mut world,
                        ),
                    ));
                }
                ChunkKind::Summary => {
                    chunks.push(FilmDataChunk::Summary(V41SummaryChunkReader::read(
                        input,
                        source_position,
                        chunk_index,
                        data,
                        transport,
                    )))
                }
                ChunkKind::Unknown(_) => chunks.push(FilmDataChunk::Unknown(Chunk {
                    source: input,
                    source_position,
                    data,
                    transport,
                    body: (),
                })),
            }
        }

        Ok(Film { registry, chunks })
    }
}

fn packet_headers(
    data: &[u8],
    kind: ChunkKind,
    source_position: usize,
) -> (Vec<(PacketSource, FilmPacketHeader)>, Vec<ByteRange>) {
    let headers = transport::read_packet_headers(data);
    let walk_end = headers
        .last()
        .map(|(source, _)| source.payload.end)
        .unwrap_or(0);
    let opaque = if walk_end < data.len() {
        tracing::warn!(
            ?kind,
            source_position,
            byte_offset = walk_end,
            remaining_bytes = data.len() - walk_end,
            "stopped parsing film chunk before the end"
        );
        vec![ByteRange {
            start: walk_end,
            end: data.len(),
        }]
    } else {
        Vec::new()
    };
    (headers, opaque)
}
