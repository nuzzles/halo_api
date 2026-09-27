"""Legacy field annotations used only by Theater Lab's byte inspector.

Full probe sources are in archive/cleanup-2026-09-10/examples-before-cleanup.zip.
The upstream Rust decoder handles film decoding and replay.
"""

from legacy.build_bandit_scene import SPAWN_COORD_PREFIX, uint

from legacy.build_octagon_scene import COORD_SUFFIX, SPAWN_BODY

CLOCKS = (
    "10100000011110110100001000000",
    "10100000011111000100001000000",
    "10100000011111010100001000000",
)


def spawn_fields(bits, offset):
    if offset < 0 or bits[offset + 20:offset + 69] != SPAWN_BODY[2:]:
        return None
    wire = uint(bits, offset + 1, 17) - 8704
    generation = uint(bits, offset + 18, 2)
    player = uint(bits, offset + 69, 5)
    if not 0 <= wire <= 255 or generation not in (1, 2) or player not in range(8):
        raise ValueError("Unsupported spawn identity")
    shifts = [s for s in (0, 32)
              if bits[offset + 192 + s:offset + 255 + s] == SPAWN_COORD_PREFIX
              and bits[offset + 314 + s:offset + 344 + s] == COORD_SUFFIX]
    if len(shifts) != 1 or shifts[0] != (32 if player == 6 else 0):
        raise ValueError("Unsupported spawn coordinate layout")
    shift = shifts[0]
    return {"wire": wire, "generation": generation, "player": player, "shift": shift,
            "xyz": [uint(bits, offset + i + shift, w)
                    for i, w in ((263, 18), (281, 18), (299, 15))]}
