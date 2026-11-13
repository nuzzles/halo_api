"""Run pinned original LevelUp player/body/wrapper readers on authored packets.

GO_BIN=... GOCACHE=... python3 export_source.py PINNED_GRAMMAR > reference.json
Only package/type qualifications are adapted. Source bit reader is unchanged.
Input payloads are authored here; expected values/closure come from Go, never Rust.
"""
import base64
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
 'player_table_record.go': '0239de11b8f8d1f340c4ed0242f0f2be5f7af9de',
 'player_table.go': 'c1d4606fb992abbf3629b4554807587d5beebc2a',
 'roster_type8.go': '8cd673cb1751607c7c670ed8785a52b53af41fd4',
 'bot_metadata.go': '1a3d6d3c206ac13fcdc1418047ce9782d2153533',
 'source/bits.go': '2951f8428d6228a7de607a8f1c1d062b7475da4d',
 'profile/build_profile.go': '947913ed0b519f905b869256396ecbd845271584',
 'profile/equipe_bot.go': '8236c0050efaf81ae1aac04674e832fec027d945',
}
SOURCES = {}
for name, blob in BLOBS.items():
 raw = (Path(sys.argv[1])/name).read_bytes()
 assert hashlib.sha1(f'blob {len(raw)}\0'.encode()+raw).hexdigest()==blob, name
 SOURCES[name] = raw.decode()

def extract(source, prefix):
 start=source.index(prefix);opening=source.index('{',start);depth=1;end=opening+1
 while depth:
  depth+=(source[end]=='{')-(source[end]=='}');end+=1
 return source[start:end]

def constants(source):
 return re.findall(r'^const \(.*?^\)',source,re.M|re.S)

selected=[]
for file, prefixes in {
 'player_table_record.go': ['type slotEnr struct','type slotReader struct','func (r *slotReader) bits(', 'func (r *slotReader) saute(', 'func decodeSlotListes(', 'func lireGamertag(', 'func decodeSlotCorps('],
 'roster_type8.go': ['type RosterEntry struct','type RosterBilan struct','func DecodeRoster(', 'func rosterLireEntree(', 'func rosterOctetsDuFlux('],
 'bot_metadata.go': ['type LectureDesBots struct','type EntreeDeBotEcrite struct','func lireLesEntreesDeBots(', 'func lireLeBloc44('],
 'profile/build_profile.go': ['func PersonnalisationOctets('],
 'profile/equipe_bot.go': ['type PositionEquipeBot struct','func PositionEquipeBotPour('],
}.items():
 selected.extend(extract(SOURCES[file],p) for p in prefixes)
for f in ['player_table.go','roster_type8.go','bot_metadata.go']:
 selected.extend(constants(SOURCES[f]))
selected.append(constants(SOURCES['profile/build_profile.go'])[0])
source='package main\nimport "unicode/utf16"\n'+'\n\n'.join(selected)
source=source.replace('types.PlayerSlot','PlayerSlot').replace('profile.PositionEquipeBot','PositionEquipeBot')

class Writer:
 def __init__(self): self.bits=[]
 def put(self,n,v): self.bits.extend((v>>i)&1 for i in range(n-1,-1,-1))
 def blob(self,n,seed):
  for i in range(n): self.put(8,(seed+37*i)&255)
 def payload(self):
  bits=self.bits+[1]*(-len(self.bits)%8)
  return bytes(sum(bits[i+j]<<(7-j) for j in range(8)) for i in range(0,len(bits),8))

def entry(w,bots,version,perso,i,wide=False):
 if bots:
  for v in [0x80000001+i,60+i,0x10203040+i]: w.put(32,v)
 else:
  if version>=21:
   w.put(1,i%2)
   if i%2==0:w.put(5,17+i)
  w.put(64,0x6945f23ff0010900+i)
 mask=2048 if wide else 1+i;w.put(11,mask-1)
 for j in range(mask):w.put(1,j%2)
 n=2048 if wide else i;w.put(12,n);w.blob(n,17)
 n=192 if wide else i;w.put(8,n);w.blob(n*4,29)
 w.blob(104,31)
 units=([0xd800,65,0] if i==0 else list(range(65,81)))
 for u in units:w.put(16,u)
 w.blob(16,71);w.put(32,0xdeadbeef);w.put(64,0x1122334455667788+i)
 for n,v in [(10,1023),(14,16382),(6,0),(8,254),(7,127),(1,1)]:w.put(n,v)
 w.blob(perso,97);w.blob(44,117)

