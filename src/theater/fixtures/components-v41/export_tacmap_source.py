"""Trace unchanged pinned LevelUp tacmap snapshot readers.

Legacy delta helpers panic. No replacement grammar can silently validate a
case. Delta point bytes are separately compared with original libfilmshell
tether traces in the Rust tests; this export covers full snapshot composition.
"""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

PIN = '413971bcc485be53a2fa8cf229e170539a5146f8'
BLOB = '699712aa7591344f6b60831506ce1e74e8c14f78'
raw = Path(sys.argv[1]).read_bytes()
assert hashlib.sha1(f'blob {len(raw)}\0'.encode() + raw).hexdigest() == BLOB
source = raw.decode()


def extract(prefix):
    start = source.index(prefix)
    opening = source.index('{', start)
    depth, end = 1, opening + 1
    while depth:
        depth += (source[end] == '{') - (source[end] == '}')
        end += 1
    return source[start:end]


selected = '\n\n'.join(extract('func ' + name + '(') for name in [
    'consumeTacmapPoiIcon', 'consumeTacmapDisplayAsset', 'consumeTacmapAreaOfInterest',
])
harness = r'''
package main
import("encoding/hex";"encoding/json";"os";"fmt")
const rawVec3Bits=96
type Lecteur struct { data []byte; at int; trace [][2]int }
func(r *Lecteur) ReadBits(n uint)uint64 { r.trace=append(r.trace,[2]int{r.at,int(n)}); var v uint64; for i:=uint(0);i<n;i++{ if r.at>=len(r.data)*8{panic("truncated fixture")};v=v<<1|uint64((r.data[r.at/8]>>uint(7-r.at%8))&1);r.at++ };return v }
func(r *Lecteur) ReadBit()bool{return r.ReadBits(1)!=0}
func(r *Lecteur) sousLaGardeSinonException()bool{return true}
func lireVecteurAncienAuNiveauDuRegistre(r *Lecteur,level uint32){panic("legacy delta is not a snapshot oracle")}
func lireCorpsDeTraverseeAncien(r *Lecteur){panic("legacy delta is not a snapshot oracle")}
func main(){var cases []struct{Name,Component,Hex string;Version uint32;BitLen int `json:"bit_len"`;Snapshot bool};b,e:=os.ReadFile(os.Args[1]);if e!=nil{panic(e)};if e=json.Unmarshal(b,&cases);e!=nil{panic(e)}
 for _,c:=range cases{if !c.Snapshot{continue};d,e:=hex.DecodeString(c.Hex);if e!=nil{panic(e)};r:=&Lecteur{data:d};switch c.Component{case "tacmap-poiicon":consumeTacmapPoiIcon(r,c.Version);case "tacmap-displayasset":consumeTacmapDisplayAsset(r);case "tacmap-areaofinterest":consumeTacmapAreaOfInterest(r);default:panic(c.Component)};if r.at!=c.BitLen{panic(c.Name)};t,_:=json.Marshal(r.trace);fmt.Printf("%s\t%d\t%s\n",c.Name,r.at,t)}
}
'''
cases_path = Path(__file__).with_name('tacmap-source.json')
with tempfile.TemporaryDirectory(prefix='halo-tacmap-oracle-') as tmp:
    root = Path(tmp)
    (root / 'main.go').write_text(harness)
    (root / 'tacmap.go').write_text('package main\n' + selected)
    (root / 'cases.json').write_bytes(cases_path.read_bytes())
    output = subprocess.check_output([
        os.environ.get('GO_BIN', 'go'), 'run', 'main.go', 'tacmap.go', 'cases.json',
    ], cwd=root, text=True)
rows = []
for line in output.splitlines():
    name, end, trace = line.split('\t')
    rows.append(dict(name=name, end=int(end), trace=json.loads(trace)))
assert len(rows) == sum(c['snapshot'] for c in json.loads(cases_path.read_text()))
print(json.dumps(dict(commit=PIN, blob=BLOB, cases=rows), indent=2))
