"""Legacy field annotations used only by Theater Lab's byte inspector.

Full probe sources are in archive/cleanup-2026-09-10/examples-before-cleanup.zip.
The upstream Rust decoder handles film decoding and replay.
"""

from legacy.build_oddball_scene import CLOCKS

from legacy.film_vitality_probe import INPUT_END

PROJECTILE_SPAWN = ('0000100100000000000110100110000001100000000001011000000010111000100000110001001000010110010010110011110011111'
                    '000000000000001000001110000000000110100000000010110000000000000000000000000000000000000000000100001110000000001111000')


PROJECTILE_DELTA = '100100000000000100'  # Checked identity 9216, generation 1, sparse mode.


PROJECTILE_END = '110000101100000000010000'  # Terminal event reference in both controls.


def grenade_fields(bits, offset, require_clock=True):
    event = bits[offset:offset + 32]
    if (len(event) != 32 or event[1:10] != '101001111'
            or event[18:20] not in ('01', '10') or event[20:27] != '0000010'):
        return None
    # As with the melee companion, the final roster bit overlaps the next
    # record's otherwise uninterpreted leading bit. These are checked windows.
    clock = bits[offset + 31:offset + 68]
    if require_clock and (len(clock) != 37 or clock[1:29] not in [c[1:] for c in CLOCKS]):
        return None
    wire, generation = int(event[10:18], 2), int(event[18:20], 2)
    return dict(wire=wire, generation=generation, serial=wire + 256 * (generation - 1),
                player=int(event[27:32], 2))


def projectile_fields(bits, offset):
    if bits[offset:offset + 18] != PROJECTILE_DELTA or len(bits) < offset + 21:
        return None
    count = int(bits[offset + 18:offset + 21], 2)
    ids = [int(bits[offset + 21 + i * 6:offset + 27 + i * 6], 2)
           for i in range(count) if len(bits) >= offset + 27 + i * 6]
    if ids not in ([0, 1, 2], [0, 1, 2, 5], [0, 1, 2, 20]):
        return None
    start = offset + 21 + 6 * count
    # Component 0: 3 flags + 15/15/17 coordinates + 2 suffix bits.
    # Components 1/2: 58 checked opaque bits together. The optional component
    # contributes 29 bits (5) or 9 bits (20); no velocity/rotation/tick decode.
    end = start + 110 + (29 if ids[-1] == 5 else 9 if ids[-1] == 20 else 0)
    if (bits[start:start + 3] != '000' or bits[start + 50:start + 52] != '00'
            or bits[end:end + 16] != INPUT_END):
        return None
    xyz = [int(bits[start + a:start + a + n], 2) for a, n in ((3, 15), (18, 15), (33, 17))]
    return dict(xyz=xyz, components=ids, coordinate_bit=start + 3, checked_end_bit=end)
