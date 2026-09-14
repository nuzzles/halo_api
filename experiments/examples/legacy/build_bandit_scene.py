"""Legacy field annotations used only by Theater Lab's byte inspector.

Full probe sources are in archive/cleanup-2026-09-10/examples-before-cleanup.zip.
The upstream Rust decoder handles film decoding and replay.
"""

CLOCK = "10100000011110110100001000000"


SPAWN_COORD_PREFIX = "110000010000110010111111000000000011000011111001101110001001000"


class Unsupported(ValueError):
    pass


def uint(bits, start, width):
    field = bits[start:start + width]
    if len(field) != width:
        raise Unsupported("truncated")
    return int(field, 2)


def delta_fields(bits, offset, clock, generation=1):
    """Read a sparse delta through component 25, returning its checked prefix.

    Returning an end offset does NOT imply the whole record was parsed if the
    component list contains indices above 25. Unknown earlier fields are skipped
    by rejecting the entire candidate, never by searching for its counter.
    The two-bit generation tag is 01 in Bandit; Oddball also checks 10 after
    wire-ID reuse. The following sparse-mode bits remain 00 in these captures.
    """
    body = offset + 14
    if bits[body:body + 4] != f"{generation:02b}00":
        return None
    count = uint(bits, body + 4, 3)
    ids = [uint(bits, body + 7 + 6 * i, 6) for i in range(count)]
    if not count or ids != sorted(set(ids)) or 25 not in ids:
        return None
    cursor = body + 7 + 6 * count
    position = aim = None
    for component in ids:
        if component == 0:
            if bits[cursor:cursor + 5] != "00000" or bits[cursor + 56:cursor + 58] != "00":
                raise Unsupported("position flags")
            position = [uint(bits, cursor + i, w) for i, w in ((5, 18), (23, 18), (41, 15))]
            cursor += 58
        elif component == 1:
            flag = bits[cursor:cursor + 2]
            if flag not in ("00", "01"):
                raise Unsupported("component 1 flags")
            cursor += 31 if flag == "00" else 2
        elif component == 4:
            # Combined components 4+5 occupy 40 bits in the checked captures.
            # 11+29 is a provisional split, not a semantic decode of either.
            if 5 not in ids:
                raise Unsupported("component 4 without 5")
            cursor += 11
        elif component == 5:
            if bits[cursor:cursor + 1] != "0":
                raise Unsupported("component 5 flags")
            cursor += 29
        elif component == 21:
            if bits[cursor:cursor + 1] != "1" or bits[cursor + 24:cursor + 25] != "0":
                raise Unsupported("aim flags")
            aim = [uint(bits, cursor + 1, 12), uint(bits, cursor + 13, 11)]
            cursor += 25
        elif component == 25:
            if bits[cursor:cursor + 10] != f"1{clock:08b}0":
                raise Unsupported("counter mismatch")
            return {"position": position, "aim": aim, "end": cursor + 10, "components": ids}
        else:
            raise Unsupported(f"component {component}")
    raise AssertionError("Missing counter")
