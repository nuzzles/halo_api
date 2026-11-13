"""Author independent Bond option-rule inputs from the original catalogue export."""
import json
from pathlib import Path
import sys

ROOT=Path(__file__).parent
catalogue=json.loads((Path(sys.argv[1]) if len(sys.argv)>1 else ROOT/'../game-catalogues-v41/reference.json').read_text())
def hash_name(name):
    b=name.lower().encode();h=0
    rot=lambda x,n:((x<<n)|(x>>(32-n)))&0xffffffff
    mix=lambda k:rot((k*0xcc9e2d51)&0xffffffff,15)*0x1b873593&0xffffffff
    for i in range(0,len(b)//4*4,4):h=(rot(h^mix(int.from_bytes(b[i:i+4],'little')),13)*5+0xe6546b64)&0xffffffff
    tail=b[len(b)//4*4:]
    if tail:h^=mix(int.from_bytes(tail,'little'))
    h^=len(b);h^=h>>16;h=h*0x85ebca6b&0xffffffff;h^=h>>13;h=h*0xc2b2ae35&0xffffffff;h^=h>>16
    return h
for row in catalogue['option_names']:assert hash_name(row['name'])==row['hash']
def vi(n):
    b=bytearray()
    while n>=128:b.append(n%128+128);n>>=7
    return bytes(b)+bytes([n])
def field(t,i,b):return (bytes([t|i<<5]) if i<6 else bytes([t|192,i]))+b
def st(fields):
    b=b''.join(fields)+b'\0';return vi(len(b))+b
def seq(t,values):
    n=len(values)
    return (bytes([t|(n+1)<<5]) if n<7 else bytes([t])+vi(n))+b''.join(values)
def blob(data):return seq(3,[bytes([b]) for b in data])
def wrapped(i):
    n=i if i<0x80000000 else i-0x100000000
    return st([field(16,0,vi(((n<<1)^(n>>31))&0xffffffff))])
def value(n=0x3eaaaaab,kind=3):return n.to_bytes(4,'little')+bytes([kind,0xaa,0xbb,0xcc])
def option(key,raw):return dict(hash=None if key is None else hash_name(key),raw=None if raw is None else list(raw),key=key)
cases=[]
def add(name,scripts,options,wrong_options=False,wrong_scripts=False,invalid_hash=False):
    members=[]
    if scripts is not None:
        data=b''.join(b'tags\\'+s.encode()+b'\0' for s in scripts)
        members.append(field(16,2,vi(0)) if wrong_scripts else field(11,2,blob(data)))
    if options is not None:
        rows=[]
        for i,o in enumerate(options):
            fields=[]
            if o['hash'] is not None:fields.append(field(10,0,wrapped(o['hash'])))
            elif invalid_hash and i==0:fields.append(field(10,0,st([field(9,0,vi(3)+b'bad')])))
            if o['raw'] is not None:fields.append(field(10,1,st([field(11,0,blob(bytes(o['raw'])))])))
            rows.append(st(fields))
        members.append(field(10,9,st([field(16,0,vi(0)) if wrong_options else field(11,0,seq(10,rows))])))
    base=st([field(10,3,st(members))])
    root=st([field(11,1,seq(10,[base]))])
    cases.append(dict(name=name,hex=root.hex(),scripts=scripts or [],options=options or []))

for m in catalogue['modes']:
    add('defaults/'+m['prefix'],[m['script']],[])
    for i,o in enumerate(m['options']):
        for j,key in enumerate(o['keys']):
            raw=value(0x80000000 if j%2 else 0x3eaaaaab)
            add(f"alias/{m['prefix']}/{i}/{j}",[m['script']],[option(key,raw)])
key='StrongholdsOptions_captureTimeSec';old='StrongholdsOptions_TimeToCapture';script=['strongholdsoptions.lua']
add('alias-priority',script,[option(key,value(0x40000000)),option(old,value(0x3f800000))])
add('duplicate-rows',script,[option(key,value()),option(key,value(0x40000000))])
add('duplicate-equal',script,[option(key,value()),option(key,value())])
add('missing-value',script,[option(old,None),option(key,value())])
add('invalid-value',script,[option(old,b'1234567'),option(key,value())])
add('unknown-payload',script,[option(old,value(kind=99)),option(key,value())])
add('unidentified-row',script,[option(None,value()),option(key,value())])
add('invalid-hash',script,[option(None,value()),option(key,value())],invalid_hash=True)
add('options-absent',script,None)
add('options-invalid',script,[],wrong_options=True)
add('prefix-only',None,[option(key,value())])
add('alias-only',None,[option(old,value())])
add('script-invalid',script,[option(key,value())],wrong_scripts=True)
add('no-evidence',[],[])
add('unknown-hash',[],[option('Unknown_Option',value())])
for raw in [value(0x7fc12345),value(0x7f800000),value(0xff800000),value(0x80000000),value(0x123456ff,2)]:
    add('raw/'+raw.hex(),script,[option(key,raw)])
(Path(sys.argv[2]) if len(sys.argv)>2 else ROOT/'cases.json').write_text(json.dumps(cases,indent=2)+'\n')
