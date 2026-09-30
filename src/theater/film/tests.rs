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
    assert!(only_registry.chunks.is_empty());
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
    assert_eq!(ChunkKind::try_from(99).unwrap(), ChunkKind::Unknown(99));
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
    assert_eq!(film.replication_chunks().next().unwrap().source_position, 2);
    assert!(matches!(
        film.replication_chunks().next().unwrap().body.packets[0].body,
        PacketRead::Opaque {
            reason: PacketDecodeError::UnsupportedLayout { packet_type: 99 }
        }
    ));
    assert_eq!(
        film.summary_chunks()
            .map(|c| c.source_position)
            .collect::<Vec<_>>(),
        vec![1, 3]
    );
    for (i, input) in chunks.iter().enumerate() {
        let source = match input.kind {
            ChunkKind::Registry => &film.registry.source,
            _ => match &film.chunks[i - 1] {
                FilmDataChunk::Replication(chunk) => &chunk.source,
                FilmDataChunk::Summary(chunk) => &chunk.source,
                FilmDataChunk::Unknown(chunk) => &chunk.source,
            },
        };
        assert_eq!(source, input);
    }
    let json = serde_json::to_value(&film).unwrap();
    assert_eq!(
        json.as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        vec!["chunks", "registry"]
    );
    assert_eq!(serde_json::from_value::<Film>(json).unwrap(), film);
    let resolved = film.resolve();
    assert!(std::ptr::eq(resolved.film(), &film));
    assert_eq!(resolved.events().first().unwrap().source.chunk, 2);
}

#[test]
fn no_archetype_keyframe_header_must_fit_the_payload() {
    // One table configuration bit, a valid namespaced ID, then the sentinel
    // archetype word. Its remaining 44 header bits must still be present.
    let mut header_bits = vec![false];
    for word in [0x4000_0007u32, u32::MAX] {
        header_bits.extend((0..32).rev().map(|shift| word & (1 << shift) != 0));
    }
    for (length, expected_stop, expected_records) in [
        (9, KeyframeChainStop::Truncated, 0),
        (14, KeyframeChainStop::End, 1),
    ] {
        let mut payload = vec![0; length];
        for (bit, set) in header_bits.iter().enumerate() {
            if *set {
                payload[bit / 8] |= 1 << (7 - bit % 8);
            }
        }
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
        let chunk = film.replication_chunks().next().unwrap();
        let PacketRead::Decoded(ReplicationStreamPacketBody::KeyframesPacketBody(table)) =
            &chunk.body.packets[0].body
        else {
            panic!()
        };
        assert_eq!(table.stop, expected_stop);
        assert_eq!(table.records.len(), expected_records);
        for record in &table.records {
            assert!(record.end_bit <= (length * 8) as i64);
            assert!(record.record.is_none());
        }
        assert_eq!(chunk.payload(&chunk.body.packets[0]).unwrap(), payload);
    }
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
    let PacketRead::Decoded(ReplicationStreamPacketBody::KeyframesPacketBody(table)) =
        &film.replication_chunks().next().unwrap().body.packets[0].body
    else {
        panic!()
    };
    assert_eq!(table.stop, KeyframeChainStop::Header);
    assert!(table.records.is_empty());
    assert_eq!(
        film.replication_chunks().next().unwrap().data,
        packet(2, &payload, 10)
    );
    assert!(film.resolve().advance_to(10).entities.is_empty());
}

