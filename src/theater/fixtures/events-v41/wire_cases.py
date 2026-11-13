"""Author event wire layouts independently of halo_api; all words are source bits."""
import json
import runpy
from pathlib import Path


def bits(width, value=0):
    assert 0 <= value < 1 << width
    return format(value, f'0{width}b')


def optional(width, present, value=None):
    return bits(1, present) + (bits(width, (1 << width) - 3 if value is None else value) if present else '')


def unless(width, present):
    return bits(1, not present) + (bits(width, (1 << width) - 3) if present else '')


KINDS = {7: [1, 8, 7], 9: [2, 8, 7], 11: [2, 8, 7], 21: [4, 8, 7],
         22: [1, 1, 7], 27: [0, 0, 7], 38: [2, 8, 7], 40: [2, 0, 7],
         41: [3, 8, 7], 46: [4, 8, 7], 47: [4, 8, 7], 58: [1, 8, 7],
         76: [1, 8, 7], 82: [0, 8, 7], 100: [1, 8, 7], 109: [0, 0, 7],
         118: [1, 8, 7], 14: [2, 2, 7], 17: [6, 6, 7], 50: [4, 4, 7],
         79: [4, 0, 7], 104: [0, 0, 7], 106: [0, 0, 7], 115: [1, 0, 7],
         32: [1, 8, 7], 64: [6, 8, 7], 78: [4, 8, 7], 116: [1, 0, 7], 102: [2, 8, 7],
         5: [5, 8, 7], 6: [5, 8, 7], 36: [1, 8, 7],
         0: [1, 1, 7], 1: [1, 8, 7], 39: [2, 8, 7], 75: [1, 8, 7],
         3: [0, 8, 7], 4: [0, 8, 7], 23: [0, 8, 7],
         24: [0, 2, 7], 25: [0, 2, 7], 26: [0, 2, 7],
         33: [3, 4, 7], 49: [3, 2, 7], 54: [1, 1, 7],
         57: [2, 0, 7], 59: [0, 8, 7], 92: [6, 8, 7], 103: [0, 0, 7]}


def header(code, present, unit):
    wire = '1' + bits(7, code)
    for slot, kind in enumerate(KINDS[code]):
        wire += bits(1, present)
        if present:
            if kind == 1:
                wire += bits(1, unit)
            width = 9 if kind in [4, 6] or (kind == 1 and unit) else 8 if kind in [2, 3, 5] else 13
            wire += bits(width, (1 << width) - 5) + bits(2, (slot + unit) % 4)
    return wire


