"""Author every six-bit managed-player participant at every bit alignment."""
import json
from pathlib import Path


def word(width, value):
    return format(value, f"0{width}b")


rows = []
for versioned in (False, True):
    for participant in range(64):
        body = ("1" + word(8, 0xE7) if versioned else "0")
        body += word(6, participant) + word(6, 0x2D) + "1"
        for shift in range(8):
            bits = "0" * shift + body + word(8, 0xD6)
            bits += "0" * (-len(bits) % 8)
            rows.append(
                dict(
                    name=f"{'versioned' if versioned else 'default'}/{participant}/{shift}",
                    hex=int(bits, 2).to_bytes(len(bits) // 8, "big").hex(),
                    at=shift,
                    bit_len=len(body),
                    participant=participant,
                )
            )
assert len(rows) == 1024
Path(__file__).with_name("cases.json").write_text(json.dumps(rows, indent=2) + "\n")
