"""Export current libfilmshell motion labels for the retained corpus.

Usage: python3 export_reference.py /path/to/libfilmshell experiments/films > reference.json
"""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile

PIN = "a13efb323b230c96b59685defbc376720043240f"
CONTAINER_BLOB = "068885d8c5fdc790ea8efd86e5e4e82873ab97f4"


def git_blob(data):
    return hashlib.sha1(f"blob {len(data)}\0".encode() + data).hexdigest()


def main(upstream, corpus):
    actual = git_blob((upstream / "src/container.rs").read_bytes())
    if actual != CONTAINER_BLOB:
        raise SystemExit(f"container.rs blob {actual} differs from {PIN}")
    source = Path(__file__).with_name("export_upstream.rs").read_text()
    paths = sorted(corpus.rglob("film.json"))
    if len(paths) != 32:
        raise SystemExit(f"expected 32 retained films, found {len(paths)}")
    with tempfile.TemporaryDirectory(prefix="halo-encoding-oracle-") as temp:
        project = Path(temp)
        (project / "src").mkdir()
        (project / "src/main.rs").write_text(source)
        (project / "Cargo.toml").write_text(
            '[package]\nname = "halo-encoding-oracle"\nversion = "0.0.0"\nedition = "2024"\n'
            f'[dependencies]\nfilmshell = {{ path = {json.dumps(str(upstream))}, features = ["research"] }}\n'
            'serde_json = "1"\n[profile.dev]\nopt-level = 2\n'
        )
        output = subprocess.check_output(
            ["cargo", "run", "--offline", "--quiet", "--manifest-path", str(project / "Cargo.toml"),
             "--", str(corpus)], text=True
        )
    labels = dict(line.split("\t", 1) for line in output.strip().splitlines())
    rows = []
    for manifest in paths:
        relative = manifest.parent.relative_to(corpus).as_posix()
        rows.append({
            "path": relative,
            "manifest_sha256": hashlib.sha256(manifest.read_bytes()).hexdigest(),
            "label": labels.pop(relative),
        })
    if labels:
        raise SystemExit(f"extra upstream labels: {labels}")
    print(json.dumps({"source": PIN, "rows": rows}, indent=2))


if __name__ == "__main__":
    main(Path(sys.argv[1]).resolve(), Path(sys.argv[2]).resolve())
