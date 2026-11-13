"""Compare admitted parent observations with the unchanged original block.
Unbound/refused observations are exported separately and never assigned a guessed
archetype for this comparison.
"""
import json,sys
local=json.load(open(sys.argv[1]));native=json.load(open(sys.argv[2]))
assert native['commit']=='8f0f8d078e513c800baa21fac416fd4c0c127d03'
assert len(local)==len(native['cases'])
counts={'observations':len(local),'attached':0,'detached':0,'unit_handles':0,'object_handles':0,'seat_present':0}
for c,n in zip(local,native['cases']):
 assert c['name']==n['name'] and c['bit_len']==n['end'] and n['complete']
 for field in ['parent','generation','node','seat']:assert c[field]==n[field],(c['name'],field,c[field],n[field])
 if c['parent'] is None:counts['detached']+=1
 else:
  counts['attached']+=1
  counts['unit_handles' if c['parent'][0] else 'object_handles']+=1
 counts['seat_present']+=c['seat'] is not None
print(json.dumps(counts,indent=2))
