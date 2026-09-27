#!/usr/bin/env python3
"""Verify the pinned Go runtime source and the extracted float32 power table."""
import argparse
import hashlib
import json
import re
from pathlib import Path


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('go_root', type=Path)
    args = parser.parse_args()
    reference = Path(__file__).resolve().parent
    pin = json.loads((reference / 'facts-json-float-runtime.json').read_text())
    root = args.go_root / 'src/internal/strconv'
    for name, expected in pin['files'].items():
        assert hashlib.sha256((root / name).read_bytes()).hexdigest() == expected, name
    native = re.findall(r'\{(0x[0-9a-f]+), 0x[0-9a-f]+\}, // 1e(-?\d+)',
                        (root / 'pow10tab.go').read_text())
    expected = [(int(k), int(hi, 16)) for hi, k in native if -31 <= int(k) <= 46]
    rust = re.findall(r'(0x[0-9a-f]+), // 1e(-?\d+)',
                      (reference.parent / 'facts_json_float.rs').read_text())
    assert [(int(k), int(hi, 16)) for hi, k in rust] == expected
    assert len(expected) == 78
    print(json.dumps({'go_version': pin['go_version'], 'source_hashes': len(pin['files']),
                      'float32_power_entries': len(expected), 'verified': True}))


if __name__ == '__main__':
    main()
