use super::*;
use crate::clients::hi::models::{RawChunkKind, RawFilmChunk};
use crate::theater::parser::packet::*;
use crate::theater::parser::test_support::empty_summary;
use crate::theater::parser::test_support::zlib_fixture_data as zlib;
use crate::theater::parser::v41::chunks::replication::source::components::field::*;
use crate::theater::parser::v41::chunks::replication::source::*;
use crate::theater::parser::{DeltaRecordSite, FilmParseContext, ParseLimits};
use std::io::{BufRead, Write};

mod limits;
mod ordering;

type ParsedFilm = crate::theater::parser::ParsedFilm;

fn parse_structural(
    chunks: impl IntoIterator<Item = RawFilmChunk>,
) -> Result<ParsedFilm, ParseError> {
    parse_structural_with_context(chunks, FilmParseContext::default())
}

fn parse_structural_with_context(
    chunks: impl IntoIterator<Item = RawFilmChunk>,
    context: FilmParseContext,
) -> Result<ParsedFilm, ParseError> {
    crate::theater::parser::ChunkReader::read_structural(chunks, ParseLimits::default(), context)
}

fn parse_structural_with_limits(
    chunks: impl IntoIterator<Item = RawFilmChunk>,
    limits: ParseLimits,
) -> Result<ParsedFilm, ParseError> {
    crate::theater::parser::ChunkReader::read_structural(
        chunks,
        limits,
        FilmParseContext::default(),
    )
}

fn parse_typed_structural(
    chunks: impl IntoIterator<Item = RawFilmChunk>,
) -> Result<Film, ParseError> {
    parse_structural(chunks)?.into_film()
}

fn indexed_summary() -> RawFilmChunk {
    RawFilmChunk {
        index: 2,
        ..empty_summary()
    }
}

fn assert_chunk_bytes(input: &RawFilmChunk, index: usize, start_ms: i64, data: &[u8]) {
    assert_eq!(index, usize::try_from(input.index).unwrap());
    assert_eq!(start_ms, input.start_ms);
    assert_eq!(data, input.clone().inflate().unwrap().data);
}

fn registry() -> RawFilmChunk {
    RawFilmChunk {
        kind: ChunkKind::Registry.into(),
        index: 0,
        start_ms: 0,
        data: crate::theater::parser::test_support::zlib_fixture_data(
            &[41u32.to_le_bytes(), 27u32.to_le_bytes()].concat(),
        ),
    }
}
fn packet(kind: u16, payload: &[u8], time: u64) -> Vec<u8> {
    [
        kind.to_le_bytes().as_slice(),
        &[0xab, 0xcd],
        &(payload.len() as u32).to_le_bytes(),
        &time.to_le_bytes(),
        payload,
    ]
    .concat()
}

#[test]
fn finished_film_serializes_only_typed_sections() {
    let film = parse_typed_structural([
        crate::theater::parser::test_support::registry_with_complete_setup(),
        crate::theater::parser::test_support::empty_replication(),
        empty_summary(),
    ])
    .unwrap();
    let value = serde_json::to_value(&film).unwrap();
    assert_eq!(
        value
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["registry", "replication_stream", "summary"]
    );
    assert_eq!(
        value["registry"]
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["components", "map", "options", "version"]
    );
    assert_eq!(
        value["replication_stream"]
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["observations"]
    );
    assert!(value.to_string().find("source_bit").is_none());
    assert!(value.to_string().find("packet_index").is_none());
}

#[test]
fn film_summary_exposes_only_guarded_highlights() {
    fn put(output: &mut [u8], start: usize, bytes: &[u8]) {
        for (index, byte) in bytes.iter().enumerate() {
            for shift in 0..8 {
                let bit = start + index * 8 + shift;
                output[bit / 8] |= ((byte >> (7 - shift)) & 1) << (7 - bit % 8);
            }
        }
    }
    let identity: usize = 64;
    let tail = identity + 14_926;
    let end = tail + 60 * 8;
    let mut payload = vec![0; (end + 32).div_ceil(8)];
    put(&mut payload, 0, &2u32.to_be_bytes());
    put(&mut payload, identity, &42u64.to_le_bytes());
    put(&mut payload, identity + 64, &[0x2d, 0xc0]);
    put(&mut payload, tail, &[b'A', 0]);
    put(&mut payload, tail + 47 * 8, &[50]);
    put(&mut payload, tail + 48 * 8, &1234u32.to_be_bytes());
    put(&mut payload, end, &0x2ee0u32.to_be_bytes());
    let film = parse_typed_structural([
        crate::theater::parser::test_support::registry_with_complete_setup(),
        crate::theater::parser::test_support::empty_replication(),
        RawFilmChunk {
            kind: ChunkKind::Summary.into(),
            index: 2,
            start_ms: 0,
            data: zlib(&packet(9, &payload, 999_999)),
        },
    ])
    .unwrap();
    assert_eq!(film.registry.version.film, 41);
    assert_eq!(film.summary.highlights.len(), 1);
    assert_eq!(
        serde_json::to_value(&film.summary)
            .unwrap()
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        vec!["highlights"]
    );
    let event = &film.summary.highlights[0];
    assert_eq!(
        serde_json::to_value(event)
            .unwrap()
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        vec!["gamertag", "payload", "timestamp_us", "xuid"]
    );
    assert_eq!(event.payload, SummaryHighlightPayload::Kill);
    assert_eq!(event.xuid, Xuid(42));
    assert_eq!(event.gamertag, "A");
    assert_eq!(event.timestamp_us, 1_234_000);
    assert_eq!(
        crate::theater::TheaterRuntime::load(film)
            .summary_highlights()
            .len(),
        1
    );
}

