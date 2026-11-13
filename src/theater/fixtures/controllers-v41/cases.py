"""Independent unit-control bodies; no controller, every ID, all optional tails."""
import json
from pathlib import Path
cases=[]
for player in [None,*range(32)]:
 for secondary in ([None] if player is None else [None,63]):
  for reference in [None,0xffffffff]:
   bits=[]
   def w(n,v):bits.extend((v>>i)&1 for i in reversed(range(n)))
   w(1,int(player is not None))
   if player is not None:
    w(5,player);w(1,int(secondary is not None))
    if secondary is not None:w(6,secondary)
   w(1,int(reference is not None))
   if reference is not None:w(32,reference)
   length=len(bits);bits += [0]*(-len(bits)%8)
   data=bytes(sum(b<<(7-i) for i,b in enumerate(bits[j:j+8])) for j in range(0,len(bits),8))
   cases.append(dict(name=f'{player}/{secondary}/{reference}',player=player,secondary=secondary,reference=reference,bit_len=length,hex=data.hex()))
(Path(__file__).parent/'cases.json').write_text(json.dumps(cases,indent=2)+'\n')
