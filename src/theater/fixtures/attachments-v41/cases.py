"""Independent parent-state bodies, including old detached and biped-only tails."""
import json
from pathlib import Path
cases=[]
for parameter in range(5):
 for archetype in [35,40,47]:
  for unit,generation in [(None,0),(None,3),*[(u,g) for u in [False,True] for g in range(4)]]:
   bits=[]
   def w(n,v):bits.extend((v>>i)&1 for i in reversed(range(n)))
   attached=unit is not None;seat=[None,0,63,7][generation]
   w(1,int(attached));index=node=None
   if attached:
    index=([0,255,256,511] if unit else [0,511,1024,8191])[generation];node=[0,1,32768,65535][generation]
    w(1,int(unit));w(9 if unit else 13,index);w(2,generation);w(16,node)
    w(1,int(generation>=2))
    if generation>=2:w(16,65535)
    w(1,generation&1);w(1,(generation>>1)&1)
    for v in [0,32768,65535]:w(16,v)
    w(1,int(generation%2==0))
    if generation%2:w(19,0x7ffff)
    w(8,255);w(1,1)
   elif parameter<2:
    w(1,int(generation!=0))
    if generation:w(13,0x1234);w(2,2)
    w(1,int(generation!=0))
    if generation:w(11,2047)
   w(1,int(seat is not None))
   if seat is not None:w(6,seat)
   w(1,1)
   if parameter<=2 or archetype==35:w(3,7)
   length=len(bits);bits += [0]*(-len(bits)%8)
   data=bytes(sum(b<<(7-i) for i,b in enumerate(bits[j:j+8])) for j in range(0,len(bits),8))
   cases.append(dict(name=f'{parameter}/{archetype}/{unit}/{generation}',parameter=parameter,archetype=archetype,unit=unit,index=index,generation=generation if attached else None,node=node,seat=seat,bit_len=length,hex=data.hex()))
Path(__file__).with_name('cases.json').write_text(json.dumps(cases,indent=2)+'\n')