CASES = []
def case(name, code, body, present=False, unit=False, complete=True, point_offsets=()):
    # Fire v3 needs an explicit configuration bit preceding the event list.
    # Its event-continuation bit must never masquerade as that configuration.
    start = 1 if code == 36 else 0
    wire = ('1' if start else '') + header(code, present, unit) + body
    record_end = len(wire)
    if complete:
        wire += header(21, False, False) + bits(2, 2) + '0'
    padded = wire.ljust((len(wire) + 7) // 8 * 8, '0')
    body_start = start + len(header(code, present, unit))
    CASES.append(dict(name=name, code=code, bit_len=len(wire), record_end=record_end,
                      indexed_points=[body_start + offset for offset in point_offsets],
                      complete=complete, hex=int(padded, 2).to_bytes(len(padded) // 8, 'big').hex()))
    if start:
        CASES[-1].update(start_bit=start, fire_v3=True)


for code in [7, 9, 11, 21, 22, 27, 38, 40, 41, 46, 47, 58, 76, 100, 109, 118]:
    for flags in range(8):
        a, b, c = [bool(flags & (1 << i)) for i in range(3)]
        if code == 7:
            body = bits(1, not a) + (optional(32, b) + bits(32, 0xDEADBEEF) if a else '')
            body += ''.join(bits(w, (1 << w) - 1) for w in [7, 7, 19, 2, 36, 19, 9, 16, 1, 1])
        elif code == 9:
            body = bits(3, 5) + optional(32, a)
        elif code == 11:
            body = unless(2, a) + optional(32, b) + bits(32, 0xABCDEF12) + bits(1, c) + unless(5, c)
        elif code in [21, 27]:
            body = bits(2, flags % 4)
        elif code == 22:
            body = bits(6, 59) + bits(1, a) + bits(3, 5)
        elif code == 38:
            body = bits(4, 13) + unless(5, a)
        elif code == 40:
            body = bits(3, 1 if a else flags) + (bits(2, 3) if a else '')
            body += bits(2, 2) + optional(32, b) + bits(32, 0xDEADBEEF) + unless(5, c)
        elif code == 41:
            body = bits(3, 5) + unless(5, a)
        elif code in [46, 47]:
            body = bits(1, c) + unless(2, a) + optional(32, b) + bits(32, 0xDEADBEEF)
            body += unless(5, c) if code == 47 else bits(1, c)
        elif code in [58, 118]:
            body = optional(32, a)
        elif code == 76:
            body = optional(32, a) + optional(32, b) + unless(5, c) + bits(32, 0x1234ABCD)
        elif code == 100:
            body = optional(32, a) + bits(32, 0xDEADBEEF) + optional(32, b)
        elif code == 109:
            body = bits(2, flags % 4) + optional(32, a) + optional(32, b) + bits(32, 0xDEADBEEF)
        case(f'body/{code}/{flags}', code, body, present=b, unit=c)

# Every possible early NUL and a full-length string without NUL. Include bytes
# outside printable ASCII; raw wire strings have no text-normalization license.
for length in range(17):
    value = bytes((0x80 + i) for i in range(length))
    if length < 16:
        value += b'\0'
    body = bits(32, 0xDEADBEEF) + bits(8, 213) + bits(3, 1)
    body += bits(32, 0xFEDCBA98) + bits(3, 5) + ''.join(bits(8, x) for x in value)
    body += '0' + bits(32, 0x98765432)
    case(f'game/string-{length}', 82, body)

for optional_player in [False, True]:
    for compact in [False, True]:
        body = bits(32, 0xDEADBEEF) + bits(8, 213) + bits(3, 7)
        for tag in range(7):
            body += bits(32, 0xABCDEF00 + tag) + bits(3, tag)
            if tag in [1, 2, 3, 6]:
                body += bits(32, 0x87654321)
            elif tag == 4:
                body += '1'
            elif tag == 5:
                body += bits(8, 0)
        body += '1' + bits(32, 0x12345678) + bits(3, 7)
        for tag in [0, 1, 2, 3, 4, 5, 7]:
            body += bits(3, tag)
            if tag == 1:
                body += unless(5, optional_player)
            elif tag == 2:
                body += bits(1, compact) + bits(24 if compact else 32, 0xABCDEF)
            elif tag > 2:
                body += bits(32, 0xFEDCBA98)
        body += bits(32, 0x1234ABCD)
        case(f'game/values/{optional_player}/{compact}', 82, body)

# Upstream establishes references even when it cannot decode the body's type.
for code in [14, 17, 50, 79, 104, 106, 115]:
    for unit in [False, True]:
        case(f'references/{code}/{unit}', code, '', present=True, unit=unit, complete=False)

def point(default):
    return ('1' + bits(22, 0x3ABCDE) + bits(22, 0x123456) + bits(22, 0x210003)
            if default else '00' + bits(15, 31001) + bits(15, 27003) + bits(17, 110005))


def add_point(body, points, default):
    if not default:
        points.append(len(body))
    return body + point(default)


for code in [32, 64, 78, 82, 116]:
    for flags in range(8):
        a, b, c = [bool(flags & (1 << i)) for i in range(3)]
        for default in [False, True]:
            body, points = '', []
            if code == 32:
                body = add_point(body, points, default)
                body = add_point(body, points, default)
                body += optional(32, a) + optional(32, b)
            elif code == 64:
                body += bits(6, 51)
                body = add_point(body, points, default)
                # Raw float words include negative zero and an explicit NaN payload.
                body += bits(32, 0x00000080) + bits(32, 0x4523C17F) + bits(32, 0x0000803F) + bits(1, a)
            elif code == 78:
                body = add_point(body, points, default)
                body += bits(1, not a) + (bits(19, 0x5ABCD) + bits(10, 771) if a else '')
                body += bits(2, flags % 4)
            elif code == 82:
                body += bits(32, 0xDEADBEEF) + bits(8, 213) + bits(3, 1) + bits(32, 0x1234ABCD) + bits(3, 7)
                body = add_point(body, points, default)
                body += '0' + bits(32, 0x87654321)
            else:
                body += bits(1, a)
                if a:
                    body += unless(19, b) + bits(8, 213)
                body += bits(1, c) + optional(32, b)
                if b:
                    body = add_point(body, points, default)
                    body = add_point(body, points, default)
            case(f'point/{code}/{flags}/{default}', code, body, present=c,
                 unit=a, point_offsets=points)

actions = runpy.run_path(str(Path(__file__).parent.parent / 'actions-v41/wire_cases.py'))['CASES']
for action in actions:
    case(f'action/{action["name"]}', 102, bits(5, action['variant']) + action['wire'],
         present=True, complete=action['complete'])

# Reduced-precision points. The detonation tail selects its eight-bit branch:
# upstream's fixed tail does not establish the local 19-bit branch.
for code, drop in [(5, 1), (6, 4)]:
    for flags in range(8):
        a, b, c = [bool(flags & (1 << i)) for i in range(3)]
        for default in [False, True]:
            body = bits(6, 51) if code == 5 else ''
            body += bits(1, not a)
            if a:
                body += optional(32, b) + bits(32, 0xDEADBEEF)
            if code == 5:
                body += optional(32, c)
            else:
                body += bits(7, 99) + bits(7, 117) + bits(19, 0x5ABCD)
            offsets = [] if default else [len(body)]
            body += bits(1, default)
            if default:
                body += ''.join(bits(22 - drop, (1 << (22 - drop)) - 1 - i) for i in range(3))
            else:
                body += bits(1, 0) + ''.join(bits(w - drop, (1 << (w - drop)) - 1 - i) for i, w in enumerate([15, 15, 17]))
            body += bits(19, 0x6ABCD)
            if code == 5:
                body += bits(5, 27) + bits(1, b) + bits(9, 479) + optional(18, c)
                body += '0' + bits(8, 213) + bits(2, flags % 4)
            else:
                body += bits(9, 479) + bits(1, b)
            case(f'projectile/{code}/{flags}/{default}', code, body, present=b,
                 unit=c, point_offsets=offsets)

fires = runpy.run_path(str(Path(__file__).with_name('fire_cases.py')))['CASES']
for fire in fires:
    case(fire['name'], 36, fire['body'], point_offsets=fire['point_offsets'])

# Empty bodies still carry typed references. Exercise absent, object-relative
# and unit-relative reference forms with an independently authored successor.
for code in [3, 4, 23, 24, 25, 26, 33, 49, 54, 57, 59, 92, 103]:
    for present, unit in [(False, False), (True, False), (True, True)]:
        case(f'empty/{code}/{present}/{unit}', code, '', present, unit)

for code, count in [(1, 4), (39, 8), (75, 16)]:
    for flags in range(count):
        a, b, c, d = [bool(flags & (1 << i)) for i in range(4)]
        if code == 1:
            body = bits(5, 29) + unless(4, a) + bits(3, 5) + optional(19, b)
        elif code == 39:
            body = bits(1, a) + (optional(32, b) + bits(4, 13) if a else bits(3, 5))
            body += unless(5, c)
        else:
            body = bits(5, 27) + optional(32, a) + unless(12, b) + unless(5, c)
            body += bits(1, d) + bits(1 + d, 2 if d else 1)
        for present, unit in [(False, False), (True, False), (True, True)]:
            case(f'conditional/{code}/{flags}/{present}/{unit}', code, body, present, unit)


def damage_body(gates, f58, flags14):
    a, b, c, d, e, f, g, h = [bool(gates & (1 << i)) for i in range(8)]
    body = optional(32, a) + unless(5, b) + bits(19, 0x5ABCD)
    body += bits(1, c) + (bits(19, 0x6ABCD) + bits(12, 3101) if c else '')
    body += bits(5, 27) + bits(5, 13) + bits(6, 51) + optional(5, d)
    # FUN_1407f15a4 records fifteen independent flags; only the last gates a word.
    body += bits(14, flags14) + optional(32, e)
    body += bits(1, 1) + bits(3, 5) + bits(5, 23) + bits(5, 11) + bits(1, 0) + bits(4, 13)
    body += unless(10, f) + bits(4, f58) + optional(32, g)
    body += bits(8, 213) if f58 == 1 else ''
    body += bits(4, 14) + bits(1, h) + (bits(13, 7919) + bits(2, 3) if h else '')
    return body


# Exhaust all eight independent presence gates with/without the f58 byte.
# Separate cases retain every other f58 value and each individual flag value.
for gates in range(256):
    for f58 in [0, 1]:
        case(f'damage/gates/{gates}/{f58}', 0, damage_body(gates, f58, 0x2AAA),
             present=bool(gates & 1), unit=bool(gates & 2))
for f58 in range(2, 16):
    for gates in [0, 255]:
        case(f'damage/f58/{f58}/{gates}', 0, damage_body(gates, f58, 0x1555))
for flag in range(14):
    for inverse in [False, True]:
        flags14 = (1 << flag) ^ (0x3FFF if inverse else 0)
        case(f'damage/flag/{flag}/{inverse}', 0, damage_body(255, 1, flags14))

assert len({c['name'] for c in CASES}) == len(CASES)
Path(__file__).with_name('cases.json').write_text(json.dumps(CASES, indent=2) + '\n')
print(f'Authored {len(CASES)} event cases')
