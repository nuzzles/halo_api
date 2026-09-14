"""Legacy field annotations used only by Theater Lab's byte inspector.

Full probe sources are in archive/cleanup-2026-09-10/examples-before-cleanup.zip.
The upstream Rust decoder handles film decoding and replay.
"""

FIRING_GUARD = "0010110010010110011110011111"


def firing_fields(bits, offset):
    """Read a checked 108-bit prefix, not the entire event record."""
    field = bits[offset:offset + 108]
    if (len(field) != 108 or field[1:12] != "10100100110"
            or field[20:22] not in ("01", "10") or field[22:26] != "0000" or field[34] != "0"
            or field[80:108] != FIRING_GUARD):
        return None
    wire, generation = int(field[12:20], 2), int(field[20:22], 2)
    return {"serial": wire + 256 * (generation - 1), "wire": wire, "generation": generation,
            "player": int(field[35:40], 2),
            # Oddball establishes modulo-256 wrap; bit 34 remains a checked zero.
            "sequence": int(field[26:33], 2) + 128 * int(field[33]),
            "weapon_window": f"{int(field[40:80], 2):010x}"}
