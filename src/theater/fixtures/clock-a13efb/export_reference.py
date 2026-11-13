"""Export latest libfilmshell's packet/highlight ticks for the retained corpus.

Usage: python3 export_reference.py /path/to/libfilmshell-a13efb /path/to/films > reference.bin.zlib
"""
import hashlib
import json
from pathlib import Path
import struct
import subprocess
import sys
import tempfile
import zlib

PIN = "a13efb323b230c96b59685defbc376720043240f"
BLOBS = {
    "src/container.rs": "068885d8c5fdc790ea8efd86e5e4e82873ab97f4",
    "src/lives.rs": "6d2be472947c4c87e9e9727315a1ae6d8f91f928",
    "src/inflate.rs": "a8b88897f08ef418146dd72707612f825c3e7b24",
}
MAGIC = b"HCLKv001"


def git_blob(data):
    return hashlib.sha1(f"blob {len(data)}\0".encode() + data).hexdigest()


def main(upstream, corpus):
    for relative, expected in BLOBS.items():
        actual = git_blob((upstream / relative).read_bytes())
        if actual != expected:
            raise SystemExit(f"{relative} blob {actual} differs from {PIN}")
    source = Path(__file__).with_name("export_upstream.rs").read_text()
    paths = sorted(corpus.rglob("film.json"))
    if len(paths) != 32:
        raise SystemExit(f"expected 32 retained films, found {len(paths)}")
    with tempfile.TemporaryDirectory(prefix="halo-clock-oracle-") as temp:
        project = Path(temp)
        (project / "src").mkdir()
        (project / "src/main.rs").write_text(source)
        (project / "Cargo.toml").write_text(
            '[package]\nname = "halo-clock-oracle"\nversion = "0.0.0"\nedition = "2024"\n'
            f'[dependencies]\nfilmshell = {{ path = {json.dumps(str(upstream))}, features = ["research"] }}\n'
            'serde_json = "1"\n[profile.dev]\nopt-level = 2\n'
        )
        output = project / "output"
        subprocess.run(
            ["cargo", "run", "--offline", "--quiet", "--manifest-path", str(project / "Cargo.toml"),
             "--", str(corpus), str(output)], check=True, stdout=sys.stderr
        )
        data = bytearray(MAGIC + struct.pack("<I", len(paths)))
        for manifest in paths:
            relative = manifest.parent.relative_to(corpus).as_posix().encode()
            ticks = (output / relative.decode() / "ticks.bin").read_bytes()
            if len(relative) > 65535 or len(ticks) % 8:
                raise AssertionError(manifest)
            data += struct.pack("<H", len(relative)) + relative
            data += hashlib.sha256(manifest.read_bytes()).digest()
            data += struct.pack("<I", len(ticks) // 8) + ticks
        sys.stdout.buffer.write(zlib.compress(data, level=9))


if __name__ == "__main__":
    main(Path(sys.argv[1]).resolve(), Path(sys.argv[2]).resolve())
