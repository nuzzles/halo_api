"""Author MSB-first point bodies from the independently exported catalogue."""
import json
from pathlib import Path
root = Path(__file__).parent
catalogue = json.loads((root / 'catalogue.json').read_text())
cases = []
for level in catalogue['levels']:
    for region in level['regions']:
        for kind, precision in [('event',16),('event',15),('event',12),('dynamic',16),('object',16)]:
            widths = [w-(16-precision) for w in region['axis_bits']]
            codes = [0,(1<<widths[1])-1, (1<<(widths[2]-1))+3]
            fields = [(0, {'event':1,'dynamic':4,'object':2}[kind]), (region['index'],level['index_bits'])] + list(zip(codes,widths))
            if kind != 'event': fields += [(2,2)]
            bits = ''.join(format(v,f'0{w}b') for v,w in fields)
            end = len(bits)
            bits += '11010011' # following marker, outside the point body
            bits += '0' * (-len(bits)%8)
            cases.append(dict(name=f"{level['module']}-{region['index']}-{kind}-{precision}",tag=level['tag'],region=region['index'],kind=kind,precision=precision,codes=codes,widths=widths,hex=int(bits,2).to_bytes(len(bits)//8,'big').hex(),bit_len=end,upstream=region['index']<4))
# Default-box words are independent of the level and must not use its widths.
for kind, precision in [('event',16),('event',15),('event',12),('object',16)]:
    w=22-(16-precision); codes=[1,(1<<w)-1,1<<(w-1)]
    bits=('01' if kind=='object' else '1')+''.join(format(v,f'0{w}b') for v in codes)
    if kind=='object': bits+='10'
    end=len(bits); bits+='11010011'; bits+='0'*(-len(bits)%8)
    cases.append(dict(name=f'default-{kind}-{precision}',tag=catalogue['levels'][0]['tag'],region=None,kind=kind,precision=precision,codes=codes,widths=[w]*3,hex=int(bits,2).to_bytes(len(bits)//8,'big').hex(),bit_len=end,upstream=True))
assert len(cases)==329
(root/'points.json').write_text(json.dumps(cases,indent=2)+'\n')
