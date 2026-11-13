"""Independent action_weapon_fire v3 wire author (FUN_14080c1f8).

Bodies only: the enclosing fixture supplies configuration 1 and event headers.
Keep branch selection here independent of both Rust readers.
"""
from itertools import product


def word(width, value=None):
    value = (1 << width) - 1 if value is None else value
    assert 0 <= value < 1 << width
    return format(value, f'0{width}b')


def opt(width, present):
    return word(1, present) + (word(width) if present else '')


def handle(kind, unit, generation):
    width = 9 if unit else 13
    return ((word(1, unit) + word(width, (1 << width) - 5))
            if kind == 1 else word(8, 239)) + word(2, generation)


def target(kind, unit, marker):
    result = word(2, kind)
    if kind == 1:
        result += handle(1, unit, 3) + opt(6, marker)
    elif kind == 2:
        result += handle(2, False, 2)
    return result


def body(*, short=False, block=False, timestamp=False, stamp=False,
         player=False, slot=False, tag=False, empty=False, explicit=False,
         markers=(), barrels=(), aim=None, second=None, direction='direct-vector',
         unit=False, marker=False, six=False, selector=2, default=True,
         extra=False, four=False, twelve=(False, False), seven=False, origin=False):
    assert block or not timestamp
    wire = word(1, short) + word(1, block) + word(7, 91) + '1'
    wire += word(1, not player) + (word(5, 27) if player else '')
    wire += word(1, not slot) + (word(2, 2) if slot else '')
    wire += opt(32, tag) + word(32, 0xDEADBEEF) + word(2, 3)
    if block:
        wire += '1' + word(1, timestamp)
    if timestamp:
        wire += opt(10, stamp)
    if short:
        return wire + word(10, 919), []
    n, m = len(barrels), len(markers)
    assert n <= 15 and m <= 15
    wire += word(1, empty)
    if empty:
        assert n == m == 0
    else:
        wire += '1' if n == 1 and not explicit else '0' + word(4, n)
        wire += ('1' if m == 0 and not explicit else
                 '01' if m == 1 and not explicit else '00' + word(4, m))
    for i, kind in enumerate(markers):
        wire += word(2, kind) + word(1, i % 2) + handle(1, bool(i % 2), i % 4)
    follow = True
    for i, entry in enumerate(barrels):
        wire += word(4, i) + word(1, entry is not None)
        if entry is None:
            continue
        count, index = entry
        follow = count != 0
        width = 12 if n == 1 else 4
        if index < m and markers[index] == 1:
            width = min(width, 6)
        wire += word(3, count) + word(1 if m < 3 else 4, index) + word(16, 0xABCD)
        wire += ''.join(word(width, (1 << width) - 1 - axis) for axis in range(3))
    if follow:
        wire += word(1, aim is not None)
        if aim is not None:
            wire += target(aim, unit, marker)
            direct = direction.startswith('direct-')
            wire += word(1, direct)
            if not direct:
                wire += word(4, 11) + word(4, 13) + word(1, direction != 'indirect-none')
            if direction != 'indirect-none':
                zero = direction.endswith('-zero')
                wire += word(1, zero)
                if not zero:
                    wire += word(15, 31001) + word(7, 117)
        wire += word(1, second is not None)
        if second is not None:
            wire += target(second, unit, marker)
        if aim is None:
            wire += word(30, 0x23456789)
    if not timestamp:
        wire += opt(6, six) + word(6, 53)
    points = []

    def point():
        nonlocal wire
        if default:
            wire += '1' + ''.join(word(22, v) for v in [0x3ABCDE, 0x123456, 0x210003])
        else:
            points.append(len(wire))
            wire += '00' + word(15, 31001) + word(15, 27003) + word(17, 110005)

    if block:
        wire += word(2, selector)
        if selector == 0:
            point()
        elif selector == 1:
            wire += word(19, 0x5ABCD)
        wire += word(4, 13)
    else:
        wire += word(2, 3) + word(1, extra) + opt(4, four)
        if extra:
            wire += opt(12, twelve[0]) + opt(12, twelve[1])
    wire += word(6, 59) + opt(7, seven) + word(1, origin)
    if origin:
        point()
    return wire, points


CASES = []


def add(name, **options):
    wire, points = body(**options)
    CASES.append(dict(name='fire/' + name, body=wire, point_offsets=points))


# Every meaningful timestamp mode, short return, and prefix gate combination.
for short, mode, flags in product([False, True], range(4), range(8)):
    add(f'prefix/{short}/{mode}/{flags}', short=short, block=mode != 0,
        timestamp=mode >= 2, stamp=mode == 3, player=bool(flags & 1),
        slot=bool(flags & 2), tag=bool(flags & 4), empty=True)

# Shorthand and explicit counts; marker kinds, handle widths/generations,
# one/multiple barrel coordinate widths, and out-of-list marker references.
for n, m, explicit in product([0, 1, 2, 15], [0, 1, 2, 3, 15], [False, True]):
    kinds = tuple(i % 4 for i in range(m))
    indices = [0, 1] if m < 3 else [0, 1, 2, 14, 15]
    for index in indices:
        add(f'counts/{n}/{m}/{explicit}/{index}', markers=kinds, explicit=explicit,
            barrels=tuple((7, index) for _ in range(n)), six=True)

# A later absent barrel does not reset the last present barrel's aim decision.
for i, entries in enumerate([
        (None,), ((0, 0),), ((7, 0),), (None, None),
        ((0, 0), None), ((7, 0), None), ((0, 0), (7, 0)),
        ((7, 0), (0, 0)), (None, (0, 0)), (None, (7, 0))]):
    add(f'barrel-order/{i}', barrels=entries, markers=(1,), aim=1, second=2)

# Both target discriminants, all vector gates, kind-1 optional marker/handle.
for aim, second, direction in product([None, 0, 1, 2, 3], [None, 0, 1, 2, 3],
                                      ['direct-zero', 'direct-vector', 'indirect-none',
                                       'indirect-zero', 'indirect-vector']):
    if aim is None and direction != 'direct-zero':
        continue
    add(f'aim/{aim}/{second}/{direction}', aim=aim, second=second, direction=direction)
for unit, marker in product([False, True], repeat=2):
    add(f'target-handle/{unit}/{marker}', aim=1, second=1, unit=unit, marker=marker)

# Both point sites, all block selectors, timestamp/no timestamp and tail gates.
for selector, default, timestamp, flags in product(range(4), [False, True],
                                                  [False, True], range(4)):
    add(f'block/{selector}/{default}/{timestamp}/{flags}', block=True,
        selector=selector, default=default, timestamp=timestamp, stamp=True,
        seven=bool(flags & 1), origin=bool(flags & 2), six=True)
for extra, four, first, second, seven, origin in product([False, True], repeat=6):
    if not extra and (first or second):
        continue
    add(f'tail/{extra}/{four}/{first}/{second}/{seven}/{origin}', extra=extra,
        four=four, twelve=(first, second), seven=seven, origin=origin)
