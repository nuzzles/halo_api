#!/usr/bin/env python3
"""Check reference film sources against the committed, pinned port inventory."""
import argparse
import hashlib
import json
from pathlib import Path


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("film_root", type=Path, help="LevelUp's halo_infinite/film directory")
    args = parser.parse_args()
    manifest = json.loads(Path(__file__).with_name("levelup-port-manifest.json").read_text())
    failures = []
    for entry in manifest["files"]:
        path = args.film_root / entry["path"]
        try:
            digest = hashlib.sha256(path.read_bytes()).hexdigest()
        except OSError as error:
            failures.append({"path": entry["path"], "error": str(error)})
            continue
        if digest != entry["sha256"]:
            failures.append({"path": entry["path"], "expected": entry["sha256"], "actual": digest})
    print(json.dumps({"reference_commit": manifest["commit"],
                      "checked_files": len(manifest["files"]), "failures": failures}, indent=2))
    return bool(failures)


if __name__ == "__main__":
    raise SystemExit(main())
