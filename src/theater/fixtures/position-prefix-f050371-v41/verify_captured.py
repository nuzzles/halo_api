"""Verify that both short fixture buffers are exact captured packet prefixes."""
import hashlib
import json
from pathlib import Path
import sys


def main() -> None:
    root = Path(sys.argv[1]) if len(sys.argv) > 1 else Path("experiments/films")
    manifest_path = root / "raids/01-hour-long-raid/film.json"
    chunks = json.loads(manifest_path.read_text())["chunks"]
    cases = json.loads(Path(__file__).with_name("cases.json").read_text())
    assert len(cases) == 2
    for case in cases:
        index = int(case["name"].split("-")[2])
        chunk = (manifest_path.parent / chunks[index]["file"]).read_bytes()
        assert hashlib.sha256(chunk).hexdigest() == case["chunk_sha256"]
        start = case["packet_payload_start"]
        size = int.from_bytes(chunk[start - 12:start - 8], "little")
        payload = chunk[start:start + size]
        bit = case["component_start_bit"]
        count = len(case["hex"]) * 4
        expected = "".join(f"{byte:08b}" for byte in bytes.fromhex(case["hex"]))
        actual = "".join(f"{byte:08b}" for byte in payload)[bit:bit + count]
        assert actual == expected, case["name"]
        print(case["name"], "verified")


if __name__ == "__main__":
    main()
