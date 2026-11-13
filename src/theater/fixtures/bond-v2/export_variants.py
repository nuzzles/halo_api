"""Run the original pinned LevelUp variant walker on valid schema fixtures.

Provide the pinned film_variante_de_partie.go as argv[1]. Its Git blob ID is
verified. Only package/type wiring changes; its walker and projection bodies
remain unchanged. The harness supplies bounded bit/string primitives.
"""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

PIN='413971bcc485be53a2fa8cf229e170539a5146f8'
BLOB='13179ea7149628e9c31c6fe827bbaec005111ab1'
raw=Path(sys.argv[1]).read_bytes()
assert hashlib.sha1(f'blob {len(raw)}\0'.encode()+raw).hexdigest()==BLOB
source=raw.decode().replace('package grammar','package main',1)
source=source.replace('import "levelup/go-api/internal/games/halo_infinite/film/internal/profile"','')
source=source.replace('profile.VarianteDePartie','Variant')
cases=json.loads(Path(__file__).with_name('variants.json').read_text())
cases=[c for c in cases if c['name'].startswith('valid/') or c['name'] in ['defaults','nonzero-bool']]
harness=r'''
package main
import("encoding/json";"encoding/hex";"os";"fmt")
type Variant struct { Lue, Presente bool; TypeDeMoteur int32; KillcamEnabled, PlayOfTheGameEnabled bool }
type Lecteur struct { data []byte; at int; over bool }
func LecteurSur(d []byte)*Lecteur{return &Lecteur{data:d}}
func(r *Lecteur) ReadBits(n uint)uint64{var v uint64;for i:=uint(0);i<n;i++{v<<=1;if r.at>=len(r.data)*8{r.over=true}else{v|=uint64((r.data[r.at/8]>>uint(7-r.at%8))&1)};r.at++};return v}
func(r *Lecteur) Skip(n int){r.at+=n;if r.at>len(r.data)*8{r.over=true}}
func(r *Lecteur) SetBitPos(p int){r.at=p;if p>len(r.data)*8{r.over=true}}
func(r *Lecteur) BitPos()int{return r.at}
func(r *Lecteur) Deborde()bool{return r.over}
func lireChaine(r *Lecteur,n int)bool{for i:=0;i<n;i++{if r.ReadBits(8)==0{return !r.over}};return false}
func main(){var cases []struct{Name,Hex string;BitLen int `json:"bit_len"`};b,_:=os.ReadFile(os.Args[1]);if e:=json.Unmarshal(b,&cases);e!=nil{panic(e)}
 for _,c:=range cases{raw,e:=hex.DecodeString(c.Hex);if e!=nil{panic(e)};d:=make([]byte,len(raw)+1);for i:=0;i<len(raw)*8;i++{d[(i+1)/8]|=((raw[i/8]>>uint(7-i%8))&1)<<uint(7-(i+1)%8)}
 v,end:=marcherLeCorps(d,1);if !v.Lue||!v.Presente||end!=c.BitLen+1{panic(c.Name)}
 fmt.Printf("%s\t%d\t%t\t%t\n",c.Name,v.TypeDeMoteur,v.KillcamEnabled,v.PlayOfTheGameEnabled)
 }}
'''
with tempfile.TemporaryDirectory(prefix='halo-variant-oracle-') as tmp:
    p=Path(tmp);(p/'variant.go').write_text(source);(p/'main.go').write_text(harness)
    (p/'cases.json').write_text(json.dumps(cases))
    output=subprocess.check_output([os.environ.get('GO_BIN','go'),'run','main.go','variant.go','cases.json'],cwd=p,text=True)
rows=[]
for line in output.splitlines():
    name,engine,kill,play=line.split('\t')
    value=[int(engine),kill=='true',play=='true']
    case=cases[len(rows)];assert name==case['name'] and value==case['expected']
    rows.append(dict(name=name,value=value))
assert len(rows)==len(cases)
print(json.dumps(dict(commit=PIN,blob=BLOB,cases=rows),indent=2))
