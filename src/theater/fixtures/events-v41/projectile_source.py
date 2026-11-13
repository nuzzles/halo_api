"""Author both projectile tail branches from the pinned LevelUp grammar.

These synthetic delta events use explicit external widths. They do not claim
snapshot/full-precision admission or establish a map catalogue's bounds.
"""
import json
from pathlib import Path


def bits(width, value=0):
    assert 0 <= value < 1 << width
    return format(value, f'0{width}b')


def optional(width, present):
    return bits(1, present) + (bits(width, (1 << width) - 3) if present else '')


def header(code, present, unit):
    wire = '1' + bits(7, code)
    # Projectile references have kinds [5, 8, 7]. Unit does not change
    # their widths, but varies generations independently of body gates.
    for slot, width in enumerate([8, 13, 13]):
        wire += bits(1, present)
        if present:
            wire += bits(width, (1 << width) - 5) + bits(2, (slot + unit) % 4)
    return wire


CASES = []
for code, drop in [(5, 1), (6, 4)]:
    for flags in range(8):
        a, b, c = [bool(flags & (1 << i)) for i in range(3)]
        for default in [False, True]:
            for wide in ([False, True] if code == 5 else [False]):
                body = bits(6, 51) if code == 5 else ''
                body += bits(1, not a)
                if a:
                    body += optional(32, b) + bits(32, 0xDEADBEEF)
                if code == 5:
                    body += optional(32, c)
                else:
                    body += bits(7, 99) + bits(7, 117) + bits(19, 0x5ABCD)
                point = len(body)
                body += bits(1, default)
                if not default:
                    body += bits(1, 0)
                widths = [22, 22, 22] if default else [15, 15, 17]
                body += ''.join(bits(w - drop, (1 << (w - drop)) - 1 - i)
                                for i, w in enumerate(widths))
                body += bits(19, 0x6ABCD)
                if code == 5:
                    body += bits(5, 27) + bits(1, b) + bits(9, 479) + optional(18, c)
                    body += bits(1, wide) + bits(19 if wide else 8, 0x5ABCD if wide else 213)
                    body += bits(2, flags % 4)
                else:
                    body += bits(9, 479) + bits(1, b)
                for present, unit in [(False, False), (True, False), (True, True)]:
                    prefix = header(code, present, unit)
                    wire = prefix + body
                    end = len(wire)
                    # Known zoom event, absent references, terminator.
                    wire += '1' + bits(7, 21) + '000' + bits(2, 2) + '0'
                    padded = wire.ljust((len(wire) + 7) // 8 * 8, '0')
                    CASES.append(dict(
                        name=f'source/projectile/{code}/{flags}/{default}/{wide}/{present}/{unit}',
                        code=code, body_start=len(prefix), bit_len=len(wire), record_end=end,
                        complete=True, wide=wide,
                        indexed_points=[] if default else [len(prefix) + point],
                        hex=int(padded, 2).to_bytes(len(padded) // 8, 'big').hex()))

assert len(CASES) == 144
Path(__file__).with_name('projectile-source.json').write_text(json.dumps(CASES, indent=2) + '\n')
print(f'Authored {len(CASES)} source projectile cases')
