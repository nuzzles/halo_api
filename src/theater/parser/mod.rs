//! Decoding implementation behind `Film::parse`.
//! Canonical models are public under `theater::film`; runtime models under `theater::runtime`.
pub(crate) mod chunk;
pub(crate) mod compression;
pub(crate) mod context;
pub(crate) mod limits;
pub(crate) mod ticks;
pub(crate) mod wire;
pub(crate) use context::{DeltaRecordSite, FilmParseContext, VehicleSnapshotSite};
pub(crate) use limits::ParseLimits;
pub(crate) use ticks::{ReplayTickSource, ReplayTickStatus};
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
    pub(crate) registry: v41::registry::ParsedRegistry,
    pub(crate) replication_stream: v41::replication::ParsedReplicationStream,
    pub(crate) summary: v41::summary::models::ParsedSummary,
}

impl From<ParsedFilm> for Film {
    fn from(parsed: ParsedFilm) -> Self {
        Self {
            registry: parsed.registry.into(),
            replication_stream: parsed.replication_stream.into(),
            summary: parsed.summary.into(),
        }
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
        match tick_status {
            ReplayTickStatus::Found(_) => Ok(parsed.into()),
            ReplayTickStatus::Absent => Err(ParseError::NoTicks),
            ReplayTickStatus::Undetermined => Err(ParseError::TickEvidenceIncomplete),
        }
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
        // Large terminal summaries can inflate while the replication workers
        // inflate the middle chunks. Tiny summaries cost less than a thread.
        #[cfg(not(target_arch = "wasm32"))]
        let (replication_stream, summary) = if summary_chunk.data.len() >= 64 * 1024
            && std::thread::available_parallelism().is_ok_and(|workers| workers.get() > 1)
        {
            std::thread::scope(|scope| {
                let summary = scope.spawn(|| summary_chunk.inflate());
                let replication_stream = Self::inflate_replication_chunks(chunks);
                let summary = summary
                    .join()
                    .expect("film summary inflation worker panicked");
                Ok::<_, ParseError>((replication_stream?, summary?))
            })?
        } else {
            (
                Self::inflate_replication_chunks(chunks)?,
                summary_chunk.inflate()?,
            )
        };
        #[cfg(target_arch = "wasm32")]
        let (replication_stream, summary) = (
            Self::inflate_replication_chunks(chunks)?,
            summary_chunk.inflate()?,
        );
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

    fn inflate_replication_chunks(
        chunks: Vec<RawFilmChunk>,
    ) -> Result<Vec<InflatedFilmChunk>, ParseError> {
        #[cfg(not(target_arch = "wasm32"))]
        if chunks.len() >= 2 {
            let workers =
                std::thread::available_parallelism().map_or(1, |count| count.get().min(8));
            if workers > 1 {
                let chunk_count = chunks.len();
                let group_size = chunks.len().div_ceil(workers);
                let groups = chunk_count.div_ceil(group_size);
                let mut chunks = chunks.into_iter();
                return std::thread::scope(|scope| {
                    let handles = (0..groups)
                        .map(|_| {
                            let group = chunks.by_ref().take(group_size).collect::<Vec<_>>();
                            scope.spawn(move || {
                                group
                                    .into_iter()
                                    .map(RawFilmChunk::inflate)
                                    .collect::<Vec<_>>()
                            })
                        })
                        .collect::<Vec<_>>();
                    let mut inflated = Vec::with_capacity(chunk_count);
                    for handle in handles {
                        for result in handle.join().expect("film inflation worker panicked") {
                            inflated.push(result?);
                        }
                    }
                    Ok(inflated)
                });
            }
            return chunks.into_iter().map(RawFilmChunk::inflate).collect();
        }
        chunks.into_iter().map(RawFilmChunk::inflate).collect()
    }
}
