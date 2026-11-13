"""Verify the pinned clock fixture's recorded manifest identities.

Usage: python3 verify_source.py /path/to/films
"""
import hashlib
from pathlib import Path
import struct
import sys
import zlib

root = Path(sys.argv[1])
fixture = Path(__file__).with_name("reference.bin.zlib")
data = memoryview(zlib.decompress(fixture.read_bytes()))
at = 0


def take(size):
    global at
    end = at + size
    if end > len(data):
        raise ValueError("truncated clock fixture")
    value = data[at:end]
    at = end
    return value


if bytes(take(8)) != b"HCLKv001":
    raise ValueError("wrong clock fixture version")
count = struct.unpack("<I", take(4))[0]
films = sorted(root.rglob("film.json"))
if count != 32 or len(films) != count:
    raise ValueError((count, len(films)))
for manifest in films:
    path_length = struct.unpack("<H", take(2))[0]
    relative = bytes(take(path_length)).decode("utf-8")
    expected = manifest.parent.relative_to(root).as_posix()
    if relative != expected:
        raise ValueError((relative, expected))
    recorded_hash = bytes(take(32))
    actual_hash = hashlib.sha256(manifest.read_bytes()).digest()
    if recorded_hash != actual_hash:
        raise ValueError(f"manifest changed: {relative}")
    frames = struct.unpack("<I", take(4))[0]
    take(frames * 8)
if at != len(data):
    raise ValueError("trailing clock fixture data")
print(f"verified {count} film manifests and clock arrays")
