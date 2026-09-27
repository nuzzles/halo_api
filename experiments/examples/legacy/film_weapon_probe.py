"""Legacy field annotations used only by Theater Lab's byte inspector.

Full probe sources are in archive/cleanup-2026-09-10/examples-before-cleanup.zip.
The upstream Rust decoder handles film decoding and replay.
"""

import re

from legacy.build_oddball_scene import CLOCKS

from legacy.film_vitality_probe import INPUT_END, vitality_window

AUXILIARY_PREFIX = '0100000000000100100001001100000'


def reload_fields(bits, offset):
    event = bits[offset:offset + 32]
    if (len(event) != 32 or event[1:10] != '101001101'
            or event[18:20] not in ('01', '10') or event[20:27] != '0001000'):
        return None
    clock_bit = offset + 31
    auxiliary = bits[clock_bit + 1:clock_bit + 32] == AUXILIARY_PREFIX
    if auxiliary:
        clock_bit += 64
    clock = bits[clock_bit:clock_bit + 37]
    # The manual control's boundary has leading 111. The following 26 signature
    # bits and eight-bit counter agree with clock-shaped records elsewhere;
    # the first three bits are left uninterpreted here, not generalized globally.
    if len(clock) != 37 or clock[:3] not in ('001', '101', '111') or clock[3:29] not in [c[3:] for c in CLOCKS]:
        return None
    wire, generation = int(event[10:18], 2), int(event[18:20], 2)
    return dict(wire=wire, generation=generation, serial=wire + 256 * (generation - 1),
                player=int(event[27:32], 2), clock_bit=clock_bit, auxiliary=auxiliary)


DELTA = re.compile(r'(?=(100010[01]{8}(?:01|10)00))')


def weapon_delta(bits, offset, ranked, clock):
    """Decode only captured sparse forms, including their exact continuation."""
    if not DELTA.match(bits, offset) or len(bits) < offset + 21:
        return None
    count = int(bits[offset + 18:offset + 21], 2)
    if not count or len(bits) < offset + 21 + 6 * count:
        return None
    ids = [int(bits[offset + 21 + 6 * i:offset + 27 + 6 * i], 2) for i in range(count)]
    if ids != sorted(set(ids)) or not set(ids) & {30, 33, 42}:
        return None
    cursor, fields = offset + 21 + 6 * count, []
    for component in ids:
        start = cursor
        if component == 0:
            width = 58 if ranked else 54
            if bits[cursor:cursor + 5] != '00000' or bits[cursor + width - 2:cursor + width] != '00':
                return None
            cursor += width
        elif component == 1:
            if bits[cursor:cursor + 2] not in ('00', '01'):
                return None
            cursor += 31 if bits[cursor:cursor + 2] == '00' else 2
        elif component in (4, 5):
            width = 11 if component == 4 else 29
            try:
                vitality_window(component, bits[cursor:cursor + width])
            except ValueError:
                return None
            cursor += width
        elif component == 21:
            if bits[cursor:cursor + 1] != '1' or bits[cursor + 24:cursor + 25] != '0':
                return None
            cursor += 25
        elif component == 25:
            if clock is None or bits[cursor:cursor + 10] != f'1{clock:08b}0':
                return None
            cursor += 10
        elif component in (30, 33):
            if bits[cursor:cursor + 2] == '11':
                amount, cursor = 0, cursor + 2
            elif len(bits) >= cursor + 10 and bits[cursor] == '0' and bits[cursor + 9] == '1':
                amount = int(bits[cursor + 1:cursor + 9], 2)
                if not 1 <= amount <= 15:
                    return None  # Only this scalar range is established by controls.
                cursor += 10
            else:
                return None
            fields.append(dict(kind='ammo', slot=(component - 30) // 3, value=amount, bit=start, end=cursor))
        elif component == 42:
            if bits[cursor:cursor + 7] != '0010011':
                return None  # Only this slot-1 selection form is established.
            fields.append(dict(kind='switch', slot=1, bit=cursor, end=cursor + 7))
            cursor += 7
        else:
            return None
    if bits[cursor:cursor + 16] != INPUT_END:
        return None
    return dict(components=ids, fields=fields, end=cursor)
