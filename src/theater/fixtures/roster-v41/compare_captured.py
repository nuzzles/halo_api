"""Compare captured canonical reads with separately exported pinned Go outputs.

python3 compare_captured.py CAPTURED_INPUT.json GO_OUTPUT.json
Does not derive expected boundaries from halo_api or bless new oracle values.
"""
import json
from pathlib import Path
import sys

actual=json.loads(Path(sys.argv[1]).read_text())
expected=json.loads(Path(sys.argv[2]).read_text())
assert len(actual)==len(expected)>0
counts=[0,0,0,0]

def word(e,kind):
 f=next(f for f in e['fields'] if f['kind']==kind)
 r=f['raw'];return int.from_bytes(bytes(r['bytes']),'big')>>(-r['bit_len']%8)

def name(e):
 units=[word(e,f['kind']) for f in e['fields'] if isinstance(f['kind'],dict) and 'GamertagUnit' in f['kind']]
 if units and units[-1]==0:units.pop()
 return b''.join(u.to_bytes(2,'big') for u in units).decode('utf-16-be',errors='replace')

for a,o in zip(actual,expected):
 assert a['Name']==o['Name']
 r=a['Actual'];complete=[e for e in r['entries'] if e['stop']=='Complete']
 counts[2*int(a['Bots'])+int(r['stop']!='Complete')]+=1
 assert r['personalization_bytes']==o['Perso']
 if a['Bots']:
  assert (r['stop']=='Complete')==o['Closed'],a['Name']
  if o['BotsRead'] is None:continue
  assert len(complete)==len(o['BotsRead']),a['Name']
  for e,v in zip(complete,o['BotsRead']):
   assert word(e,'Slot')==v['Slot'] and word(e,'BotId')==v['BotID']
   assert name(e)==v['Nom']
   if v['EquipeLue']:
    offsets=(37,38) if a['Build']=='HI_1_13_0' else (4,6)
    for offset,key in zip(offsets,['Equipe','Jumeau']):
     n=word(e,{'Block44Byte':offset});assert (n if n<128 else n-256)==v[key]
 else:
  b=o['Report']
  assert len(complete)==b['Entrees'],a['Name']
  assert (r['stop']=='Complete')==(b['Refusees']==0 and not b['Debordement']),a['Name']
  if r['stop']=='Complete':assert r['end_bit']==b['BitsLus'],a['Name']
  for e,v in zip(complete,o['Roster'] or []):
   slot=v['Joueur'];shorts=slot['Shorts']
   assert e['start_bit']==slot['Bit'] and e['end_bit']-e['start_bit']==slot['TotalBits']
   assert name(e)==slot['Gamertag']
   assert int.from_bytes(word(e,'Identity').to_bytes(8,'big'),'little')==v['Identite']
   option=next((word(e,'Option') for f in e['fields'] if f['kind']=='Option'),-1)
   assert option==v['Option']
   assert word(e,'Representation')==shorts['Repr'] and word(e,'JoinKey')==shorts['Q64']
   for i,key in enumerate(['F10','F14','F6','F8','F7','F1']):
    assert word(e,{'Short':i})-int(i==2)==shorts[key]
print(f'{len(actual)} packets agree with pinned Go readers; roster complete/partial {counts[:2]}, bots complete/partial {counts[2:]}')
