"""Inventory type-6/10 bodies using only sequential recorded packet envelopes.

Usage: python3 packet_bodies.py CORPUS_ROOT
This measures bytes, not a proposed body grammar or a no-state-change policy.
"""
from collections import Counter
import hashlib
import json
from pathlib import Path
import struct
import sys
import zlib


def inventory(root):
    results = {k: {'sizes': Counter(), 'payloads': Counter(), 'neighbors': Counter(),
                   'examples': {}, 'flags': Counter(), 'after_previous_time_us': Counter()}
               for k in [6, 10]}
    inputs = []
    for path in sorted(root.rglob('film.json')):
        name = path.parent.relative_to(root).as_posix()
        manifest = json.loads(path.read_text())
        hashes = []
        for chunk in manifest['chunks']:
            if chunk['chunk_type'] == 1:
                continue
            raw = (path.parent / chunk['file']).read_bytes()
            hashes.append({'file': chunk['file'], 'sha256': hashlib.sha256(raw).hexdigest()})
            compressed = (len(raw) >= 2 and raw[0] & 15 == 8 and raw[0] >> 4 <= 7
                          and int.from_bytes(raw[:2], 'big') % 31 == 0)
            data = zlib.decompress(raw) if compressed else raw
            packets, pos = [], 0
            while pos + 16 <= len(data):
                kind, flags, size, timestamp = struct.unpack_from('<HHIQ', data, pos)
                end = pos + 16 + size
                assert end <= len(data), (name, chunk['file'], pos)
                packets.append((kind, flags, timestamp, pos, data[pos+16:end]))
                pos = end
            assert pos == len(data), (name, chunk['file'], pos, len(data))
            for i, (kind, flags, timestamp, offset, body) in enumerate(packets):
                if kind not in results:
                    continue
                result = results[kind]
                result['sizes'][len(body)] += 1
                result['payloads'][body.hex()] += 1
                result['flags'][flags] += 1
                previous = packets[i-1] if i else None
                following = packets[i+1] if i+1 < len(packets) else None
                result['neighbors'][str((previous[0] if previous else None,
                                         following[0] if following else None))] += 1
                if previous:
                    result['after_previous_time_us'][timestamp-previous[2]] += 1
                examples = result['examples'].setdefault(len(body), [])
                if len(examples) < 3 and not any(e['hex'] == body.hex() for e in examples):
                    examples.append({'film': name, 'file': chunk['file'], 'packet': i,
                                     'offset': offset, 'time_us': timestamp, 'flags': flags,
                                     'hex': body.hex()})
        inputs.append({'film': name, 'manifest_sha256': hashlib.sha256(path.read_bytes()).hexdigest(),
                       'chunks': hashes})
    assert inputs
    out = {'inputs': inputs, 'packet_types': {}}
    for kind, r in results.items():
        times = r['after_previous_time_us']
        out['packet_types'][kind] = {
            'count': sum(r['sizes'].values()), 'sizes': dict(sorted(r['sizes'].items())),
            'distinct_payloads': len(r['payloads']),
            'most_common_payloads': r['payloads'].most_common(8),
            'header_flags': dict(sorted(r['flags'].items())),
            'neighbors_previous_next': dict(sorted(r['neighbors'].items())),
            'after_previous_time_us': {'min': min(times), 'max': max(times),
                                       'most_common': times.most_common(8)},
            'examples_by_size': r['examples'],
        }
    return out


if __name__ == '__main__':
    print(json.dumps(inventory(Path(sys.argv[1])), indent=2, sort_keys=True))
