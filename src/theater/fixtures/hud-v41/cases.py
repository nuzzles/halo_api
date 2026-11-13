"""Author full static-component wire bodies without consulting parser output."""
import json

def case(name, args=None, key=0xFEDCBA98, present=True, body=False):
    bits=[]
    def word(v,n):
        assert 0 <= v < 1 << n
        bits.extend((v >> i) & 1 for i in reversed(range(n)))
    word(int(present),1)
    if present:
        word(key,32);word(len(args or []),3)
        for kind,value in args or []:
            word(kind,3)
            if kind==1:
                word(int(value is None),1)
                if value is not None:word(value,5)
            elif kind==2:
                real,number=value;word(int(real),1);word(number,24 if real else 32)
            elif kind>=3:word(value,32)
    prefix=len(bits)
    word(0xFEDCBA,24);word(int(body),1)
    if body:
        word(1,3);word(0xABCD,16);word(0xEF,8);word(0x12,8);word(1,1);word(0xDEADBEEF,32)
        word(0,2);word(0,3);word(0,1);word(0,1)
    size=len(bits);bits.extend([0]*(-size%8))
    data=bytes(sum(bits[i+j]<<(7-j) for j in range(8)) for i in range(0,len(bits),8))
    return dict(name=name,hex=data.hex(),bit_len=size,prefix_bits=prefix)

cases=[case('absent',present=False),case('empty',[]),case('zero-key',[],key=0),case('empty-argument',[(0,None)])]
for v in [None,0,1,15,31]:cases.append(case(f'player-{v}',[(1,v)]))
for v in [0,1,0x7FFFFFFF,0x80000000,0xFFFFFFFF]:cases.append(case(f'int-{v}',[(2,(False,v))]))
for v in [0,1,0x7FFFFF,0x800000,0x800001,0xFFFFFE,0xFFFFFF]:cases.append(case(f'real-{v}',[(2,(True,v))]))
for kind in range(3,8):
    for v in [0,0xFFFFFFFF]:cases.append(case(f'kind-{kind}-{v}',[(kind,v)]))
cases.extend([
    case('mixed-seven',[(0,None),(1,31),(1,None),(2,(False,0x80000000)),(2,(True,0xFFFFFF)),(3,0),(7,0x12345678)],body=True),
    case('repeated-players',[(1,7)]*7),
    case('all-unknown',[(4,1),(5,2),(6,3),(7,4)]),
    case('absent-with-body',present=False,body=True),
])
print(json.dumps(cases,indent=2))
