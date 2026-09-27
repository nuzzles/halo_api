//! Independent oracle for native loaded-film chunk-number lookup and packet ranges.
use super::{FilmSource, FilmSourceMetadata};
use serde::Deserialize;
use std::io::Read;

#[test]
fn native_chunk_bridge() {
    #[derive(Deserialize)]
    struct Packet {
        index: usize,
        #[serde(rename = "type")]
        kind: u16,
        start: usize,
        size: usize,
        timestamp: u64,
        hex: String,
    }
    #[derive(Deserialize)]
    struct Lookup {
        number: i64,
        position: i64,
        ok: bool,
        hex: String,
        packets: Vec<Packet>,
    }
    #[derive(Deserialize)]
    struct Case {
        chunks: Vec<String>,
        metadata: Vec<FilmSourceMetadata>,
        numbers: Vec<i64>,
        lookups: Vec<Lookup>,
        registry: String,
        has_registry: bool,
    }
    fn unhex(s: &str) -> Vec<u8> {
        s.as_bytes()
            .as_chunks::<2>()
            .0
            .iter()
            .map(|v| u8::from_str_radix(std::str::from_utf8(v).unwrap(), 16).unwrap())
            .collect()
    }
    let mut json = Vec::new();
    flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/chunk-bridge-v41.json.zlib")[..])
        .read_to_end(&mut json)
        .unwrap();
    let cases: Vec<Case> = serde_json::from_slice(&json).unwrap();
    assert_eq!(cases.len(), 512);
    let (mut lookups, mut packets, mut mismatched_numbers) = (0, 0, 0);
    for (i, c) in cases.into_iter().enumerate() {
        let chunks: Vec<_> = c.chunks.iter().map(|s| unhex(s)).collect();
        let source = FilmSource::load(&chunks, &c.metadata).unwrap();
        // The reference ScanClockOrigin uses exactly this lookup's first packet.
        // The oracle includes absent, short, long, reordered and duplicate metadata.
        let clock_lookup = c.lookups.iter().find(|l| l.number == 1).unwrap();
        let expected_clock = clock_lookup
            .ok
            .then(|| clock_lookup.packets.first().map(|p| p.timestamp))
            .flatten();
        assert_eq!(
            super::scan_loaded_replay_clock_origin(&source).ok(),
            expected_clock,
            "clock {i}"
        );
        assert_eq!(source.data_chunk_numbers(), c.numbers, "numbers {i}");
        let reg = unhex(&c.registry);
        assert_eq!(
            source.registry_chunk(),
            c.has_registry.then_some(reg.as_slice()),
            "registry {i}"
        );
        for l in c.lookups {
            lookups += 1;
            assert_eq!(
                source.chunk_position(l.number).map(|n| n as i64),
                l.ok.then_some(l.position),
                "position {i}/{}",
                l.number
            );
            let found = source.chunk_by_number(l.number);
            assert_eq!(found.is_some(), l.ok, "lookup {i}/{}", l.number);
            let Some((bytes, actual)) = found else {
                continue;
            };
            assert_eq!(bytes, unhex(&l.hex), "bytes {i}/{}", l.number);
            assert_eq!(actual.len(), l.packets.len());
            let isolated = super::walk_film_packets(bytes);
            assert_eq!(isolated.len(), l.packets.len());
            mismatched_numbers += usize::from(l.number != l.position);
            for (j, (actual, expected)) in actual.iter().zip(l.packets).enumerate() {
                packets += 1;
                let mut at_zero = *actual;
                at_zero.chunk_index = 0;
                assert_eq!(isolated[j], at_zero);
                assert_eq!(j, expected.index);
                assert_eq!(
                    (
                        actual.packet_type,
                        actual.payload_offset,
                        actual.payload_size,
                        actual.timestamp_us
                    ),
                    (
                        expected.kind,
                        expected.start,
                        expected.size,
                        expected.timestamp
                    ),
                    "packet {i}/{j}"
                );
                assert_eq!(i64::from(actual.chunk_index), l.position);
                assert_eq!(source.payload(actual).unwrap(), unhex(&expected.hex));
            }
        }
    }
    assert!(mismatched_numbers > 0 && packets > 0);
    println!(
        "native chunk bridge: {lookups} lookups, {packets} packets, {mismatched_numbers} number/position differences"
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn native_packet_files() {
    #[derive(Deserialize)]
    struct Input {
        name: String,
        hex: String,
    }
    #[derive(Deserialize)]
    struct Expected {
        number: i64,
        ok: bool,
        hex: String,
    }
    #[derive(Deserialize)]
    struct Case {
        inputs: Vec<Input>,
        count: usize,
        reads: Vec<Expected>,
    }
    fn unhex(s: &str) -> Vec<u8> {
        s.as_bytes()
            .as_chunks::<2>()
            .0
            .iter()
            .map(|v| u8::from_str_radix(std::str::from_utf8(v).unwrap(), 16).unwrap())
            .collect()
    }
    let c: Case = serde_json::from_slice(include_bytes!("fixtures/packet-files-v41.json")).unwrap();
    let path = std::env::temp_dir().join(format!("halo-packet-files-{}", std::process::id()));
    std::fs::create_dir(&path).unwrap();
    for input in c.inputs {
        std::fs::write(path.join(input.name), unhex(&input.hex)).unwrap();
    }
    let count = super::count_film_chunks(&path);
    let reads: Vec<_> = c
        .reads
        .iter()
        .map(|r| super::read_film_chunk(&path, r.number))
        .collect();
    std::fs::remove_dir_all(&path).unwrap();
    assert_eq!(count, c.count);
    for (actual, expected) in reads.into_iter().zip(c.reads) {
        assert_eq!(actual.is_ok(), expected.ok, "chunk {}", expected.number);
        if let Ok(bytes) = actual {
            assert_eq!(bytes, unhex(&expected.hex), "chunk {}", expected.number);
        }
    }
}
