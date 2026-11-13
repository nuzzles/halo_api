"""Pinned native version classification and event-head admission oracle.
GO_BIN=... GOCACHE=... python3 export_source.py PINNED_GRAMMAR > reference.json
"""
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile

PIN='413971bcc485be53a2fa8cf229e170539a5146f8'
BLOBS={
 'vue_a_versions.go':'68123731583e8a618c55d9b5e0b6338f434cb917',
 'frame_vue_messages.go':'06eafc58e7d747ca2fec6efdfba7a4c963142730',
 'vue_a_lecture.go':'6aeb697049e4b4e7181f7c2fd855429cdc4d7097',
 'vue_a_genres.go':'ee099013bd3359d439801d45df3b724a98ea1870',
 'vue_a_charges.go':'d022e4cd4b92c755c4210fed655423129ecc175e',
 'source/bits.go':'2951f8428d6228a7de607a8f1c1d062b7475da4d',
}
sources={}
for name,blob in BLOBS.items():
 raw=(Path(sys.argv[1])/name).read_bytes()
 assert hashlib.sha1(f'blob {len(raw)}\0'.encode()+raw).hexdigest()==blob
 sources[name]=raw.decode()


def extract(name,prefix):
 s=sources[name];start=s.index(prefix);end=s.index('{',start)+1;depth=1
 while depth:
  depth+=(s[end]=='{')-(s[end]=='}');end+=1
 return s[start:end]


selected=[extract(n,p) for n,p in [
 ('vue_a_versions.go','func versionNative('),
 ('vue_a_versions.go','func classeDesGenres('),
 ('vue_a_versions.go','func classeSousLaMajeure('),
 ('vue_a_versions.go','func premierGenrePresume('),
 ('frame_vue_messages.go','func placeDisponible('),
 ('vue_a_lecture.go','func lireLaVueA('),
 ('vue_a_lecture.go','func lireUnMessage('),
 ('vue_a_genres.go','func descripteurDuGenre('),
 ('vue_a_charges.go','func chargeZoom('),
]]
s=sources['vue_a_versions.go']
selected.append(s[s.index('type classeDeLaVueA uint8'):s.index('// versionMajeureJouee est')])
selected.append(s[s.index('const versionsNativesDesGenres ='):s.index('// versionNative rend')])
for prefix in ['const versionMajeureJouee =','const dernierGenreAVersionDistinctive =']:
 selected.append(next(l for l in s.splitlines() if l.startswith(prefix)))
for prefix in ['const GenresVueA =','const LargeurGenreVueA =']:
 selected.append(next(l for l in sources['frame_vue_messages.go'].splitlines() if l.startswith(prefix)))
s=sources['vue_a_genres.go']
selected.append(s[s.index('const tableDesGenresVueA ='):s.index('// largeurEntreeDeGenre :')])
selected+=['const largeurEntreeDeGenre = 4',"const marqueGenreVide = 'v'"]

cases=[dict(count=n,major=41,read_major=True) for n in range(126)]
for code in range(123):
 for value in [0,9,4294967295]:
  cases.append(dict(count=123,major=41,read_major=True,changed=code,value=value))
for major in [0,40,41,42]:
 for read in [False,True]:
  cases.append(dict(count=123,major=major,read_major=read))


def pack(bits):
 shift=(-len(bits))%8
 padded='0'*shift+bits
 return ''.join(f'{int(padded[i:i+8],2):02x}' for i in range(0,len(padded),8)),shift+1


frames=[]
for count in [0,122,123,124]:
 for config in [False,True]:
  for code in [None,3,21,123,127]:
   for corruption in [False,True]:
    bits=str(int(config))
    if code is not None:
     bits+='1'+f'{code:07b}'+'000'
     if code==21:
      bits+='11'
      if corruption:bits+='1'+f'{0x12345678:032b}'
     bits+='1'+f'{3:07b}'+'000'
    bits+='0'
    data,start=pack(bits)
    frames.append(dict(count=count,config=config,code=code,corruption=corruption,hex=data,start=start))
