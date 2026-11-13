"""Control-list boundary cases authored independently of halo_api readers."""
import json
from pathlib import Path


def main():
    cases = []
    for count in [0, 1, 63, 64, 65, 128]:
        for form in ["kind0", "kind1", "kind2", "mixed"]:
            kinds = [(i % 3 if form == "mixed" else int(form[-1])) for i in range(count)]
            bits = ""
            for i, kind in enumerate(kinds):
                bits += "1" + f"{kind:02b}" + "0"  # presence, kind, no baseline
                if kind == 2:
                    bits += "0"  # no kind-2 value
                else:
                    bits += f"{i % 32:05b}" + "0"  # index, absent input/commands
                    if kind == 0:
                        bits += "0"  # no secondary block
            bits += "0"  # terminating continuation bit
            padded = bits.ljust((len(bits) + 7) // 8 * 8, "0")
            cases.append(dict(
                name=f"{form}/{count}", hex=int(padded, 2).to_bytes(len(padded) // 8, "big").hex(),
                bit_len=len(bits), kinds=kinds, command_counts=[None for k in kinds if k == 1],
                secondary=[False for k in kinds if k == 0], index_bits=1, region_widths={},
            ))
    Path(__file__).with_name("cases.json").write_text(json.dumps(cases, indent=2) + "\n")


if __name__ == "__main__":
    main()
