"""Compare runtime assignments, classifying only source-proven scanner omissions.
Optional third input is the retained pinned LevelUp output; its payloads must be
byte-identical before its independent identity/option results may corroborate.
"""
import hashlib
import json
import sys
local=json.load(open(sys.argv[1]))
native=json.load(open(sys.argv[2]))
assert native['commit']=='8f0f8d078e513c800baa21fac416fd4c0c127d03'
assert len(local)==len(native['cases'])
go={c['Name']:c for c in json.load(open(sys.argv[3]))} if len(sys.argv)>3 else None
counts={'packets':len(local),'human_assignments':0,'bot_assignments':0,'native_agreements':0,'missing_engine_ids':0,'scanner_omissions':0,'retained_go_agreements':0}
omissions=[]
for case,reference in zip(local,native['cases']):
    assert case['Name']==reference['name']
    assert case['Actual']['stop']=='Complete'
    a=case['Assignments']; n=reference['assignments']
    assert len(a)==len(case['Actual']['entries'])
    by_id={v['identity']:v for v in n}
    assert len(by_id)==len(n)
    assert all(any(v['identity']==w['identity'] for w in a) for v in n),case['Name']
    data=bytes.fromhex(case['Data'])
    def bits(at,width):
        return sum(((data[(at+i)//8]>>(7-(at+i)%8))&1)<<(width-1-i) for i in range(width))
    if go is not None:
        original=go[case['Name']]
        assert original['Data']==case['Data'] and original['Version']==case['Version'] and original['Build']==case['Build']
        if not case['Bots']:
            assert len(original['Roster'])==len(a)
            for v,r in zip(a,original['Roster']):
                assert v['identity']==r['Identite'] and v['engine']==r['Option']
                counts['retained_go_agreements']+=1
    for ordinal,(assignment,entry) in enumerate(zip(a,case['Actual']['entries'])):
        counts['bot_assignments' if case['Bots'] else 'human_assignments']+=1
        original=by_id.get(assignment['identity'])
        if original is None:
            assert not case['Bots']
            f=next(f for f in entry['fields'] if f['kind']=={'Short':0})
            assert f['width']==10 and bits(f['bit'],10)==21
            identity=next(f for f in entry['fields'] if f['kind']=='Identity')
            join=next(f for f in entry['fields'] if f['kind']=='JoinKey')
            option=next(f for f in entry['fields'] if f['kind']=='Option')
            assert bits(identity['bit'],64)==bits(join['bit'],64)
            assert int.from_bytes(bits(identity['bit'],64).to_bytes(8,'big'),'little')==assignment['identity']
            assert option['width']==5 and bits(option['bit'],5)==assignment['engine']
            counts['scanner_omissions']+=1
            omissions.append(dict(name=case['Name'],entry=ordinal,entry_bits=[entry['start_bit'],entry['end_bit']],short_bit=f['bit'],short_value=21,payload_sha256=hashlib.sha256(data).hexdigest()))
        elif assignment['engine'] is None:
            counts['missing_engine_ids']+=1
        else:
            assert assignment==original,(case['Name'],assignment,original)
            counts['native_agreements']+=1
print(json.dumps(dict(commit=native['commit'],counts=counts,scanner_omissions=omissions),indent=2))
