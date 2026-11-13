"""Independently author game-variant schema cases inside complete options."""
import json
from pathlib import Path
root = Path(__file__).parent

def varint(n):
    out = bytearray()
    while n >= 128:
        out.append((n & 127) | 128); n >>= 7
    return bytes(out) + bytes([n])
def field(t, i, value):
    header = bytes([t | i << 5]) if i < 6 else bytes([t | 0xc0, i])
    return header + value
def struct(fields):
    body = b''.join(fields) + b'\0'
    return varint(len(body)) + body
def integer(i, n):
    return field(16, i, varint(((n << 1) ^ (n >> 31)) & 0xffffffff))
def boolean(i, n):
    return field(2, i, bytes([n]))
def variant(engine=2, kill=0, play=0, *, header=True, settings=True, base=False, duplicate=False, wrong=False):
    h = [] if engine is None else [integer(0, engine)]
    if duplicate: h += [integer(0, 1)]
    if wrong: h = [boolean(0, 1)]
    playback = ([boolean(0, 1), boolean(2, 1), b'\1'] if base else [])
    playback += ([] if kill is None else [boolean(0, kill)]) + ([] if play is None else [boolean(2, play)])
    fields = [field(10, 0, struct(h))] if header else []
    if settings: fields += [field(10, 1, struct([field(10, 0, struct([field(10, 5, struct(playback))]))]))]
    return struct(fields)
def selected(i, values):
    return field(11, i, bytes([10 | ((len(values) + 1) << 5)]) + b''.join(values))

base = json.loads((root/'game-options.json').read_text())[2]
wire = ''.join(format(b,'08b') for b in bytes.fromhex(base['hex']))
target = [x for x in base['entries'] if x['kind']=='bond'][1]
cases=[]
def add(name, value, expected):
    message = ''.join(format(b,'08b') for b in value)
    bits = wire[:target['start']] + message + wire[target['end']:base['bit_len']]
    n=len(bits); bits=(bits+'1001011010010110').ljust((n+16+7)//8*8,'0')
    cases.append(dict(name=name,hex=int(bits,2).to_bytes(len(bits)//8,'big').hex(),bit_len=n,expected=expected))
for i in range(4):
    for engine in [0,1,2,-1]:
        for kill in [0,1]:
            for play in [0,1]:
                add(f'valid/{i}/{engine}/{kill}/{play}', struct([selected(j,[variant(engine,kill,play)] if i==j else []) for j in range(4)]),[engine,bool(kill),bool(play)])
for name, v, expected in [
    ('defaults',variant(None,None,None),[0,False,False]),
    ('base-ids-are-not-derived',variant(None,None,None,base=True),[0,False,False]),
    ('nonzero-bool',variant(2,255,2),[2,True,True]),
    ('missing-header',variant(header=False),[None,False,False]),
    ('missing-settings',variant(settings=False),[2,None,None]),
    ('duplicate-engine',variant(duplicate=True),[None,None,None]),
    ('wrong-engine-type',variant(wrong=True),[None,None,None]),
]: add(name,struct([selected(1,[v])]),expected)
for name,value in [
    ('empty',struct([])),
    ('empty-lists',struct([selected(i,[]) for i in range(4)])),
    ('multiple-instances',struct([selected(0,[variant()]),selected(1,[variant()])])),
    ('multiple-in-list',struct([selected(0,[variant(),variant()])])),
    ('duplicate-list',struct([selected(0,[variant()]),selected(0,[])])),
    ('wrong-list-type',struct([field(10,0,variant())])),
]: add(name,value,[None,None,None])
(root/'variants.json').write_text(json.dumps(cases,indent=2)+'\n')
