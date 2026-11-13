"""Authored orientation branches from instruction-addressed LevelUp analysis.

Unlike libfilmshell, C overrides raw mode and a clear delta-roll gate has no
payload. Versions below 2 do not carry C. No halo_api output is used.
"""
import json
from pathlib import Path
cases=[]
def b(n,v=0):return format(v,f'0{n}b')
def case(name,version,wire):
    padded=wire+'0'*(-len(wire)%8)
    cases.append(dict(name=name,component='object-forward-and-up-dynamic-precision-component',version=version,hex=int(padded,2).to_bytes(len(padded)//8,'big').hex(),bit_len=len(wire),complete=True,snapshot=False))
for v in [0,1,2,4]:
    for keep,delta in [(1,0),(0,0),(0,1)]:
        for tail in [0,1]:
            wire='01'+('0' if v>=2 else '')+b(1,keep)
            if not keep:wire+=b(1,delta)+(b(8,0xE3) if delta else b(19,0x54321))
            wire+=b(1,tail)+(b(4,0xD) if tail else '')
            case(f'delta/{v}/{keep}/{delta}/{tail}',v,wire)
    # Plain raw mode, with no C below version 2.
    case(f'raw/{v}',v,'1'+('0' if v>=2 else '')+b(192,0x7FC12345))
    if v>=2:
        for default in [0,1]:
            case(f'raw-overridden/{v}/{default}',v,'11'+b(1,default)+('' if default else b(30,0x3456789))+b(30,0x2345678))
assert len(cases)==32
Path(__file__).with_name('orientation-source.json').write_text(json.dumps(cases,indent=2)+'\n')
