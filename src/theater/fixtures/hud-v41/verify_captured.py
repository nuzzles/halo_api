"""Compare a captured source projection with export_reference.py's original output."""
import json
from pathlib import Path
import sys

captured=json.loads(Path(sys.argv[1]).read_text())
native=json.loads(Path(sys.argv[2]).read_text())
assert native['commit']=='18067987ebc3628ad499c819c6aee814692df43b'
assert len(captured)==len(native['cases'])
for local,original in zip(captured,native['cases']):
    assert local['name']==original['name']
    assert local['message']==original['message'],local['name']
print(f"Matched {len(captured)} original HUD observations, including absent text and exact f64 numeric bits.")
