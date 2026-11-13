"""Independent wire fields for the one-bit-shifted identification section."""
import json
from pathlib import Path

root = Path(__file__).parent
headers = json.loads((root / 'cases.json').read_text())
cases = []
for zero in [False, True]:
    for header in headers:
        raw = bytes.fromhex(header['hex'])
        wire = ''.join(format(x, '08b') for x in raw)[:header['bit_len']]
        fields = []

        def add(kind, data):
            global wire
            fields.append(dict(kind=kind, bit=len(wire), width=len(data) * 8, hex=data.hex()))
            wire += ''.join(format(x, '08b') for x in data)

        for index in range(2):
            add({'NameBuffer': index}, bytes((i * 17 + index) % 256 for i in range(256)))
        start = 0 if zero else 0xFEDCBA98
        add('StartTime', start.to_bytes(4, 'little'))
        for index, value in enumerate([0, 0xFFFFFFFF, 0x1234ABCD]):
            add({'UnknownWord': index}, value.to_bytes(4, 'little'))
        for index in range(3):
            add({'UnknownBlock': index}, bytes((i * 19 + index) % 256 for i in range(4096)))
        asset = bytes(16) if zero else bytes.fromhex('0123456789abcdef0123456789abcdef')
        version = bytes(16) if zero else bytes.fromhex('fedcba9876543210fedcba9876543210')
        add('MapAssetId', asset)
        add('MapVersionId', version)
        bit_len = len(wire)
        # The following game-options prefix is deliberately distinctive. The
        # identification reader must not consume it. Remaining bytes allow the
        # upstream header reader's own conservative size guard to pass.
        wire += '101011' + '11001010' * 16
        padded = wire.ljust((len(wire) + 7) // 8 * 8, '0')
        cases.append(dict(name=f'{header["name"]}/zero-{zero}', start_bit=header['bit_len'],
                          bit_len=bit_len, fields=fields, timestamp=start,
                          asset_hex=asset.hex(), version_hex=version.hex(),
                          hex=int(padded, 2).to_bytes(len(padded) // 8, 'big').hex()))

(root / 'identification.json').write_text(json.dumps(cases, indent=2) + '\n')
