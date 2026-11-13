"""Compare the fixed bootstrap prefix with the older build-anchor observation.

Read-only corpus audit. The anchor comparison is evidence, not parser admission.
Usage: python3 bootstrap_inventory.py experiments/films
"""
import hashlib
import json
from pathlib import Path
import sys
import zlib

REGISTRY_END = 0xCB208
BUILD = REGISTRY_END + 123 * 4
FLAG = BUILD + 3 * 32 + 2 * 4


def main():
    root = Path(sys.argv[1])
    rows = []
    for path in sorted(root.rglob('film.json')):
        manifest = json.loads(path.read_text())
        chunk = next(c for c in manifest['chunks'] if c['chunk_type'] == 1)
        data = (path.parent / chunk['file']).read_bytes()
        if len(data) > 1 and data[0] & 15 == 8 and int.from_bytes(data[:2], 'big') % 31 == 0:
            data = zlib.decompress(data)
        assert len(data) == chunk['decompressed_size'] and len(data) > FLAG
        word = lambda at: int.from_bytes(data[at:at + 4], 'little')
        # This independently reproduces the older runtime's aligned anchor scan.
        anchors = [at for at in range(REGISTRY_END + 32, min(REGISTRY_END + 32 + 0x1000, len(data) - 32), 4)
                   if data[at:at + 3] == b'HI_']
        rows.append(dict(film=str(path.parent.relative_to(root)),
                         header=[word(0), word(4)],
                         header_sha256=hashlib.sha256(data).hexdigest(),
                         registry_prefix_sha256=hashlib.sha256(data[:REGISTRY_END]).hexdigest(),
                         anchored_event_count=(anchors[0] - 32 - REGISTRY_END) // 4 if anchors else None,
                         event_versions=[word(REGISTRY_END + 4 * code) for code in range(123)],
                         build_version=data[BUILD:BUILD + 32].hex(),
                         build_branch=data[BUILD + 32:BUILD + 64].hex(),
                         corruption_checks=bool(data[FLAG] & 0x80)))
    assert rows, 'No corpus manifests found'
    print(json.dumps(dict(registry_end_byte=REGISTRY_END, event_count=123,
                          header_end_bit=FLAG * 8 + 1, films=rows), indent=2))


if __name__ == '__main__':
    main()
