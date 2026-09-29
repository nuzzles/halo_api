use super::*;
use std::io::{BufRead, Write};

fn registry() -> FilmChunk {
    FilmChunk {
        kind: ChunkKind::Registry,
        index: None,
        start_ms: None,
        data: [41u32.to_le_bytes(), 27u32.to_le_bytes()].concat(),
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
fn dispatch_requires_one_registry_and_rejects_unsupported_versions() {
    assert!(matches!(Film::parse([]), Err(ParseError::MissingRegistry)));
    for kind in [ChunkKind::Replication, ChunkKind::Summary] {
        assert!(matches!(
            Film::parse([
                FilmChunk {
                    kind,
                    index: None,
                    start_ms: None,
                    data: Vec::new()
                },
                registry()
            ]),
            Err(ParseError::RegistryNotFirst)
        ));
    }
    let only_registry = Film::parse([registry()]).unwrap();
    assert!(only_registry.replication.chunks.is_empty());
    assert!(only_registry.summaries.chunks.is_empty());
    assert!(matches!(
        Film::parse([registry(), registry()]),
        Err(ParseError::MultipleRegistries)
    ));
    assert!(matches!(
        Film::parse([FilmChunk {
            kind: ChunkKind::Registry,
            index: None,
            start_ms: None,
            data: 75u32.to_le_bytes().to_vec()
        }]),
        Err(ParseError::UnsupportedVersion(75))
    ));
    assert!(matches!(
        Film::parse([FilmChunk {
            kind: ChunkKind::Registry,
            index: None,
            start_ms: None,
            data: vec![41]
        }]),
        Err(ParseError::TruncatedRegistryHeader)
    ));
    assert!(matches!(
        ChunkKind::try_from(99),
        Err(ParseError::ChunkKind(99))
    ));
}

#[test]
fn sections_preserve_transport_metadata_positions_and_unknown_bytes() {
    let bootstrap = registry().data;
    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(&bootstrap).unwrap();
    let mut compressed = encoder.finish().unwrap();
    compressed.extend(b"transport suffix");
    let mut stream = packet(99, b"opaque", 0);
    stream.extend([0xff; 3]);
    let chunks = vec![
        FilmChunk {
            kind: ChunkKind::Registry,
            index: None,
            start_ms: None,
            data: compressed.clone(),
        },
        FilmChunk {
            kind: ChunkKind::Summary,
            index: Some(8),
            start_ms: Some(-10),
            data: packet(9, &0u32.to_be_bytes(), 200),
        },
        FilmChunk {
            kind: ChunkKind::Replication,
            index: None,
            start_ms: None,
            data: stream.clone(),
        },
        FilmChunk {
            kind: ChunkKind::Summary,
            index: None,
            start_ms: None,
            data: packet(9, &0u32.to_be_bytes(), 100),
        },
    ];
    let film = Film::parse(chunks.clone()).unwrap();
    assert_eq!(film.registry.source_position, 0);
    assert_eq!(film.registry.source.data, compressed);
    assert_eq!(film.registry.data, bootstrap);
    assert_eq!(film.replication.chunks[0].source_position, 2);
    assert!(matches!(
        film.replication.chunks[0].packets[0].body,
        FilmPacketBody::Opaque
    ));
    assert_eq!(
        film.summaries
            .chunks
            .iter()
            .map(|c| c.source_position)
            .collect::<Vec<_>>(),
        vec![1, 3]
    );
    for (i, input) in chunks.iter().enumerate() {
        assert_eq!(film.chunk(i).unwrap().source(), input);
    }
    let json = serde_json::to_value(&film).unwrap();
    assert_eq!(
        json.as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        vec!["registry", "replication", "summaries"]
    );
    assert_eq!(serde_json::from_value::<Film>(json).unwrap(), film);
    let resolved = film.resolve();
    assert!(std::ptr::eq(resolved.film(), &film));
    assert_eq!(resolved.events().first().unwrap().source.chunk, 2);
}

#[test]
fn keyframes_stop_without_searching_past_invalid_header() {
    let mut payload = vec![0; 32];
    payload[16..20].copy_from_slice(&0x40000007u32.to_be_bytes());
    payload[20..24].copy_from_slice(&3u32.to_be_bytes());
    let film = Film::parse([
        registry(),
        FilmChunk {
            kind: ChunkKind::Replication,
            index: None,
            start_ms: None,
            data: packet(2, &payload, 10),
        },
    ])
    .unwrap();
    let FilmPacketBody::Keyframes(table) = &film.replication.chunks[0].packets[0].body else {
        panic!()
    };
    assert_eq!(table.stop, KeyframeChainStop::Header);
    assert!(table.records.is_empty());
    assert_eq!(film.replication.chunks[0].data, packet(2, &payload, 10));
    assert!(film.resolve().advance_to(10).entities.is_empty());
}

#[test]
fn event_gate_is_unresolved_in_native_film() {
    // Event list: config bit zero, present bit one, event code 15.
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
    let film = Film::parse([
        registry(),
        FilmChunk {
            kind: ChunkKind::Replication,
            index: None,
            start_ms: None,
            data: packet(0, &payload, 10),
        },
    ])
    .unwrap();
    let read = film.replication.chunks[0].packets[0]
        .event_list
        .as_ref()
        .unwrap();
    assert_eq!(read.gate15, None);
    let before = serde_json::to_value(&film).unwrap();
    let resolved = film.resolve();
    assert_eq!(
        resolved.interpretations().event_gate15.policy,
        NativeEventGate15Policy::ReferenceInference
    );
    assert_eq!(serde_json::to_value(resolved.film()).unwrap(), before);
}

#[test]
#[ignore = "requires the separately retained 32-film v41 corpus"]
fn captured_v41_corpus() {
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
    let (mut frames, mut summaries) = (0, 0);
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
                FilmChunk {
                    kind: ChunkKind::try_from(c["chunk_type"].as_i64().unwrap() as i32).unwrap(),
                    index: c["index"].as_i64(),
                    start_ms: c["start_time_offset_ms"].as_i64(),
                    data: std::fs::read(p).unwrap(),
                }
            })
            .collect();
        let film = Film::parse(input.clone()).unwrap();
        assert_eq!(film.registry.definition.registry.archetypes.len(), 50);
        let mut baseline_world = FilmWorld::default();
        let baseline_config = NativeFrameConfig::default();
        for (i, original) in input.iter().enumerate() {
            let chunk = film.chunk(i).unwrap();
            assert_eq!(chunk.source(), original);
            baseline_world.current_chunk = original.index.unwrap_or(i as i64);
            for packet in chunk.packets() {
                let payload = &chunk.data()[packet.header.payload_offset
                    ..packet.header.payload_offset + packet.header.payload_size];
                match &packet.body {
                    FilmPacketBody::Frame(_) => {
                        let baseline = baseline_config
                            .decode_production_views(
                                payload,
                                2,
                                &film.registry.definition.registry,
                                &mut baseline_world,
                            )
                            .unwrap();
                        let oracle = expected
                            .remove(&(files[i].clone(), packet.header.payload_offset))
                            .unwrap();
                        assert_eq!(
                            (baseline.views_completed, baseline.end_bit),
                            oracle,
                            "{}:{}",
                            files[i],
                            packet.header.payload_offset
                        );
                        frames += 1;
                    }
                    FilmPacketBody::Keyframes(table) => {
                        for attempt in &table.records {
                            if attempt.record.is_some() {
                                baseline_world.bind_keyframe(
                                    attempt.id >> 30,
                                    attempt.id & 0x3fffffff,
                                    attempt.archetype,
                                );
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        let resolved = film.resolve();
        let events: Vec<_> = resolved
            .query(crate::theater::resolved::EventFilter {
                kind: Some(crate::theater::resolved::EventKind::Summary),
                ..Default::default()
            })
            .collect();
        assert_eq!(events.len(), film.summaries.events().count());
        for event in events {
            let Some(crate::theater::resolved::Record::Summary(summary)) =
                resolved.record(event.source)
            else {
                panic!()
            };
            assert_eq!(event.timestamp_us, summary.time_us);
            if let Some(player) = event.player_index {
                assert_eq!(
                    resolved.player(player).unwrap().xuid.to_string(),
                    summary.xuid
                );
            }
            summaries += 1;
        }
        eprintln!("verified {}", path.strip_prefix(&root).unwrap().display());
    }
    assert!(expected.is_empty());
    assert_eq!(frames, 403465);
    assert_eq!(summaries, 3667);
}

use crate::theater::parser::v41::{FilmWorld, NativeEventGate15Policy, NativeFrameConfig};
