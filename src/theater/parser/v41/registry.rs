//! Bootstrap entity-component registry.
use crate::theater::film::ByteRange;

pub(crate) use crate::theater::film::chunks::registry::{
    FilmArchetype, FilmRegistry, FilmRegistryRead, FilmRegistryReadError, RegistryComponent,
    RegistryStop,
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
    let mut registry = FilmRegistry {
        archetypes: Vec::new(),
    };
    let mut registry_end_byte = header.map_or(0, |_| 8);
    let mut stop = if header.is_some() {
        RegistryStop::SourceBoundary
    } else {
        RegistryStop::TruncatedHeader
    };
    if header.is_some() {
        for (index, block) in data[8..]
            .as_chunks::<REGISTRY_BLOCK_SIZE>()
            .0
            .iter()
            .enumerate()
        {
            let start = 8 + index * REGISTRY_BLOCK_SIZE;
            let mut components = Vec::new();
            for (slot_index, slot) in block.as_chunks::<REGISTRY_SLOT_SIZE>().0.iter().enumerate() {
                if registry_slot_name(slot).is_none() {
                    break;
                }
                let slot_start = start + slot_index * REGISTRY_SLOT_SIZE;
                components.push(RegistryComponent {
                    name_bytes: slot[..256].to_vec(),
                    precision_level: u32::from_le_bytes(slot[256..260].try_into().unwrap()),
                    source: ByteRange {
                        start: slot_start,
                        end: slot_start + REGISTRY_SLOT_SIZE,
                    },
                });
            }
            let tail_start = components.len() * REGISTRY_SLOT_SIZE;
            if block[tail_start..].iter().any(|byte| *byte != 0) {
                stop = RegistryStop::BoundaryBlock;
                break;
            }
            let end = start + REGISTRY_BLOCK_SIZE;
            registry.archetypes.push(FilmArchetype {
                index,
                components,
                source: ByteRange { start, end },
                padding: ByteRange {
                    start: start + tail_start,
                    end,
                },
            });
            registry_end_byte = end;
        }
    }
    warn_unknown_registry(&registry);
    Ok(FilmRegistryRead {
        registry,
        header,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_components_own_wire_fields_and_block_ranges() {
        let mut data = [41u32.to_le_bytes(), 27u32.to_le_bytes()].concat();
        data.resize(8 + REGISTRY_BLOCK_SIZE * 2, 0);
        data[8..13].copy_from_slice(b"alpha");
        // Bytes after the name terminator are still part of the recorded slot.
        data[20] = 0xee;
        data[264..268].copy_from_slice(&7u32.to_le_bytes());
        data[268..272].copy_from_slice(b"beta");
        data[524..528].copy_from_slice(&11u32.to_le_bytes());
        let boundary = 8 + REGISTRY_BLOCK_SIZE;
        data[boundary] = 0xff;
        let read = parse_registry_chunk(&data).unwrap();
        assert_eq!(read.stop, RegistryStop::BoundaryBlock);
        assert_eq!(read.registry_end_byte, boundary);
        assert_eq!(read.registry.archetypes.len(), 1);
        let archetype = &read.registry.archetypes[0];
        assert_eq!(
            archetype.source,
            ByteRange {
                start: 8,
                end: boundary
            }
        );
        assert_eq!(
            archetype.padding,
            ByteRange {
                start: 528,
                end: boundary
            }
        );
        assert!(
            data[archetype.padding.start..archetype.padding.end]
                .iter()
                .all(|b| *b == 0)
        );
        assert_eq!(archetype.components.len(), 2);
        for (component, name, level, start) in [
            (&archetype.components[0], "alpha", 7, 8),
            (&archetype.components[1], "beta", 11, 268),
        ] {
            assert_eq!(component.name(), Some(name));
            assert_eq!(component.precision_level, level);
            assert_eq!(
                component.source,
                ByteRange {
                    start,
                    end: start + 260
                }
            );
            assert_eq!(component.name_bytes, data[start..start + 256]);
        }
        assert_eq!(archetype.components[0].name_bytes[12], 0xee);
        // Fingerprints use accepted names and levels, excluding retained tails.
        let mut expected = 0xcbf29ce484222325u64;
        for (name, level) in [("alpha", 7u32), ("beta", 11)] {
            for byte in level.to_le_bytes().iter().chain(name.as_bytes()) {
                expected = (expected ^ u64::from(*byte)).wrapping_mul(0x100000001b3);
            }
        }
        assert_eq!(read.registry.fingerprint(), Some(expected));
        let roundtrip: FilmRegistryRead =
            serde_json::from_slice(&serde_json::to_vec(&read).unwrap()).unwrap();
        assert_eq!(roundtrip, read);
    }

    #[test]
    fn registry_keeps_empty_blocks_and_stops_before_incomplete_blocks() {
        let mut data = [41u32.to_le_bytes(), 27u32.to_le_bytes()].concat();
        data.resize(8 + REGISTRY_BLOCK_SIZE + 100, 0);
        data[8 + REGISTRY_BLOCK_SIZE] = 0x42;
        let read = parse_registry_chunk(&data).unwrap();
        assert_eq!(read.stop, RegistryStop::SourceBoundary);
        assert_eq!(read.registry.archetypes.len(), 1);
        let archetype = &read.registry.archetypes[0];
        assert!(archetype.components.is_empty());
        assert_eq!(archetype.padding, archetype.source);
        assert_eq!(read.registry_end_byte, 8 + REGISTRY_BLOCK_SIZE);
        let truncated = parse_registry_chunk(&[41, 0, 0, 0]).unwrap();
        assert_eq!(truncated.stop, RegistryStop::TruncatedHeader);
        assert!(truncated.registry.archetypes.is_empty());
    }
}
