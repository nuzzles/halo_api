"""Independent script-v1 writer layouts, including the non-playback client form."""
import json
from pathlib import Path

def bits(width, value):
    return format(value, f'0{width}b') if width else ''

def header(code, present):
    wire = '1' + bits(7, code)
    # Script uses three 13-bit references; the following zoom leaves all absent.
    for index in range(3):
        wire += bits(1, present)
        if present:
            wire += bits(13, 8001 + index) + bits(2, index)
    return wire

cases = []
for prefix in [False, True]:
    for count in [0, 1, 7, 8, 31, 64, 511, 1023]:
        for present in [False, True]:
            wire = '1' + header(15, present)
            body_start = len(wire)
            widths = ([15] if prefix else []) + [13, 10, count]
            body = (bits(15, 0x56AB) if prefix else '') + bits(13, 0x1234) + bits(10, count)
            body += ''.join(str((i * 5 + i // 3) % 2) for i in range(count))
            wire += body
            record_end = len(wire)
            wire += header(21, False) + '10' + '0'
            bit_len = len(wire)
            wire += '0000'  # entity terminator and empty controls
            wire = wire.ljust((len(wire)+7)//8*8, '0')
            cases.append(dict(name=f'script/{prefix}/{count}/{present}',code=15,
                script_prefix=prefix,body_start=body_start,body_widths=widths,
                bit_len=bit_len,record_end=record_end,start_bit=1,complete=True,
                hex=int(wire,2).to_bytes(len(wire)//8,'big').hex()))
Path(__file__).with_name('script.json').write_text(json.dumps(cases,indent=2)+'\n')
