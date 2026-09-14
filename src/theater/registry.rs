//! Bootstrap entity-component registry.

use crate::clients::hi::models::FilmChunkData;

const REGISTRY_SLOT_SIZE: usize = 260;
const REGISTRY_BLOCK_SLOTS: usize = 64;
const REGISTRY_BLOCK_SIZE: usize = REGISTRY_SLOT_SIZE * REGISTRY_BLOCK_SLOTS;

/// Ordered replication components for one ECS entity archetype.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FilmArchetype {
    pub index: usize,
    pub components: Vec<String>,
}

/// Entity-component schema serialized in a Theater film's bootstrap chunk.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FilmRegistry {
    pub archetypes: Vec<FilmArchetype>,
}

impl FilmRegistry {
    pub fn archetype(&self, index: usize) -> Option<&FilmArchetype> {
        self.archetypes.get(index)
    }
}

/// Parses the fixed-width ECS component registry from the film bootstrap chunk.
pub fn decode_registry(chunks: &[FilmChunkData]) -> Option<FilmRegistry> {
    let data = &chunks
        .iter()
        .find(|chunk| chunk.metadata.chunk_type == 1)?
        .data;
    let archetypes = data
        .as_chunks::<REGISTRY_BLOCK_SIZE>()
        .0
        .iter()
        .enumerate()
        .map(|(index, block)| {
            let components = block
                .as_chunks::<REGISTRY_SLOT_SIZE>()
                .0
                .iter()
                .map(|slot| registry_slot_name(slot))
                .take_while(Option::is_some)
                .flatten()
                .collect();
            FilmArchetype { index, components }
        })
        .collect();
    Some(FilmRegistry { archetypes })
}

fn registry_slot_name(slot: &[u8]) -> Option<String> {
    let bytes = slot.get(8..)?;
    let end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    let name = bytes.get(..end)?;
    if name.is_empty() || !name.iter().all(|byte| byte.is_ascii_graphic()) {
        return None;
    }
    String::from_utf8(name.to_vec()).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clients::hi::models::FilmChunk;

    #[test]
    fn parses_registry_slots() {
        let mut data = vec![0; REGISTRY_BLOCK_SIZE];
        let name = b"example-component";
        data[8..8 + name.len()].copy_from_slice(name);
        let chunks = [FilmChunkData {
            metadata: FilmChunk {
                index: 0,
                start_time_offset_ms: 0,
                duration_ms: 0,
                size: data.len() as i64,
                file_relative_path: String::new(),
                chunk_type: 1,
            },
            data,
        }];
        let registry = decode_registry(&chunks).unwrap();
        assert_eq!(
            registry.archetypes[0].components,
            [String::from_utf8_lossy(name)]
        );
    }
}
