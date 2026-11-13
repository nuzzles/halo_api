"""Count sequential packet envelopes in the separately retained film corpus.

Usage: python3 packet_inventory.py /path/to/experiments/films
No marker search, body inference or player data is included in the output.
"""
from collections import Counter
import json
from pathlib import Path
import struct
import sys
import zlib


def inventory(root):
    counts, zero, after, suffixes = (Counter() for _ in range(4))
    films = sorted(root.rglob('film.json'))
    assert films, 'No film manifests found'
    chunks = 0
    for path in films:
        manifest = json.loads(path.read_text())
        for chunk in manifest['chunks']:
            if chunk['chunk_type'] == 1:  # Registry has a different envelope.
                continue
            raw = (path.parent / chunk['file']).read_bytes()
            # The retained corpus uses ordinary zlib or uncompressed chunks.
            is_zlib = (len(raw) >= 2 and raw[0] & 15 == 8 and raw[0] >> 4 <= 7
                       and int.from_bytes(raw[:2], 'big') % 31 == 0)
            data = zlib.decompress(raw) if is_zlib else raw
            pos, seen_end = 0, False
            while pos + 16 <= len(data):
                kind, _, size, _ = struct.unpack_from('<HHIQ', data, pos)
                end = pos + 16 + size
                assert end <= len(data), f'{path}: packet exceeds chunk'
                counts[kind] += 1
                if size == 0:
                    zero[kind] += 1
                if seen_end:
                    after[kind] += 1
                if kind == 7:
                    seen_end = True
                    suffixes[len(data) - end] += 1
                pos = end
            assert pos == len(data), f'{path}: incomplete packet walk'
            chunks += 1
    return dict(films=len(films), chunks=chunks, packets=dict(sorted(counts.items())),
                zero_payloads=dict(zero), packets_after_7=dict(after),
                bytes_after_7=dict(suffixes))


if __name__ == '__main__':
    print(json.dumps(inventory(Path(sys.argv[1])), indent=2))
