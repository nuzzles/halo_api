//! Bootstrap entity-component registry.

pub(crate) use crate::theater::film::chunks::registry::{
    FilmArchetype, FilmRegistry, FilmRegistryRead, FilmRegistryReadError, RegistryBlockRead,
    RegistrySlotRead, RegistryStop,
};
use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};

/// Structural v41 component registry (50 blocks, 1,067 named slots in the corpus).
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

/// Parse already-inflated registry bytes with the reference error/truncation contract.
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
        let registry = FilmRegistry { archetypes: vec![] };
        warn_unknown_registry(&registry);
        registry
    };
    let registry_end_byte = block_reads
        .iter()
        .take_while(|block| block.accepted)
        .last()
        .map_or(header.map_or(0, |_| 8), |block| block.end_byte);
    let stop = if header.is_none() {
        RegistryStop::TruncatedHeader
    } else if block_reads.last().is_some_and(|block| !block.accepted) {
        RegistryStop::BoundaryBlock
    } else {
        RegistryStop::SourceBoundary
    };
    Ok(FilmRegistryRead {
        registry,
        header,
        blocks: block_reads,
        registry_end_byte,
        stop,
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
    mut trace: Option<&mut Vec<RegistryBlockRead>>,
) -> Option<FilmRegistry> {
    if registry_looks_compressed(data) {
        return None; // Compatibility API intentionally loses the reference error category.
    }
    data.get(..8)?;
    let mut registry = FilmRegistry {
        archetypes: Vec::new(),
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
                slot_reads.push(RegistrySlotRead {
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
            trace.push(RegistryBlockRead {
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
            break;
        }
        registry.archetypes.push(FilmArchetype {
            index,
            components,
            levels,
        });
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
