//! Bootstrap entity-component registry.
mod bootstrap;
pub(crate) mod source;
pub(crate) use source::*;
/// Read the v41 registry's recorded blocks, slots, and retained bootstrap bytes.
pub struct V41RegistryChunkReader;

/// Structural type-1 read kept inside the parser until public setup is built.
#[derive(Debug)]
pub(crate) struct ParsedRegistry {
    pub(crate) index: usize,
    pub(crate) start_ms: i64,
    pub(crate) data: Vec<u8>,
    pub(crate) body: crate::theater::parser::v41::chunks::registry::source::FilmRegistryRead,
}

impl ParsedRegistry {
    /// Project recorded setup after all readers have used the source context.
    pub(crate) fn public_registry(
        &self,
    ) -> Result<crate::theater::film::Registry, crate::theater::film::ParseError> {
        use self::source::{
            BootstrapIdentificationFieldKind, GameOptionsEntry, GameOptionsFieldKind,
        };
        use crate::theater::film::registry::{
            Registry, RegistryArchetype, RegistryComponentDefinition, RegistryGameMode,
            RegistryMap, RegistryOptions, RegistryVersion,
        };

        let version = self
            .body
            .header
            .map(|[film, format]| RegistryVersion { film, format })
            .expect("chunk reader requires a complete registry version header");
        let components = self
            .body
            .registry
            .archetypes
            .iter()
            .map(|archetype| RegistryArchetype {
                index: crate::theater::film::ArchetypeId(archetype.index as u32),
                components: archetype
                    .components
                    .iter()
                    .map(|component| RegistryComponentDefinition {
                        name: component
                            .name()
                            .expect("reader accepted registry name")
                            .to_owned(),
                        precision_level: component.precision_level,
                    })
                    .collect(),
            })
            .collect();
        let identification = self
            .body
            .bootstrap_header
            .as_ref()
            .and_then(|header| header.identification.as_ref());
        let options = identification
            .and_then(|id| id.game_options.as_ref())
            .and_then(|read| {
                let value = |kind, width| {
                    read.entries.iter().find_map(|entry| match entry {
                        GameOptionsEntry::Field {
                            kind: recorded,
                            bits,
                        } if *recorded == kind && bits.width == width => bits
                            .raw
                            .as_ref()
                            .filter(|raw| raw.bit_len == width)
                            .map(|raw| raw.low_u64()),
                        _ => None,
                    })
                };
                Some((
                    value(GameOptionsFieldKind::GameMode, 3)? as u8,
                    value(GameOptionsFieldKind::RecorderSimulation, 3)? as u8,
                    crate::theater::film::TagId(value(GameOptionsFieldKind::LevelTag, 32)? as u32),
                ))
            });
        let map = identification.and_then(|id| {
            let bytes = |kind| {
                id.fields.iter().find_map(|field| {
                    if field.kind != kind || field.width != 128 {
                        return None;
                    }
                    let raw = field.raw.as_ref()?;
                    if raw.bit_len != 128 {
                        return None;
                    }
                    raw.bytes.as_slice().try_into().ok()
                })
            };
            Some(RegistryMap {
                asset_id: crate::theater::film::MapAssetId(bytes(
                    BootstrapIdentificationFieldKind::MapAssetId,
                )?),
                version_id: crate::theater::film::MapVersionId(bytes(
                    BootstrapIdentificationFieldKind::MapVersionId,
                )?),
            })
        });
        let map = map.ok_or(crate::theater::film::ParseError::MissingRegistryMap)?;
        let (game_mode, recorder_simulation, level_tag) =
            options.ok_or(crate::theater::film::ParseError::IncompleteRegistryOptions)?;
        let options = RegistryOptions {
            game_mode: RegistryGameMode::try_from(game_mode)?,
            recorder_simulation,
            level_tag,
        };
        Ok(Registry {
            version,
            components,
            options,
            map,
        })
    }
}