cases=[]
builds=[('HI_1_13_0',27,1852),('HI_1_12_0',27,1852),('HI_1_11_0',25,1492),('HI_1_10_0',24,1492),('HI_1_9_0',24,1312),('HI_1_8_0',24,1312),('HI_1_4_1',21,2052),('HI_1_4_1',20,2052)]
for build,version,perso in builds:
 for bots in [False,True]:
  w=Writer();w.put(32,2)
  for i in range(2):entry(w,bots,version,perso,i)
  cases.append(dict(Name=f'{build}-v{version}-{bots}',Build=build,Version=version,Bots=bots,Data=base64.b64encode(w.payload()).decode()))
# Maximum admitted variable lists, empty population, and nonzero arbitrary tails.
for bots in [False,True]:
 for kind in ['wide','empty','tail','truncated','byte-limit','word-limit','count-limit']:
  w=Writer();w.put(32,65 if kind=='count-limit' else 0 if kind=='empty' else 1)
  if kind not in ['empty','count-limit']:
   entry(w,bots,27,1852,0,wide=kind=='wide')
  data=w.payload()
  if kind=='tail':data+=b'\xa5\xff'
  if kind=='truncated':data=data[:-7]
  if kind in ['byte-limit','word-limit']:
   bits=list(w.bits);offset=32+(96 if bots else 70)+12
   if kind=='word-limit':offset+=12
   width,value=(12,2049) if kind=='byte-limit' else (8,193)
   bits[offset:offset+width]=[(value>>i)&1 for i in range(width-1,-1,-1)]
   w.bits=bits;data=w.payload()
  cases.append(dict(Name=f'{kind}-{bots}',Build='HI_1_13_0',Version=27,Bots=bots,Data=base64.b64encode(data).decode()))

if len(sys.argv)>2:
 cases=json.loads(Path(sys.argv[2]).read_text())
 for c in cases: c['Data']=base64.b64encode(bytes.fromhex(c['Data'])).decode()

HARNESS=r'''
package main
import("encoding/json";"os")
type Lecteur = Bits
func LecteurSur(d []byte)*Lecteur{return NewBits(d)}
type PlayerSlot struct { XUID uint64; Bit,TotalBits int; Gamertag string; Shorts Shorts }
type Shorts struct{Repr uint32;Q64 uint64;F10,F14 uint32;F6 int;F8,F7,F1 uint32}
type Case struct{Name,Build string;Version int;Bots bool;Data []byte}
type Output struct{Case;Perso int;Roster []RosterEntry;Report RosterBilan;BotsRead []EntreeDeBotEcrite;Closed bool}
func main(){
 var cases []Case;if e:=json.NewDecoder(os.Stdin).Decode(&cases);e!=nil{panic(e)}
 out:=[]Output{}
 for _,c:=range cases {
  p,ok:=PersonnalisationOctets(c.Build);if !ok{panic(c.Build)}
  o:=Output{Case:c,Perso:p}
  if c.Bots {pos,known:=PositionEquipeBotPour(c.Build);o.BotsRead,o.Closed=lireLesEntreesDeBots(c.Data,LectureDesBots{PersoBits:p*8,PersoConnue:true,Equipe:pos,EquipeConnue:known})
  } else {o.Roster,o.Report=DecodeRoster(c.Data,c.Version,p*8)}
  out=append(out,o)
 }
 if e:=json.NewEncoder(os.Stdout).Encode(out);e!=nil{panic(e)}
}
'''
with tempfile.TemporaryDirectory(prefix='halo-roster-source-') as tmp:
 root=Path(tmp);(root/'main.go').write_text(HARNESS);(root/'grammar.go').write_text(source)
 (root/'bits.go').write_text(SOURCES['source/bits.go'].replace('package source','package main',1))
 output=subprocess.check_output([os.environ.get('GO_BIN','go'),'run','main.go','grammar.go','bits.go'],cwd=root,input=json.dumps(cases).encode())
# Human-reviewable deterministic JSON; payload bytes remain base64.
rows=json.loads(output)
for row in rows: row['Data'] = base64.b64decode(row['Data']).hex()
print(json.dumps(rows,indent=2,ensure_ascii=True))
