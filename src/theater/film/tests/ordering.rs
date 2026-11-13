use super::*;
use crate::theater::film::ChunkKind;
use std::io::Write;

fn chunk(kind: ChunkKind, index: i32, data: Vec<u8>) -> RawFilmChunk {
    RawFilmChunk {
        kind: kind.into(),
        index,
        start_ms: 0,
        data: crate::theater::parser::test_support::zlib_fixture_data(&data),
    }
}

fn registry() -> RawFilmChunk {
    chunk(
        ChunkKind::Registry,
        0,
        [41u32.to_le_bytes(), 27u32.to_le_bytes()].concat(),
    )
}

fn packet(payload: &[u8]) -> Vec<u8> {
    [
        0u16.to_le_bytes().as_slice(),
        &[0, 0],
        &(payload.len() as u32).to_le_bytes(),
        &[0; 8],
        payload,
    ]
    .concat()
}

fn tick_payload() -> Vec<u8> {
    let bits = format!(
        "00{:027b}00101010000",
        crate::theater::parser::ticks::TICK_SIGNATURE
    );
    let mut payload = vec![0u8; bits.len().div_ceil(8)];
    for (i, bit) in bits.bytes().enumerate() {
        if bit == b'1' {
            payload[i / 8] |= 1 << (7 - i % 8);
        }
    }
    payload
}

#[test]
fn indexes_out_of_order_chunks_by_manifest_index() {
    let replay = chunk(ChunkKind::Replication, 1, packet(&tick_payload()));
    let film = Film::parse([
        replay,
        chunk(ChunkKind::Summary, 2, vec![]),
        crate::theater::parser::test_support::registry_with_complete_setup(),
    ])
    .unwrap();
    assert_eq!(film.registry.version.film, 41);
    // Indexed admission succeeding also proves the replay tick was found.
}

#[test]
fn rejects_duplicate_and_missing_indices() {
    assert!(matches!(
        Film::parse([registry(), chunk(ChunkKind::Replication, 0, vec![])]),
        Err(ParseError::DuplicateIndex { index: 0 })
    ));
    assert!(matches!(
        Film::parse([registry(), chunk(ChunkKind::Replication, 2, vec![])]),
        Err(ParseError::MissingIndex { index: 1 })
    ));
}

#[test]
fn rejects_negative_manifest_index_before_inflation() {
    let negative = chunk(ChunkKind::Replication, -1, vec![]);
    assert!(matches!(
        Film::parse([
            registry(),
            negative.clone(),
            chunk(ChunkKind::Summary, 2, vec![])
        ]),
        Err(ParseError::InvalidIndex(-1))
    ));
    assert!(matches!(
        negative.inflate(),
        Err(ParseError::InvalidIndex(-1))
    ));
}

#[test]
fn inflation_and_packet_budget_have_distinct_errors() {
    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(&packet(&tick_payload())).unwrap();
    let mut compressed = encoder.finish().unwrap();
    compressed.pop();
    assert!(matches!(
        Film::parse([
            registry(),
            RawFilmChunk {
                kind: ChunkKind::Replication.into(),
                index: 1,
                start_ms: 0,
                data: compressed,
            },
            chunk(ChunkKind::Summary, 2, vec![])
        ]),
        Err(ParseError::Inflate {
            index: 1,
            cause: crate::theater::film::InflateError::Stream,
        })
    ));

    let limits = ParseLimits {
        max_packets_per_chunk: 0,
        ..ParseLimits::default()
    };
    assert!(matches!(
        crate::theater::parser::ChunkReader::read(
            [
                registry(),
                chunk(ChunkKind::Replication, 1, packet(&tick_payload())),
                chunk(ChunkKind::Summary, 2, vec![])
            ],
            limits,
            FilmParseContext::default(),
        ),
        Err(ParseError::PacketLimit { index: 1, limit: 0 })
    ));
}

#[test]
fn distinguishes_no_ticks_from_unread_replay_bytes() {
    assert!(matches!(
        Film::parse([
            crate::theater::parser::test_support::registry_with_complete_setup(),
            chunk(ChunkKind::Replication, 1, packet(&[0; 8])),
            chunk(ChunkKind::Summary, 2, vec![])
        ]),
        Err(ParseError::NoTicks)
    ));
    assert!(matches!(
        Film::parse([
            crate::theater::parser::test_support::registry_with_complete_setup(),
            chunk(
                ChunkKind::Replication,
                1,
                [packet(&[0; 8]), vec![0xff]].concat()
            ),
            chunk(ChunkKind::Summary, 2, vec![])
        ]),
        Err(ParseError::TickEvidenceIncomplete)
    ));
}

#[test]
#[ignore = "requires the separately retained v41 film corpus"]
fn captured_film_admits_with_reversed_manifest_order() {
    let root = std::env::var_os("HALO_FILM_CORPUS")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("experiments/films"));
    let directory = root.join("aim/01-pitch-up");
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(directory.join("film.json")).unwrap()).unwrap();
    let mut chunks = manifest["chunks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| RawFilmChunk {
            kind: crate::clients::hi::models::RawChunkKind(
                i32::try_from(entry["chunk_type"].as_i64().unwrap()).unwrap(),
            ),
            index: i32::try_from(entry["index"].as_i64().expect("film chunk index"))
                .expect("film chunk index fits i32"),
            start_ms: entry["start_time_offset_ms"]
                .as_i64()
                .expect("film chunk start time"),
            data: std::fs::read(directory.join(entry["file"].as_str().unwrap())).unwrap(),
        })
        .collect::<Vec<_>>();
    chunks.reverse();
    let film = Film::parse(chunks).unwrap();
    assert_eq!(film.registry.version.film, 41);
}
