"""Independently authored creation data, never generated from halo_api output."""
import json
from pathlib import Path


def bits(width, value=0):
    assert 0 <= value < 1 << width
    return format(value, f"0{width}b")


def handle(width, value, generation):
    return bits(width, value) + bits(2, generation)


def common(count=0, extras=False):
    wire = bits(9, 381) + bits(32, 0xFEDCBA98)
    wire += ("0" + bits(32, 0x1234ABCD)) if extras else "1"
    wire += ("1" + bits(18, 200003) + "1" + bits(13, 7003)) if extras else "00"
    wire += bits(2, 3) + bits(5, 29) + bits(3, count)
    if count > 4:
        return wire
    for i in range(count):
        wire += bits(5, 25 + i) + bits(1, i % 2)
        if i % 2:
            wire += bits(32, 0xDEADBEEF + i)
    if extras:
        wire += "10" + bits(5, 19) + "1" + bits(8, 237) + bits(8, 179) + bits(8, 227)
        wire += "1" + bits(32, 0xABCDEF12) + "1" + bits(32, 0x12345678) + bits(14, 12001)
    else:
        wire += "00"
    return wire


def point(snapshot, default_region):
    if snapshot:
        return bits(32, 0x00000080) + bits(32, 0x4523C17F) + bits(32, 0x0000803F)
    if default_region:
        return "1" + bits(22, 0x3ABCDE) + bits(22, 1234) + bits(22, 0x210001)
    return "00" + bits(15, 31001) + bits(15, 17003) + bits(17, 110005)


def projectile(snapshot=False, version=None, count=0, extras=False,
               target=None, unit=False, target_index=False, position=False,
               default_region=False, relative=None, relative_tail=False):
    wire = ("1" + bits(8, version)) if version is not None else "0"
    wire += common(count, extras)
    if count > 4:
        return wire
    wire += ("1" + bits(5, 27)) if extras else "0"
    if extras:
        wire += "11" + bits(1, unit) + handle(9 if unit else 13, 301, 0) + "0" + bits(2, 2)
    else:
        wire += "0"
    wire += bits(1, target is not None)
    if target is not None:
        wire += bits(2, target)
        if target in [1, 2]:
            if snapshot:
                wire += bits(32, 0x87654321)
            elif target == 1:
                wire += bits(1, unit) + handle(9 if unit else 13, 303, 3)
            else:
                wire += handle(8, 213, 0)
            if target == 1:
                wire += bits(1, target_index) + (bits(6, 53) if target_index else "")
    wire += "1" + bits(1, extras) + (bits(5, 27) + bits(5, 19) if extras else "") + bits(5, 23)
    wire += bits(1, position)
    if position:
        wire += point(snapshot, default_region) + bits(19, 0x5ABCD) + bits(12, 3701)
    wire += "11" + handle(13, 7001, 0) if extras else "0"
    wire += bits(1, relative is not None)
    if relative is not None:
        wire += bits(1, relative)
        if relative:
            wire += bits(2, 3) + bits(13, 7001) + bits(13, 5103) + bits(13, 4097)
            wire += bits(1, relative_tail) + (bits(16, 55003) if relative_tail else "")
        else:
            wire += point(snapshot, default_region)
    if version is not None and version > 2:
        wire += "1"
    wire += ("10" + bits(5, 29) + bits(7, 99) + "1" + bits(4, 13)) if extras else "0"
    return wire


