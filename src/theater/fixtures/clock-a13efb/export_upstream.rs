//! Disposable exact packet/highlight clock exporter from the upstream reader.
use filmshell::container::{Film, Framing};
use filmshell::inflate;
use std::path::{Path, PathBuf};

fn manifests(root: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(root).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            manifests(&path, out);
        } else if path.file_name().is_some_and(|name| name == "film.json") {
            out.push(path);
        }
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let root = PathBuf::from(args.next().expect("corpus root"));
    let output = PathBuf::from(args.next().expect("output root"));
    let selected = args.next();
    let mut paths = Vec::new();
    manifests(&root, &mut paths);
    paths.sort();
    for path in paths {
        let relative = path.parent().unwrap().strip_prefix(&root).unwrap();
        if selected
            .as_deref()
            .is_some_and(|name| relative != Path::new(name))
        {
            continue;
        }
        let manifest: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        let chunks = manifest["chunks"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| {
                let raw = std::fs::read(path.parent().unwrap().join(row["file"].as_str().unwrap()))
                    .unwrap();
                inflate::zlib(&raw, 256 << 20).unwrap_or(raw)
            })
            .collect::<Vec<_>>();
        let film = Film::new(chunks, Framing::Packets);
        assert_eq!(film.framing, Framing::Packets, "{}", path.display());
        let game = film.frame_ticks();
        let highlight = film.highlight_ticks();
        assert_eq!(game.len(), highlight.len());
        let mut bytes = Vec::with_capacity(game.len() * 8);
        for (&g, &h) in game.iter().zip(&highlight) {
            bytes.extend_from_slice(&g.to_le_bytes());
            bytes.extend_from_slice(&h.to_le_bytes());
        }
        let target = output.join(relative).join("ticks.bin");
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::write(&target, bytes).unwrap();
        println!("{} {}", relative.display(), game.len());
    }
}
