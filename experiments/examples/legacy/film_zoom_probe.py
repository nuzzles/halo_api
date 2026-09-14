"""Legacy field annotations used only by Theater Lab's byte inspector.

Full probe sources are in archive/cleanup-2026-09-10/examples-before-cleanup.zip.
The upstream Rust decoder handles film decoding and replay.
"""

from legacy.build_oddball_scene import CLOCKS

from legacy.film_weapon_probe import AUXILIARY_PREFIX

def zoom_header(bits, offset):
    """Candidate fields only; a following boundary is required for acceptance."""
    event = bits[offset:offset + 25]
    if (len(event) != 25 or event[1:11] != '1001010110'
            or event[19:21] not in ('01', '10') or event[21:23] != '00'
            or event[23:25] not in ('00', '01', '10')):
        return None
    wire, generation = int(event[11:19], 2), int(event[19:21], 2)
    return dict(wire=wire, generation=generation,
                serial=wire + 256 * (generation - 1), level=int(event[23:25], 2))


def zoom_fields(bits, offset):
    """25-bit window; its final stage bit overlaps the following leading bit."""
    event = zoom_header(bits, offset)
    if event is None:
        return None
    cursor, following = offset + 24, []
    # At most two additional consecutive scope records are established here.
    for _ in range(2):
        if zoom_header(bits, cursor) is None:
            break
        following.append(cursor)
        cursor += 24
    auxiliary = bits[cursor + 1:cursor + 32] == AUXILIARY_PREFIX
    auxiliary_bit = cursor if auxiliary else None
    if auxiliary:
        cursor += 64
    clock = bits[cursor:cursor + 37]
    if (len(clock) != 37 or clock[:3] not in ('001', '101', '111')
            or clock[3:29] not in {c[3:] for c in CLOCKS}):
        return None
    return dict(**event, clock_bit=cursor, auxiliary_bit=auxiliary_bit,
                following_zoom_bits=following)