impl V41RegistryChunkReader {
    pub(crate) fn read(
        input: crate::theater::film::InflatedFilmChunk,
    ) -> Result<ParsedRegistry, crate::theater::film::ParseError> {
        let body = parse_registry_chunk(&input.data)?;
        Ok(ParsedRegistry {
            index: input.index,
            start_ms: input.start_ms,
            data: input.data,
            body,
        })
    }
}
use crate::theater::film::FilmRegistryReadError;
use crate::theater::parser::packet::ByteRange;

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
    let bootstrap_header = bootstrap::read(data, header, &registry, registry_end_byte);
    Ok(FilmRegistryRead {
        registry,
        header,
        registry_end_byte,
        stop,
        bootstrap_header,
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
    use super::{ParsedRegistry, REGISTRY_BLOCK_SIZE, parse_registry_chunk};
    use crate::theater::parser::packet::ByteRange;
    use crate::theater::parser::v41::chunks::registry::source::{
        FilmArchetype, FilmRegistryRead, RegistryComponent, RegistryStop,
    };

    #[test]
    fn public_registry_projects_typed_setup_without_source_bytes() {
        use crate::theater::film::registry::RegistryVersion;
        use crate::theater::parser::v41::chunks::registry::source::{
            BondBits, GameOptionsEntry, GameOptionsFieldKind, GameOptionsRead, GameOptionsStop,
        };
        use crate::theater::parser::v41::chunks::registry::source::{
            BootstrapHeaderRead, BootstrapHeaderStop, BootstrapIdentificationField,
            BootstrapIdentificationFieldKind, BootstrapIdentificationRead,
            BootstrapIdentificationStop,
        };
        use crate::theater::parser::v41::chunks::replication::source::components::field::RawBits;

        let mut body =
            parse_registry_chunk(&[41u32.to_le_bytes(), 27u32.to_le_bytes()].concat()).unwrap();
        let mut name_bytes = vec![0; 256];
        name_bytes[..5].copy_from_slice(b"alpha");
        body.registry.archetypes.push(FilmArchetype {
            index: 4,
            components: vec![RegistryComponent {
                name_bytes,
                precision_level: 7,
                source: ByteRange { start: 8, end: 268 },
            }],
            source: ByteRange {
                start: 8,
                end: 16_648,
            },
            padding: ByteRange {
                start: 268,
                end: 16_648,
            },
        });
        let map_field = |kind, byte| BootstrapIdentificationField {
            kind,
            bit: 0,
            width: 128,
            raw: Some(RawBits {
                bit_len: 128,
                bytes: vec![byte; 16],
            }),
        };
        let option = |kind, width, value| GameOptionsEntry::Field {
            kind,
            bits: BondBits {
                bit: 0,
                width,
                raw: Some(RawBits::from_low(value, width)),
            },
        };
        body.bootstrap_header = Some(BootstrapHeaderRead {
            start_bit: 0,
            end_bit: 0,
            fields: vec![],
            stop: BootstrapHeaderStop::Complete,
            identification: Some(BootstrapIdentificationRead {
                start_bit: 0,
                end_bit: 0,
                fields: vec![
                    map_field(BootstrapIdentificationFieldKind::MapAssetId, 1),
                    map_field(BootstrapIdentificationFieldKind::MapVersionId, 2),
                ],
                stop: BootstrapIdentificationStop::Complete,
                game_options: Some(GameOptionsRead {
                    start_bit: 0,
                    end_bit: 0,
                    entries: vec![
                        option(GameOptionsFieldKind::GameMode, 3, 2),
                        option(GameOptionsFieldKind::RecorderSimulation, 3, 1),
                        option(GameOptionsFieldKind::LevelTag, 32, 123),
                    ],
                    stop: GameOptionsStop::Complete,
                }),
            }),
        });
        let parsed = ParsedRegistry {
            index: 0,
            start_ms: 5,
            data: vec![0xaa],
            body,
        };
        let public = parsed.public_registry().unwrap();
        assert_eq!(
            public.version,
            RegistryVersion {
                film: 41,
                format: 27
            }
        );
        assert_eq!(public.components.len(), 1);
        assert_eq!(
            public.components[0].index,
            crate::theater::film::ArchetypeId(4)
        );
        assert_eq!(public.components[0].components[0].name, "alpha");
        assert_eq!(public.components[0].components[0].precision_level, 7);
        let options = public.options;
        assert_eq!(
            (
                options.game_mode,
                options.recorder_simulation,
                options.level_tag
            ),
            (
                crate::theater::film::RegistryGameMode::Multiplayer,
                1,
                crate::theater::film::TagId(123)
            )
        );
        let map = public.map;
        assert_eq!(
            (map.asset_id.bytes(), map.version_id.bytes()),
            ([1; 16], [2; 16])
        );
        let value = serde_json::to_value(parsed.public_registry().unwrap()).unwrap();
        assert_eq!(
            value
                .as_object()
                .unwrap()
                .keys()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            vec!["components", "map", "options", "version"]
        );

        let mut partial = parsed;
        partial
            .body
            .bootstrap_header
            .as_mut()
            .unwrap()
            .identification
            .as_mut()
            .unwrap()
            .game_options = None;
        assert!(matches!(
            partial.public_registry(),
            Err(crate::theater::film::ParseError::IncompleteRegistryOptions)
        ));
        partial
            .body
            .bootstrap_header
            .as_mut()
            .unwrap()
            .identification = None;
        assert!(matches!(
            partial.public_registry(),
            Err(crate::theater::film::ParseError::MissingRegistryMap)
        ));
    }

    #[test]
    fn finished_registry_requires_each_game_option() {
        use super::V41RegistryChunkReader;
        use crate::theater::film::ParseError;
        use crate::theater::parser::v41::chunks::registry::source::{
            GameOptionsEntry, GameOptionsFieldKind,
        };

        for missing in [
            GameOptionsFieldKind::GameMode,
            GameOptionsFieldKind::RecorderSimulation,
            GameOptionsFieldKind::LevelTag,
        ] {
            let input = crate::theater::parser::test_support::registry_with_complete_setup()
                .inflate()
                .unwrap();
            let mut parsed = V41RegistryChunkReader::read(input).unwrap();
            let entries = &mut parsed
                .body
                .bootstrap_header
                .as_mut()
                .unwrap()
                .identification
                .as_mut()
                .unwrap()
                .game_options
                .as_mut()
                .unwrap()
                .entries;
            let before = entries.len();
            entries.retain(
                |entry| !matches!(entry, GameOptionsEntry::Field { kind, .. } if *kind == missing),
            );
            assert!(entries.len() < before);
            assert!(matches!(
                parsed.public_registry(),
                Err(ParseError::IncompleteRegistryOptions)
            ));
        }

        let input = crate::theater::parser::test_support::registry_with_complete_setup()
            .inflate()
            .unwrap();
        let mut parsed = V41RegistryChunkReader::read(input).unwrap();
        let entries = &mut parsed
            .body
            .bootstrap_header
            .as_mut()
            .unwrap()
            .identification
            .as_mut()
            .unwrap()
            .game_options
            .as_mut()
            .unwrap()
            .entries;
        let mode = entries
            .iter_mut()
            .find(|entry| {
                matches!(
                    entry,
                    GameOptionsEntry::Field {
                        kind: GameOptionsFieldKind::GameMode,
                        ..
                    }
                )
            })
            .unwrap();
        if let GameOptionsEntry::Field { bits, .. } = mode {
            bits.raw = Some(
                crate::theater::parser::v41::chunks::replication::source::components::field::RawBits::from_low(1, 3),
            );
        }
        assert!(matches!(
            parsed.public_registry(),
            Err(ParseError::UnsupportedGameMode(1))
        ));
    }

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
