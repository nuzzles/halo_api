//! Version-specific chunk dispatch in manifest-index order.
use super::registry::V41RegistryChunkReader;
use super::replication::ParsedReplicationStream;
use super::summary::V41SummaryChunkReader;
use crate::theater::film::{
    FilmReplication, FilmSummary, InflatedFilmChunk, ParseError, Threading,
};
use crate::theater::parser::{FilmParseContext, InflatedFilmChunks, ParseLimits, ParsedFilm};

/// v41 backend selected from the first registry's recorded version.
pub struct V41ChunkReader;

impl V41ChunkReader {
    pub(crate) fn read_replication(
        self,
        registry: InflatedFilmChunk,
        chunks: Vec<InflatedFilmChunk>,
        limits: ParseLimits,
        context: FilmParseContext,
        threading: Threading,
    ) -> Result<FilmReplication, ParseError> {
        let registry = V41RegistryChunkReader::read(registry)?;
        let replication = ParsedReplicationStream::parse(
            &registry,
            chunks,
            &context,
            limits.max_packets_per_chunk,
            limits.max_control_records,
            threading,
        );
        let replication = replication?;
        crate::theater::parser::ChunkReader::validate_ticks(&replication)?;
        Ok(FilmReplication {
            registry: registry.into(),
            replication_stream: replication.into(),
        })
    }

    pub(crate) fn read_summary(
        self,
        registry: InflatedFilmChunk,
        summary: InflatedFilmChunk,
        limits: ParseLimits,
    ) -> Result<FilmSummary, ParseError> {
        let registry = V41RegistryChunkReader::read(registry)?;
        let summary = V41SummaryChunkReader::read(summary, limits.max_packets_per_chunk)?;
        Ok(FilmSummary {
            registry: registry.into(),
            summary: summary.into(),
        })
    }

    pub(crate) fn read(
        self,
        chunks: InflatedFilmChunks,
        limits: ParseLimits,
        parse_context: FilmParseContext,
        threading: Threading,
    ) -> Result<ParsedFilm, ParseError> {
        let registry = V41RegistryChunkReader::read(chunks.registry)?;
        #[cfg(not(target_arch = "wasm32"))]
        let (parsed_summary, parsed_replication) = if threading == Threading::Multi {
            std::thread::scope(|scope| {
                let summary = scope.spawn(|| {
                    V41SummaryChunkReader::read(chunks.summary, limits.max_packets_per_chunk)
                });
                let replication = ParsedReplicationStream::parse(
                    &registry,
                    chunks.replication_stream,
                    &parse_context,
                    limits.max_packets_per_chunk,
                    limits.max_control_records,
                    threading,
                );
                // Keep summary errors ahead of replication errors, as in the
                // sequential reader, while letting both readers run concurrently.
                let summary = summary.join().expect("film summary worker panicked")?;
                Ok::<_, ParseError>((summary, replication?))
            })?
        } else {
            (
                V41SummaryChunkReader::read(chunks.summary, limits.max_packets_per_chunk)?,
                ParsedReplicationStream::parse(
                    &registry,
                    chunks.replication_stream,
                    &parse_context,
                    limits.max_packets_per_chunk,
                    limits.max_control_records,
                    threading,
                )?,
            )
        };
        #[cfg(target_arch = "wasm32")]
        let (parsed_summary, parsed_replication) = (
            V41SummaryChunkReader::read(chunks.summary, limits.max_packets_per_chunk)?,
            ParsedReplicationStream::parse(
                &registry,
                chunks.replication_stream,
                &parse_context,
                limits.max_packets_per_chunk,
                limits.max_control_records,
                threading,
            )?,
        );
        Ok(ParsedFilm {
            registry,
            replication_stream: parsed_replication,
            summary: parsed_summary,
        })
    }
}
