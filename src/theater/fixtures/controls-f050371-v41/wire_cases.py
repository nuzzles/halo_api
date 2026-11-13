"""Authored secondary-control envelopes; no halo_api or upstream code is imported."""
import itertools
import json
from pathlib import Path


def bits(width, value=0):
    assert 0 <= value < (1 << width)
    return format(value, f'0{width}b')


def handle(width, generation):
    return bits(width, (1 << width) - 1) + bits(2, generation)


def reference(form, generation):
    if form == 'none':
        return '0'
    return '1' + ('0' + handle(13, generation) if form == 'object' else '1' + handle(9, generation))


def point(mode):
    if mode == 'local':
        return '1' + bits(14, 0) + bits(14, 0x1555) + bits(14, 0x3fff)
    if mode == 'default':
        return '01' + bits(22, 0) + bits(22, 0x234567) + bits(22, 0x3fffff)
    if mode == 'world':
        return '000' + bits(15, 0x7123) + bits(15, 0) + bits(17, 0x1ffff)
    return '00' + bits(2, 3) + bits(19, 0x71234) + bits(19, 0) + bits(16, 0xffff)


def prefix(state, unit, generation):
    return bits(5, state) + bits(2, 3) + bits(17, 100001) + bits(16, 0xabcd) + reference(unit, generation)


cases = []


def add(name, body, generation, mode, unit, second, target):
    # A complete kind-0 input (no first block) followed by a distinctive kind-2
    # value and the list terminator. Baseline absent/present-zero are distinct.
    baseline = '0' if generation % 2 == 0 else '1' + bits(7, 0)
    wire = '100' + baseline + bits(5, 21) + '01' + body
    secondary_end = len(wire)
    wire += '11001' + bits(13, 0x15ab) + '0'
    padded = wire.ljust((len(wire) + 7) // 8 * 8, '0')
    cases.append(dict(
        name=name, hex=int(padded, 2).to_bytes(len(padded) // 8, 'big').hex(),
        bit_len=len(wire), kinds=[0, 2], command_counts=[], secondary=[True],
        index_bits=2 if mode == 'region3' else 1,
        region_widths={'3': [19, 19, 16]} if mode == 'region3' else {},
        generation=generation, position_mode=mode, unit=unit, reference=second,
        target=target, secondary_end=secondary_end,
    ))


for state, unit, generation in itertools.product([0, 1, 4, 29], ['none', 'object', 'unit'], range(4)):
    add(f'short/{state}/{unit}/{generation}', prefix(state, unit, generation), generation,
        'none', unit, 'none', False)

for unit, second, mode, target, generation in itertools.product(
        ['none', 'object', 'unit'], ['none', 'object', 'unit'],
        ['local', 'default', 'world', 'region3'], [False, True], range(4)):
    body = prefix(2 if generation % 2 == 0 else 31, unit, generation)
    body += reference(second, generation) + bits(2, 2) + bits(3, 5)
    count = [0, 1, 3, 7][generation]
    body += bits(3, count)
    for i in range(count):
        body += ('0' if i % 2 else '1' + handle(13, generation)) + bits(4, i) + bits(4, 15 - i)
    body += bits(5, 29) + bits(3, 6)
    body += '0' + bits(2, 2) + '1' + bits(6, 47) + bits(3, 5) + bits(3, 3)
    body += ('0' + bits(6, 56)) if generation % 2 else '1'
    body += point(mode)
    body += ('0' + bits(19, 0x5abcd)) if generation % 2 else '1'
    body += bits(8, 0xaa)
    for width in [10, 8]:
        body += ('0' + bits(19, 0x23456) + bits(width, (1 << width) - 1)) if generation % 2 else '1'
    body += ('1' + handle(13, generation) + bits(11, 0x7ff)) if target else '0'
    body += bits(7, 101)
    add(f'full/{unit}/{second}/{mode}/{int(target)}/{generation}', body, generation,
        mode, unit, second, target)

assert len(cases) == 336
assert len({c['name'] for c in cases}) == len(cases)
Path(__file__).with_name('cases.json').write_text(json.dumps(cases, indent=2) + '\n')
