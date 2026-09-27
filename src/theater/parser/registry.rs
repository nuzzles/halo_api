//! Bootstrap entity-component registry.

use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};

/// LevelUp's reference v41 component registry (50 blocks, 1,067 named slots).
pub const KNOWN_REGISTRY_FINGERPRINT: u64 = 0x36ca8c3d2a2f9b88;

fn first_unknown_registry(fingerprint: u64, seen: &mut HashSet<u64>) -> bool {
    fingerprint != KNOWN_REGISTRY_FINGERPRINT && seen.insert(fingerprint)
}

fn warn_unknown_registry(registry: &FilmRegistry) {
    static WARNED: OnceLock<Mutex<HashSet<u64>>> = OnceLock::new();
    let Some(fingerprint) = registry.fingerprint() else {
        return;
    };
    let first = {
        let mut seen = WARNED
            .get_or_init(Mutex::default)
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        first_unknown_registry(fingerprint, &mut seen)
    };
    if first {
        tracing::warn!(
            fingerprint,
            known_fingerprint = KNOWN_REGISTRY_FINGERPRINT,
            blocks = registry.archetypes.len(),
            named_slots = registry
                .archetypes
                .iter()
                .map(|a| a.components.len())
                .sum::<usize>(),
            "Unknown film ECS registry fingerprint; component grammar may differ; decoding continues"
        );
    }
}

const REGISTRY_SLOT_SIZE: usize = 260;
const REGISTRY_BLOCK_SLOTS: usize = 64;
const REGISTRY_BLOCK_SIZE: usize = REGISTRY_SLOT_SIZE * REGISTRY_BLOCK_SLOTS;

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
    /// Version words preceding the registry (distinct from component entries).
    #[serde(default)]
    pub major_version: u32,
    #[serde(default)]
    pub format_version: u32,
    /// First byte of the section following the registry.
    #[serde(default)]
    pub end_byte: usize,
    /// Input ended before a structural registry terminator was encountered.
    #[serde(default)]
    pub truncated: bool,
}

impl FilmRegistry {
    pub(crate) fn archetype(&self, index: usize) -> Option<&FilmArchetype> {
        self.archetypes.get(index)
    }

    /// LevelUp-compatible FNV-1a over each named entry's LE level and name.
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

/// Parses decompressed bootstrap bytes using LevelUp's structural registry boundary.
/// Port source: `docs/VALIDATION.md`; MIT attribution alongside it.
/// Native ParseRegistryChunk failure, distinct from a truncated registry result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum FilmRegistryReadError {
    #[error("registry chunk is still compressed")]
    StillCompressed,
}
/// One name read in a candidate registry block. Rejected names do not read a
/// level. Byte ranges refer to the retained bootstrap, not a recovered schema.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NativeRegistrySlotRead {
    pub index: usize,
    pub start_byte: usize,
    /// All 256 bytes, including terminators and otherwise discarded string tails.
    pub name_bytes: Vec<u8>,
    pub name: Option<String>,
    /// Wire bytes of the level, only when the native name reader accepted.
    pub level_bytes: Option<[u8; 4]>,
}
/// Ordered attempt to recognize one complete registry block. A rejected block
/// belongs to the following section; its tentative reads are not archetypes.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NativeRegistryBlockRead {
    pub index: usize,
    pub start_byte: usize,
    pub end_byte: usize,
    pub slots: Vec<NativeRegistrySlotRead>,
    pub tail_start_byte: usize,
    /// Exclusive end actually inspected by the zero-tail predicate. It stops
    /// immediately after the first nonzero byte, if any.
    pub tail_checked_end_byte: usize,
    pub first_nonzero_byte: Option<usize>,
    pub accepted: bool,
}
/// Complete native registry parse outcome, including truncation information.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FilmRegistryRead {
    pub registry: FilmRegistry,
    pub truncated_bytes: usize,
    /// None means fewer than eight physical header bytes were available. The
    /// compatibility registry's zero version words must not be read as recorded.
    pub header: Option<[u32; 2]>,
    /// None for old exports. Includes the rejected boundary block, if present;
    /// incomplete trailing blocks are not read by the native grammar.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub block_reads: Option<Vec<NativeRegistryBlockRead>>,
}
/// Parse already-inflated registry bytes with the native error/truncation contract.
/// Short input succeeds with an empty truncated registry, rather than becoming
/// indistinguishable from a compressed-input error. This does not enable decoding
/// earlier film major versions.
pub(crate) fn parse_registry_chunk(data: &[u8]) -> Result<FilmRegistryRead, FilmRegistryReadError> {
    if registry_looks_compressed(data) {
        return Err(FilmRegistryReadError::StillCompressed);
    }
    let header = data.get(..8).map(|h| {
        [
            u32::from_le_bytes(h[..4].try_into().unwrap()),
            u32::from_le_bytes(h[4..].try_into().unwrap()),
        ]
    });
    let mut block_reads = Vec::new();
    let registry = if let Some(registry) = parse_registry_with_trace(data, Some(&mut block_reads)) {
        registry
    } else {
        let registry = FilmRegistry {
            archetypes: vec![],
            major_version: 0,
            format_version: 0,
            end_byte: 0,
            truncated: true,
        };
        warn_unknown_registry(&registry);
        registry
    };
    let truncated_bytes = if !registry.truncated {
        0
    } else if data.len() < 8 {
        data.len()
    } else {
        (data.len() - 8) % REGISTRY_BLOCK_SIZE
    };
    Ok(FilmRegistryRead {
        registry,
        truncated_bytes,
        header,
        block_reads: Some(block_reads),
    })
}
fn registry_looks_compressed(data: &[u8]) -> bool {
    data.len() >= 2
        && data[0] & 15 == 8
        && data[1] & 32 == 0
        && u16::from_be_bytes([data[0], data[1]]).is_multiple_of(31)
}

