//! Decoding implementation behind `Film::parse`.
//! Canonical models are public under `theater::film`; runtime models under `theater::runtime`.
pub(crate) mod bits;
pub(crate) mod chunk;
pub(crate) mod context;
pub(crate) mod limits;
pub(crate) mod ticks;
pub(crate) use context::{DeltaRecordSite, FilmParseContext, VehicleSnapshotSite};
pub(crate) use limits::ParseLimits;
pub(crate) use ticks::{ReplayTickSource, ReplayTickStatus};
pub(crate) mod packet;
#[cfg(test)]
pub(crate) mod test_support;
pub(crate) mod v41;

use crate::clients::hi::models::{RawChunkKind, RawFilmChunk};
use crate::theater::film::{ChunkKind, Film, InflatedFilmChunk, ParseError};

/// Validated chunk roles after each downloaded buffer has been consumed by inflation.
pub(crate) struct InflatedFilmChunks {
    pub registry: InflatedFilmChunk,
    pub replication_stream: Vec<InflatedFilmChunk>,
    pub summary: InflatedFilmChunk,
}

/// Structural source reads kept only while parsing and in parser tests.
#[derive(Debug)]
pub(crate) struct ParsedFilm {
    pub(crate) registry: v41::chunks::registry::ParsedRegistry,
    pub(crate) replication_stream: v41::chunks::replication::ParsedReplicationStream,
    pub(crate) summary: v41::chunks::summary::models::ParsedSummary,
}

impl ParsedFilm {
    pub(crate) fn into_film(self) -> Result<Film, ParseError> {
        Ok(Film {
            registry: self.registry.public_registry()?,
            replication_stream: self.replication_stream.into_typed(),
            summary: v41::chunks::summary::summarize(&self.summary),
        })
    }
}

/// Version-selected reader used internally by [`Film::parse`].
///
/// The enum makes the version dispatch explicit while preserving one public
/// parsing entry point. Add future film versions as variants here.
pub(crate) enum ChunkReader {
    V41(v41::V41ChunkReader),
}

impl ChunkReader {
    pub(crate) fn read(
        chunks: impl IntoIterator<Item = RawFilmChunk>,
        limits: ParseLimits,
        context: FilmParseContext,
    ) -> Result<Film, ParseError> {
        let chunks = Self::sort_chunks(chunks)?;
        for (expected, chunk) in chunks.iter().enumerate() {
            if usize::try_from(chunk.index).ok() != Some(expected) {
                return Err(ParseError::MissingIndex { index: expected });
            }
        }
        let parsed = Self::read_ordered(chunks, limits, context)?;
        let tick_status = parsed.replication_stream.replay_tick_status();
        let film = parsed.into_film()?;
        match tick_status {
            ReplayTickStatus::Found(_) => Ok(film),
            ReplayTickStatus::Absent => Err(ParseError::NoTicks),
            ReplayTickStatus::Undetermined => Err(ParseError::TickEvidenceIncomplete),
        }
    }

    #[cfg(test)]
    pub(crate) fn read_structural(
        chunks: impl IntoIterator<Item = RawFilmChunk>,
        limits: ParseLimits,
        context: FilmParseContext,
    ) -> Result<ParsedFilm, ParseError> {
        let chunks = Self::sort_chunks(chunks)?;
        Self::read_ordered(chunks, limits, context)
    }

    fn sort_chunks(
        chunks: impl IntoIterator<Item = RawFilmChunk>,
    ) -> Result<Vec<RawFilmChunk>, ParseError> {
        let mut chunks = chunks.into_iter().collect::<Vec<_>>();
        if let Some(chunk) = chunks.iter().find(|chunk| chunk.index < 0) {
            return Err(ParseError::InvalidIndex(chunk.index));
        }
        chunks.sort_unstable_by_key(|chunk| chunk.index);

        // Sorted chunks with equal adjacent indices have duplicate manifest entries.
        for pair in chunks.windows(2) {
            if pair[0].index == pair[1].index {
                return Err(ParseError::DuplicateIndex {
                    index: usize::try_from(pair[0].index).expect("validated nonnegative index"),
                });
            }
        }

        Ok(chunks)
    }

    fn read_ordered(
        mut chunks: Vec<RawFilmChunk>,
        limits: ParseLimits,
        context: FilmParseContext,
    ) -> Result<ParsedFilm, ParseError> {
        // Registry chunk should always come first
        let registry_chunk = chunks.first().ok_or(ParseError::MissingRegistry)?;
        if registry_chunk.kind != RawChunkKind::REGISTRY {
            return Err(ParseError::RegistryNotFirst);
        }
        let registry_chunk = chunks.remove(0);

        // Summary chunk should always come last.
        let summary_chunk = chunks.last().ok_or(ParseError::MissingSummary)?;
        if summary_chunk.kind != RawChunkKind::SUMMARY {
            return Err(ParseError::SummaryNotLast);
        }
        let summary_chunk = chunks.remove(chunks.len() - 1);

        // Get the version from the registry here, so we know what parser to use.
        let registry_chunk = registry_chunk.inflate()?;
        let header = registry_chunk
            .data
            .get(..8)
            .ok_or(ParseError::TruncatedRegistryHeader)?;
        let version = u32::from_le_bytes(header[..4].try_into().unwrap());

        // Parse with the version-specific reader.
        let reader = match version {
            41 => Self::V41(v41::V41ChunkReader),
            other => return Err(ParseError::UnsupportedVersion(other)),
        };

        // Validate remaining chunks are all Replication chunks.
        for chunk in &chunks {
            match ChunkKind::try_from(chunk.kind)? {
                ChunkKind::Registry => return Err(ParseError::RegistryNotFirst),
                ChunkKind::Summary => return Err(ParseError::SummaryNotLast),
                ChunkKind::Replication => {}
            }
        }
        // Ensure we have at least one replication chunk.
        if chunks.is_empty() {
            return Err(ParseError::MissingReplication);
        }

        // Replication chunks are in the middle.
        let replication_stream = chunks
            .into_iter()
            .map(RawFilmChunk::inflate)
            .collect::<Result<Vec<_>, _>>()?;
        let summary = summary_chunk.inflate()?;
        let chunks = InflatedFilmChunks {
            registry: registry_chunk,
            replication_stream,
            summary,
        };

        // Finish parsing film
        match reader {
            Self::V41(reader) => reader.read(chunks, limits, context),
        }
    }
}
