"""Native category-handle values and prefixes, independent of halo_api.
GO_BIN=... GOCACHE=... python3 export_source.py PINNED_GRAMMAR > reference.json
"""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

PIN = '413971bcc485be53a2fa8cf229e170539a5146f8'
BLOBS = {
    'varwidth.go': 'ed8ddb806fc86634d87fd68490eb47d3c96b5f8d',
    'bit_leaf_readers.go': 'a0bbe36af9f333cc35d5b9f3cd42307097ccb0e2',
    'quantize.go': '66dc765b8189a00a5cc86b09e1a7294605a645b5',
    'unit_ref_probe.go': '60a564eb1f31363f7fa33c29f1aae8058f404a57',
    'source/bits.go': '2951f8428d6228a7de607a8f1c1d062b7475da4d',
}
sources = {}
for name, blob in BLOBS.items():
    raw = (Path(sys.argv[1]) / name).read_bytes()
    assert hashlib.sha1(f'blob {len(raw)}\0'.encode() + raw).hexdigest() == blob
    sources[name] = raw.decode()


def extract(name, prefix):
    source = sources[name]
    start = source.index(prefix)
    end = source.index('{', start) + 1
    depth = 1
    while depth:
        depth += (source[end] == '{') - (source[end] == '}')
        end += 1
    return source[start:end]


selected = [extract(n, p) for n, p in [
    ('bit_leaf_readers.go', 'func readVarWidthInt('),
    ('bit_leaf_readers.go', 'func consume1408f0ac4Probe('),
    ('varwidth.go', 'func varWidthRange('),
    ('varwidth.go', 'func varWidthBits('),
    ('quantize.go', 'func bitLen('),
    ('unit_ref_probe.go', 'type UnitRefRead struct'),
]]
unit_refs = sources['unit_ref_probe.go']
selected.append(unit_refs[unit_refs.index('type UnitRefKind int'):unit_refs.index('// String rend')])
for prefix in ['const varWidthDefaultRange ', 'const varWidthProbeSlot ',
               'const varWidthProbeCategory ']:
    selected.append(next(line for line in sources['varwidth.go'].splitlines()
                         if line.startswith(prefix)))

cases = []
for category in range(9):
    for optional, present in [(False, True), (True, False), (True, True)]:
        for probe in ([False, True] if category == 1 and present else [False]):
            selected_category = 4 if category == 1 and probe else category
            width = 8 if selected_category in (2, 3, 5) else 9 if selected_category in (4, 6) else 13
            for value, tail in ([(0, 0), ((1 << width) - 1, 3)] if present else [(0, 0)]):
                body = str(int(present)) if optional else ''
                if present:
                    if category == 1:
                        body += str(int(probe))
                    body += f'{value:0{width}b}{tail:02b}'
                shift = (-len(body)) % 8
                aligned = '0' * shift + body
                payload = ''.join(f'{int(aligned[i:i+8], 2):02x}' for i in range(0, len(aligned), 8))
                cases.append(dict(category=category, optional=optional, present=present,
                                  probe=probe, value=value, tail=tail, start=shift,
                                  bits=len(body), hex=payload))

HARNESS = r'''
package main
import("encoding/json";"encoding/hex";"os";"math/bits")
type Observer struct{Refs []UnitRefRead}
func(o *Observer)publishUnitRef(r UnitRefRead){o.Refs=append(o.Refs,r)}
type Lecteur struct{*Bits;obs Observer}
type Case struct{Category,Start int;Optional bool;Hex string}
type Result struct{End int;Value,Tail uint64;Present bool;Refs []UnitRefRead}
func main(){
 var cases []Case;if e:=json.NewDecoder(os.Stdin).Decode(&cases);e!=nil{panic(e)}
 out:=[]Result{}
 for _,c:=range cases{
  b,e:=hex.DecodeString(c.Hex);if e!=nil{panic(e)}
  r:=&Lecteur{Bits:NewBits(b)};r.SetBitPos(c.Start)
  result:=Result{Present:true}
  if c.Optional{result.Value,result.Tail,result.Present=consume1408f0ac4Probe(r,c.Category)
  }else{result.Value,result.Tail=readVarWidthInt(r,c.Category)}
  result.End=r.BitPos();result.Refs=r.obs.Refs;out=append(out,result)
 }
 if e:=json.NewEncoder(os.Stdout).Encode(out);e!=nil{panic(e)}
}
'''

with tempfile.TemporaryDirectory(prefix='halo-reference-configuration-') as tmp:
    root = Path(tmp)
    (root / 'go.mod').write_text('module reference_configuration\ngo 1.23\n')
    (root / 'bits.go').write_text(sources['source/bits.go'].replace('package source', 'package main', 1))
    (root / 'main.go').write_text(HARNESS + '\n' + '\n\n'.join(selected))
    output = subprocess.run([os.environ.get('GO_BIN', 'go'), 'run', '.'], cwd=root,
                            input=json.dumps(cases), text=True, capture_output=True, check=True)
    results = json.loads(output.stdout)
    for case, result in zip(cases, results):
        assert result['End'] == case['start'] + case['bits']
        assert result['Value'] == case['value']
        assert result['Tail'] == case['tail']
        assert result['Present'] == case['present']
    print(json.dumps(dict(commit=PIN, blobs=BLOBS, cases=cases, results=results), indent=2))
