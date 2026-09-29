//! Reference registry models.

/// Ordered replication components for one ECS entity archetype.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FilmArchetype {
    pub index: usize,
    pub components: Vec<String>,
    /// Per-component precision levels, stored after each 256-byte name.
    #[serde(default)]
    pub levels: Vec<u32>,
}

/// Entity-component schema serialized in a Theater film's bootstrap chunk.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FilmRegistry {
    pub archetypes: Vec<FilmArchetype>,
}

/// Failure reported before a structural registry result can be produced.
/// Reference ParseRegistryChunk failure, distinct from a truncated registry result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum FilmRegistryReadError {
    #[error("registry chunk is still compressed")]
    StillCompressed,
}

/// One name read in a candidate registry block. Rejected names do not read a
/// level. Byte ranges refer to the retained bootstrap, not a recovered schema.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RegistrySlotRead {
    pub index: usize,
    pub start_byte: usize,
    /// All 256 bytes, including terminators and otherwise discarded string tails.
    pub name_bytes: Vec<u8>,
    pub name: Option<String>,
    /// Wire bytes of the level, only when the reference name reader accepted.
    pub level_bytes: Option<[u8; 4]>,
}

/// Ordered attempt to recognize one complete registry block. A rejected block
/// belongs to the following section; its tentative reads are not archetypes.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RegistryBlockRead {
    pub index: usize,
    pub start_byte: usize,
    pub end_byte: usize,
    pub slots: Vec<RegistrySlotRead>,
    pub tail_start_byte: usize,
    /// Exclusive end actually inspected by the zero-tail predicate. It stops
    /// immediately after the first nonzero byte, if any.
    pub tail_checked_end_byte: usize,
    pub first_nonzero_byte: Option<usize>,
    pub accepted: bool,
}

/// Complete reference registry parse outcome, including truncation information.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FilmRegistryRead {
    pub registry: FilmRegistry,
    /// None means fewer than eight physical header bytes were available. The
    /// compatibility registry's zero version words must not be read as recorded.
    pub header: Option<[u32; 2]>,
    /// None for old exports. Includes the rejected boundary block, if present;
    /// incomplete trailing blocks are not read by the reference grammar.
    pub blocks: Vec<RegistryBlockRead>,
    pub registry_end_byte: usize,
    pub stop: RegistryStop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RegistryStop {
    BoundaryBlock,
    SourceBoundary,
    TruncatedHeader,
}

impl FilmRegistry {
    pub(crate) fn archetype(&self, index: usize) -> Option<&FilmArchetype> {
        self.archetypes.get(index)
    }

    /// FNV-1a over each named entry's little-endian level and name.
    /// Older exports without levels have no comparable fingerprint.
    pub(crate) fn fingerprint(&self) -> Option<u64> {
        let mut hash = 0xcbf29ce484222325u64;
        for archetype in &self.archetypes {
            if archetype.components.len() != archetype.levels.len() {
                return None;
            }
            for (name, level) in archetype.components.iter().zip(&archetype.levels) {
                for byte in level.to_le_bytes().iter().chain(name.as_bytes()) {
                    hash = (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3);
                }
            }
        }
        Some(hash)
    }
}
