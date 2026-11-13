"""Independent v41/27 HI_1_13_0 packet author, using the structural writer grammar."""
import json
from pathlib import Path

class Wire:
    def __init__(self): self.bits=[]
    def word(self,n,v): self.bits.extend((v>>i)&1 for i in reversed(range(n)))
    def zeros(self,n): self.bits.extend([0]*n)
    def hex(self):
        self.bits.extend([1]*(-len(self.bits)%8))
        return bytes(sum(b<<(7-i) for i,b in enumerate(self.bits[j:j+8])) for j in range(0,len(self.bits),8)).hex()

def packet(name,bots,rows,marker=183):
    w=Wire(); w.word(32,len(rows))
    expected=[]
    for i,(identity,engine) in enumerate(rows):
        start=len(w.bits)
        if bots:
            w.word(32,i); w.word(32,engine); w.word(32,identity)
        else:
            w.word(1,int(engine is None))
            if engine is not None: w.word(5,engine)
            w.word(64,int.from_bytes(identity.to_bytes(8,'little'),'big'))
        w.zeros(11+1+12+8+832)
        w.word(16,65+i%26); w.word(16,0)
        w.zeros(128+32)
        w.word(64,0 if bots else int.from_bytes(identity.to_bytes(8,'little'),'big'))
        # Actual source scanner requires this inner marker, unlike canonical grammar.
        w.word(10,marker); w.zeros(14+6+8+7+1+1852*8+44*8)
        expected.append(dict(identity=identity,engine=engine,start=start,end=len(w.bits)))
    return dict(name=name,bots=bots,hex=w.hex(),expected=expected)

base=0x0009000012340000
cases=[packet('session-all-five-bit-values',False,[(base+i,i) for i in range(32)]),
       packet('bots-all-scanner-admitted-slots',True,[(1000+i,i) for i in range(64)]),
       packet('session-absent-and-present',False,[(base+i,None if i%2==0 else i%32) for i in range(16)]),
       packet('session-duplicate-slot',False,[(base+1,7),(base+2,7)]),
       packet('bots-duplicate-slot',True,[(19,7),(20,7)]),
       packet('session-empty',False,[]),packet('bots-empty',True,[]),
       packet('session-short-21-without-scanner-marker',False,[(base+1,7)],marker=21)]
cases[-1]['scanner_omitted']=True
(Path(__file__).parent/'cases.json').write_text(json.dumps(cases,indent=2)+'\n')
