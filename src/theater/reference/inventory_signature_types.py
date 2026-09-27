#!/usr/bin/env python3
"""Emit additional named types exposed through the pinned facade signatures.

Usage: inventory_signature_types.py OUTPUT.json
Pass that inventory to inventory_public_types.go.txt to resolve declarations.
This is a declaration audit input, not a Rust coverage claim.
"""
import json
from pathlib import Path
import re
import sys

root = Path(__file__).resolve().parent
inventory = json.loads((root / "decfilm-export-inventory.json").read_text())
aliases = {e["target"] for e in inventory["exports"] if e["kind"] == "type"}
references = {}
for entry in inventory["exports"]:
    if entry["kind"] != "func":
        continue
    for target in re.findall(r"\b\w+\.\w+\b", entry["signature"]):
        if target not in aliases:
            references.setdefault(target, set()).add(entry["name"])
external = {key: references.pop(key) for key in ("context.Context", "time.Duration")}
result = {
    "reference_commit": inventory["reference_commit"],
    "scope": "Named signature types absent from the 50 facade type aliases; declarations only.",
    "external_contracts": {key: sorted(value) for key, value in external.items()},
    "exports": [
        {"kind": "type", "name": key, "target": key, "referenced_by": sorted(value)}
        for key, value in sorted(references.items())
    ],
}
Path(sys.argv[1]).write_text(json.dumps(result, indent=2) + "\n")
