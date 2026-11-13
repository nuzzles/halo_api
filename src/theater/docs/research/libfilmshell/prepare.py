"""Prepare the exact manifest-selected samples for the libfilmshell study.

Usage: python3 prepare.py /path/to/experiments/films /tmp/study-inputs
Only the output directory is written. No downloads or credentials are needed.
"""

import hashlib
import json
from pathlib import Path
import sys
import zlib


def main():
    corpus, output = map(Path, sys.argv[1:])
    expected = json.loads(Path(__file__).with_name("inputs.json").read_text())
    output.mkdir(parents=True, exist_ok=True)
    for sample in expected:
        source = corpus / sample["film"]
        manifest = json.loads((source / "film.json").read_text())
        header = next(c for c in manifest["chunks"] if c["chunk_type"] == 1)
        replay = next(c for c in manifest["chunks"] if c["file"] == sample["replay"])
        destination = output / (
            sample["film"].replace("/", "--") + f"--chunk-{replay['index']:03}"
        )
        destination.mkdir(exist_ok=True)
        for entry, filename in [(header, "header.bin"), (replay, "replay.bin")]:
            data = (source / entry["file"]).read_bytes()
            if len(data) > 1 and data[0] & 15 == 8 and int.from_bytes(data[:2], "big") % 31 == 0:
                data = zlib.decompress(data)
            if len(data) != entry["decompressed_size"]:
                raise ValueError(f"Unexpected size: {source / entry['file']}")
            if hashlib.sha256(data).hexdigest() != sample["sha256_inflated"][filename]:
                raise ValueError(f"Unexpected hash: {source / entry['file']}")
            (destination / filename).write_bytes(data)
    print(f"Prepared and hash-verified {len(expected)} replay chunks")


if __name__ == "__main__":
    main()
