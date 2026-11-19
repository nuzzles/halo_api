//! Version-specific chunk dispatch in manifest-index order.
use super::registry::V41RegistryChunkReader;
use super::replication::ParsedReplicationStream;
use super::summary::V41SummaryChunkReader;
use crate::theater::film::ParseError;
use crate::theater::parser::{FilmParseContext, InflatedFilmChunks, ParseLimits, ParsedFilm};

/// v41 backend selected from the first registry's recorded version.
pub struct V41ChunkReader;

impl V41ChunkReader {
    pub(crate) fn read(
        self,
        chunks: InflatedFilmChunks,
        limits: ParseLimits,
        parse_context: FilmParseContext,
    ) -> Result<ParsedFilm, ParseError> {
        let registry = V41RegistryChunkReader::read(chunks.registry)?;
        let parsed_summary =
            V41SummaryChunkReader::read(chunks.summary, limits.max_packets_per_chunk)?;
        let parsed_replication = ParsedReplicationStream::parse(
            &registry,
            chunks.replication_stream,
            &parse_context,
            limits.max_packets_per_chunk,
            limits.max_control_records,
        )?;
        Ok(ParsedFilm {
            registry,
            replication_stream: parsed_replication,
            summary: parsed_summary,
        })
    }
}