#[test]
fn exact_delta_context_decodes_only_its_record_without_recording_an_archetype() {
    fn payload(two_deltas: bool) -> Vec<u8> {
        let mut bits = vec![true, false]; // configuration, event terminator
        let mut delta = || {
            bits.push(true);
            bits.extend((0..13).rev().map(|shift| 7 & (1 << shift) != 0));
            bits.extend([false, true]); // generation 1
            bits.extend([false, false, false, false, false]); // baseline, sparse empty mask
        };
        delta();
        if two_deltas {
            delta();
        }
        bits.extend([false, false, false, false]); // entity and control terminators
        let mut bytes = vec![0; bits.len().div_ceil(8)];
        for (index, bit) in bits.into_iter().enumerate() {
            bytes[index / 8] |= u8::from(bit) << (7 - index % 8);
        }
        bytes
    }
    fn parse(payload: &[u8], context: FilmParseContext) -> Result<ParsedFilm, ParseError> {
        parse_structural_with_context(
            [
                crate::theater::parser::test_support::zlib_fixture_chunk(
                    crate::theater::parser::test_support::registry_with_corruption_policy(false),
                ),
                RawFilmChunk {
                    kind: ChunkKind::Replication.into(),
                    index: 4,
                    start_ms: 0,
                    data: zlib(&packet(0, payload, 10)),
                },
                empty_summary(),
            ],
            context,
        )
    }
    fn frame(film: &ParsedFilm) -> &ProductionFrame {
        let PacketRead::Decoded(ReplicationStreamPacketBody::FramePacketBody(packet)) =
            &film.replication_stream.0[0].body.packets[0].body
        else {
            panic!("frame packet expected")
        };
        packet.frame.decoded().expect("decoded frame")
    }
    let id = (1 << 30) | 7;
    let site = DeltaRecordSite {
        chunk_index: 4,
        packet_index: 0,
        start_bit: 2,
        id,
    };
    let encoded = payload(false);
    assert!(matches!(
        frame(&parse(&encoded, FilmParseContext::default()).unwrap()).entity_end,
        Some(ProductionEntityEnd::Rejected {
            reason: DeltaRejectionReason::Unbound,
            ..
        })
    ));
    for wrong in [
        DeltaRecordSite {
            chunk_index: 5,
            ..site
        },
        DeltaRecordSite {
            packet_index: 1,
            ..site
        },
        DeltaRecordSite {
            start_bit: 3,
            ..site
        },
        DeltaRecordSite { id: id + 1, ..site },
    ] {
        let context = FilmParseContext {
            delta_record_archetypes: [(wrong, 0)].into(),
            ..Default::default()
        };
        assert!(matches!(
            frame(&parse(&encoded, context).unwrap()).entity_end,
            Some(ProductionEntityEnd::Rejected {
                reason: DeltaRejectionReason::Unbound,
                ..
            })
        ));
    }
    let context = FilmParseContext {
        delta_record_archetypes: [(site, 0)].into(),
        ..Default::default()
    };
    let film = parse(&encoded, context.clone()).unwrap();
    let decoded = frame(&film);
    assert_eq!(decoded.records.len(), 1);
    assert_eq!(decoded.records[0].archetype, None);
    assert_eq!(decoded.records[0].mask, Some(0));
    assert_eq!(decoded.records[0].stop, EntityViewStop::Complete);
    assert_eq!(
        decoded.external_delta_schemas,
        vec![ExternalDeltaSchemaUse {
            start_bit: 2,
            id,
            archetype: 0
        }]
    );
    let film = parse(&payload(true), context).unwrap();
    let decoded = frame(&film);
    assert_eq!(decoded.records.len(), 1);
    assert!(matches!(
        decoded.entity_end,
        Some(ProductionEntityEnd::Rejected {
            reason: DeltaRejectionReason::Unbound,
            ..
        })
    ));
    assert_eq!(decoded.external_delta_schemas.len(), 1);
    assert!(matches!(
        parse(
            &encoded,
            FilmParseContext {
                delta_record_archetypes: [(site, 50)].into(),
                ..Default::default()
            }
        ),
        Err(ParseError::InvalidDeltaContextArchetype { archetype: 50 })
    ));
}

#[test]
fn end_packet_admits_only_empty_body_and_preserves_unsupported_source() {
    for payload in [vec![], vec![0], vec![0xff], packet(7, &[], 99)] {
        for suffix in [vec![], packet(7, &[], 100)] {
            let end = packet(7, &payload, 42);
            let data = [end.clone(), suffix.clone()].concat();
            let film = parse_structural([
                registry(),
                RawFilmChunk {
                    kind: ChunkKind::Replication.into(),
                    index: 3,
                    start_ms: 4,
                    data: zlib(&data),
                },
                empty_summary(),
            ])
            .unwrap();
            let chunk = &film.replication_stream.0[0];
            assert_eq!(chunk.data, data);
            assert_eq!(chunk.data, data);
            assert_eq!(
                chunk.body.packets.len(),
                if suffix.is_empty() { 1 } else { 2 }
            );
            let p = &chunk.body.packets[0];
            assert_eq!(p.header.timestamp_us, 42);
            assert_eq!(p.header.unknown_2, [0xab, 0xcd]);
            assert_eq!(chunk.payload(p).unwrap(), payload);
            if payload.is_empty() {
                assert_eq!(
                    p.body,
                    PacketRead::Decoded(ReplicationStreamPacketBody::EndPacketBody)
                );
                assert_eq!(p.source_coverage(&chunk.data), Some(vec![]));
            } else {
                assert_eq!(
                    p.body,
                    PacketRead::Opaque {
                        reason: PacketDecodeError::UnsupportedLayout { packet_type: 7 }
                    }
                );
                assert_eq!(
                    p.source_coverage(&chunk.data),
                    Some(vec![SourceRegion {
                        source: BitRange {
                            start: 0,
                            end: payload.len() * 8
                        },
                        kind: SourceRegionKind::Opaque,
                    }])
                );
            }
            // A complete successor envelope is framed by its declared size;
            // the payload is never scanned for a plausible header.
            assert_eq!(chunk.body.opaque, vec![]);
            assert_eq!(
                serde_json::from_slice::<ReplicationStreamPacket>(&serde_json::to_vec(p).unwrap())
                    .unwrap(),
                *p
            );
        }
    }
}

#[test]
fn empty_envelopes_and_end_successors_preserve_sequential_boundaries() {
    let first = packet(99, &[], 1);
    let end = packet(7, &[], 2);
    let last = packet(88, b"opaque", 3);
    let data = [first.clone(), end.clone(), last.clone(), vec![0xee; 3]].concat();
    let film = parse_structural([
        registry(),
        RawFilmChunk {
            kind: ChunkKind::Replication.into(),
            index: 3,
            start_ms: 4,
            data: zlib(&data),
        },
        empty_summary(),
    ])
    .unwrap();
    let chunk = &film.replication_stream.0[0];
    assert_eq!(chunk.body.packets.len(), 3);
    assert_eq!(chunk.body.packets[0].source.payload.start, first.len());
    assert_eq!(chunk.body.packets[0].source.payload.end, first.len());
    assert_eq!(chunk.body.packets[1].source.header.start, first.len());
    assert_eq!(
        chunk.body.packets[2].source.header.start,
        first.len() + end.len()
    );
    assert!(matches!(
        chunk.body.packets[0].body,
        PacketRead::Opaque { .. }
    ));
    assert!(chunk.payload(&chunk.body.packets[0]).unwrap().is_empty());
    assert_eq!(chunk.payload(&chunk.body.packets[2]).unwrap(), b"opaque");
    assert_eq!(
        chunk.body.opaque,
        vec![ByteRange {
            start: data.len() - 3,
            end: data.len()
        }]
    );
}

