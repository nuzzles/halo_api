"""Independently authored Bond script blobs and all reference rule combinations."""
import json
from pathlib import Path

ROOT=Path(__file__).parent
def vi(n):
    out=bytearray()
    while n>=128:out.append((n&127)|128);n>>=7
    return bytes(out)+bytes([n])
def field(t,i,b):return bytes([t|(i<<5)])+b
def struct(fields):
    b=b''.join(fields)+b'\0'
    return vi(len(b))+b
def seq(t,values):
    n=len(values)
    return (bytes([t|((n+1)<<5)]) if n<7 else bytes([t])+vi(n))+b''.join(values)
def blob(data):return seq(3,[bytes([b]) for b in data])
def base(data):return struct([field(10,3,struct([field(11,2,blob(data))]))])
cases=[]
def add(name,data,body=None):
    body=base(data) if body is None else body
    root=struct([field(11,1,seq(10,[body]))])
    cases.append(dict(name=name,blob_hex=data.hex(),hex=root.hex()))

for name,data in [
    ('empty',b''),('repeats',b'prefix\\SLAYEREvents.lua\0other/SlayerEvents.lua\0x/slayerevents.lua\0'),
    ('ascii-space',b'x/Script Name_42.lua\0'),('no-separator',b'slayerevents.lua\0'),
    ('empty-name',b'x/.lua\0'),('uppercase-extension',b'x/SlayerEvents.LUA\0'),
    ('unsupported-name-char',b'x/a-b.lua\0x/c.d.lua\0x/\xffd.lua\0'),
    ('following-byte',b'x/slayerevents.luac\0'),('end-of-blob',b'x/slayerevents.lua'),
    ('unrecognized',b'x/something_else.lua\0'),
]:add(name,data)
add('absent',b'',struct([]))
add('wrong-type',b'',struct([field(10,3,struct([field(9,2,vi(4)+b'nope')]))]))
add('duplicate-blob',b'',struct([field(10,3,struct([field(11,2,blob(b'x/a.lua')),field(11,2,blob(b'x/b.lua'))]))]))
add('base-only-blob',b'',struct([field(10,3,struct([field(11,2,blob(b'x/a.lua')),b'\1']))]))
add('base-derived-blob',b'x/slayerevents.lua',struct([field(10,3,struct([field(11,2,blob(b'x/oddballevents.lua')),b'\1',field(11,2,blob(b'x/slayerevents.lua'))]))]))
add('outside-blob',b'x/slayerevents.lua',struct([
    field(9,0,vi(len(b'x/oddballevents.lua\0\0'))+b'x/oddballevents.lua\0\0'),
    field(10,3,struct([field(11,2,blob(b'x/slayerevents.lua'))]))]))
names=['firefightevents.lua','fbastionevents.lua','arenaflagevents.lua','oddballevents.lua',
       'bastionevents.lua','parcel_stronghold.lua','slayerevents.lua','btbslayerevents.lua']
for mask in range(1<<len(names)):
    data=b''.join(b'tags\\'+n.encode()+b'\0' for i,n in enumerate(names) if mask&(1<<i))
    add(f'modes/{mask:02x}',data)
(ROOT/'cases.json').write_text(json.dumps(cases,indent=2)+'\n')
