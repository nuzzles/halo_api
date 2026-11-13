"""Add independent packet-envelope checks and input hashes to a vehicle audit.

Usage: seal_vehicle_context.py CORPUS AUDIT_JSON [REPLACEMENT_FILM_AUDIT_JSON]
Prints a sealed report; never changes expected parser results or supplies context.
The optional replacement contains the final focused rerun for one film.
"""
import hashlib
import json
from pathlib import Path
import struct
import sys

root=Path(sys.argv[1])
films=json.loads(Path(sys.argv[2]).read_text())
if len(sys.argv)>3:
    replacement=json.loads(Path(sys.argv[3]).read_text())
    assert len(replacement)==1
    indexes=[i for i,f in enumerate(films) if f['film']==replacement[0]['film']]
    assert len(indexes)==1
    films[indexes[0]]=replacement[0]
assert len(films)==32 and len({f['film'] for f in films})==32

inputs={}
packets={}
def inspect_source(source):
    name=source['file']
    if name not in packets:
        data=(root/name).read_bytes()
        inputs[name]=dict(bytes=len(data),sha256=hashlib.sha256(data).hexdigest())
        headers=[];offset=0
        while offset+16<=len(data):
            kind,flags,size,timestamp=struct.unpack_from('<HHIQ',data,offset)
            assert offset+16+size<=len(data),name
            headers.append((kind,timestamp,offset+16,size))
            offset+=16+size
        assert offset==len(data),name
        packets[name]=headers
    kind,timestamp,offset,size=packets[name][source['packet']]
    assert kind in [0,2] and timestamp==source['timestamp_us'],source
    assert 0<=source['start_bit']<=source['end_bit']<=size*8,source
    source.update(packet_type=kind,payload_offset=offset,payload_bytes=size)

def walk(value):
    if isinstance(value,dict):
        if 'file' in value and 'packet' in value and 'start_bit' in value:inspect_source(value)
        else:
            for child in value.values():walk(child)
    elif isinstance(value,list):
        for child in value:walk(child)
walk(films)
for film in films:
    for filename in ['film.json','chunk-000-type-1.bin']:
        name=film['film']+'/'+filename
        data=(root/name).read_bytes()
        inputs[name]=dict(bytes=len(data),sha256=hashlib.sha256(data).hexdigest())
print(json.dumps(dict(
    schema_version=1,
    writer_source_commit='413971bcc485be53a2fa8cf229e170539a5146f8',
    original_reference_commit='18067987ebc3628ad499c819c6aee814692df43b',
    refreshed_reference_commit='8f0f8d078e513c800baa21fac416fd4c0c127d03',
    policy='Source-order audit only. No asset gate is enabled; same-ID NEWs are candidates, not established lifetime links.',
    inputs=dict(sorted(inputs.items())),films=films,
),indent=2))
