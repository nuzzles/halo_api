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

/// An explicitly empty summary for synthetic recordings that exercise replication only.
pub(crate) fn empty_summary() -> FilmChunk {
    FilmChunk {
        kind: ChunkKind::Summary,
        index: None,
        start_ms: None,
        data: Vec::new(),
    }
}

pub(crate) fn test_chunks(source: &FixtureFilmSource) -> Vec<FilmChunk> {
    let mut chunks: Vec<_> = source
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
        .collect();
    // Legacy test fixtures often contain only the section under test.
    if !chunks.iter().any(|chunk| chunk.kind == ChunkKind::Summary) {
        chunks.push(empty_summary());
    }
    chunks
}

pub(crate) fn recharge_device_baseline() -> crate::theater::film::KeyframeRecord {
    use crate::theater::parser::v41::{
        chunks::replication::replication_stream::keyframes::record::decode_reference_keyframe_record_contextual,
        context::V41DecodeConfig,
    };
    let declarations: Vec<(String, u32)> = serde_json::from_str(include_str!(
        "../fixtures/movement-schema-v41/recharge-device-registry.json"
    ))
    .unwrap();
    let registry = component_registry(&[(43, declarations)]);
    let mut config = V41DecodeConfig::default();
    config.profile.grammar.baseline_scope = true;
    config.profile.grammar.writer_absolute = true;
    let payload = include_bytes!("../fixtures/movement-schema-v41/recharge-device-keyframe.bin");
    decode_reference_keyframe_record_contextual(
        payload,
        0,
        &registry,
        &config.record_layout(),
        Some(&config.profile),
    )
    .unwrap()
}

pub(crate) fn component_registry(
    declarations: &[(usize, Vec<(String, u32)>)],
) -> crate::theater::film::FilmRegistry {
    use crate::theater::film::{ByteRange, FilmArchetype, FilmRegistry, RegistryComponent};
    let range = ByteRange { start: 0, end: 0 };
    FilmRegistry {
        archetypes: (0..=declarations.iter().map(|(index, _)| *index).max().unwrap())
            .map(|index| FilmArchetype {
                index,
                source: range,
                padding: range,
                components: declarations.iter().find(|(i, _)| *i == index).map_or_else(
                    Vec::new,
                    |(_, components)| {
                        components
                            .iter()
                            .map(|(name, level)| RegistryComponent {
                                name_bytes: name.as_bytes().to_vec(),
                                precision_level: *level,
                                source: range,
                            })
                            .collect()
                    },
                ),
            })
            .collect(),
    }
}
