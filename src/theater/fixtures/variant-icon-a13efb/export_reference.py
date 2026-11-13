"""Run the unchanged a13efb libfilmshell variant reader on retained film headers.

Usage: python3 export_reference.py /path/to/libfilmshell-a13efb /path/to/films
"""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import zlib

PIN = "a13efb323b230c96b59685defbc376720043240f"
BLOB = "51c7626e939444345753f366b3eb1177980d9034"
SOURCE = "src/variant.rs"

def git_blob(data):
    return hashlib.sha1(f"blob {len(data)}\0".encode() + data).hexdigest()

def inflated(raw):
    zlib_header = (len(raw) >= 2 and raw[0] & 15 == 8 and raw[0] >> 4 <= 7
                   and int.from_bytes(raw[:2], "big") % 31 == 0)
    return zlib.decompress(raw) if zlib_header else raw

upstream = Path(sys.argv[1])
corpus = Path(sys.argv[2])
source = (upstream / SOURCE).read_bytes()
if git_blob(source) != BLOB:
    raise SystemExit(f"unexpected {SOURCE} blob: {git_blob(source)}")

MANIFEST = r'''
mod game {
    #[derive(Clone, Debug, PartialEq)]
    pub enum OptionValue { Number(f64), Bool(bool) }
}
mod variant;
fn main() {
    for path in std::env::args().skip(1) {
        let data = std::fs::read(path).unwrap();
        let icon = variant::read(&data).and_then(|v| v.icon);
        println!("{}", icon.map(|x| x.to_string()).unwrap_or_else(|| "null".into()));
    }
}
'''
rows = []
with tempfile.TemporaryDirectory(prefix="halo-variant-icon-") as temp:
    project = Path(temp)
    (project / "variant.rs").write_bytes(source)
    (project / "main.rs").write_text(MANIFEST)
    inputs = []
    for manifest_path in sorted(corpus.rglob("film.json")):
        manifest_bytes = manifest_path.read_bytes()
        manifest = json.loads(manifest_bytes)
        headers = [c for c in manifest["chunks"] if c["chunk_type"] == 1]
        if len(headers) != 1:
            raise AssertionError((manifest_path, len(headers)))
        header = headers[0]
        raw = (manifest_path.parent / header["file"]).read_bytes()
        data = inflated(raw)
        path = project / f"chunk-{len(rows):03}.bin"
        path.write_bytes(data)
        inputs.append(str(path))
        rows.append({
            "film": manifest_path.parent.relative_to(corpus).as_posix(),
            "manifest_sha256": hashlib.sha256(manifest_bytes).hexdigest(),
            "registry_sha256": hashlib.sha256(raw).hexdigest(),
            "icon": None,
        })
    if not rows:
        raise AssertionError("no captured films")
    subprocess.run(["rustc", "--edition=2024", "-Awarnings", str(project / "main.rs"),
                    "-o", str(project / "oracle")], check=True)
    result = subprocess.check_output([str(project / "oracle"), *inputs], text=True).splitlines()
    if len(result) != len(rows):
        raise AssertionError((len(result), len(rows)))
    for row, value in zip(rows, result):
        row["icon"] = None if value == "null" else int(value)
print(json.dumps({"repository": "https://github.com/OpenSpartan/libfilmshell",
                  "commit": PIN, "source_blob": BLOB, "films": rows},
                 indent=2) + "\n")
