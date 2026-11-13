"""Compare rules using original decoder output, without rounding local values."""
import collections
import json
from pathlib import Path
import sys

local=json.loads(Path(sys.argv[1]).read_text())
native=json.loads(Path(sys.argv[2]).read_text())
assert len(local)==32
counts=collections.Counter()
for film,original in zip(local,native['cases'],strict=True):
    assert film['name']==original['name']
    rules=[r for group in film['rule_groups'] if group['evidence'] for r in group['rules']]
    assert [r['name'] for r in rules]==[r['name'] for r in original['rules']],film['name']
    for rule,expected in zip(rules,original['rules'],strict=True):
        assert rule['default']==expected['default'],(film['name'],rule['name'])
        state=rule['state'];evaluation=rule['evaluation']
        counts[evaluation['kind']]+=1
        for alias in rule['aliases']:
            assert alias['rows']==[i for i,o in enumerate(film['options']) if o['hash']==alias['hash']]
        if state['kind']=='recorded':
            row=state['row'];alias=state['alias']
            assert rule['aliases'][alias]['rows']==[row]
            assert all(not a['rows'] for a in rule['aliases'][:alias])
            assert state['raw']==film['options'][row]['raw']
            assert evaluation==dict(kind='recorded',raw=state['raw'])
            assert original['decoded_rows'][str(row)]==expected['value']==expected['effective']
        else:
            # This corpus has complete, identified option rows. Unknown states
            # must fail this check instead of being reclassified as defaults.
            assert state['kind']=='absent',(film['name'],rule['name'],state)
            assert expected['value']=='-'
            assert expected['effective']==rule['default']
            assert evaluation==(dict(kind='no-literal-default') if rule['default']=='-' else dict(kind='default',value=rule['default']))
print(f'Original rule selection/default/effective projection matches all 32 films and {sum(counts.values())} evidenced rules: {dict(counts)}')
print('Recorded values retain all eight bytes; original decoder output is compared separately without replacing local f32 words')
