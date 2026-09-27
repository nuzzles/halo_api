#!/usr/bin/env python3
"""Extract literal registry catalog data from the pinned native Go source.

This does not execute the reference classifier. Its result is independently
compared with the native public table and classifier by the Rust oracle test.
"""
import json
from pathlib import Path
import re
import sys


def extract(source):
    text = Path(source).read_text()
    constants = dict(re.findall(r'^\s*(\w+)\s*=\s*("(?:[^"\\]|\\.)*")', text, re.M))
    constants = {key: json.loads(value) for key, value in constants.items()}
    pattern = re.compile(
        r'Cle:\s*(\w+),\s*Empreinte:\s*(0x[0-9a-f]+),\s*'
        r'Blocs:\s*(\d+),\s*SlotsNommes:\s*(\d+),\s*'
        r'Statut:\s*(\w+),\s*Source:\s*(\w+),\s*'
        r'Temoins:\s*\[\]string\{([^}]+)\},\s*'
        r'Preuve:\s*((?:"(?:[^"\\]|\\.)*"\s*\+?\s*)+),\s*Date:\s*(\w+),'
    )
    rows = []
    for key, fingerprint, blocks, slots, status, origin, witnesses, proof, date in pattern.findall(text):
        assert origin == 'ProvenanceMesuree'
        rows.append(dict(Cle=constants[key], Empreinte=int(fingerprint, 16), Blocs=int(blocks),
                         SlotsNommes=int(slots), Statut={'StatutCatalogueConnue': 'connue',
                         'StatutCataloguePresumee': 'presumee'}[status], Source='mesuree',
                         Temoins=json.loads('[' + witnesses + ']'),
                         Preuve=''.join(json.loads(s) for s in re.findall(r'"(?:[^"\\]|\\.)*"', proof)),
                         Date=constants[date]))
    assert len(rows) == 9, f'Expected all nine native catalog rows, got {len(rows)}'
    return rows


if __name__ == '__main__':
    Path(sys.argv[2]).write_text(json.dumps(extract(sys.argv[1]), ensure_ascii=False, indent=2) + '\n')
