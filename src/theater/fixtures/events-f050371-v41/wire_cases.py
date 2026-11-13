"""Independent source-bit layouts for the f050371 event-reader additions."""
import itertools
import json
from pathlib import Path

def b(n, v=0):
    assert 0 <= v < 1 << n
    return f'{v:0{n}b}'
def opt(n, present, inverted=False):
    return b(1, not present if inverted else present) + (b(n, (1<<n)-1) if present else '')
KINDS={2:[1,8,7],8:[2,3,7],12:[8,8,7],13:[4,8,7],14:[2,2,7],80:[8,8,7],104:[0,0,7],105:[0,0,7],106:[0,0,7],21:[4,8,7]}
def header(code, form):
    s='1'+b(7,code)
    for k in KINDS[code]:
        s+=b(1,form>0)
        if form:
            if k==1:s+=b(1,form==2)
            w=9 if k in [4,6] or (k==1 and form==2) else 8 if k in [2,3,5] else 13
            s+=b(w,(1<<w)-3)+b(2,form)
    return s

def point(precision, default):
    widths=[precision+6]*3 if default else [15-(16-precision),15-(16-precision),17-(16-precision)]
    return b(1,default)+('' if default else b(1,0))+''.join(b(w,(1<<w)-2) for w in widths)

bodies=[]
for code in [2,13]:
    for wide in [False,True]:
        w=(5 if code==2 else 7) if wide else 2
        for v in [0,(1<<w)-1]:
            for tail in ([0,1] if code==13 else [0]):
                bodies.append((code,f'{wide}/{v}/{tail}',b(1,wide)+b(w,v)+(b(1,tail) if code==13 else ''),[]))
for v in [0,63]:bodies.append((8,str(v),b(6,v),[]))
for v in [0,0xffffffff]:bodies.append((14,str(v),b(32,v),[]))
for a,c,t,d in itertools.product([False,True],repeat=4):
    prefix=opt(5,a,True)+opt(5,c,True)+opt(32,t)
    bodies.append((12,f'{a}/{c}/{t}/{d}',prefix+point(12,d)+b(19,0x5abcd),[] if d else [len(prefix)]))
for tag,form,default in itertools.product([False,True],range(4),[False,True]):
    prefix=opt(32,tag)+b(2,form)
    n={0:1,1:3}.get(form,0)
    p=point(16,default)
    bodies.append((80,f'{tag}/{form}/{default}',prefix+p*n,[] if default else [len(prefix)+i*len(p) for i in range(n)]))
for zero in [False,True]:bodies.append((104,str(zero),b(1,zero)+('' if zero else b(19,0x5abcd)+b(10,999)),[]))
for tag in [False,True]:bodies.append((105,str(tag),opt(32,tag),[]))
for d in [False,True]:bodies.append((106,str(d),point(12,d)+b(10,1023)+b(9,511),[] if d else [0]))
cases=[]
for code,name,body,points in bodies:
    for form in range(3):
        s=header(code,form)+body
        end=len(s)
        s+=header(21,0)+b(2,2)+'0'
        padded=s.ljust((len(s)+7)//8*8,'0')
        cases.append(dict(name=f'{code}/{name}/refs-{form}',code=code,hex=int(padded,2).to_bytes(len(padded)//8,'big').hex(),bit_len=len(s),record_end=end,complete=True,indexed_points=[len(header(code,form))+p for p in points]))
Path(__file__).with_name('cases.json').write_text(json.dumps(cases,indent=2)+'\n')
print(len(cases))
