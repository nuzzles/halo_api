"""Extract unchanged pinned guard functions; callbacks isolate envelope grammar.
GO_BIN=... GOCACHE=... python3 export_source.py LEVELUP > reference.json
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
    'vue_a_lecture.go': '6aeb697049e4b4e7181f7c2fd855429cdc4d7097',
    'vue_a_genres.go': 'ee099013bd3359d439801d45df3b724a98ea1870',
    'vue_a_charges.go': 'd022e4cd4b92c755c4210fed655423129ecc175e',
    'traverse.go': '41f10d2d885e8e2b66c50174723af8837c67a787',
    'keyframe_fullstate_loop.go': 'd9d6d39b643e61c0c5e5107c1e94d20522ef2252',
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
    ('vue_a_lecture.go', 'func lireUnMessage('),
    ('vue_a_genres.go', 'func descripteurDuGenre('),
    ('vue_a_charges.go', 'func chargeZoom('),
    ('traverse.go', 'func consumeCorruptionCheck('),
    ('keyframe_fullstate_loop.go', 'func consumeFullStateDefaultBlock('),
]]
genres = sources['vue_a_genres.go']
selected.append(genres[genres.index('const tableDesGenresVueA ='):genres.index('// largeurEntreeDeGenre :')])
selected += ['const largeurEntreeDeGenre = 4', "const marqueGenreVide = 'v'"]
table = ''.join(re.findall(r'"([0-9.v]+)"', selected[-3]))
empty_codes = [i for i in range(123) if table[i*4+3] == 'v']


def pack(words):
    bits = ''.join(f'{v:0{w}b}' for w, v in words)
    return bytes(int((bits+'0'*7)[i:i+8], 2) for i in range(0, len(bits), 8)).hex(), len(bits)


cases = []
for enabled in [False, True]:
    for code in empty_codes + [21]:
        for gate in ([0, 1] if code == 21 and enabled else [0]):
            for sentinel in ([0, 0x0bcddcba, 0xffffffff, 0x12345678] if gate else [0]):
                words = [(1,1),(7,code),(3,0)]
                if code == 21:
                    words += [(2,3)]
                    if enabled:
                        words += [(1,gate)]
                        if gate: words += [(32,sentinel)]
                # Empty successor, then list terminator.
                words += [(1,1),(7,3),(3,0),(1,0)]
                payload, bits = pack(words)
                cases.append(dict(kind='event', enabled=enabled, code=code, gate=gate,
                                  sentinel=sentinel, hex=payload, bits=bits))
    for gate in [0,1]:
        words = [(1,gate)] + ([(32,0x12345678)] if gate else [])
        payload,bits=pack(words)
        cases.append(dict(kind='component',enabled=enabled,hex=payload,bits=bits))
    for n1 in [0,1,0xffffffff,0x80000000]:
        words=[(32,n1)]
        if enabled and n1 == 1: words += [(32,0x12345678)]
        words += [(32,1)]
        payload,bits=pack(words)
        cases.append(dict(kind='default',enabled=enabled,hex=payload,bits=bits,n1=n1))

HARNESS = r'''
package main
import("encoding/json";"encoding/hex";"os")
type Profile struct{Grammaire struct{ControleDeCorruption bool}}
type refDeMessage struct{presente bool;index,gen int}
type Lecteur struct{*Bits;p Profile;vueA struct{genres int};refsDuMessage [3]refDeMessage;portee bool}
func(*Lecteur)cadre()struct{MotDeTailleBits int}{return struct{MotDeTailleBits int}{32}}
func consumeKeyframeDefaultState(*Lecteur,uint32){} // ti0 default stub
func readVarWidthInt(*Lecteur,int)(int,int){panic("reference absent in guard fixtures")}
func chargeDuGenre(code int)func(*Lecteur)bool{if code==21{return chargeZoom};panic("unselected body")}
type Case struct{Kind string;Enabled bool;Hex string}
type Result struct{End int `json:"end"`;OK bool `json:"ok"`;Ends []int `json:"ends"`}
func main(){
 var cases []Case;if e:=json.NewDecoder(os.Stdin).Decode(&cases);e!=nil{panic(e)}
 results:=[]Result{}
 for _,c:=range cases{
  b,e:=hex.DecodeString(c.Hex);if e!=nil{panic(e)}
  r:=&Lecteur{Bits:NewBits(b)};r.p.Grammaire.ControleDeCorruption=c.Enabled;r.vueA.genres=123
  result:=Result{OK:true,Ends:[]int{}}
  switch c.Kind{
  case "event":
   for r.ReadBit(){code:=int(r.ReadBits(7));if !lireUnMessage(r,code){result.OK=false;break};result.Ends=append(result.Ends,r.BitPos())}
  case "component":consumeCorruptionCheck(r)
  case "default":result.OK=consumeFullStateDefaultBlock(r,0,false)
  }
  result.End=r.BitPos();result.OK=result.OK&&!r.Deborde();results=append(results,result)
 }
 if e:=json.NewEncoder(os.Stdout).Encode(results);e!=nil{panic(e)}
}
'''
with tempfile.TemporaryDirectory(prefix='halo-corruption-oracle-') as tmp:
    root = Path(tmp)
    (root/'main.go').write_text(HARNESS)
    (root/'guards.go').write_text('package main\n'+'\n\n'.join(selected))
    (root/'bits.go').write_text(sources['source/bits.go'].replace('package source','package main',1))
    output = subprocess.check_output([os.environ.get('GO_BIN','go'),'run','main.go','guards.go','bits.go'],cwd=root,input=json.dumps(cases).encode())
print(json.dumps(dict(commit=PIN,blobs=BLOBS,empty_codes=empty_codes,cases=cases,results=json.loads(output)),indent=2))
