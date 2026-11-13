"""Execute unchanged pinned projectile/point readers with bounded primitives.

The original functions/constants are extracted only after Git blob checks.
External delta/precision context and observation sinks are supplied by the
harness. No projectile branches are substituted or patched.
"""
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile

PIN = '413971bcc485be53a2fa8cf229e170539a5146f8'
BLOBS = {
    'vue_a_charges_armes.go': '2a1a7be5519ca9e483247ad587781adc7bc33dd0',
    'lecteur_position.go': '6033b0366c28cf899883ab35681ba4eb1f5f8b16',
    'bit_leaf_readers.go': 'a0bbe36af9f333cc35d5b9f3cd42307097ccb0e2',
    'vue_a_charges.go': 'd022e4cd4b92c755c4210fed655423129ecc175e',
}
SELECT = {
    'vue_a_charges_armes.go': [
        'func lireArmeEtVariante(', 'func lireVarianteSiAbsente(',
        'func chargeDetonation(', 'func chargeImpact(',
    ],
    'lecteur_position.go': [
        'type positionQuantifiee struct', 'type tablesDePosition struct',
        'func lireE494Sur(', 'func lireE524Sur(',
    ],
    'bit_leaf_readers.go': ['func consumeGateR('],
    'vue_a_charges.go': [],
}


def extract(source, prefix):
    start = source.index(prefix)
    opening = source.index('{', start)
    depth, end = 1, opening + 1
    while depth:
        depth += (source[end] == '{') - (source[end] == '}')
        end += 1
    return source[start:end]


selected = []
for name, blob in BLOBS.items():
    raw = (Path(sys.argv[1]) / name).read_bytes()
    assert hashlib.sha1(f'blob {len(raw)}\0'.encode() + raw).hexdigest() == blob
    source = raw.decode()
    selected.extend(extract(source, prefix) for prefix in SELECT[name])
    if name == 'vue_a_charges_armes.go':
        blocks = re.findall(r'const \(\n.*?\n\)', source, re.S)
        assert len(blocks) == 2
        selected.extend(blocks)
    if name == 'vue_a_charges.go':
        selected.append(re.search(r'^const largeurDirection = \d+$', source, re.M).group())

MODULE = 'levelup/go-api/internal/games/halo_infinite/film/internal/grammar'
HARNESS = r'''
package main
import("encoding/hex";"encoding/json";"os";"fmt";
 "levelup/go-api/internal/games/halo_infinite/film/internal/grammar/lecture")
const rawVec3Bits=96
type Observation struct{}
func(o *Observation) compterIndexAbsolu(index int){}
type VueA struct{positions tablesDePosition}
type Lecteur struct{
 data []byte; at,limit int; trace [][2]int; obs *Observation
 vueA VueA; detonationLue lecture.MessageDeDetonation
}
func(r *Lecteur) ReadBits(n uint)uint64{
 if int(n)>r.limit-r.at{panic("truncated fixture")}
 r.trace=append(r.trace,[2]int{r.at,int(n)})
 var v uint64
 for i:=uint(0);i<n;i++{v=v<<1|uint64((r.data[r.at/8]>>uint(7-r.at%8))&1);r.at++}
 return v
}
func(r *Lecteur) ReadBit()bool{return r.ReadBits(1)!=0}
func(r *Lecteur) Skip(n int){r.ReadBits(uint(n))}
func fullPrecisionGate(r *Lecteur)bool{return false}
func dequantDetonationPosition(r *Lecteur,p positionQuantifiee){}
func largeursDeLaLigne(t tablesDePosition,index,niveau int)[3]uint{
 if niveau!=15 && niveau!=12{panic("unexpected precision")}
 widths:=t.axesCarte
 if index == -1{widths=[3]uint{22,22,22}}else if index!=0{panic("unexpected region")}
 for i:=range widths{widths[i]-=uint(16-niveau)}
 return widths
}
func main(){
 var cases []struct{Name,Hex string; Code int; BodyStart int `json:"body_start"`;RecordEnd int `json:"record_end"`}
 b,e:=os.ReadFile(os.Args[1]);if e!=nil{panic(e)};if e=json.Unmarshal(b,&cases);e!=nil{panic(e)}
 for _,c:=range cases{
  d,e:=hex.DecodeString(c.Hex);if e!=nil{panic(e)}
  r:=&Lecteur{data:d,at:c.BodyStart,limit:c.RecordEnd}
  r.vueA.positions=tablesDePosition{indexW:1,indexLisible:true,regionSeule:true,region:0,axesCarte:[3]uint{15,15,17}}
  var ok bool
  switch c.Code{case 5:ok=chargeDetonation(r);case 6:ok=chargeImpact(r);default:panic(c.Name)}
  if !ok || r.at!=c.RecordEnd{panic(fmt.Sprintf("%s: %d != %d",c.Name,r.at,c.RecordEnd))}
  t,_:=json.Marshal(r.trace);fmt.Printf("%s\t%d\t%s\n",c.Name,r.at,t)
 }
}
'''
cases_path = Path(__file__).with_name('projectile-source.json')
with tempfile.TemporaryDirectory(prefix='halo-projectile-source-') as tmp:
    root = Path(tmp)
    (root / 'go.mod').write_text(f'module {MODULE}\n\ngo 1.24\n')
    (root / 'lecture').mkdir()
    # This observation-only sink is reset by the original readers; it consumes no bits.
    (root / 'lecture/lecture.go').write_text('package lecture\ntype MessageDeDetonation struct{}\n')
    (root / 'main.go').write_text(HARNESS)
    (root / 'source.go').write_text(f'package main\nimport "{MODULE}/lecture"\n' + '\n\n'.join(selected))
    (root / 'cases.json').write_bytes(cases_path.read_bytes())
    output = subprocess.check_output([
        os.environ.get('GO_BIN', 'go'), 'run', 'main.go', 'source.go', 'cases.json',
    ], cwd=root, text=True)
rows = []
for line in output.splitlines():
    name, end, trace = line.split('\t')
    rows.append(dict(name=name, end=int(end), trace=json.loads(trace)))
assert len(rows) == len(json.loads(cases_path.read_text()))
print(json.dumps(dict(commit=PIN, blobs=BLOBS, cases=rows), indent=2))
