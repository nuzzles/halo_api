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
use crate::theater::film::{
    ChunkKind, Film, FilmReplication, FilmSummary, InflatedFilmChunk, ParseError, Threading,
};

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
/// All full and partial parsing entry points share this version dispatch.
/// Add future film versions as variants here.
pub(crate) enum ChunkReader {
    V41(v41::V41ChunkReader),
}

impl ChunkReader {
    pub(crate) fn read(
        chunks: impl IntoIterator<Item = RawFilmChunk>,
        limits: ParseLimits,
        context: FilmParseContext,
        threading: Threading,
    ) -> Result<Film, ParseError> {
        let chunks = Self::sort_chunks(chunks)?;
        for (expected, chunk) in chunks.iter().enumerate() {
            if usize::try_from(chunk.index).ok() != Some(expected) {
                return Err(ParseError::MissingIndex { index: expected });
            }
        }
        let parsed = Self::read_ordered(chunks, limits, context, threading)?;
        Self::validate_ticks(&parsed.replication_stream)?;
        Ok(parsed.into())
    }

    fn validate_ticks(
        replication: &v41::replication::ParsedReplicationStream,
    ) -> Result<(), ParseError> {
        match replication.replay_tick_status() {
            ReplayTickStatus::Found(_) => Ok(()),
            ReplayTickStatus::Absent => Err(ParseError::NoTicks),
            ReplayTickStatus::Undetermined => Err(ParseError::TickEvidenceIncomplete),
        }
    }

    pub(crate) fn read_replication(
        chunks: impl IntoIterator<Item = RawFilmChunk>,
        limits: ParseLimits,
        context: FilmParseContext,
        threading: Threading,
    ) -> Result<FilmReplication, ParseError> {
        let mut chunks = Self::sort_chunks(
            chunks
                .into_iter()
                .filter(|chunk| chunk.kind != RawChunkKind::SUMMARY),
        )?;
        Self::validate_registry_first(&chunks)?;
        Self::validate_replication_chunks(&chunks[1..])?;
        for (expected, chunk) in chunks.iter().enumerate() {
            if chunk.index as usize != expected {
                return Err(ParseError::MissingIndex { index: expected });
            }
        }
        let registry = chunks.remove(0);
        let (reader, registry) = Self::select_reader(registry)?;
        let replication = Self::inflate_replication_chunks(chunks, threading)?;
        match reader {
            Self::V41(reader) => {
                reader.read_replication(registry, replication, limits, context, threading)
            }
        }
    }

    pub(crate) fn read_summary(
        chunks: impl IntoIterator<Item = RawFilmChunk>,
        limits: ParseLimits,
        _threading: Threading,
    ) -> Result<FilmSummary, ParseError> {
        let mut chunks = Self::sort_chunks(
            chunks
                .into_iter()
                .filter(|chunk| chunk.kind != RawChunkKind::REPLICATION),
        )?;
        Self::validate_registry_first(&chunks)?;
        if chunks[0].index != 0 {
            return Err(ParseError::MissingIndex { index: 0 });
        }
        let registry = chunks.remove(0);
        if chunks.is_empty() {
            return Err(ParseError::MissingSummary);
        }
        for chunk in &chunks {
            match ChunkKind::try_from(chunk.kind)? {
                ChunkKind::Registry => return Err(ParseError::RegistryNotFirst),
                ChunkKind::Summary => {}
                ChunkKind::Replication => unreachable!("filtered above"),
            }
        }
        if chunks.len() != 1 {
            return Err(ParseError::SummaryNotLast);
        }
        let (reader, registry) = Self::select_reader(registry)?;
        let summary = chunks.pop().expect("one summary").inflate()?;
        match reader {
            Self::V41(reader) => reader.read_summary(registry, summary, limits),
        }
    }

    fn validate_registry_first(chunks: &[RawFilmChunk]) -> Result<(), ParseError> {
        let registry = chunks.first().ok_or(ParseError::MissingRegistry)?;
        if registry.kind != RawChunkKind::REGISTRY {
            return Err(ParseError::RegistryNotFirst);
        }
        Ok(())
    }

    fn select_reader(registry: RawFilmChunk) -> Result<(Self, InflatedFilmChunk), ParseError> {
        let registry = registry.inflate()?;
        let header = registry
            .data
            .get(..8)
            .ok_or(ParseError::TruncatedRegistryHeader)?;
        let version = u32::from_le_bytes(header[..4].try_into().unwrap());
        let reader = match version {
            41 => Self::V41(v41::V41ChunkReader),
            other => return Err(ParseError::UnsupportedVersion(other)),
        };
        Ok((reader, registry))
    }

    fn validate_replication_chunks(chunks: &[RawFilmChunk]) -> Result<(), ParseError> {
        for chunk in chunks {
            match ChunkKind::try_from(chunk.kind)? {
                ChunkKind::Registry => return Err(ParseError::RegistryNotFirst),
                ChunkKind::Summary => return Err(ParseError::SummaryNotLast),
                ChunkKind::Replication => {}
            }
        }
        if chunks.is_empty() {
            return Err(ParseError::MissingReplication);
        }
        Ok(())
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
        threading: Threading,
    ) -> Result<ParsedFilm, ParseError> {
        // Registry chunk should always come first
        Self::validate_registry_first(&chunks)?;
        let registry_chunk = chunks.remove(0);

        // Summary chunk should always come last.
        let summary_chunk = chunks.last().ok_or(ParseError::MissingSummary)?;
        if summary_chunk.kind != RawChunkKind::SUMMARY {
            return Err(ParseError::SummaryNotLast);
        }
        let summary_chunk = chunks.remove(chunks.len() - 1);

        // Get the version from the registry here, so we know what parser to use.
        let (reader, registry_chunk) = Self::select_reader(registry_chunk)?;

        // Validate remaining chunks are all Replication chunks.
        Self::validate_replication_chunks(&chunks)?;

        // Replication chunks are in the middle.
        // Large terminal summaries can inflate while the replication workers
        // inflate the middle chunks. Tiny summaries cost less than a thread.
        #[cfg(not(target_arch = "wasm32"))]
        let (replication_stream, summary) = if threading == Threading::Multi
            && summary_chunk.data.len() >= 64 * 1024
            && std::thread::available_parallelism().is_ok_and(|workers| workers.get() > 1)
        {
            std::thread::scope(|scope| {
                let summary = scope.spawn(|| summary_chunk.inflate());
                let replication_stream = Self::inflate_replication_chunks(chunks, threading);
                let summary = summary
                    .join()
                    .expect("film summary inflation worker panicked");
                Ok::<_, ParseError>((replication_stream?, summary?))
            })?
        } else {
            (
                Self::inflate_replication_chunks(chunks, threading)?,
                summary_chunk.inflate()?,
            )
        };
        #[cfg(target_arch = "wasm32")]
        let (replication_stream, summary) = (
            Self::inflate_replication_chunks(chunks, threading)?,
            summary_chunk.inflate()?,
        );
        let chunks = InflatedFilmChunks {
            registry: registry_chunk,
            replication_stream,
            summary,
        };

        // Finish parsing film
        match reader {
            Self::V41(reader) => reader.read(chunks, limits, context, threading),
        }
    }

    fn inflate_replication_chunks(
        chunks: Vec<RawFilmChunk>,
        threading: Threading,
    ) -> Result<Vec<InflatedFilmChunk>, ParseError> {
        #[cfg(target_arch = "wasm32")]
        let _ = threading;
        #[cfg(not(target_arch = "wasm32"))]
        if threading == Threading::Multi && chunks.len() >= 2 {
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
