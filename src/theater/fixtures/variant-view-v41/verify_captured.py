"""Compare locally exported source rows with original upstream value traces.
Usage: verify_captured.py LOCAL_EXPORT NATIVE_EXPORT
The trace must be produced by export_reference.py from the same source spans.
"""
import json
from pathlib import Path
import sys

local=json.loads(Path(sys.argv[1]).read_text())
native=json.loads(Path(sys.argv[2]).read_text())
assert len(local)==32
count=0
definitions=columns=omitted=0
string_entries=localized_values=matched_labels=0
for row,original in zip(local,native['cases'],strict=True):
    assert row['name']==original['name']
    hashes=[v['value']&0xffffffff for v in original['values'] if v['path']==[3,9,0,0,0]]
    values=[list((v['value']&0xffffffffffffffff).to_bytes(8,'little')) for v in original['values'] if v['path']==[3,9,0,1,0]]
    pairs=[dict(hash=h,raw=v) for h,v in zip(hashes,values,strict=True)]
    assert pairs==row['options'],row['name']
    assert original['end_byte']==len(bytes.fromhex(row['base_hex']))
    count+=len(values)
    for slot,(local_stat,native_stat) in enumerate(zip(row['definitions'],original['definitions'],strict=True)):
        assert local_stat['slot']==slot
        for key in ['id','name_hex','base','aggregation','sort','display','precision']:
            recorded=local_stat[key]
            if recorded is None:
                # Compare the original API's fallback without filling local data.
                assert native_stat[key]==('' if key=='name_hex' else 0),(row['name'],slot,key)
                omitted+=1
            else:
                assert recorded==native_stat[key],(row['name'],slot,key)
        definitions+=1
    assert [c['slot'] for c in row['columns']]==original['columns'],row['name']
    for column in row['columns']:
        assert row['definitions'][column['slot']]['id']==column['id']
        columns+=1
    strings=row['strings']
    offsets=[v['value'] for v in original['values'] if v['path']==[3,0,1,0,1]]
    assert offsets==[t['offset'] for entry in strings for t in entry['texts']],row['name']
    flags=[v['value'] for v in original['values'] if v['path']==[3,0,1,0,0]]
    assert flags==[int(entry['flag']) for entry in strings],row['name']
    text=original['text']
    assert strings[0]['texts'][0]['hex']==text['name_hex'],row['name']
    assert strings[1]['texts'][0]['hex']==text['description_hex'],row['name']
    english=[entry['texts'][0]['hex'] for entry in strings]
    # Compare text availability only. A string-table index is not a stat slot;
    # template strings and Alpha also occur in the indexed table.
    for label in text['labels']:
        assert english.count(label)==1,(row['name'],label)
        matched_labels+=1
    string_entries+=len(strings)
    localized_values+=len(offsets)
    assert original['spawn']==('None' if row['spawn'] is None else f"Some({row['spawn']})")
print(f'Original upstream traces match {len(local)} variants and {count} option rows, including all eight bytes and source order')
print(f'Stat projection matches {definitions} definitions and {columns} column references; {omitted} local fields remain absent where upstream supplies defaults')

print(f'Indexed strings match {string_entries} entry flags and {localized_values} offsets; all 32 names/descriptions and {matched_labels} native label texts are present without assuming label-slot mappings')
