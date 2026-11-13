"""Author the fixed v41/27 bootstrap prefix independently of the Rust parser."""
import json
from pathlib import Path

CASES = []
for flag in [False, True]:
    fields = []
    wire = ''

    def add(kind, data, width=None):
        global wire
        width = len(data) * 8 if width is None else width
        fields.append(dict(kind=kind, bit=len(wire), width=width, hex=data.hex()))
        wire += ''.join(format(x, '08b') for x in data)[:width]

    for code in range(123):
        value = 3 if code == 36 else [0, 1, 0xFFFFFFFF, 0x1234ABCD][code % 4]
        add({'EventVersion': code}, value.to_bytes(4, 'little'))
    # Retain bytes after a NUL, non-ASCII data and a buffer with no NUL.
    add('BuildVersion', b'6.10026.18411.0\0'.ljust(32, b'\xff'))
    add('BuildBranch', b'HI_1_13_0\0'.ljust(32, b'\xfe'))
    add('BuildFlavor', bytes(range(0x80, 0xA0)))
    add('BuildId', (0xFECDAB98).to_bytes(4, 'little'))
    add('Changelist', (0x87654321).to_bytes(4, 'little'))
    add('CorruptionChecks', bytes([0x80 if flag else 0]), 1)
    padded = wire.ljust((len(wire) + 7) // 8 * 8, '0')
    CASES.append(dict(name=f'flag/{flag}', bit_len=len(wire), fields=fields,
                      hex=int(padded, 2).to_bytes(len(padded) // 8, 'big').hex()))

Path(__file__).with_name('cases.json').write_text(json.dumps(CASES, indent=2) + '\n')
