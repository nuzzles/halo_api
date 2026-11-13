"""Run the unchanged LevelUp width law and indexed point reader, including R(0)."""
from pathlib import Path
import hashlib
import json
import os
import subprocess
import sys
import tempfile

PIN = "413971bcc485be53a2fa8cf229e170539a5146f8"
BLOBS = {
    "profile/loi_largeurs.go": "623a55741a4578c57bacb3a7abf47fffe8d5737d",
    "lecteur_position.go": "6033b0366c28cf899883ab35681ba4eb1f5f8b16",
}
root = Path(sys.argv[1])
go = sys.argv[2] if len(sys.argv) > 2 else "go"
sources = {}
for name, blob in BLOBS.items():
    b = (root / name).read_bytes()
    assert hashlib.sha1(f"blob {len(b)}\0".encode() + b).hexdigest() == blob, name
    sources[name] = b.decode()

def block(prefix):
    s = sources["lecteur_position.go"]
    start = s.index(prefix)
    end = s.index("{", start) + 1
    depth = 1
    while depth:
        depth += (s[end] == "{") - (s[end] == "}")
        end += 1
    return s[start:end]

reader = "\n".join(block(p) for p in [
    "type positionQuantifiee struct", "type tablesDePosition struct",
    "func largeursDeLaLigne(", "func lireE524Sur(",
])
main = r'''
type observer struct{}
func (observer) compterIndexAbsolu(int) {}
type Lecteur struct { data []byte; at uint; over bool; trace [][2]uint; obs observer }
func (r *Lecteur) ReadBits(n uint) uint64 {
    if r.at+n > uint(len(r.data)*8) { r.over=true; return 0 }
    r.trace=append(r.trace,[2]uint{r.at,n}); var v uint64
    for i:=uint(0);i<n;i++ { v=v<<1|uint64((r.data[r.at/8]>>(7-r.at%8))&1);r.at++ }; return v
}
func(r *Lecteur) ReadBit() bool{return r.ReadBits(1)!=0}
type wire struct { bits []bool }
func(w *wire) put(n uint,v uint64) {for i:=int(n)-1;i>=0;i--{w.bits=append(w.bits,v&(uint64(1)<<uint(i))!=0)}}
func(w *wire) bytes() []byte {b:=make([]byte,(len(w.bits)+7)/8);for i,v:=range w.bits{if v{b[i/8]|=1<<uint(7-i%8)}};return b}
type row struct {
    Name string `json:"name"`; Precision int `json:"precision"`
    Min [3]uint32 `json:"min"`; Max [3]uint32 `json:"max"`
    Widths [3]uint `json:"widths"`; Codes [3]uint64 `json:"codes"`
    Hex string `json:"hex"`; BitLen uint `json:"bit_len"`; Trace [][2]uint `json:"trace"`
}
func main() {
    tiny:=[2]float32{0,0.001}; normal:=[2]float32{-1,1}
    boxes:=[][3][2]float32{
        {tiny,tiny,tiny},{tiny,normal,tiny},{normal,tiny,normal},{normal,normal,tiny},
        {{0.125,0.126},{-0.126,-0.125},tiny},
        {{0,math.Float32frombits(0x3c888888)},{0,math.Float32frombits(0x3c888889)},{0,math.Float32frombits(0x3c88888a)}},
        {{0,0.016},{0,0.017},{0,0.032}},
        {{0,1e7},{0,200},{0,40000}},
    }
    var rows []row
    for bi,b:=range boxes {for _,level:=range []int{12,15,16,22,23} {for variant:=0;variant<2;variant++ {
        widths:=profile.LargeursAxeDuNiveau(b,level)
        t:=tablesDePosition{indexW:2,axesCarte:widths,bornesCarte:b,indexLisible:true,regionSeule:true,region:3}
        var w wire;w.put(1,0);w.put(2,3)
        r:=row{Name:fmt.Sprintf("box%d-level%d-v%d",bi,level,variant),Precision:level,Widths:widths}
        for a,n:=range widths {
            q:=(uint64(1)<<n)-1;if variant==0 {q/=2};r.Codes[a]=q;w.put(n,q)
            r.Min[a]=math.Float32bits(b[a][0]);r.Max[a]=math.Float32bits(b[a][1])
        }
        r.BitLen=uint(len(w.bits));w.put(8,0xd3);data:=w.bytes();r.Hex=hex.EncodeToString(data)
        br:=Lecteur{data:data};p,ok:=lireE524Sur(&br,level,t)
        if !ok||br.over||br.at!=r.BitLen||p.idx!=3||p.q!=r.Codes||p.w!=widths {panic("point")}
        r.Trace=br.trace;if br.ReadBits(8)!=0xd3 {panic("marker")};rows=append(rows,r)
    }}}
    if err:=json.NewEncoder(os.Stdout).Encode(rows);err!=nil{panic(err)}
}
'''
with tempfile.TemporaryDirectory(prefix="halo-zero-axis-oracle-") as tmp:
    p = Path(tmp)
    (p / "profile").mkdir()
    (p / "profile/loi_largeurs.go").write_text(sources["profile/loi_largeurs.go"])
    (p / "go.mod").write_text("module zerooracle\n\ngo 1.23\n")
    (p / "main.go").write_text('package main\nimport("encoding/hex";"encoding/json";"fmt";"math";"os";"zerooracle/profile")\n' + reader + main)
    env = dict(os.environ, GOPROXY="off", GOWORK="off", GOTOOLCHAIN="local", GOCACHE=str(p / "cache"))
    cases = json.loads(subprocess.check_output([go, "run", "."], cwd=p, env=env, text=True))
assert len(cases) == 80
print(json.dumps(dict(commit=PIN, blobs=BLOBS, cases=cases), indent=2))
