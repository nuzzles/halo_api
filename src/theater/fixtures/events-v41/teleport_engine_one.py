"""Engine-1 teleport composition from FUN_142efa2a8/FUN_141f86118.

The orientation decoder is FUN_142e29bac, independently traced from the pinned
LevelUp orientation implementation. Remaining gates/points follow the same
teleport body used by other engines; no captured engine-1 film is claimed.
"""
import json
from pathlib import Path
cases=[]
def b(n,v=0):return format(v,f'0{n}b')
def point(default):
    return ('1'+b(22,1)+b(22,(1<<22)-1)+b(22,12345)) if default else ('00'+b(15,1)+b(15,32767)+b(17,65537))
for refs in [0,1,2]:
    header='1'+b(7,116)
    for kind in [1,0,7]:
        header+=b(1,refs!=0)
        if refs:
            if kind==1:header+=b(1,refs==2)
            header+=b(9 if kind==1 and refs==2 else 13,17)+b(2,3)
    for default in [0,1]:
        for flag in [0,1]:
            for points in ['none','default','indexed','mixed']:
                wire=header+'1';start=len(wire)
                wire+=b(1,default)+('' if default else b(30,0x3456789))+b(30,0x2345678);end=len(wire)
                wire+=b(1,flag)+b(1,points!='none')
                if points!='none':wire+=b(32,0xDEADBEEF)+point(points in ['default','mixed'])+point(points=='default')
                record_end=len(wire)
                wire+='1'+b(7,21)+'000'+b(2,2)+'0'
                padded=wire+'0'*(-len(wire)%8)
                cases.append(dict(name=f'{refs}/{default}/{flag}/{points}',hex=int(padded,2).to_bytes(len(padded)//8,'big').hex(),bit_len=len(wire),record_end=record_end,orientation_bit=start,orientation_end=end))
assert len(cases)==48
Path(__file__).with_name('teleport-engine-one.json').write_text(json.dumps(cases,indent=2)+'\n')