#[test]
fn dispatch_requires_one_registry_and_rejects_unsupported_versions() {
    assert!(matches!(Film::parse([]), Err(ParseError::MissingRegistry)));
    for kind in [ChunkKind::Replication, ChunkKind::Summary] {
        assert!(matches!(
            Film::parse([
                RawFilmChunk {
                    kind: kind.into(),
                    index: 0,
                    start_ms: 0,
                    data: Vec::new()
                },
                RawFilmChunk {
                    index: 1,
                    ..registry()
                }
            ]),
            Err(ParseError::RegistryNotFirst)
        ));
    }
    assert!(matches!(
        Film::parse([registry()]),
        Err(ParseError::MissingSummary)
    ));
    let film = parse_typed_structural([
        crate::theater::parser::test_support::registry_with_complete_setup(),
        crate::theater::parser::test_support::empty_replication(),
        RawFilmChunk {
            index: 2,
            ..empty_summary()
        },
    ])
    .unwrap();
    assert!(film.replication_stream.observations.is_empty());
    assert!(film.summary.highlights.is_empty());
    assert!(matches!(
        Film::parse([
            registry(),
            RawFilmChunk {
                index: 1,
                ..empty_summary()
            }
        ]),
        Err(ParseError::MissingReplication)
    ));
    assert!(matches!(
        Film::parse([registry(), empty_summary(), empty_summary()]),
        Err(ParseError::DuplicateIndex { index }) if index == i32::MAX as usize
    ));
    assert!(matches!(
        Film::parse([
            registry(),
            RawFilmChunk {
                index: 1,
                ..empty_summary()
            },
            RawFilmChunk {
                index: 2,
                ..empty_summary()
            },
        ]),
        Err(ParseError::SummaryNotLast)
    ));
    assert!(matches!(
        Film::parse([
            registry(),
            RawFilmChunk {
                index: 1,
                ..empty_summary()
            },
            RawFilmChunk {
                kind: ChunkKind::Replication.into(),
                index: 2,
                ..crate::theater::parser::test_support::empty_replication()
            },
        ]),
        Err(ParseError::SummaryNotLast)
    ));
    let unknown = RawFilmChunk {
        kind: crate::clients::hi::models::RawChunkKind(99),
        index: 1,
        ..empty_summary()
    };
    assert!(matches!(
        Film::parse([registry(), unknown, indexed_summary()]),
        Err(ParseError::UnsupportedChunkKind(99))
    ));
    assert!(matches!(
        Film::parse([registry(), registry()]),
        Err(ParseError::DuplicateIndex { index: 0 })
    ));
    assert!(matches!(
        Film::parse([
            registry(),
            RawFilmChunk {
                index: 1,
                ..registry()
            },
            indexed_summary(),
        ]),
        Err(ParseError::RegistryNotFirst)
    ));
    assert!(matches!(
        Film::parse([
            RawFilmChunk {
                kind: ChunkKind::Registry.into(),
                index: 0,
                start_ms: 0,
                data: zlib(&[75u32.to_le_bytes(), 27u32.to_le_bytes()].concat())
            },
            crate::theater::parser::test_support::empty_replication(),
            indexed_summary(),
        ]),
        Err(ParseError::UnsupportedVersion(75))
    ));
    assert!(matches!(
        Film::parse([
            RawFilmChunk {
                kind: ChunkKind::Registry.into(),
                index: 0,
                start_ms: 0,
                data: zlib(&[41])
            },
            crate::theater::parser::test_support::empty_replication(),
            indexed_summary(),
        ]),
        Err(ParseError::TruncatedRegistryHeader)
    ));
    assert!(matches!(
        Film::parse([
            RawFilmChunk {
                kind: ChunkKind::Registry.into(),
                index: 0,
                start_ms: 0,
                data: zlib(&41u32.to_le_bytes())
            },
            crate::theater::parser::test_support::empty_replication(),
            indexed_summary(),
        ]),
        Err(ParseError::TruncatedRegistryHeader)
    ));
    assert_eq!(RawChunkKind::from(99), RawChunkKind(99));
    assert!(matches!(
        ChunkKind::try_from(RawChunkKind(99)),
        Err(ParseError::UnsupportedChunkKind(99))
    ));
    assert!(matches!(
        ChunkKind::try_from(RawChunkKind(-1)),
        Err(ParseError::UnsupportedChunkKind(-1))
    ));
}

#[test]
fn a_finished_film_requires_recorded_map_ids() {
    assert!(matches!(
        Film::parse([
            registry(),
            crate::theater::parser::test_support::empty_replication(),
            indexed_summary(),
        ]),
        Err(ParseError::MissingRegistryMap)
    ));
}

#[test]
fn a_finished_film_requires_complete_game_options() {
    let mut registry = crate::theater::parser::test_support::registry_with_complete_setup();
    let mut data = registry.clone().inflate().unwrap().data;
    let body = crate::theater::parser::v41::chunks::registry::parse_registry_chunk(&data).unwrap();
    let end_bit = body
        .bootstrap_header
        .unwrap()
        .identification
        .unwrap()
        .end_bit;
    data.truncate(end_bit.div_ceil(8));
    registry.data = zlib(&data);
    assert!(matches!(
        Film::parse([
            registry,
            crate::theater::parser::test_support::empty_replication(),
            indexed_summary(),
        ]),
        Err(ParseError::IncompleteRegistryOptions)
    ));
}

#[test]
fn missing_summary_is_rejected() {
    let input = RawFilmChunk {
        kind: ChunkKind::Replication.into(),
        index: 1,
        start_ms: 1200,
        data: packet(0, &[], 1_200_000),
    };
    assert!(matches!(
        Film::parse([registry(), input]),
        Err(ParseError::SummaryNotLast)
    ));
}

#[test]
fn parse_orders_chunks_by_manifest_index() {
    let first = crate::theater::parser::test_support::empty_replication();
    let second = RawFilmChunk {
        index: 2,
        ..crate::theater::parser::test_support::empty_replication()
    };
    let summary = RawFilmChunk {
        index: 3,
        ..empty_summary()
    };
    let film = parse_structural([summary, second, registry(), first]).unwrap();
    assert_eq!(film.registry.index, 0);
    assert_eq!(
        film.replication_stream
            .0
            .iter()
            .map(|c| c.index)
            .collect::<Vec<_>>(),
        [1, 2]
    );
    assert!(film.summary.body.packets.is_empty());
}

