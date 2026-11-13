"""Compare bounded local evidence with unchanged native blob/registry searches."""
import json
from pathlib import Path
import sys

local=json.loads(Path(sys.argv[1]).read_text())
native=json.loads(Path(sys.argv[2]).read_text())
assert len(local)==32
names=occurrences=0
for row,original in zip(local,native['cases'],strict=True):
    assert row['name']==original['name']
    assert row['script_names']==original['names'],row['name']
    assert row['script_mode']==original['mode'],row['name']
    # This corpus comparison must expose any names found outside the canonical
    # blob or missed by the original fixed-shift whole-registry search.
    assert original['registry_names']==original['names'],row['name']
    assert original['registry_mode']==original['mode'],row['name']
    assert sorted({v['name'].lower() for v in row['script_occurrences']})==row['script_names']
    assert all(v['end']-v['start']==8*len(v['name'].encode('ascii')) for v in row['script_occurrences'])
    names+=len(row['script_names']);occurrences+=len(row['script_occurrences'])
print(f'All 32 bounded searches and mode choices match original blob and whole-registry APIs: {names} per-film distinct names, {occurrences} source occurrences')