#[test]
fn event_gate_is_unresolved_in_reference_film() {
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
    let PacketRead::Decoded(ReplicationStreamPacketBody::FramePacketBody(frame)) =
        &film.replication_chunks().next().unwrap().body.packets[0].body
    else {
        panic!("missing frame packet")
    };
    let read = &frame.events;
    assert_eq!(read.gate15, None);
    let before = serde_json::to_value(&film).unwrap();
    let resolved = film.resolve();
    assert_eq!(
        resolved.interpretations().event_gate15.policy,
        EventGate15Policy::CandidateCountInference
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
        fn world(&self) -> FilmWorld {
            FilmWorld {
                slots: self
                    .slots
                    .iter()
                    .map(|(&slot, binding)| {
                        (
                            slot,
                            crate::theater::parser::v41::FilmWorldSlot {
                                archetype: binding.archetype,
                                full_id: binding.full_id,
                                soft: binding.soft,
                                generation_any: binding.generation_any,
                                position: None,
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
        assert_eq!(film.registry.body.registry.archetypes.len(), 50);
        let mut reference_context: Option<ReferenceContext> = None;
        let baseline_config = FrameConfig::default();
        for (i, original) in input.iter().enumerate() {
            match original.kind {
                ChunkKind::Registry => assert_eq!(&film.registry.source, original),
                ChunkKind::Summary => {
                    let chunk = film
                        .summary_chunks()
                        .find(|chunk| chunk.source_position == i)
                        .unwrap();
                    assert_eq!(&chunk.source, original);
                    for packet in &chunk.body.packets {
                        assert!(packet.source_coverage(&chunk.data).is_some());
                    }
                }
                ChunkKind::Replication => {
                    let chunk = film
                        .replication_chunks()
                        .find(|chunk| chunk.source_position == i)
                        .unwrap();
                    assert_eq!(&chunk.source, original);
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
                                }
                            }
                            let key = (files[i].clone(), packet.source.payload.start);
                            let (context, runtime_components) = contexts.remove(&key).unwrap();
                            if let Some(context) = context {
                                reference_context = Some(context);
                            }
                            // Compare reader behavior under the same independently
                            // captured context, not one changed by earlier bounded reads.
                            let mut baseline_world = reference_context.as_ref().unwrap().world();
                            let baseline = baseline_config
                                .decode_production_views(
                                    payload,
                                    2,
                                    &film.registry.body.registry,
                                    &mut baseline_world,
                                )
                                .unwrap();
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
                                    crate::theater::parser::v41::production_frame::completed_views(
                                        &baseline
                                    ) <= oracle.0
                                );
                                runtime_refusals += 1;
                            } else if oracle.1 <= (payload.len() * 8) as i64 {
                                assert_eq!(
                                        (crate::theater::parser::v41::production_frame::completed_views(&baseline), baseline.end_bit),
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
                                    crate::theater::parser::v41::production_frame::completed_views(
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
                ChunkKind::Unknown(_) => {}
            }
        }
        let resolved = film.resolve();
        let events: Vec<_> = resolved
            .query(crate::theater::resolved::EventFilter {
                kind: Some(crate::theater::resolved::EventKind::Summary),
                ..Default::default()
            })
            .collect();
        assert_eq!(
            events.len(),
            resolved
                .interpretations()
                .summary_packets
                .iter()
                .map(|packet| packet.events.len())
                .sum::<usize>()
        );
        for event in events {
            let Some(crate::theater::resolved::Record::Summary(summary)) =
                resolved.record(event.source)
            else {
                panic!()
            };
            assert_eq!(event.timestamp_us, u64::from(summary.timestamp_ms) * 1000);
            if let Some(player) = event.player_index {
                assert_eq!(resolved.player(player).unwrap().xuid, summary.xuid);
            }
            summaries += 1;
        }
        eprintln!("verified {}", path.strip_prefix(&root).unwrap().display());
    }
    assert!(expected.is_empty());
    assert!(contexts.is_empty());
    assert_eq!(frames, 403465);
    assert!(runtime_refusals > 0);
    eprintln!(
        "verified {frames} reference contexts; {runtime_refusals} source-verified runtime refusals"
    );
    assert_eq!(summaries, 3667);
}

use crate::theater::parser::v41::{FilmWorld, FrameConfig};
use crate::theater::resolved::interpretation::EventGate15Policy;

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
    invalid.body = PacketRead::Decoded(ReplicationStreamPacketBody::RosterPacketBody);
    assert!(invalid.payload(&bytes).is_none());
    assert!(valid.payload(&bytes[..15]).is_none());
}

#[test]
fn source_coverage_keeps_opaque_summary_and_unparsed_partial_event_bits() {
    let summary_payload = [2u32.to_be_bytes().as_slice(), &[0xaa, 0xbb]].concat();
    let film = Film::parse([
        registry(),
        FilmChunk {
            kind: ChunkKind::Summary,
            index: None,
            start_ms: None,
            data: packet(9, &summary_payload, 0),
        },
        FilmChunk {
            kind: ChunkKind::Replication,
            index: None,
            start_ms: None,
            // configuration=0, continuation=1, code=85, references=000.
            // Its bounded partial body does not establish the remaining layout.
            data: packet(0, &[0x6a, 0x8e], 0),
        },
    ])
    .unwrap();
    let summary = film.summary_chunks().next().unwrap();
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
    let replication = film.replication_chunks().next().unwrap();
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
    let film = Film::parse([
        registry(),
        FilmChunk {
            kind: ChunkKind::Replication,
            index: None,
            start_ms: None,
            data: packet(0, &[0x6a, 0x8f, 0], 0),
        },
    ])
    .unwrap();
    let chunk = film.replication_chunks().next().unwrap();
    let coverage = chunk.body.packets[0].source_coverage(&chunk.data).unwrap();
    assert_eq!(
        coverage.last().unwrap(),
        &SourceRegion {
            source: BitRange { start: 14, end: 24 },
            kind: SourceRegionKind::Unparsed,
        }
    );
}
