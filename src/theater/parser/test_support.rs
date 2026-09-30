//! Test-only captured source adapters.
use crate::theater::film::{ChunkKind, FilmChunk};
// Legacy fixture adapters are test-only; production parsing consumes FilmChunk directly.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct FixtureChunkMetadata {
    pub index: i64,
    pub chunk_type: i64,
    pub start_ms: i64,
}

pub(crate) struct FixtureFilmSource {
    original_chunks: Vec<Vec<u8>>,
    metadata: Vec<FixtureChunkMetadata>,
}
impl FixtureFilmSource {
    pub(crate) fn load(
        chunks: &[impl AsRef<[u8]>],
        metadata: &[FixtureChunkMetadata],
    ) -> Result<Self, &'static str> {
        if chunks.is_empty() {
            return Err("empty film input");
        }
        Ok(Self {
            original_chunks: chunks.iter().map(|c| c.as_ref().to_vec()).collect(),
            metadata: metadata.to_vec(),
        })
    }
    pub(crate) fn original_chunks(&self) -> &[Vec<u8>] {
        &self.original_chunks
    }
    pub(crate) fn metadata(&self) -> &[FixtureChunkMetadata] {
        &self.metadata
    }
}

pub(crate) fn test_chunks(source: &FixtureFilmSource) -> Vec<FilmChunk> {
    source
        .original_chunks()
        .iter()
        .enumerate()
        .map(|(i, bytes)| {
            let metadata = source.metadata().get(i);
            FilmChunk {
                kind: match metadata
                    .map(|m| m.chunk_type)
                    .unwrap_or(if i == 0 { 1 } else { 2 })
                {
                    1 => ChunkKind::Registry,
                    3 => ChunkKind::Summary,
                    _ => ChunkKind::Replication,
                },
                index: metadata.map(|m| m.index),
                start_ms: metadata.map(|m| m.start_ms),
                data: bytes.clone(),
            }
        })
        .collect()
}