#[test]
fn parse_requires_downloaded_zlib_bytes() {
    let mut clear_registry = registry();
    clear_registry.data = clear_registry.clone().inflate().unwrap().data;
    assert!(matches!(
        Film::parse([
            clear_registry,
            crate::theater::parser::test_support::empty_replication(),
            indexed_summary()
        ]),
        Err(ParseError::Inflate {
            index: 0,
            cause: InflateError::Header
        })
    ));
}

#[test]
fn sections_preserve_transport_metadata_positions_and_unknown_bytes() {
    let bootstrap = crate::theater::parser::test_support::registry_with_complete_setup()
        .inflate()
        .unwrap()
        .data;
    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(&bootstrap).unwrap();
    let mut compressed = encoder.finish().unwrap();
    compressed.extend(b"transport suffix");
    let mut stream = packet(99, b"opaque", 0);
    stream.extend([0xff; 3]);
    let chunks = vec![
        RawFilmChunk {
            kind: ChunkKind::Registry.into(),
            index: 0,
            start_ms: 0,
            data: compressed.clone(),
        },
        RawFilmChunk {
            kind: ChunkKind::Replication.into(),
            index: 1,
            start_ms: 0,
            data: zlib(&stream),
        },
        RawFilmChunk {
            kind: ChunkKind::Summary.into(),
            index: 8,
            start_ms: -10,
            data: zlib(&packet(9, &0u32.to_be_bytes(), 200)),
        },
    ];
    let parsed = parse_structural(chunks.clone()).unwrap();
    let film = parse_typed_structural(chunks.clone()).unwrap();
    assert_eq!(parsed.registry.index, 0);
    assert_eq!(parsed.registry.data, bootstrap);
    assert_eq!(parsed.replication_stream.0.first().unwrap().index, 1);
    assert!(matches!(
        parsed.replication_stream.0.first().unwrap().body.packets[0].body,
        PacketRead::Opaque {
            reason: PacketDecodeError::UnsupportedLayout { packet_type: 99 }
        }
    ));
    assert!(film.summary.highlights.is_empty());
    for input in &chunks {
        let (index, start_ms, data) = match input.kind {
            RawChunkKind::REGISTRY => (
                parsed.registry.index,
                parsed.registry.start_ms,
                &parsed.registry.data,
            ),
            RawChunkKind::SUMMARY => continue,
            RawChunkKind::REPLICATION => {
                let c = &parsed.replication_stream.0[0];
                (c.index, c.start_ms, &c.data)
            }
            _ => unreachable!(),
        };
        assert_chunk_bytes(input, index, start_ms, data);
    }
    let json = serde_json::to_value(&film).unwrap();
    assert_eq!(
        json.as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        vec!["registry", "replication_stream", "summary"]
    );
    assert_eq!(serde_json::from_value::<Film>(json).unwrap(), film);
    let resolved = crate::theater::TheaterRuntime::load(film.clone());
    assert!(resolved.observations().is_empty());
}

#[test]
fn no_archetype_keyframe_header_must_fit_the_payload() {
    // One table configuration bit, a valid namespaced ID, then a vacant
    // archetype word. Its remaining 44 header bits must still be present.
    let mut header_bits = vec![false];
    for word in [0x4000_0007u32, u32::MAX] {
        header_bits.extend((0..32).rev().map(|shift| word & (1 << shift) != 0));
    }
    for (length, expected_headers) in [(9, 0), (14, 1)] {
        let mut payload = vec![0; length];
        for (bit, set) in header_bits.iter().enumerate() {
            if *set {
                payload[bit / 8] |= 1 << (7 - bit % 8);
            }
        }
        let film = parse_structural([
            registry(),
            RawFilmChunk {
                kind: ChunkKind::Replication.into(),
                index: 1,
                start_ms: 0,
                data: zlib(&packet(2, &payload, 10)),
            },
            empty_summary(),
        ])
        .unwrap();
        let chunk = film.replication_stream.0.first().unwrap();
        let PacketRead::Decoded(ReplicationStreamPacketBody::KeyframesPacketBody(table)) =
            &chunk.body.packets[0].body
        else {
            panic!()
        };
        assert_eq!(table.stop, KeyframeChainStop::Truncated);
        assert!(table.records.is_empty());
        assert_eq!(
            table.slots.as_ref().unwrap().headers.len(),
            expected_headers
        );
        assert_eq!(chunk.payload(&chunk.body.packets[0]).unwrap(), payload);
    }
}

#[test]
fn keyframes_stop_without_searching_past_invalid_header() {
    let mut payload = vec![0; 32];
    // Invalid whole archetype at bit 33; a zero-generation ID alone is valid.
    payload[4..8].fill(0x7f);
    payload[16..20].copy_from_slice(&0x40000007u32.to_be_bytes());
    payload[20..24].copy_from_slice(&3u32.to_be_bytes());
    let film = parse_structural([
        registry(),
        RawFilmChunk {
            kind: ChunkKind::Replication.into(),
            index: 1,
            start_ms: 0,
            data: zlib(&packet(2, &payload, 10)),
        },
        empty_summary(),
    ])
    .unwrap();
    let PacketRead::Decoded(ReplicationStreamPacketBody::KeyframesPacketBody(table)) =
        &film.replication_stream.0.first().unwrap().body.packets[0].body
    else {
        panic!()
    };
    assert_eq!(table.stop, KeyframeChainStop::Header);
    assert!(table.records.is_empty());
    assert_eq!(
        film.replication_stream.0.first().unwrap().data,
        packet(2, &payload, 10)
    );
    assert!(
        film.replication_stream.0[0].body.packets[0]
            .body
            .decoded()
            .is_some()
    );
}

#[test]
fn missing_event_version_keeps_source_prefix() {
    // The short synthetic frame lacks the registry event-version context.
    // Keep its source-backed prefix rather than claiming a later gate.
    let mut bits = vec![false, true];
    for bit in (0..5).rev() {
        bits.push((15 >> bit) & 1 != 0);
    }
    bits.extend([false; 121]);
    let mut payload = vec![0; bits.len().div_ceil(8)];
    for (i, bit) in bits.into_iter().enumerate() {
        if bit {
            payload[i / 8] |= 1 << (7 - i % 8);
        }
    }
    let film = parse_structural([
        registry(),
        RawFilmChunk {
            kind: ChunkKind::Replication.into(),
            index: 1,
            start_ms: 0,
            data: zlib(&packet(0, &payload, 10)),
        },
        empty_summary(),
    ])
    .unwrap();
    let PacketRead::Decoded(ReplicationStreamPacketBody::FramePacketBody(frame)) =
        &film.replication_stream.0.first().unwrap().body.packets[0].body
    else {
        panic!("missing frame packet")
    };
    let read = &frame.events;
    assert_eq!(read.gate15, None);
    assert_eq!(
        read.stop,
        EventListStop::MissingEventVersion { event_code: 0 }
    );
    assert_eq!(read.records.len(), 1);
    assert!(read.records[0].end_bit > read.records[0].start_bit);
    assert!(!read.records[0].layout_complete);
}

