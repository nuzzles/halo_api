"""Authored CBv2 crate properties; never reads halo_api output."""
import json

def uleb(v):
    b=[]
    while v>=128: b.append((v&127)|128); v>>=7
    return bytes(b+[v])
def field(i,t,p): return (bytes([(i<<5)|t]) if i<6 else bytes([0xc0|t,i]))+p
def struct(fs):
    b=b''.join(fs)+b'\0'
    return uleb(len(b))+b
def number(i,v,wrapped=False):
    p=uleb((v<<1)^(v>>63))
    return field(i,10,struct([field(0,17,p)])) if wrapped else field(i,17,p)
def seq(values):return bytes([((len(values)+1)<<5)|10])+b''.join(values)
def geo(kind=3,dims=(65536,131072,196608,-32768),wrapped=False):
    return struct([number(0,kind,wrapped)]+[number(i,v,wrapped) for i,v in zip([5,6,7,8],dims) if v is not None])
def item(g,team=1,lists=False):return struct([field(0,11,seq([g])) if lists else field(0,10,g)]+([] if team is None else [number(8,team)]))
def root(items,lists=False):return struct([field(0,11,seq(items))] if lists else [field(0,10,items[0])])
cases=[]
def case(name,b,policy='single'):
    bits='0'+'10101'+''.join(f'{v:08b}' for v in b)
    raw=int(bits.ljust((len(bits)+7)//8*8,'0'),2).to_bytes((len(bits)+7)//8,'big')
    cases.append(dict(name=name,hex=raw.hex(),bit_len=len(bits),bond_hex=b.hex(),policy=policy))
case('box',root([item(geo())]))
case('cylinder-recorded-right',root([item(geo(2))]))
case('missing-dimensions',root([item(geo(dims=(65536,None,None,None)),team=None)]))
case('wrapped-integers',root([item(geo(wrapped=True))]))
case('single-lists',root([item(geo(),lists=True)],lists=True))
case('unknown-shape',root([item(geo(99))]))
case('multiple-items',root([item(geo()),item(geo(2),2)],lists=True),'multiple')
case('empty-items',root([],lists=True),'empty')
case('duplicate-root',struct([field(0,10,item(geo())),field(0,10,item(geo(2)))]),'duplicate-root')
case('duplicate-dimension',root([item(struct([number(0,3),number(5,65536),number(5,131072)]))]),'duplicate-dimension')
case('inherited-root',struct([bytes([1]),field(0,10,item(geo()))]),'inheritance')
b=root([item(geo())]);case('root-short',b'\x01'+b[1:],'invalid-bond')
case('root-long',uleb(len(b)+20)+b[1:],'invalid-bond')
case('nested-short',root([item(b'\x01'+geo()[1:])]),'invalid-bond')
case('empty-wrapped-right',root([item(struct([number(0,2),field(6,10,struct([]))]))]),'empty-wrapper')
case('multiple-geometries',root([struct([field(0,12,seq([geo(),geo(2)]))])]),'geometries')
cases.append(dict(name='absent',hex='80',bit_len=1,policy='absent'))
print(json.dumps(cases,indent=2))
