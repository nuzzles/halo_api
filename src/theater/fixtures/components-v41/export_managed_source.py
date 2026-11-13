"""Trace pinned, unchanged manager readers where libfilmshell differs.

Timer/TLV functions and constants are extracted verbatim; music is one unchanged
dispatch arm wrapped in a standalone switch. Only bounded bit I/O is supplied.
No halo_api code, inferred widths, replacement grammar or map context is used.
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
    'components_moteur_de_partie.go': '4b6f127a8941c9f16097c7d512c2ec08b49e16c5',
    'lecteur_minuteur.go': 'dc382adaa1bbfe38e4f8004f094aaf04da636bd6',
    'dispatch_player.go': '3e58e6eb74179f6f69c96e20592b1dfb2c98d2dc',
    'components_batch7.go': '73d02925cc5051252605236dfb3290af79c37a84',
    'tlv_mode2.go': '3709d03b18693a463257d899d9b34804ab418b64',
}
sources = {}
for name, blob in BLOBS.items():
    raw = (Path(sys.argv[1]) / name).read_bytes()
    assert hashlib.sha1(f'blob {len(raw)}\0'.encode() + raw).hexdigest() == blob
    sources[name] = raw.decode()


def extract(source, prefix):
    start = source.index(prefix)
    opening = source.index('{', start)
    depth, end = 1, opening + 1
    while depth:
        depth += (source[end] == '{') - (source[end] == '}')
        end += 1
    return source[start:end]


motor = sources['components_moteur_de_partie.go']
selected = [extract(motor, 'func consumeManagedEngineTimers(')]
for marker in ['// Largeurs lues', '// Les etiquettes d une fente']:
    start = motor.index('const (', motor.index(marker))
    end = motor.index('\n)', start) + 2
    selected.append(motor[start:end])
for name in ['lecteur_minuteur.go', 'tlv_mode2.go']:
    selected.append(sources[name].removeprefix('package grammar'))
for name in ['consumeObjectMultiplayerProperties', 'consumeManagedSplashMessage']:
    selected.append(extract(sources['components_batch7.go'], 'func ' + name + '('))
dispatch = sources['dispatch_player.go']
start = dispatch.index('\tcase "music-state-component":')
end = dispatch.index('\tcase "flock-destroying-component":', start)
arm = dispatch[start:end]
start_alliance = dispatch.index('\tcase \"game-engine-alliance-component\":')
end_alliance = dispatch.index('\tcase compGameEngineCurrentRound:', start_alliance)
arm += dispatch[start_alliance:end_alliance]
selected.append('func music(br *Lecteur, name string) (variant uint32, dead *int, ported bool) {\n'
                'switch name {\n' + arm + '\n}; panic("unexpected component")\n}')

harness = r'''
package main
import("encoding/hex";"encoding/json";"os";"fmt")
type Lecteur struct { data []byte; at,limit int; trace [][2]int }
func(r *Lecteur) ReadBits(n uint)uint64 {
 if int(n)>r.limit-r.at {panic("truncated fixture")}
 r.trace=append(r.trace,[2]int{r.at,int(n)})
 var v uint64
 for i:=uint(0);i<n;i++{v=v<<1|uint64((r.data[r.at/8]>>uint(7-r.at%8))&1);r.at++}
 return v
}
func(r *Lecteur) ReadBit()bool{return r.ReadBits(1)!=0}
func(r *Lecteur) Skip(n int){if n<0{panic("negative skip")};r.ReadBits(uint(n))}
func main(){
 var cases []struct{Name,Component,Hex string;BitLen int `json:"bit_len"`;Complete bool}
 b,e:=os.ReadFile(os.Args[1]);if e!=nil{panic(e)};if e=json.Unmarshal(b,&cases);e!=nil{panic(e)}
 for _,c:=range cases{
  if !c.Complete {panic(c.Name)}
  d,e:=hex.DecodeString(c.Hex);if e!=nil{panic(e)}
  r:=&Lecteur{data:d,limit:c.BitLen}
  switch c.Component {
  case "music-state-component", "game-engine-alliance-component":music(r,c.Component)
  case "managed-engine-timers-component":consumeManagedEngineTimers(r)
  case "managed-object-networked-splash-message-static-component":consumeManagedSplashMessage(r)
  case "object-multiplayer-properties-component":consumeObjectMultiplayerProperties(r)
  default:panic(c.Component)
  }
  if r.at!=c.BitLen{panic(fmt.Sprintf("%s: %d != %d",c.Name,r.at,c.BitLen))}
  t,_:=json.Marshal(r.trace);fmt.Printf("%s\t%d\t%s\n",c.Name,r.at,t)
 }
}
'''
cases_path = Path(__file__).with_name('managed-source.json')
with tempfile.TemporaryDirectory(prefix='halo-managed-oracle-') as tmp:
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