CASES = []
def case(name, snapshot=False, **kwargs):
    wire = projectile(snapshot=snapshot, **kwargs)
    padded = wire.ljust((len(wire) + 7) // 8 * 8, '0')
    CASES.append(dict(name=name, entity_type=41, snapshot=snapshot,
                      bit_len=len(wire), complete=kwargs.get('count', 0) <= 4,
                      hex=int(padded, 2).to_bytes(len(padded) // 8, 'big').hex()))

for snapshot in [False, True]:
    form = 'snapshot' if snapshot else 'tick'
    for version in [None, 0, 1, 2, 3, 255]:
        for extras in [False, True]:
            case(f'{form}/version-{version}/{extras}', snapshot, version=version, extras=extras)
    for count in range(8):
        case(f'{form}/common-count-{count}', snapshot, count=count, extras=True)
    for target in [0, 1, 2, 3]:
        for unit in [False, True]:
            for index in [False, True]:
                case(f'{form}/target-{target}/{unit}/{index}', snapshot,
                     target=target, unit=unit, target_index=index, extras=True)
    for default_region in [False, True]:
        for relative in [None, False, True]:
            for tail in [False, True]:
                case(f'{form}/positions-{default_region}/{relative}/{tail}', snapshot,
                     version=3, extras=True, count=4, position=True,
                     default_region=default_region, relative=relative, relative_tail=tail)
# The same invalid count must stop every common-object caller before its
# type-specific tail. These are upstream refusals, not synthetic valid bodies.
for snapshot in [False, True]:
    for entity_type in [35, 36, 37, 38, 39, 40, 42, 43]:
        for count in [5, 6, 7]:
            prefix = "001" if entity_type == 35 else "00" if entity_type in [37, 42] else "0"
            wire = prefix + common(count, extras=True)
            padded = wire.ljust((len(wire) + 7) // 8 * 8, '0')
            CASES.append(dict(name=f'common-invalid/{entity_type}/{snapshot}/{count}',
                              entity_type=entity_type, snapshot=snapshot,
                              bit_len=len(wire), complete=False,
                              hex=int(padded, 2).to_bytes(len(padded) // 8, 'big').hex()))
for snapshot in [False, True]:
    for present in [False, True]:
        for default_region in [False, True]:
            for list_count in [None, 0, 1, 3]:
                wire = "0" + common(4, extras=True) + bits(1, present)
                if present:
                    wire += point(snapshot, default_region) + "0" + bits(19, 0x5ABCD) + bits(8, 217)
                wire += bits(19, 0x6ABCD) + bits(1, list_count is not None)
                if list_count is not None:
                    wire += bits(2, list_count)
                for i in range(1 if list_count is None else list_count):
                    wire += bits(1, i % 2 == 0) + (bits(32, 0xFEEDABCD + i) if i % 2 == 0 else "")
                padded = wire.ljust((len(wire) + 7) // 8 * 8, '0')
                CASES.append(dict(name=f'vehicle/{snapshot}/{present}/{default_region}/{list_count}',
                                  entity_type=40, snapshot=snapshot, bit_len=len(wire), complete=True,
                                  hex=int(padded, 2).to_bytes(len(padded) // 8, 'big').hex()))
def raw_case(name, entity_type, snapshot, wire):
    padded = wire.ljust((len(wire) + 7) // 8 * 8, '0')
    CASES.append(dict(name=name, entity_type=entity_type, snapshot=snapshot,
                      bit_len=len(wire), complete=True,
                      hex=int(padded, 2).to_bytes(len(padded) // 8, 'big').hex()))

# Every version threshold, present/absent references and inverse player gates.
for snapshot in [False, True]:
    for version in [None, 0, 5, 6, 10, 11, 12, 13, 255]:
        for present in [False, True]:
            for variant in [0, 1]:
                v = 13 if version is None else version
                wire = "0" if version is None else "1" + bits(8, version)
                wire += bits(1, variant) + (bits(32, 0xDEADBEEF) if variant else "")
                if v > 10:
                    wire += bits(1, variant) + ("" if variant else bits(5, 27))
                wire += common(4 if variant else 0, extras=bool(variant))
                wire += bits(1, variant) + (bits(6, 59) if variant else "") + "1"
                wire += bits(1, present)
                if present:
                    wire += bits(32, 0x87654321) + point(snapshot, bool(variant))
                    wire += bits(1, variant) + ("" if variant else bits(5, 29))
                wire += bits(19, 0x5ABCD)
                if v > 5:
                    wire += bits(1, variant)
                if v >= 12:
                    wire += bits(1, variant) + (bits(32, 0x1234ABCD) if variant else "")
                raw_case(f'biped/{snapshot}/v{version}/{present}/{variant}', 35, snapshot, wire)

# Property kinds and values interleave; array mode changes the domain, not widths.
for snapshot in [False, True]:
    for masked in [False, True]:
        for seed in range(16):
            wire = "1" + bits(8, 3) + bits(32, 0xDEADBEEF) + bits(1, masked)
            for i in range(32 if masked else 1):
                tag = (seed + i) % 16
                wire += bits(4, tag)
                if tag != 0 and (tag < 7) != masked:
                    width = {1: 4, 11: 4, 2: 1, 10: 1, 3: 24, 7: 24}.get(tag, 32)
                    wire += bits(width, ((1 << width) - 1) ^ (i % (1 << width)))
            raw_case(f'property/{snapshot}/{masked}/{seed}', 13, snapshot, wire)

# Remaining nonempty manager readers supported by the pinned upstream.
# Each gate has both states; explicit versions include a nonzero byte.
for snapshot in [False, True]:
    for entity_type in [3, 5, 6, 9, 10, 11, 12, 14, 17, 21, 29, 47, 49]:
        for present in [False, True]:
            wire = "" if entity_type in [6, 21] else ("1" + bits(8, 255) if present else "0")
            if entity_type in [3, 10]:
                wire += bits(1, present) + (handle(13, 7001, 0) if present else "")
            if entity_type == 3:
                wire += bits(1, present) + (bits(8, 217) if present else "")
                wire += bits(1, not present) + bits(8, 179)
            elif entity_type in [5, 6, 14, 17, 47]:
                width = {5: 6, 6: 6, 14: 5, 17: 7, 47: 5}[entity_type]
                wire += bits(width, (1 << width) - 3)
            elif entity_type == 9:
                wire += bits(6, 51) + bits(6, 37) + bits(1, present)
            elif entity_type == 21:
                wire += bits(18, 200003)
            raw_case(f'manager/{entity_type}/{snapshot}/{present}', entity_type, snapshot, wire)

# Item/device, rigid-body, equipment and weapon tails, following their common
# body. The weapon's maximum tag count tests each optional entry in sequence.
for snapshot in [False, True]:
    for entity_type in [36, 37, 38, 39, 42, 43]:
        for present in [False, True]:
            for extras in [False, True]:
                version = "1" + bits(8, 255) if present else "0"
                prefix = version * (2 if entity_type in [37, 42] else 1)
                prefix += common(4 if extras else 0, extras)
                variants = [None, 0, 1, 15] if entity_type == 42 else [None]
                for count in variants:
                    wire = prefix
                    if entity_type in [38, 39]:
                        wire += bits(1, present) + (handle(13, 7001, 3) if present else "")
                    elif entity_type == 37:
                        wire += bits(1, not present) + (bits(5, 29) if present else "")
                        wire += bits(1, extras) + (bits(32, 0xDEADBEEF) if extras else "")
                    elif entity_type == 42:
                        wire += bits(12, 3701) + bits(7, 99)
                        wire += bits(1, count is not None)
                        if count is not None:
                            wire += bits(4, count)
                        for i in range(1 if count is None else count):
                            tag_present = present != (i % 2 == 1)
                            wire += bits(1, tag_present)
                            if tag_present:
                                wire += bits(32, 0xDEAD0000 + i)
                        wire += bits(1, not extras) + (bits(5, 27) if extras else "")
                    raw_case(f'item/{entity_type}/{snapshot}/{present}/{extras}/{count}',
                             entity_type, snapshot, wire)

assert len({c['name'] for c in CASES}) == len(CASES)
Path(__file__).with_name('cases.json').write_text(json.dumps(CASES, indent=2) + '\n')
print(f'Authored {len(CASES)} creation cases')
