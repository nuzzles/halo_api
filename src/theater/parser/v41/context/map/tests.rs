use super::*;
use crate::clients::hi::models::RawFilmChunk;
use crate::theater::film::{catalog::maps::levels, *};
use crate::theater::parser::v41::chunks::registry::source::{
    BootstrapHeaderFieldKind, GameOptionsEntry, GameOptionsFieldKind,
};
use crate::theater::parser::v41::{
    chunks::registry::parse_registry_chunk,
    context::events::tests::{copy_bits, registry},
};

#[test]
fn level_binding_requires_exact_recorded_build_tag_and_complete_sections() {
    let mut data = registry(3, 1);
    let parsed = parse_registry_chunk(&data).unwrap();
    let header = parsed.bootstrap_header.as_ref().unwrap();
    let (version, branch) = ("6.10026.19225.0", levels::BRANCH);
    let mut locations = Vec::new();
    for (kind, text) in [
        (BootstrapHeaderFieldKind::BuildVersion, version),
        (BootstrapHeaderFieldKind::BuildBranch, branch),
    ] {
        let field = header.fields.iter().find(|f| f.kind == kind).unwrap();
        let mut buf = vec![0; field.width / 8];
        buf[..text.len()].copy_from_slice(text.as_bytes());
        copy_bits(&mut data, field.bit, &buf, field.width);
        if kind == BootstrapHeaderFieldKind::BuildBranch {
            locations.push(field.bit);
        }
    }
    let build_id = header
        .fields
        .iter()
        .find(|f| f.kind == BootstrapHeaderFieldKind::BuildId)
        .unwrap();
    copy_bits(&mut data, build_id.bit, &levels::BUILD_ID.to_le_bytes(), 32);
    locations.push(build_id.bit);
    let options = header
        .identification
        .as_ref()
        .unwrap()
        .game_options
        .as_ref()
        .unwrap();
    let tag = options
        .entries
        .iter()
        .find_map(|e| match e {
            GameOptionsEntry::Field {
                kind: GameOptionsFieldKind::LevelTag,
                bits,
            } => Some(bits),
            _ => None,
        })
        .unwrap();
    locations.push(tag.bit);
    for level in levels::all() {
        copy_bits(&mut data, tag.bit, &level.tag.to_be_bytes(), 32);
        let read = parse_registry_chunk(&data).unwrap();
        assert_eq!(recorded_level(&read), Some(level));
    }
    // Product version text is not the engine build identifier.
    let product = header
        .fields
        .iter()
        .find(|f| f.kind == BootstrapHeaderFieldKind::BuildVersion)
        .unwrap();
    let mut other_product = data.clone();
    copy_bits(&mut other_product, product.bit, b"6.10026.18411.0", 15 * 8);
    assert!(recorded_level(&parse_registry_chunk(&other_product).unwrap()).is_some());
    for at in locations {
        let mut altered = data.clone();
        altered[at / 8] ^= 1 << (7 - at % 8);
        assert!(recorded_level(&parse_registry_chunk(&altered).unwrap()).is_none());
    }
    for len in [0xcb208 + 123 * 4, 0xce68c, 0xce68c + 2, data.len() - 1] {
        assert!(recorded_level(&parse_registry_chunk(&data[..len]).unwrap()).is_none());
    }
    let mut missing = parse_registry_chunk(&data).unwrap();
    missing.bootstrap_header = None;
    assert!(recorded_level(&missing).is_none());
}

#[test]
#[ignore = "requires the separately retained 32-film corpus"]
fn captured_level_context() {
    use std::path::{Path, PathBuf};
    fn visit(path: &Path, out: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(path).unwrap() {
            let p = entry.unwrap().path();
            if p.is_dir() {
                visit(&p, out)
            } else if p.file_name().is_some_and(|n| n == "film.json") {
                out.push(p)
            }
        }
    }
    let root = std::env::var_os("HALO_FILM_CORPUS")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("experiments/films"));
    let mut paths = Vec::new();
    visit(&root, &mut paths);
    paths.sort();
    assert_eq!(paths.len(), 32);
    let mut matched = 0;
    for path in paths {
        let manifest: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        let c = manifest["chunks"]
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
        film.registry.public_registry().unwrap();
        let level = recorded_level(&film.registry.body);
        matched += usize::from(level.is_some());
        let header = film.registry.body.bootstrap_header.as_ref().unwrap();
        let build: Vec<_> = header
            .fields
            .iter()
            .filter(|f| {
                matches!(
                    f.kind,
                    BootstrapHeaderFieldKind::BuildVersion | BootstrapHeaderFieldKind::BuildBranch
                )
            })
            .map(|f| {
                String::from_utf8_lossy(&f.raw.as_ref().unwrap().bytes)
                    .trim_end_matches('\0')
                    .to_string()
            })
            .collect();
        eprintln!(
            "{}: {build:?} => {:?}",
            path.strip_prefix(&root).unwrap().display(),
            level.map(|l| l.module)
        );
    }
    eprintln!("exact catalogue matches: {matched}/32");
    assert_eq!(matched, 32);
}
