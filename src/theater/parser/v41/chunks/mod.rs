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
        inputs: impl IntoIterator<Item = FilmChunk>,
    ) -> Result<Film, ParseError> {
        let registry = V41RegistryChunkReader::read(registry_source, registry_data)?;
        let config = FrameConfig::default();
        let mut world = FilmWorld::default();
        let mut replication = Vec::new();
        let mut summaries = Vec::new();

        for (index, input) in inputs.into_iter().enumerate() {
            let source_position = index + 1;
            let chunk_index = i32::try_from(source_position)
                .map_err(|_| ParseError::Source("too many chunks".into()))?;
            let data = transport::inflate_film_chunk(&input.data).into_owned();
            match input.kind {
                ChunkKind::Registry => return Err(ParseError::MultipleRegistries),
                ChunkKind::Replication => {
                    world.current_chunk = input.index.unwrap_or(source_position as i64);
                    replication.push(V41ReplicationStreamChunkReader::read(
                        input,
                        source_position,
                        chunk_index,
                        data,
                        &config,
                        &registry.definition.registry,
                        &mut world,
                    ));
                }
                ChunkKind::Summary => summaries.push(V41SummaryChunkReader::read(
                    input,
                    source_position,
                    chunk_index,
                    data,
                )),
            }
        }

        Ok(Film {
            registry,
            replication: ReplicationStream {
                chunks: replication,
            },
            summaries: SummaryEvents { chunks: summaries },
        })
    }
}

fn packet_headers(
    data: &[u8],
    chunk_index: i32,
    kind: ChunkKind,
    source_position: usize,
) -> Vec<FilmPacketHeader> {
    let headers = transport::read_packet_headers(data, chunk_index);
    let walk_end = headers
        .last()
        .map(|header| header.payload_offset + header.payload_size)
        .unwrap_or(0);
    if walk_end < data.len() {
        tracing::warn!(
            ?kind,
            source_position,
            byte_offset = walk_end,
            remaining_bytes = data.len() - walk_end,
            "stopped parsing film chunk before the end"
        );
    }
    headers
}
