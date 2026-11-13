"""Trace unchanged pinned LevelUp predicted-ability and point readers.

External precision/table context is fixed explicitly, not a catalogue oracle.
The branch grammar is extracted verbatim after checking both source Git blobs.
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
    'components_biped_spartan.go': '973e443021d6048ce4569e11c33924cb23a68162',
    'lecteur_position.go': '6033b0366c28cf899883ab35681ba4eb1f5f8b16',
}
SELECT = {
    'components_biped_spartan.go': [
        'func consumeBipedSpartanAbility(', 'func consumeSpartanAbilityTag3(',
    ],
    'lecteur_position.go': [
        'type positionQuantifiee struct', 'type tablesDePosition struct',
        'func lireE494(', 'func lireE494Sur(', 'func lireE524Sur(',
    ],
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
    selected.extend(extract(raw.decode(), prefix) for prefix in SELECT[name])

harness = r'''
package main
import("encoding/hex";"encoding/json";"os";"fmt")
const rawVec3Bits=96
const niveauPosition=16
const EtatCapaciteActive=0
type Observation struct { SpartanAbilityHook func(uint64,uint64,uint64,bool) }
func(o *Observation) compterIndexAbsolu(index int) {}
type Lecteur struct { data []byte; at,limit int; snapshot bool; trace [][2]int; obs *Observation }
func(r *Lecteur) ReadBits(n uint)uint64 {
 if int(n)>r.limit-r.at {panic("truncated fixture")}
 r.trace=append(r.trace,[2]int{r.at,int(n)})
 var v uint64
 for i:=uint(0);i<n;i++{v=v<<1|uint64((r.data[r.at/8]>>uint(7-r.at%8))&1);r.at++}
 return v
}
func(r *Lecteur) ReadBit()bool{return r.ReadBits(1)!=0}
func(r *Lecteur) publishEtatMouvement(kind int, values ...uint64) {}
func fullPrecisionGate(r *Lecteur)bool{return r.snapshot}
func(r *Lecteur) tablesDuProfil()tablesDePosition {
 return tablesDePosition{indexW:1,indexLisible:true,regionSeule:true,region:0,axesCarte:[3]uint{15,15,17}}
}
func largeursDeLaLigne(t tablesDePosition,index,niveau int)[3]uint {
 if niveau!=16 {panic("unexpected precision")}
 if index == -1 {return [3]uint{22,22,22}}
 if index != 0 {panic("unexpected external region")}
 return t.axesCarte
}
func main(){
 var cases []struct{Name,Component,Hex string;BitLen int `json:"bit_len"`;Snapshot,Complete bool}
 b,e:=os.ReadFile(os.Args[1]);if e!=nil{panic(e)};if e=json.Unmarshal(b,&cases);e!=nil{panic(e)}
 for _,c:=range cases{
  if c.Component!="biped-spartan-ability-component" || !c.Complete {panic(c.Name)}
  d,e:=hex.DecodeString(c.Hex);if e!=nil{panic(e)}
  r:=&Lecteur{data:d,limit:c.BitLen,snapshot:c.Snapshot}
  consumeBipedSpartanAbility(r)
  if r.at!=c.BitLen{panic(fmt.Sprintf("%s: %d != %d",c.Name,r.at,c.BitLen))}
  t,_:=json.Marshal(r.trace);fmt.Printf("%s\t%d\t%s\n",c.Name,r.at,t)
 }
}
'''
cases_path = Path(__file__).with_name('predicted-ability-source.json')
with tempfile.TemporaryDirectory(prefix='halo-predicted-ability-oracle-') as tmp:
    root = Path(tmp)
    (root / 'main.go').write_text(harness)
    (root / 'source.go').write_text('package main\n' + '\n\n'.join(selected))
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