# Header truncation before all seven code bits are available.
for cut in range(2,9):
 data,start=pack(('11'+f'{21:07b}')[:cut])
 frames.append(dict(count=123,config=True,code=21,corruption=False,hex=data,start=start,header_cut=cut))

HARNESS=r'''
package main
import("encoding/json";"encoding/hex";"os")
type ProfilDeBalayage struct{Grammaire struct{ControleDeCorruption bool}}
type grammaireDeLaVueA struct{classe classeDeLaVueA;genres int}
type Lecteur struct{*Bits;p ProfilDeBalayage;vueA grammaireDeLaVueA;refsDuMessage [3]refDeMessage}
type refDeMessage struct{presente bool;index,gen uint64}
type FluxVueA struct{Debut,Fin,PremierPresume int;Vide,Porte bool;Genres []int}
func LecteurSur(b []byte)*Lecteur{return &Lecteur{Bits:NewBits(b)}}
func(r *Lecteur)PoserProfil(p ProfilDeBalayage){r.p=p}
func readVarWidthInt(*Lecteur,int)(uint64,uint64){panic("no present references in these admission fixtures")}
func chargeDuGenre(code int)func(*Lecteur)bool{if code==21{return chargeZoom};panic("unselected body")}
func recueillirLeMessage(*Lecteur,*FluxVueA,int,int){}
type Case struct{Count,Major int;ReadMajor bool `json:"read_major"`;Changed *int;Value uint32}
type Frame struct{Count,Start int;Corruption bool;Hex string}
type Input struct{Cases []Case;Frames []Frame}
type Classification struct{Class,Count int}
type FrameResult struct{End int;Complete,Empty bool;Codes []int}
func versions(n int)[]uint32{v:=make([]uint32,n);for i:=range v{if i<GenresVueA{v[i]=versionNative(i)}else{v[i]=1}};return v}
func main(){
 var in Input;if e:=json.NewDecoder(os.Stdin).Decode(&in);e!=nil{panic(e)}
 result:=struct{Native []uint32;Classes []Classification;Frames []FrameResult}{Native:versions(GenresVueA)}
 for _,c:=range in.Cases{
  v:=versions(c.Count);if c.Changed!=nil{v[*c.Changed]=c.Value};class,count:=classeDesGenres(v)
  class=classeSousLaMajeure(class,c.Major,c.ReadMajor);result.Classes=append(result.Classes,Classification{int(class),count})
 }
 for _,f:=range in.Frames{
  pay,e:=hex.DecodeString(f.Hex);if e!=nil{panic(e)};class,count:=classeDesGenres(versions(f.Count))
  p:=ProfilDeBalayage{};p.Grammaire.ControleDeCorruption=f.Corruption
  r:=lireLaVueA(pay,f.Start,p,grammaireDeLaVueA{class,count})
  result.Frames=append(result.Frames,FrameResult{r.Fin,r.Porte,r.Vide,r.Genres})
 }
 if e:=json.NewEncoder(os.Stdout).Encode(result);e!=nil{panic(e)}
}
'''
with tempfile.TemporaryDirectory(prefix='halo-event-admission-') as tmp:
 root=Path(tmp);(root/'main.go').write_text(HARNESS)
 (root/'source.go').write_text('package main\n'+'\n\n'.join(selected))
 (root/'bits.go').write_text(sources['source/bits.go'].replace('package source','package main',1))
 output=subprocess.check_output([os.environ.get('GO_BIN','go'),'run','main.go','source.go','bits.go'],cwd=root,input=json.dumps(dict(cases=cases,frames=frames)).encode())
print(json.dumps(dict(commit=PIN,blobs=BLOBS,cases=cases,frames=frames,results=json.loads(output)),indent=2))
