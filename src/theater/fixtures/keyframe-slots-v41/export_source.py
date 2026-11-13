"""Exercise pinned native table traversal and full-state guard readers.

GO_BIN=... GOCACHE=... python3 export_source.py PINNED_GRAMMAR > reference.json
Extracted Go functions are unchanged apart from the source package qualifier.
The harness supplies registry/context and reader adapters. Default/component
callbacks panic: these fixtures deliberately use nonpositive body guards.
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
    'keyframe_entrees.go': '5a09df3b557e047a3a2d22200a9834e281c13667',
    'keyframe_fullstate_loop.go': 'd9d6d39b643e61c0c5e5107c1e94d20522ef2252',
    'source/bits.go': '2951f8428d6228a7de607a8f1c1d062b7475da4d',
}
sources = {}
for name, blob in BLOBS.items():
    raw = (Path(sys.argv[1]) / name).read_bytes()
    assert hashlib.sha1(f'blob {len(raw)}\0'.encode() + raw).hexdigest() == blob
    sources[name] = raw.decode()


def extract(source, prefix):
    start = source.index(prefix)
    end = source.index('{', start) + 1
    depth = 1
    while depth:
        depth += (source[end] == '{') - (source[end] == '}')
        end += 1
    return source[start:end]


selected = []
selected.append(next(line for line in sources['keyframe_entrees.go'].splitlines()
                     if line.startswith('const keyframeEntreesDeTable = ')))
for name, prefixes in {
    'keyframe_entrees.go': ['func lireEntreeDImageCle(', 'func lireLesEntrees('],
    'keyframe_fullstate_loop.go': ['type keyframeFullStateTemoin struct',
                                 'func WalkKeyframeFullState(',
                                 'func walkKeyframeFullState(',
                                 'func consumeFullStateDefaultBlock('],
}.items():
    selected.extend(extract(sources[name], p) for p in prefixes)

# Compact recipes keep the fixture readable; Go authors bits from these inputs.
cases = [
    dict(name='empty-payload', count=0, truncate_bytes=0),
    dict(name='prefix-only', count=0),
    dict(name='one-vacant', count=1),
    dict(name='all-vacant', count=8191),
    dict(name='extra-slot', count=8192),
    dict(name='nonzero-alignment', count=8191, nonzero_tail=True),
    dict(name='vacant-arbitrary-header', count=8191,
         entries=[dict(slot=0, id=0, archetype=4294967295, word=123, flags=15, byte=231)]),
    dict(name='interleaved-live', count=8191, entries=[
        dict(slot=1, id=4294967295, archetype=0, word=7, flags=15, byte=221, n1=0, n2=0),
        dict(slot=7, id=5, archetype=0, n1=4294967295, n2=2147483648),
        dict(slot=8190, id=2, archetype=0, n1=0, n2=0)]),
    dict(name='invalid-archetype', count=8191, entries=[dict(slot=4, id=0, archetype=50)]),
    dict(name='truncated-live-guard', count=2,
         entries=[dict(slot=1, id=0, archetype=0, n1=0, n2=0)], truncate_bytes=33),
]
# Every byte boundary of the final required header, including <8 tail bits.
cases += [dict(name=f'last-header-minus-{n}', count=8191,
               truncate_bytes=(1 + 8191 * 108 + 7)//8 - n) for n in range(1, 15)]

HARNESS = r'''
package main
import("encoding/json"; "os")
const keyframeArchetypeNone = 0xffffffff
const kfArchMax = 50
const objectArchetypeCount = 50
const keyframeRecordTIBit = 58
type genreDEntree uint8
const(entreeIllisible genreDEntree=iota; entreeSansCorps; entreeRecord)
type Cadre struct{EnTeteBits, MotDeTailleBits int}
type Profile struct{Cadre Cadre; Grammaire struct{ControleDeCorruption bool}}
type ContexteDeLecture struct{Profil Profile; Obs any}
type Registry struct{}
type Archetype struct{}
func(*Registry)Archetype(n int)(Archetype,bool){return Archetype{},n==0}
type EntityTrace struct{DesyncAt,EndBit int;TypeIndex uint32;Mask uint64}
type Lecteur struct{*Bits;p Profile;portee,etatComplet bool}
func LecteurSur(b []byte)*Lecteur{return &Lecteur{Bits:NewBits(b)}}
func(r *Lecteur)PoserContexte(c ContexteDeLecture){r.p=c.Profil}
func(r *Lecteur)cadre()Cadre{return r.p.Cadre}
func consumeKeyframeDefaultState(*Lecteur,uint32){panic("fixture reached default body")}
func traverserSousLaPortee(*Lecteur,Archetype,*EntityTrace){panic("fixture reached component body")}
type Entry struct{Slot int;Id,Archetype,Word uint32;Flags,Byte uint8;N1,N2 uint32}
type Case struct{Name string;Count int;Entries []Entry;TruncateBytes *int `json:"truncate_bytes"`;NonzeroTail bool `json:"nonzero_tail"`}
type Result struct{Name string `json:"name"`;Bits int `json:"bits"`;Headers int `json:"headers"`;End int `json:"end"`;OK bool `json:"ok"`;Live []int `json:"live"`}
func main(){
 var cases []Case;if e:=json.NewDecoder(os.Stdin).Decode(&cases);e!=nil{panic(e)}
 var out []Result
 ctx:=ContexteDeLecture{Profil:Profile{Cadre:Cadre{108,32}}}
 for _,c:=range cases{
  var bits []bool
  put:=func(n int,v uint64){for i:=n-1;i>=0;i--{bits=append(bits,v>>i&1!=0)}}
  put(1,1)
  for i:=0;i<c.Count;i++{
   e:=Entry{Id:0xffffffff,Archetype:0xffffffff,Word:0xffffffff}
   for _,v:=range c.Entries{if v.Slot==i{e=v}}
   put(32,uint64(e.Id));put(32,uint64(e.Archetype));put(32,uint64(e.Word));put(4,uint64(e.Flags));put(8,uint64(e.Byte))
   if e.Archetype<50{put(32,uint64(e.N1));put(32,uint64(e.N2))}
  }
  for len(bits)%8!=0{bits=append(bits,c.NonzeroTail)}
  pay:=make([]byte,len(bits)/8);for i,b:=range bits{if b{pay[i/8]|=1<<uint(7-i%8)}}
  if c.TruncateBytes!=nil{pay=pay[:*c.TruncateBytes]}
  r:=Result{Name:c.Name,Bits:len(pay)*8,Live:[]int{}}
  pos:=1
  for i:=0;i<keyframeEntreesDeTable;i++{
   kind:=lireEntreeDImageCle(pay,pos,len(pay)*8,108)
   if pos+108<=len(pay)*8{r.Headers++}
   if kind==entreeRecord{r.Live=append(r.Live,pos)}
   next,ok:=lireLesEntrees(pay,pos,1,&Registry{},ctx)
   if !ok{break};pos=next
  }
  r.End,r.OK=lireLesEntrees(pay,1,keyframeEntreesDeTable,&Registry{},ctx)
  r.OK=r.OK&&len(pay)*8-r.End<8
  out=append(out,r)
 }
 enc:=json.NewEncoder(os.Stdout);if e:=enc.Encode(out);e!=nil{panic(e)}
}
'''
with tempfile.TemporaryDirectory(prefix='halo-keyframe-slots-') as tmp:
    root = Path(tmp)
    (root / 'main.go').write_text(HARNESS)
    (root / 'source.go').write_text('package main\n' + '\n\n'.join(selected).replace('source.', ''))
    (root / 'bits.go').write_text(sources['source/bits.go'].replace('package source', 'package main', 1))
    output = subprocess.check_output([os.environ.get('GO_BIN', 'go'), 'run',
                                      'main.go', 'source.go', 'bits.go'],
                                     cwd=root, input=json.dumps(cases).encode())
print(json.dumps(dict(commit=PIN, blobs=BLOBS, cases=cases, results=json.loads(output)), indent=2))
