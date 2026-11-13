"""Verify the manifests referenced by the retained upstream label oracle."""
import hashlib
import json
from pathlib import Path
import sys


def main(corpus):
    data = json.loads(Path(__file__).with_name("reference.json").read_text())
    paths = sorted(corpus.rglob("film.json"))
    assert len(paths) == len(data["rows"]) == 32
    for path, row in zip(paths, data["rows"]):
        assert path.parent.relative_to(corpus).as_posix() == row["path"]
        assert hashlib.sha256(path.read_bytes()).hexdigest() == row["manifest_sha256"]
    print(f"verified {len(paths)} motion-label oracle manifests")


if __name__ == "__main__":
    main(Path(sys.argv[1]).resolve())
