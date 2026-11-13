"""Author complete game-options envelopes from the pinned writer field map."""
import json
from pathlib import Path
import runpy

root=Path(__file__).parent
bond=runpy.run_path(str(root/'wire_cases.py'))
structure,field,sequence=bond['structure'],bond['field'],bond['sequence']
CASES=[]
for flags in range(8):
    wire=''
    entries=[]
    def add(kind,value,width):
        global wire
        start=len(wire);wire+=format(value,f'0{width}b')
        entries.append(dict(kind=kind,start=start,end=len(wire)))
    def raw(kind,data):
        global wire
        start=len(wire);wire+=''.join(format(b,'08b') for b in data)
        entries.append(dict(kind=kind,start=start,end=len(wire)))
    for i,w in enumerate([3,3,2,7,64,32,3,32,32,32]): add('field',(1<<w)-1-i%(1<<w),w)
    raw('bond',structure([field(t,i,bond['scalar'](t)) for i,t in enumerate(range(2,19))]))
    raw('string',(b'\xff'*127 if flags&4 else b'')+b'\0')
    add('field',0xFEDCBA98,32);add('field',0x1234ABCD,32)
    add('field',bool(flags&1),1)
    if flags&1: raw('field',bytes(range(224)))
    for w in [1,1,2,1,32,32,1]:add('field',(1<<w)-1,w)
    add('field',bool(flags&2),1)
    if flags&2: raw('bond',structure([field(11,i,sequence(10,i%2)) for i in range(4)]))
    raw('string',(b'Z'*255 if flags&4 else b'ab')+b'\0')
    raw('string',b'hello\xff\0')
    add('field',0xDEADBEEF,32);raw('field',bytes((i*7)%256 for i in range(216)))
    bit_len=len(wire);wire+='1001011010010110'
    padded=wire.ljust((len(wire)+7)//8*8,'0')
    CASES.append(dict(name=f'flags/{flags}',hex=int(padded,2).to_bytes(len(padded)//8,'big').hex(),bit_len=bit_len,entries=entries))
(root/'game-options.json').write_text(json.dumps(CASES,indent=2)+'\n')