fn parse_registry_with_trace(
    data: &[u8],
    mut trace: Option<&mut Vec<NativeRegistryBlockRead>>,
) -> Option<FilmRegistry> {
    if registry_looks_compressed(data) {
        return None; // Compatibility API intentionally loses the native error category.
    }
    let header = data.get(..8)?;
    let mut registry = FilmRegistry {
        archetypes: Vec::new(),
        major_version: u32::from_le_bytes(header[..4].try_into().ok()?),
        format_version: u32::from_le_bytes(header[4..].try_into().ok()?),
        end_byte: 8,
        truncated: true,
    };
    for (index, block) in data[8..]
        .as_chunks::<REGISTRY_BLOCK_SIZE>()
        .0
        .iter()
        .enumerate()
    {
        let start_byte = 8 + index * REGISTRY_BLOCK_SIZE;
        let mut slot_reads = Vec::new();
        let mut components = Vec::new();
        let mut levels = Vec::new();
        for (slot_index, slot) in block.as_chunks::<REGISTRY_SLOT_SIZE>().0.iter().enumerate() {
            let name = registry_slot_name(slot);
            if trace.is_some() {
                slot_reads.push(NativeRegistrySlotRead {
                    index: slot_index,
                    start_byte: start_byte + slot_index * REGISTRY_SLOT_SIZE,
                    name_bytes: slot[..256].to_vec(),
                    name: name.clone(),
                    level_bytes: name.as_ref().map(|_| slot[256..260].try_into().unwrap()),
                });
            }
            let Some(name) = name else {
                break;
            };
            components.push(name);
            levels.push(u32::from_le_bytes(slot[256..260].try_into().ok()?));
        }
        // A terminated list must have zero padding through the end of its block.
        // Nonzero bytes here belong to the next bootstrap section, not an archetype.
        let tail_start = components.len() * REGISTRY_SLOT_SIZE;
        let first_nonzero = block[tail_start..]
            .iter()
            .position(|b| *b != 0)
            .map(|offset| start_byte + tail_start + offset);
        if let Some(trace) = &mut trace {
            trace.push(NativeRegistryBlockRead {
                index,
                start_byte,
                end_byte: start_byte + REGISTRY_BLOCK_SIZE,
                slots: slot_reads,
                tail_start_byte: start_byte + tail_start,
                tail_checked_end_byte: first_nonzero
                    .map_or(start_byte + REGISTRY_BLOCK_SIZE, |at| at + 1),
                first_nonzero_byte: first_nonzero,
                accepted: first_nonzero.is_none(),
            });
        }
        if first_nonzero.is_some() {
            registry.truncated = false;
            break;
        }
        registry.archetypes.push(FilmArchetype {
            index,
            components,
            levels,
        });
        registry.end_byte += REGISTRY_BLOCK_SIZE;
    }
    warn_unknown_registry(&registry);
    Some(registry)
}

fn registry_slot_name(slot: &[u8]) -> Option<String> {
    let bytes = slot.get(..256)?;
    let end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    let name = bytes.get(..end)?;
    if name.is_empty() || !name.iter().all(|byte| (0x20..=0x7e).contains(byte)) {
        return None;
    }
    String::from_utf8(name.to_vec()).ok()
}
