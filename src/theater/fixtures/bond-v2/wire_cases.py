"""Independent CBv2 field author; no halo_api decoder is involved."""
import json
from pathlib import Path

def varint(n):
    out = bytearray()
    while n >= 128:
        out.append((n & 127) | 128)
        n >>= 7
    out.append(n)
    return bytes(out)

def field(ty, id, value):
    header = (bytes([ty | id << 5]) if id < 6 else bytes([ty | 192, id]) if id < 256
              else bytes([ty | 224]) + id.to_bytes(2, 'little'))
    return header + value

def structure(fields):
    body = b''.join(fields) + b'\0'
    return varint(len(body)) + body

def scalar(ty):
    if ty in [2, 3, 14]: return b'\xff'
    if ty in [4, 15]: return varint(65535)
    if ty in [5, 16]: return varint(0xFFFFFFFF)
    if ty in [6, 17]: return varint(0xFFFFFFFFFFFFFFFF)
    if ty == 7: return bytes.fromhex('4523c17f')  # explicit NaN payload
    if ty == 8: return bytes.fromhex('0000000000000080')  # negative zero
    if ty == 9: return varint(3) + b'\xff\0A'  # raw bytes, not UTF-8 normalization
    if ty == 18: return varint(2) + bytes.fromhex('00d84100')  # unpaired UTF-16 surrogate
    if ty == 10: return structure([field(2, 7, b'\1'), b'\1', field(5, 0, b'\x80\0')])
    if ty in [11, 12]: return sequence(3, 8)
    if ty == 13: return bytes([3, 9, 1, 255, 1, 65])
    raise ValueError(ty)

def sequence(ty, count, explicit=False):
    header = bytes([ty | (count + 1) << 5]) if count < 7 and not explicit else bytes([ty]) + varint(count)
    return header + scalar(ty) * count

CASES = []
def add(name, data):
    CASES.append(dict(name=name, hex=data.hex(), bit_len=len(data) * 8))

add('empty', structure([]))
for ty in range(2, 19):
    add(f'type/{ty}', structure([field(ty, 0, scalar(ty))]))
for id in [0, 5, 6, 255, 256, 65535]:
    add(f'id/{id}', structure([field(5, id, varint(0xABCDEF))]))
add('extended-zero-and-duplicates', structure([b'\xc5\0\x80\0', field(5, 2, b'\1'), b'\1', b'\xe5\0\0\0', field(5, 2, b'\2')]))
for ty in range(2, 19):
    for count in [0, 1, 6, 7]:
        add(f'list/{ty}/{count}', structure([field(11, 0, sequence(ty, count))]))
    add(f'set/{ty}/explicit-zero', structure([field(12, 0, sequence(ty, 0, True))]))
for kt in range(2, 19):
    vt = 20 - kt
    add(f'map/{kt}/{vt}', structure([field(13, 0, bytes([kt, vt]) + varint(2) + (scalar(kt) + scalar(vt)) * 2)]))
add('string/empty', structure([field(9, 0, b'\0'), field(18, 1, b'\0')]))
add('string/long', structure([field(9, 0, varint(128) + bytes(range(128)))]))
add('packed-bytes/4096', structure([field(11, 0, bytes([3]) + varint(4096) + bytes(range(256)) * 16)]))
(Path(__file__).parent / 'cases.json').write_text(json.dumps(CASES, indent=2) + '\n')
