//! Disposable exporter using the unmodified current libfilmshell vote reader.
use filmshell::container::{detect_encoding_mode, Film, Framing};
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
    let root = PathBuf::from(std::env::args().nth(1).expect("corpus root"));
    let mut paths = Vec::new();
    manifests(&root, &mut paths);
    paths.sort();
    for path in paths {
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
            .collect();
        let film = Film::new(chunks, Framing::Packets);
        let relative = path.parent().unwrap().strip_prefix(&root).unwrap();
        println!("{}\t{:?}", relative.display(), detect_encoding_mode(&film, None));
    }
}
