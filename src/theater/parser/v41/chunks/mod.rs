//! Version-specific chunk dispatch, preserving source order.
use crate::theater::film::{ChunkKind, Film, FilmChunk, ParseError};
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
        let mut replication_stream = Vec::new();
        let mut summary = None;
        for (index, input) in inputs.into_iter().enumerate() {
            // Validate cardinality before preparing or decoding duplicate chunks.
            match input.kind {
                ChunkKind::Registry => return Err(ParseError::MultipleRegistries),
                ChunkKind::Summary if summary.is_some() => {
                    return Err(ParseError::MultipleSummaries);
                }
                ChunkKind::Unknown(kind) => return Err(ParseError::UnsupportedChunkKind(kind)),
                ChunkKind::Replication if summary.is_some() => {
                    return Err(ParseError::SummaryNotLast);
                }
                _ => {}
            }
            let input = PreparedChunk::new(input, index + 1);
            match input.source.kind {
                ChunkKind::Replication => {
                    context.state.current_chunk =
                        input.source.index.unwrap_or(input.source_position as i64);
                    replication_stream
                        .push(V41ReplicationStreamChunkReader::read(input, &mut context));
                }
                ChunkKind::Summary => summary = Some(V41SummaryChunkReader::read(input)),
                ChunkKind::Registry | ChunkKind::Unknown(_) => {
                    unreachable!("unsupported chunk rejected before preparation")
                }
            }
        }
        Ok(Film {
            registry,
            replication_stream,
            summary: summary.ok_or(ParseError::MissingSummary)?,
        })
    }
}
