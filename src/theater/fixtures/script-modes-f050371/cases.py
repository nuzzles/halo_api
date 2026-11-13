"""Author every presence combination of f050371's eleven mode filenames."""
import json
from pathlib import Path

ROOT = Path(__file__).parent


def varint(n):
    out = bytearray()
    while n >= 128:
        out.append((n & 127) | 128)
        n >>= 7
    return bytes(out) + bytes([n])


def structure(fields):
    body = b"".join(fields) + b"\0"
    return varint(len(body)) + body


def sequence(type_, values):
    count = len(values)
    head = (bytes([type_ | ((count + 1) << 5)]) if count < 7
            else bytes([type_]) + varint(count))
    return head + b"".join(values)


def variant(data):
    script_data = sequence(3, [bytes([value]) for value in data])
    nested = structure([bytes([11 | (2 << 5)]) + script_data])
    base = structure([bytes([10 | (3 << 5)]) + nested])
    return structure([bytes([11 | (1 << 5)]) + sequence(10, [base])])


NAMES = [
    "firefightevents.lua", "fbastionevents.lua", "arenaflagevents.lua",
    "oddballevents.lua", "bastionevents.lua", "parcel_stronghold.lua",
    "attritionevents.lua", "eevents.lua", "gungameevents.lua",
    "slayerevents.lua", "btbslayerevents.lua",
]
cases = []
for mask in range(1 << len(NAMES)):
    data = b"".join(
        b"tags\\" + name.encode() + b"\0"
        for i, name in enumerate(NAMES) if mask & (1 << i)
    )
    cases.append({"name": f"modes/{mask:03x}", "blob_hex": data.hex(),
                  "hex": variant(data).hex()})

(ROOT / "cases.json").write_text(json.dumps(cases, separators=(",", ":")) + "\n")