#[test]
#[ignore = "requires the separately retained 32-film v41 corpus"]
fn captured_v41_corpus() {
    check_captured_v41_corpus(true);
}

#[test]
#[ignore = "requires the separately retained 32-film v41 corpus"]
fn captured_v41_source_coverage() {
    check_captured_v41_corpus(false);
}

#[test]
#[ignore = "requires the retained Octagon film"]
fn captured_first_frame_clear_creation_gate_matches_recorded_successors() {
    let root = std::env::var_os("HALO_FILM_CORPUS")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("experiments/films")
        });
    let folder = root.join("octagon/03-first-to-50");
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(folder.join("film.json")).unwrap()).unwrap();
    let film = parse_structural(manifest["chunks"].as_array().unwrap().iter().map(|row| {
        RawFilmChunk {
            kind: crate::clients::hi::models::RawChunkKind(
                i32::try_from(row["chunk_type"].as_i64().unwrap()).unwrap(),
            ),
            index: i32::try_from(row["index"].as_i64().expect("film chunk index"))
                .expect("film chunk index fits i32"),
            start_ms: row["start_time_offset_ms"]
                .as_i64()
                .expect("film chunk start time"),
            data: std::fs::read(folder.join(row["file"].as_str().unwrap())).unwrap(),
        }
    }))
    .unwrap();
    let chunk = &film.replication_stream.0[0];
    let packet = &chunk.body.packets[8];
    let PacketRead::Decoded(ReplicationStreamPacketBody::FramePacketBody(frame)) = &packet.body
    else {
        panic!("expected first type-0 body");
    };
    let FrameRead::Decoded(view) = &frame.frame else {
        panic!("expected decoded first frame");
    };
    assert_eq!(frame.events.end_bit, 2);
    assert_eq!(view.records.len(), 207);
    assert_eq!(view.records[0].header.kind, RecordKind::New);
    assert_eq!(view.records[0].header.id, Some(0x4000_0000));
    assert_eq!(view.records[0].archetype, Some(22));
    assert_eq!(view.records[0].end_bit, 27);
    assert_eq!(view.records[0].mask, None);
    assert_eq!(view.records[1].header.start_bit, 27);
    assert!(matches!(
        view.entity_end,
        Some(ProductionEntityEnd::Marker(_))
    ));
    assert_eq!(view.end_bit, 53_521);
    assert!(packet.source_coverage(&chunk.data).is_some());
}

