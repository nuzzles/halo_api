use super::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct Case {
    name: String,
    hex: String,
    bit_len: usize,
    entries: Vec<Entry>,
}
#[derive(Deserialize)]
struct Entry {
    kind: String,
    start: usize,
    end: usize,
}
fn bytes(hex: &str) -> Vec<u8> {
    hex.as_bytes()
        .chunks(2)
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}
fn cases() -> Vec<Case> {
    serde_json::from_str(include_str!(
        "../../../../../../fixtures/bond-v2/game-options.json"
    ))
    .unwrap()
}
fn bounds(entry: &GameOptionsEntry) -> (usize, usize, &str) {
    match entry {
        GameOptionsEntry::Field { bits, .. } => (bits.bit, bits.bit + bits.width, "field"),
        GameOptionsEntry::Bond { read, .. } => {
            (read.root.source.start, read.root.source.end, "bond")
        }
        GameOptionsEntry::String { bytes, .. } => {
            (bytes[0].bit, bytes.last().unwrap().bit + 8, "string")
        }
    }
}
#[test]
fn game_options_preserve_both_bond_messages_and_the_player_boundary() {
    for case in cases() {
        let source = bytes(&case.hex);
        for shift in [0, 1, 3, 7] {
            let mut data = vec![0; (source.len() * 8 + shift).div_ceil(8)];
            for i in 0..source.len() * 8 {
                data[(i + shift) / 8] |=
                    ((source[i / 8] >> (7 - i % 8)) & 1) << (7 - (i + shift) % 8);
            }
            let result = read(&data, shift);
            assert_eq!(result.stop, GameOptionsStop::Complete, "{}", case.name);
            assert_eq!(result.end_bit, case.bit_len + shift);
            assert_eq!(result.entries.len(), case.entries.len());
            for (actual, expected) in result.entries.iter().zip(&case.entries) {
                assert_eq!(
                    bounds(actual),
                    (
                        expected.start + shift,
                        expected.end + shift,
                        expected.kind.as_str()
                    )
                );
                match actual {
                    GameOptionsEntry::Field { bits, .. } => {
                        assert_eq!(bits.raw, RawBits::from_source(&data, bits.bit, bits.width))
                    }
                    GameOptionsEntry::String {
                        bytes, terminated, ..
                    } => {
                        assert!(*terminated);
                        for b in bytes {
                            assert_eq!(b.raw, RawBits::from_source(&data, b.bit, 8));
                        }
                    }
                    GameOptionsEntry::Bond { read, .. } => {
                        assert_eq!(read.stop, BondStop::Complete)
                    }
                }
            }
            assert_eq!(
                RawBits::from_source(&data, result.end_bit, 16)
                    .unwrap()
                    .low_u64(),
                0x9696
            );
            assert_eq!(
                serde_json::from_slice::<GameOptionsRead>(&serde_json::to_vec(&result).unwrap())
                    .unwrap(),
                result
            );
        }
    }
}

#[test]
fn game_options_stop_on_every_truncated_envelope_field_and_missing_string_terminator() {
    for case in cases() {
        let source = bytes(&case.hex);
        for entry in &case.entries {
            for cut in [entry.start, entry.start + 1, entry.end - 1] {
                let shift = (8 - cut % 8) % 8;
                let mut data = vec![0; (cut + shift) / 8];
                for i in 0..cut {
                    data[(i + shift) / 8] |=
                        ((source[i / 8] >> (7 - i % 8)) & 1) << (7 - (i + shift) % 8);
                }
                let result = read(&data, shift);
                assert_ne!(
                    result.stop,
                    GameOptionsStop::Complete,
                    "{}: {cut}",
                    case.name
                );
                assert!(result.end_bit <= cut + shift);
                assert!(
                    result.entries.len()
                        <= case.entries.iter().take_while(|e| e.start <= cut).count()
                );
            }
        }
        if case.name == "flags/4" {
            let mut data = source.clone();
            let string = case.entries.iter().find(|e| e.kind == "string").unwrap();
            for bit in string.end - 8..string.end {
                data[bit / 8] |= 1 << (7 - bit % 8);
            }
            let result = read(&data, 0);
            assert_eq!(result.stop, GameOptionsStop::UnterminatedString);
            assert_eq!(result.end_bit, string.end);
        }
    }
}

#[test]
#[ignore = "requires the separately retained 32-film corpus"]
fn captured_game_options_boundaries() {
    use crate::clients::hi::models::RawFilmChunk;
    use crate::theater::film::ChunkKind;
    use std::path::{Path, PathBuf};
    fn manifests(root: &Path, out: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(root).unwrap() {
            let p = entry.unwrap().path();
            if p.is_dir() {
                manifests(&p, out);
            } else if p.file_name().is_some_and(|n| n == "film.json") {
                out.push(p);
            }
        }
    }
    let root = std::env::var_os("HALO_FILM_CORPUS")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("experiments/films"));
    let mut paths = Vec::new();
    manifests(&root, &mut paths);
    paths.sort();
    assert_eq!(paths.len(), 32);
    for path in paths {
        let m: serde_json::Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        let c = m["chunks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["chunk_type"] == 1)
            .unwrap();
        let data = std::fs::read(path.parent().unwrap().join(c["file"].as_str().unwrap())).unwrap();
        let film = crate::theater::parser::test_support::parse_fixture_structural([
            RawFilmChunk {
                kind: ChunkKind::Registry.into(),
                data,
                index: 0,
                start_ms: 0,
            },
            crate::theater::parser::test_support::empty_replication(),
            crate::theater::parser::test_support::empty_summary(),
        ])
        .unwrap();
        let options = film
            .registry
            .body
            .bootstrap_header
            .as_ref()
            .unwrap()
            .identification
            .as_ref()
            .unwrap()
            .game_options
            .as_ref()
            .unwrap();
        assert_eq!(
            options.stop,
            GameOptionsStop::Complete,
            "{}: {}",
            path.display(),
            options.end_bit
        );
        assert!(options.end_bit > options.start_bit, "{}", path.display());
        eprintln!(
            "verified game-options/player boundary {}: {}",
            path.parent()
                .unwrap()
                .strip_prefix(&root)
                .unwrap()
                .display(),
            options.end_bit
        );
    }
}
