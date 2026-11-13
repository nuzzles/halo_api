use super::*;

fn replication(data: Vec<u8>) -> RawFilmChunk {
    RawFilmChunk {
        kind: ChunkKind::Replication.into(),
        index: 1,
        start_ms: -1,
        data: crate::theater::parser::test_support::zlib_fixture_data(&data),
    }
}

fn compressed(data: &[u8]) -> Vec<u8> {
    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::best());
    encoder.write_all(data).unwrap();
    encoder.finish().unwrap()
}

fn limits(packets: usize) -> ParseLimits {
    ParseLimits {
        max_packets_per_chunk: packets,
        max_control_records: ParseLimits::default().max_control_records,
    }
}

#[test]
fn damaged_zlib_is_an_error() {
    let payload = packet(99, &[0; 4096], 1);
    let wire = compressed(&payload);
    let film = parse_structural_with_limits(
        [
            registry(),
            RawFilmChunk {
                data: wire.clone(),
                ..replication(Vec::new())
            },
            empty_summary(),
        ],
        limits(1),
    )
    .unwrap();
    assert_eq!(film.replication_stream.0[0].data, payload);
    let mut damaged = wire;
    damaged.pop();
    assert!(matches!(
        Film::parse([
            registry(),
            RawFilmChunk {
                data: damaged,
                ..replication(Vec::new())
            },
            indexed_summary()
        ]),
        Err(ParseError::Inflate {
            index: 1,
            cause: InflateError::Stream
        })
    ));
}

#[test]
fn packet_budget_errors_instead_of_returning_a_partial_film() {
    let p = packet(99, &[0xa0, 0x7b, 0x42], 7);
    for kind in [ChunkKind::Replication, ChunkKind::Summary] {
        let input = RawFilmChunk {
            kind: kind.into(),
            index: if kind == ChunkKind::Summary { 2 } else { 1 },
            ..replication(p.repeat(2))
        };
        let chunks = if kind == ChunkKind::Replication {
            vec![registry(), input, empty_summary()]
        } else {
            vec![registry(), replication(vec![]), input]
        };
        assert!(matches!(
            Film::parse_with_limits(chunks, limits(1)),
            Err(ParseError::PacketLimit { index: _, limit: 1 })
        ));
    }
    let film =
        parse_structural_with_limits([registry(), replication(p), empty_summary()], limits(1))
            .unwrap();
    assert_eq!(film.replication_stream.0[0].body.packets.len(), 1);
    assert!(film.replication_stream.0[0].body.opaque.is_empty());
}
