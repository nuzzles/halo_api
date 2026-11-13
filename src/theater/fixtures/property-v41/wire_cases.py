"""Authored property bodies from the instruction-addressed variant grammar."""
import json
from pathlib import Path
ROOT = Path(__file__).parent

def bits(n, v):
    return format(v, f'0{n}b')

def variant(tag, masked, seed):
    width = 0
    if masked and tag >= 7 or not masked and 1 <= tag <= 6:
        width = 4 if tag == 1 or tag >= 11 else 1 if tag in (2,10) else 24 if tag in (3,7) else 32
    return bits(4,tag) + (bits(width, seed & ((1<<width)-1)) if width else '')

def case(name, wire, **extra):
    padded=wire+'0'*(-len(wire)%8)
    return dict(name=name,hex=int(padded,2).to_bytes(len(padded)//8,'big').hex(),bit_len=len(wire),complete=True,**extra)

components=[];defaults=[]
for snapshot in (False,True):
    for masked in (False,True):
        for tag in range(16):
            for full in (False,True):
                seed=0xffffffff if full else 0
                components.append(case(f'property/{snapshot}/{masked}/{tag}/{full}',variant(tag,masked,seed),component='managed-object-'+('player-masked-' if masked else '')+'property-component',version=1,archetype=13,snapshot=snapshot))
            for version in (False,True):
                wire=('1'+bits(8,255)) if version else '0'
                wire+=bits(32,0x7b7315d7)+bits(1,int(masked))
                for i in range(32 if masked else 1):
                    wire+=variant((tag+i)%16,masked,0xffffffff-i)
                defaults.append(case(f'property-default/{snapshot}/{masked}/{tag}/{version}',wire,entity_type=13,snapshot=snapshot))
(ROOT/'components.json').write_text(json.dumps(components,indent=2)+'\n')
(ROOT/'defaults.json').write_text(json.dumps(defaults,indent=2)+'\n')
