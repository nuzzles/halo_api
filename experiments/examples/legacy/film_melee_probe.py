"""Legacy field annotations used only by Theater Lab's byte inspector.

Full probe sources are in archive/cleanup-2026-09-10/examples-before-cleanup.zip.
The upstream Rust decoder handles film decoding and replay.
"""

from legacy.film_firing_probe import FIRING_GUARD

def melee_fields(bits, offset):
    event = bits[offset:offset + 94]
    companion = bits[offset - 31:offset + 1] if offset >= 31 else ''
    if (len(event) != 94 or event[1:10] != '101010001'
            or event[18:20] not in ('01', '10') or event[20:26] != '000010'
            or event[66:94] != FIRING_GUARD or len(companion) != 32
            or companion[1:10] != '101001101' or companion[10:20] != event[10:20]
            or companion[20:27] != '0011000'):
        return None
    wire, generation = int(event[10:18], 2), int(event[18:20], 2)
    return dict(serial=wire + 256 * (generation - 1), wire=wire, generation=generation,
                player=int(companion[27:32], 2), weapon_window=f'{int(event[26:66], 2):010x}')