// Source fidelity and the historical reader oracle are distinct checks. Run
// source fidelity across the whole corpus even when the old oracle disagrees.
fn check_captured_v41_corpus(compare_reference: bool) {
    #[derive(Deserialize)]
    struct Baseline {
        file: String,
        offset: usize,
        views: usize,
        end: i64,
    }
    let fixture = std::fs::File::open(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/theater/fixtures/native-baseline-d61443e-v41.jsonl.zlib"),
    )
    .unwrap();
    let mut expected = std::collections::BTreeMap::new();
    for line in std::io::BufReader::new(flate2::read::ZlibDecoder::new(fixture)).lines() {
        let row: Baseline = serde_json::from_str(&line.unwrap()).unwrap();
        expected.insert((row.file, row.offset), (row.views, row.end));
    }
    #[derive(Clone, Deserialize)]
    struct ReferenceSlot {
        #[serde(rename = "TypeIndex")]
        archetype: u32,
        #[serde(rename = "FullID")]
        full_id: u32,
        #[serde(rename = "Soft")]
        soft: bool,
        #[serde(rename = "GenAny")]
        generation_any: bool,
        #[serde(rename = "Vue")]
        view: i8,
    }
    #[derive(Clone, Deserialize)]
    struct ReferenceContext {
        slots: std::collections::BTreeMap<u32, ReferenceSlot>,
        namespace: i8,
    }
    impl ReferenceContext {
        fn world(&self) -> ReplicationDecodeState {
            ReplicationDecodeState {
                slots: self
                    .slots
                    .iter()
                    .map(|(&slot, binding)| {
                        (
                            slot,
                            crate::theater::parser::v41::chunks::replication::state::DecodeBinding {
                                archetype: binding.archetype,
                                full_id: binding.full_id,
                                soft: binding.soft,
                                generation_any: binding.generation_any,
                                view: (binding.view >= 0).then_some(binding.view),
                            },
                        )
                    })
                    .collect(),
                keyframe_namespace: u8::try_from(self.namespace).ok(),
                ..Default::default()
            }
        }
    }
    #[derive(Deserialize)]
    struct ReferenceRuntimeComponent {
        record: usize,
        name: String,
        start_bit: i64,
    }
    #[derive(Deserialize)]
    struct ContextRow {
        #[serde(flatten)]
        baseline: Baseline,
        context: Option<ReferenceContext>,
        #[serde(default)]
        runtime_components: Option<Vec<ReferenceRuntimeComponent>>,
    }
    let fixture = std::fs::File::open(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/theater/fixtures/reference-contexts-d61443e-v41.jsonl.zlib"),
    )
    .unwrap();
    let mut contexts = std::collections::BTreeMap::new();
    for line in std::io::BufReader::new(flate2::read::ZlibDecoder::new(fixture)).lines() {
        let row: ContextRow = serde_json::from_str(&line.unwrap()).unwrap();
        let key = (row.baseline.file, row.baseline.offset);
        assert_eq!(
            expected.get(&key),
            Some(&(row.baseline.views, row.baseline.end))
        );
        assert!(
            contexts
                .insert(
                    key,
                    (row.context, row.runtime_components.unwrap_or_default())
                )
                .is_none()
        );
    }
    assert_eq!(contexts.len(), expected.len());
    fn assert_source_fields(payload: &[u8], fields: &[ComponentField]) {
        for field in fields {
            assert!(field.bit.checked_add(field.width).unwrap() <= payload.len() * 8);
            assert_eq!(field.raw.bit_len, field.width);
            assert_eq!(field.raw.bytes.len(), field.width.div_ceil(8));
            for index in 0..field.width {
                let source = field.bit + index;
                assert_eq!(
                    (payload[source / 8] >> (7 - source % 8)) & 1,
                    (field.raw.bytes[index / 8] >> (7 - index % 8)) & 1,
                    "field {} source bit {}",
                    field.name,
                    source,
                );
            }
        }
    }
    fn manifests(root: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        for e in std::fs::read_dir(root).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                manifests(&p, out)
            } else if p.file_name().unwrap() == "film.json" {
                out.push(p)
            }
        }
    }
    let root = std::env::var_os("HALO_FILM_CORPUS")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("experiments/films")
        });
    let mut paths = Vec::new();
    manifests(&root, &mut paths);
    paths.sort();
    assert_eq!(paths.len(), 32);
    let (mut frames, mut summaries, mut runtime_refusals) = (0, 0, 0);
    let mut control_kinds = [0usize; 3];
    let mut secondary_blocks = 0;
    let (mut full_secondary_blocks, mut local_secondary_positions, mut secondary_targets) =
        (0, 0, 0);
    let (mut maximum_control_records, mut control_budget_stops) = (0, 0);
    let (mut keyframe_tables, mut closed_tables, mut vacant_headers, mut later_records) =
        (0, 0, 0, 0);
    let mut keyframe_stops = std::collections::BTreeMap::<String, usize>::new();
    for path in paths {
        let manifest: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        let mut files = Vec::new();
        let input: Vec<_> = manifest["chunks"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| {
                let p = path.parent().unwrap().join(c["file"].as_str().unwrap());
                files.push(p.strip_prefix(&root).unwrap().to_str().unwrap().to_owned());
                RawFilmChunk {
                    kind: crate::clients::hi::models::RawChunkKind(
                        i32::try_from(c["chunk_type"].as_i64().unwrap()).unwrap(),
                    ),
                    index: i32::try_from(c["index"].as_i64().expect("film chunk index"))
                        .expect("film chunk index fits i32"),
                    start_ms: c["start_time_offset_ms"]
                        .as_i64()
                        .expect("film chunk start time"),
                    data: std::fs::read(p).unwrap(),
                }
            })
            .collect();
        let typed = Film::parse(input.clone()).unwrap();
        let film = parse_structural(input.clone()).unwrap();
        assert_eq!(film.registry.body.registry.archetypes.len(), 50);
        assert_eq!(
            film.registry
                .body
                .bootstrap_header
                .as_ref()
                .and_then(|h| h.corruption_checks()),
            Some(false),
            "retained corpus guard policy"
        );
        let mut reference_context: Option<ReferenceContext> = None;
        let mut baseline_config = V41DecodeConfig::default();
        // Preserve the pinned historical policy; canonical simulation bodies
        // now have an independent complete grammar and mode/point fixtures.
        baseline_config.profile.grammar.simulation_complete = false;
        for (i, original) in input.iter().enumerate() {
            match original.kind {
                RawChunkKind::REGISTRY => assert_chunk_bytes(
                    original,
                    film.registry.index,
                    film.registry.start_ms,
                    &film.registry.data,
                ),
                RawChunkKind::SUMMARY => {
                    let chunk =
                        crate::theater::parser::v41::chunks::summary::V41SummaryChunkReader::read(
                            original.clone().inflate().unwrap(),
                            ParseLimits::default().max_packets_per_chunk,
                        )
                        .unwrap();
                    assert_eq!((chunk.index), i);
                    assert_chunk_bytes(original, chunk.index, chunk.start_ms, &chunk.data);
                    for packet in &chunk.body.packets {
                        assert!(packet.source_coverage(&chunk.data).is_some());
                    }
                }
                RawChunkKind::REPLICATION => {
                    let chunk = film
                        .replication_stream
                        .0
                        .iter()
                        .find(|chunk| (chunk.index) == i)
                        .unwrap();
                    assert_chunk_bytes(original, chunk.index, chunk.start_ms, &chunk.data);
                    for packet in &chunk.body.packets {
                        let payload = chunk.payload(packet).unwrap();
                        assert!(
                            packet.source_coverage(&chunk.data).is_some(),
                            "coverage {}:{}",
                            files[i],
                            packet.source.payload.start
                        );
                        if let PacketRead::Decoded(
                            ReplicationStreamPacketBody::KeyframesPacketBody(table),
                        ) = &packet.body
                        {
                            keyframe_tables += 1;
                            closed_tables += usize::from(table.stop == KeyframeChainStop::End);
                            let slots = table.slots.as_ref().unwrap();
                            assert!(slots.headers.len() <= 8191);
                            let mut seen_vacant = false;
                            for h in &slots.headers {
                                for (offset, width, value) in [
                                    (0, 32, h.id),
                                    (32, 32, h.archetype),
                                    (64, 32, h.word),
                                    (96, 4, u32::from(h.flags)),
                                    (100, 8, u32::from(h.byte)),
                                ] {
                                    assert_eq!(
                                        crate::theater::parser::bits::Bits(payload)
                                            .read(h.start_bit + offset, width),
                                        Some(u64::from(value))
                                    );
                                }
                                if h.archetype == u32::MAX {
                                    vacant_headers += 1;
                                    seen_vacant = true;
                                } else if seen_vacant {
                                    later_records += 1;
                                }
                            }
                            if table.stop == KeyframeChainStop::End {
                                assert_eq!(slots.headers.len(), 8191);
                                assert!(slots.tail.end - slots.tail.start < 8);
                            }
                            *keyframe_stops
                                .entry(format!("{:?}", table.stop))
                                .or_default() += 1;
                            for attempt in &table.records {
                                assert!(
                                    attempt.start_bit >= 0
                                        && attempt.end_bit <= (payload.len() * 8) as i64
                                );
                                if let Some(record) = &attempt.record {
                                    assert_source_fields(payload, &record.fields);
                                    if let Some(default_state) = &record.default_state {
                                        assert_source_fields(payload, &default_state.fields);
                                        assert!(
                                            default_state.source.start <= default_state.source.end
                                                && default_state.source.end <= payload.len() * 8
                                        );
                                    }
                                    for component in &record.components {
                                        assert_source_fields(payload, &component.fields);
                                    }
                                }
                            }
                        }
                        if let PacketRead::Decoded(ReplicationStreamPacketBody::FramePacketBody(
                            actual,
                        )) = &packet.body
                        {
                            if let Some(frame) = actual.frame.decoded() {
                                assert!(
                                    frame.end_bit >= 0
                                        && frame.end_bit <= (payload.len() * 8) as i64
                                );
                                for record in &frame.records {
                                    assert!(
                                        record.header.start_bit >= 0
                                            && record.header.end_bit <= record.end_bit
                                    );
                                    assert!(record.end_bit <= (payload.len() * 8) as i64);
                                    assert_source_fields(payload, &record.fields);
                                    if let Some(default_state) = &record.default_state {
                                        assert_source_fields(payload, &default_state.fields);
                                        assert!(
                                            default_state.source.start <= default_state.source.end
                                                && default_state.source.end <= payload.len() * 8
                                        );
                                    }
                                    for component in &record.components {
                                        assert_source_fields(payload, &component.fields);
                                    }
                                }
                                if let Some(controls) = &frame.controls {
                                    assert_source_fields(payload, &controls.fields);
                                    maximum_control_records =
                                        maximum_control_records.max(controls.control_entries.len());
                                    control_budget_stops +=
                                        usize::from(controls.stop == FrameViewStop::RecordLimit);
                                    for entry in &controls.control_entries {
                                        assert!(controls.start_bit <= entry.start_bit);
                                        assert!(entry.start_bit <= entry.end_bit);
                                        assert!(entry.end_bit <= controls.end_bit);
                                        match &entry.payload {
                                            ControlPayload::Input(input) => {
                                                control_kinds[0] += 1;
                                                if let Some(range) = &input.secondary {
                                                    assert!(entry.start_bit <= range.start as i64);
                                                    assert!(range.start < range.end);
                                                    assert!(range.end as i64 <= entry.end_bit);
                                                    secondary_blocks += 1;
                                                    let field = |name: &str| {
                                                        controls.fields.iter().find(|f| {
                                                            f.bit >= range.start
                                                                && f.bit + f.width <= range.end
                                                                && f.name == name
                                                        })
                                                    };
                                                    let state =
                                                        field("control.secondary.state").unwrap();
                                                    if state.raw.low_u64() & 2 != 0 {
                                                        full_secondary_blocks += 1;
                                                        local_secondary_positions += usize::from(
                                                            field(
                                                                "control.secondary.local_position",
                                                            )
                                                            .unwrap()
                                                            .raw
                                                            .low_u64()
                                                                != 0,
                                                        );
                                                        secondary_targets += usize::from(
                                                            field(
                                                                "control.secondary.target.present",
                                                            )
                                                            .unwrap()
                                                            .raw
                                                            .low_u64()
                                                                != 0,
                                                        );
                                                    }
                                                }
                                            }
                                            ControlPayload::Commands { records, .. } => {
                                                control_kinds[1] += 1;
                                                let mut end = entry.start_bit;
                                                for range in records.iter().flatten() {
                                                    assert!(end <= range.start as i64);
                                                    assert!(range.start < range.end);
                                                    assert!(range.end as i64 <= entry.end_bit);
                                                    end = range.end as i64;
                                                }
                                            }
                                            ControlPayload::Kind2 { .. } => {
                                                control_kinds[2] += 1;
                                                assert_eq!(entry.index(), None);
                                            }
                                        }
                                    }
                                }
                            }
                            if !compare_reference {
                                frames += 1;
                                continue;
                            }
                            let key = (files[i].clone(), packet.source.payload.start);
                            let (context, runtime_components) = contexts.remove(&key).unwrap();
                            if let Some(context) = context {
                                reference_context = Some(context);
                            }
                            // Compare reader behavior under the same independently
                            // captured context, not one changed by earlier bounded reads.
                            let mut baseline_world = reference_context.as_ref().unwrap().world();
                            let mut baseline = baseline_config
                                .decode_production_views(
                                    payload,
                                    2,
                                    &film.registry.body.registry,
                                    &mut baseline_world,
                                )
                                .unwrap();
                            // The d61443e oracle predates control kinds 1/2 and
                            // secondary decoding, and continues through kind 3.
                            // Keep that historical comparison exact. Production
                            // Film fields/coverage were checked above; upgraded
                            // controls have a separate pinned reference reader oracle.
                            if let Some(control) = &baseline.controls {
                                let mut legacy = crate::theater::parser::v41::reference::controls::decode_control_view_contextual(
                                    payload,
                                    control.start_bit,
                                    baseline_config.record_layout().position.as_ref(),
                                    Some(&baseline_config.profile),
                                );
                                let source_end = (payload.len() * 8) as i64;
                                if legacy.end_bit > source_end {
                                    legacy.end_bit = source_end;
                                    legacy.stop = FrameViewStop::Truncated;
                                }
                                baseline.end_bit = legacy.end_bit;
                                baseline.controls = Some(legacy);
                            }
                            let oracle = expected
                                .remove(&(files[i].clone(), packet.source.payload.start))
                                .unwrap();
                            if let Some(ProductionEntityEnd::Failure(
                                EntityViewStop::RuntimeContextUnavailable { index, name, field },
                            )) = &baseline.entity_end
                            {
                                assert_eq!(name, "vehicle-type-physics-component");
                                assert_eq!(field, "vehicle+0x818");
                                let record_index = baseline.records.len() - 1;
                                let record = &baseline.records[record_index];
                                let component = record.components.last().unwrap();
                                let reference = runtime_components
                                    .iter()
                                    .find(|component| {
                                        component.record == record_index && component.name == *name
                                    })
                                    .unwrap();
                                assert_eq!(component.index, *index);
                                assert_eq!(component.start_bit, reference.start_bit);
                                assert_eq!(component.start_bit, component.end_bit);
                                assert_eq!(baseline.end_bit, reference.start_bit);
                                assert!(component.fields.is_empty());
                                assert_eq!(component.status, ComponentReadStatus::Unsupported);
                                assert!(
                                    crate::theater::parser::v41::chunks::replication::replication_stream::frame::entities::completed_views(
                                        &baseline
                                    ) <= oracle.0
                                );
                                runtime_refusals += 1;
                            } else if oracle.1 <= (payload.len() * 8) as i64 {
                                assert_eq!(
                                        (crate::theater::parser::v41::chunks::replication::replication_stream::frame::entities::completed_views(&baseline), baseline.end_bit),
                                        oracle,
                                        "{}:{} actual={:?}",
                                        files[i],
                                        packet.source.payload.start,
                                        baseline
                                    );
                            } else {
                                // The pinned reader consumed synthetic tail bits. The
                                // canonical reader must stay bounded and stop explicitly.
                                assert!(baseline.end_bit <= (payload.len() * 8) as i64);
                                assert!(
                                    crate::theater::parser::v41::chunks::replication::replication_stream::frame::entities::completed_views(
                                        &baseline
                                    ) <= oracle.0,
                                    "{}:{} actual={:?} oracle={:?}",
                                    files[i],
                                    packet.source.payload.start,
                                    baseline,
                                    oracle
                                );
                                assert!(
                                    matches!(
                                        baseline.entity_end,
                                        Some(
                                            ProductionEntityEnd::Truncated
                                                | ProductionEntityEnd::PayloadBoundary
                                                | ProductionEntityEnd::Failure(_)
                                        )
                                    ) || baseline
                                        .controls
                                        .as_ref()
                                        .is_some_and(|c| c.stop == FrameViewStop::Truncated)
                                );
                            }
                            frames += 1;
                        }
                    }
                }
                _ => {}
            }
        }
        let resolved = crate::theater::TheaterRuntime::load(typed);
        summaries += resolved.summary_highlights().len();
        eprintln!("verified {}", path.strip_prefix(&root).unwrap().display());
    }
    assert_eq!(frames, 403465);
    eprintln!(
        "keyframe tables={keyframe_tables}, closed={closed_tables}, vacant headers={vacant_headers}, live headers after vacant={later_records}, stops={keyframe_stops:?}"
    );
    if compare_reference {
        assert!(expected.is_empty());
        assert!(contexts.is_empty());
        assert!(runtime_refusals > 0);
        eprintln!(
            "verified {frames} reference contexts; {runtime_refusals} source-verified runtime refusals"
        );
    } else {
        eprintln!("verified source coverage across {frames} frame packets");
        eprintln!(
            "complete control kinds 0/1/2: {control_kinds:?}; secondary blocks: {secondary_blocks}; maximum records per view: {maximum_control_records}; budget stops: {control_budget_stops}"
        );
        eprintln!(
            "complete full secondary blocks: {full_secondary_blocks}; local positions: {local_secondary_positions}; present targets: {secondary_targets}"
        );
    }
    assert_eq!(summaries, 3667);
}

