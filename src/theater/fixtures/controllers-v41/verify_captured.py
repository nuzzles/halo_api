"""Check every exported canonical controller body against the pinned native block."""
import json,sys
local=json.load(open(sys.argv[1]));native=json.load(open(sys.argv[2]))
assert native['commit']=='8f0f8d078e513c800baa21fac416fd4c0c127d03'
assert len(local)==len(native['cases'])
counts={'observations':len(local),'present':0,'cleared':0,'holders':{}}
for c,n in zip(local,native['cases']):
 assert c['name']==n['name'] and c['bit_len']==n['end'] and n['complete']
 assert c['player']==n['player'],(c,n)
 counts['cleared' if c['player'] is None else 'present']+=1
 holder=c['holder']
 if holder is not None:
  # Only aggregate status classes, not participant identities.
  status=holder.split('(')[0];counts['holders'][status]=counts['holders'].get(status,0)+1
print(json.dumps(counts,indent=2))
