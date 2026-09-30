//! Version-specific chunk dispatch, preserving source order.
use crate::theater::film::{ChunkKind, Film, FilmChunk, FilmDataChunk, ParseError};
use crate::theater::parser::transport::PreparedChunk;
use crate::theater::parser::v41::context::V41DecodeConfig;
use registry::V41RegistryChunkReader;
use replication::{V41ReplicationStreamChunkReader, state::ReplicationDecodeState};
use summary::V41SummaryChunkReader;

pub(crate) mod registry;
pub(crate) mod replication;
pub(crate) mod summary;

/// v41 backend selected from the first registry's recorded version.
pub struct V41ChunkReader;

impl V41ChunkReader {
    pub(crate) fn read(
        self,
        first: PreparedChunk,
        inputs: impl IntoIterator<Item = FilmChunk>,
    ) -> Result<Film, ParseError> {
        let registry = V41RegistryChunkReader::read(first)?;
        let config = V41DecodeConfig::default();
        let mut state = ReplicationDecodeState::default();
        let mut context = replication::replication_stream::DecodeContext {
            config: &config,
            registry: &registry.body.registry,
            state: &mut state,
            event_gate15: None,
        };
        let mut chunks = Vec::new();
        for (index, input) in inputs.into_iter().enumerate() {
            if input.kind == ChunkKind::Registry {
                return Err(ParseError::MultipleRegistries);
            }
            let input = PreparedChunk::new(input, index + 1);
            chunks.push(match input.source.kind {
                ChunkKind::Registry => {
                    unreachable!("additional registry rejected before inflation")
                }
                ChunkKind::Replication => {
                    context.state.current_chunk =
                        input.source.index.unwrap_or(input.source_position as i64);
                    FilmDataChunk::Replication(V41ReplicationStreamChunkReader::read(
                        input,
                        &mut context,
                    ))
                }
                ChunkKind::Summary => FilmDataChunk::Summary(V41SummaryChunkReader::read(input)),
                ChunkKind::Unknown(_) => FilmDataChunk::Unknown(input.with_body(())),
            });
        }
        Ok(Film { registry, chunks })
    }
}
