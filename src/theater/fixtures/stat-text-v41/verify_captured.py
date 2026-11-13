"""Check original per-row indices and existing native English label projections.
Usage: verify_captured.py LOCAL_CAPTURE INDEX_ORACLE VARIANT_ORACLE
"""
import json
from pathlib import Path
import sys

local=json.loads(Path(sys.argv[1]).read_text())
indices=json.loads(Path(sys.argv[2]).read_text())['cases']
labels=json.loads(Path(sys.argv[3]).read_text())['cases']
assert len(local)==32
definitions=references=texts=matched=0
extras=[]
for film,native,variant in zip(local,indices,labels,strict=True):
    assert film['name']==native['name']==variant['name']
    rows=film['stat_texts']
    assert len(rows)==len(film['definitions'])==len(native['indices'])
    assert len(variant['text']['labels'])<=len(rows)
    for slot,(row,expected) in enumerate(zip(rows,native['indices'],strict=True)):
        assert row['slot']==slot
        assert row['indices']==expected
        assert len(expected)==4 and all(i==expected[0] for i in expected)
        assert row['texts']==[t['hex'] for t in film['strings'][expected[0]]['texts']]
        if slot<len(variant['text']['labels']):
            assert row['texts'][0]==variant['text']['labels'][slot]
            matched+=1
        else:
            extras.append(dict(name=film['name'],slot=slot,index=expected[0],text=row['texts'][0]))
        definitions+=1;references+=len(expected);texts+=len(row['texts'])
assert extras==[dict(name='ranked-arena/02-oddball/film.json',slot=46,index=55,text=b'Carry Time'.hex())],extras
oddball=next(f for f in local if f['name']==extras[0]['name'])
assert oddball['stat_texts'][45]['indices']==oddball['stat_texts'][46]['indices']==[55]*4
assert oddball['definitions'][45]['id']!=oddball['definitions'][46]['id']
print(f'Original per-row reader matches {references} references across {definitions} definitions; {texts} localized stat-text values resolve through recorded table indices')
print(f'All {matched} native English stat labels match; one additional association preserves Oddball slots 45 and 46 sharing Carry Time')