use crate::theater::parser::v41::chunks::replication::state::ReplicationDecodeState;
use crate::theater::parser::v41::context::V41DecodeConfig;

#[test]
fn packet_payload_rejects_inconsistent_public_envelopes() {
    let bytes = packet(7, &[], 123);
    let valid = ReplicationStreamPacket {
        source: PacketSource {
            header: ByteRange { start: 0, end: 16 },
            payload: ByteRange { start: 16, end: 16 },
        },
        header: FilmPacketHeader {
            packet_type: 7,
            unknown_2: [0xab, 0xcd],
            payload_size: 0,
            timestamp_us: 123,
        },
        body: PacketRead::Decoded(ReplicationStreamPacketBody::EndPacketBody),
    };
    assert_eq!(valid.payload(&bytes), Some([].as_slice()));
    let mut invalid = valid.clone();
    invalid.header.timestamp_us = 124;
    assert!(invalid.payload(&bytes).is_none());
    let mut invalid = valid.clone();
    invalid.source.header.start = usize::MAX;
    assert!(invalid.payload(&bytes).is_none());
    let mut invalid = valid.clone();
    invalid.source.payload.start = 15;
    assert!(invalid.payload(&bytes).is_none());
    let mut invalid = valid.clone();
    invalid.source.payload.end = 17;
    assert!(invalid.payload(&bytes).is_none());
    let mut invalid = valid.clone();
    invalid.body = PacketRead::Decoded(ReplicationStreamPacketBody::DatumsPacketBody(Box::new(
        DatumTable {
            entries: vec![],
            component_masks: vec![],
            tail_words: [0; 5],
            tail_source: BitRange { start: 0, end: 0 },
            padding_source: BitRange { start: 0, end: 0 },
            padding: 0,
        },
    )));
    assert!(invalid.payload(&bytes).is_none());
    assert!(valid.payload(&bytes[..15]).is_none());
}

