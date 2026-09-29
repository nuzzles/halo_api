//! Reference registry models.

use crate::theater::film::ByteRange;

/// One recorded registry slot: fixed-width name followed by its precision level.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RegistryComponent {
    /// All 256 name bytes, including the terminator and string tail.
    pub name_bytes: Vec<u8>,
    /// Recorded little-endian 32-bit encoding parameter immediately after the
    /// 256-byte name. Its meaning depends on the named component: it can select
    /// quantization widths, gate additional fields, or select a layout. Some
    /// components ignore it; it is not a universal number of precision bits.
    ///
    /// For example, some v41 quantized vectors use `min(6 + level, 26)` bits per
    /// axis, while other readers include extra fields when the level exceeds a
    /// threshold. This value is retained exactly from the registry, independently
    /// of any world-coordinate conversion performed during resolution.
    pub precision_level: u32,
    /// Complete 260-byte slot in the decompressed registry chunk.
    pub source: ByteRange,
}

impl RegistryComponent {
    /// Borrow the ASCII name accepted by the registry grammar.
    /// Invalid manually constructed names return None.
    pub fn name(&self) -> Option<&str> {
        let end = self
            .name_bytes
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(self.name_bytes.len());
        let bytes = &self.name_bytes[..end];
        if bytes.is_empty() || !bytes.iter().all(|byte| (0x20..=0x7e).contains(byte)) {
            return None;
        }
        std::str::from_utf8(bytes).ok()
    }
}

/// One accepted registry block with components in recorded slot order.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FilmArchetype {
    pub index: usize,
    pub components: Vec<RegistryComponent>,
    /// Complete fixed-size block, including its zero-filled tail.
    pub source: ByteRange,
    /// Recorded zero-filled remainder after the last component slot.
    pub padding: ByteRange,
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

/// Complete reference registry parse outcome, including truncation information.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FilmRegistryRead {
    pub registry: FilmRegistry,
    /// None means fewer than eight physical header bytes were available. The
    /// remaining header bytes are retained in the enclosing chunk.
    pub header: Option<[u32; 2]>,
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
            for component in &archetype.components {
                let name = component.name()?;
                let level = component.precision_level;
                for byte in level.to_le_bytes().iter().chain(name.as_bytes()) {
                    hash = (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3);
                }
            }
        }
        Some(hash)
    }
}
