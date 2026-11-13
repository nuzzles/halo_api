"""Author independent Bond stat/string references with deliberate non-slot order."""
import json
from pathlib import Path
ROOT=Path(__file__).parent
def vi(n):
    out=bytearray()
    while n>=128:out.append(n%128+128);n>>=7
    return bytes(out)+bytes([n])
def f(t,i,b):return (bytes([t|i<<5]) if i<6 else bytes([t|192,i]))+b
def st(fields):
    b=b''.join(fields)+b'\0';return vi(len(b))+b
def seq(t,values):
    n=len(values)
    return (bytes([t|(n+1)<<5]) if n<7 else bytes([t])+vi(n))+b''.join(values)
def integer(n):return vi(((n<<1)^(n>>31))&0xffffffff)
def blob(b):return seq(3,[bytes([x]) for x in b])
def definition(indices,ident=7,extra=()):
    return st(([f(5,0,vi(ident))] if ident is not None else [])+
              [f(16,i+8,integer(n)) for i,n in enumerate(indices) if n is not None]+list(extra))
def table(texts,missing_blob=False):
    data=bytearray();entries=[]
    for languages in texts:
        offsets=[]
        for text in languages:offsets.append(len(data));data.extend(text+b'\0')
        entries.append(st([f(2,0,b'\1'),f(11,1,seq(16,[integer(i) for i in offsets]))]))
    return st(([] if missing_blob else [f(10,0,st([f(11,0,blob(data))]))])+[f(10,1,st([f(11,0,seq(10,entries))]))])
cases=[]
def add(name,defs,strings,extra=()):
    deps=([] if strings is None else [f(10,0,strings)])+([] if defs is None else [f(10,21,st([f(11,0,seq(10,defs))]))])+list(extra)
    base=st([f(10,3,st(deps))]);root=st([f(11,1,seq(10,[base]))])
    cases.append(dict(name=name,hex=root.hex(),base_hex=base.hex()))
texts=[[b'Carry Time',b'Tiempo'],[b'Points',b'Puntos'],[b'',b'\xff\0raw'],[b'Rounds Won']]
add('reordered',[definition([1]*4),definition([3]*4),definition([0]*4)],table(texts))
add('different-fields',[definition([0,1,2,3])],table(texts))
add('same-text-different-indices',[definition([0,1,0,1])],table([[b'same'],[b'same']]))
add('missing-index',[definition([0,None,0,0])],table(texts))
add('negative-index',[definition([-1]*4)],table(texts))
add('large-index',[definition([2147483647]*4)],table(texts))
add('wrong-index-type',[definition([None,0,0,0],extra=[f(5,8,vi(0))])],table(texts))
add('duplicate-index',[definition([0]*4,extra=[f(16,8,integer(1))])],table(texts))
add('base-only-index',[st([f(16,8,integer(1)),b'\1',f(16,9,integer(0)),f(16,10,integer(0)),f(16,11,integer(0))])],table(texts))
add('base-derived-index',[st([f(16,8,integer(1)),b'\1',*[f(16,i,integer(0)) for i in range(8,12)]])],table(texts))
add('missing-id',[definition([0]*4,ident=None)],table(texts))
add('duplicate-ids',[definition([0]*4),definition([1]*4)],table(texts))
add('shared-reference',[definition([0]*4),definition([0]*4,ident=8)],table(texts))
add('empty-invalid-text',[definition([2]*4)],table(texts))
add('missing-definitions',None,table(texts))
add('empty-definitions',[],table(texts))
add('missing-strings',[definition([0]*4)],None)
add('empty-strings',[definition([0]*4)],table([]))
add('missing-blob',[definition([0]*4)],table(texts,missing_blob=True))
add('duplicate-string-table',[definition([0]*4)],table(texts),extra=[f(10,0,table(texts))])
(ROOT/'cases.json').write_text(json.dumps(cases,indent=2)+'\n')