#[test]
fn source_coverage_keeps_opaque_summary_and_unparsed_partial_event_bits() {
    let summary_payload = [2u32.to_be_bytes().as_slice(), &[0xaa, 0xbb]].concat();
    let summary_chunk = RawFilmChunk {
        kind: ChunkKind::Summary.into(),
        index: 2,
        start_ms: 0,
        data: zlib(&packet(9, &summary_payload, 0)),
    };
    let summary = crate::theater::parser::v41::chunks::summary::V41SummaryChunkReader::read(
        summary_chunk.clone().inflate().unwrap(),
        ParseLimits::default().max_packets_per_chunk,
    )
    .unwrap();
    let film = parse_structural([
        crate::theater::parser::test_support::zlib_fixture_chunk(
            crate::theater::parser::test_support::registry_with_corruption_policy(false),
        ),
        RawFilmChunk {
            kind: ChunkKind::Replication.into(),
            index: 1,
            start_ms: 0,
            // configuration=1, continuation=1, code=85, references=000.
            // Its bounded partial body does not establish the remaining layout.
            data: zlib(&packet(0, &[0xea, 0x8e], 0)),
        },
        summary_chunk,
    ])
    .unwrap();
    let coverage = summary.body.packets[0]
        .source_coverage(&summary.data)
        .unwrap();
    assert_eq!(
        coverage,
        vec![
            SourceRegion {
                source: BitRange { start: 0, end: 32 },
                kind: SourceRegionKind::Fields
            },
            SourceRegion {
                source: BitRange { start: 32, end: 48 },
                kind: SourceRegionKind::Opaque
            },
        ]
    );
    let replication = film.replication_stream.0.first().unwrap();
    let coverage = replication.body.packets[0]
        .source_coverage(&replication.data)
        .unwrap();
    assert_eq!(
        coverage,
        vec![
            SourceRegion {
                source: BitRange { start: 0, end: 14 },
                kind: SourceRegionKind::Fields
            },
            SourceRegion {
                source: BitRange { start: 14, end: 16 },
                kind: SourceRegionKind::Unparsed
            }
        ]
    );
    // More source bytes still cannot complete the preceding 32-bit field.
    // Later bit patterns must not select flags or references after that failure.
    let film = parse_structural([
        crate::theater::parser::test_support::zlib_fixture_chunk(
            crate::theater::parser::test_support::registry_with_corruption_policy(false),
        ),
        RawFilmChunk {
            kind: ChunkKind::Replication.into(),
            index: 1,
            start_ms: 0,
            data: zlib(&packet(0, &[0xea, 0x8f, 0], 0)),
        },
        empty_summary(),
    ])
    .unwrap();
    let chunk = film.replication_stream.0.first().unwrap();
    let coverage = chunk.body.packets[0].source_coverage(&chunk.data).unwrap();
    assert_eq!(
        coverage.last().unwrap(),
        &SourceRegion {
            source: BitRange { start: 14, end: 24 },
            kind: SourceRegionKind::Unparsed,
        }
    );
}
