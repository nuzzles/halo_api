"""Legacy field annotations used only by Theater Lab's byte inspector.

Full probe sources are in archive/cleanup-2026-09-10/examples-before-cleanup.zip.
The upstream Rust decoder handles film decoding and replay.
"""

INPUT_END = "0001000000001101"


def vitality_window(component, field):
    """Decode observed windows; guards intentionally exclude unobserved forms."""
    if component == 4:
        if len(field) != 11 or field[0] != "1" or field[8:] not in ("000", "110", "100"):
            raise ValueError("Unsupported body vitality form")
        return {"body_raw": int(field[1:8], 2), "body_state": field[8:], "body_bits": field}
    if component == 5:
        if (len(field) != 29 or field[0] != "0" or field[8:16] != "00000000"
                or int(field[:8], 2) > 64 or int(field[16:25], 2) > 300
                or field[25:] not in ("0000", "0001", "1100")):
            raise ValueError("Unsupported shield vitality form")
        return {"shield_raw": int(field[:8], 2), "shield_delay_ticks": int(field[16:25], 2),
                "shield_state": field[25:], "shield_bits": field}
    raise ValueError("Not a vitality component")


def read_windows(bits, offset, ids, ranked=False):
    cursor = offset + 21 + 6 * len(ids)
    result = {}
    for component in ids:
        if component > 5:
            break
        if component == 0 and ranked:
            cursor += 58  # Already checked by delta_fields.
        elif component == 1 and ranked:
            cursor += 31 if bits[cursor:cursor + 2] == "00" else 2
        elif component in (4, 5):
            width = 11 if component == 4 else 29
            result.update(vitality_window(component, bits[cursor:cursor + width]))
            result["body_bit" if component == 4 else "shield_bit"] = cursor
            cursor += width
        else:
            raise ValueError("Unknown component before vitality")
    return result, cursor


def isolated_vitality(bits, offset):
    """Checked complete Octagon delta, including the following End/input guard."""
    body = offset + 14
    if bits[offset:offset + 7] != "1000100" or bits[body:body + 4] != "0100":
        return None
    if len(bits) < body + 7:
        return None
    count = int(bits[body + 4:body + 7], 2)
    if count not in (1, 2) or len(bits) < body + 7 + 6 * count:
        return None
    ids = [int(bits[body + 7 + i * 6:body + 13 + i * 6], 2) for i in range(count)]
    if ids not in ([5], [4, 5]):
        return None
    try:
        result, end = read_windows(bits, offset, ids)
    except ValueError:
        return None
    if bits[end:end + len(INPUT_END)] != INPUT_END:
        return None
    return {**result, "serial": int(bits[offset + 7:offset + 14], 2),
            "components": ids, "checked_end_bit": end, "boundary": "End + input"}
