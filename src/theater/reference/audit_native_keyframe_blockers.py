#!/usr/bin/env python3
"""Inventory actual pinned-reader stops, without guessing component widths.

Uses the independently generated Go fixtures, not Rust output. Candidate reads
are deliberately counted separately from sequential tables: they may overlap
and cannot measure canonical record coverage or establish continuation points.
"""

import argparse
import collections
import hashlib
import json
from pathlib import Path
import zlib


ROOT = Path(__file__).resolve().parent
FIXTURES = ROOT.parent / "fixtures"


def rows(path):
    """Stream the compressed JSONL oracle without retaining all traces."""
    decoder = zlib.decompressobj()
    pending = b""
    with path.open("rb") as source:
        while compressed := source.read(64 * 1024):
            pending += decoder.decompress(compressed)
            lines = pending.split(b"\n")
            pending = lines.pop()
            for line in lines:
                if line:
                    yield json.loads(line)
    pending += decoder.flush()
    if not decoder.eof or decoder.unused_data:
        raise ValueError(f"invalid compressed oracle: {path}")
    if pending.strip():
        yield json.loads(pending)


def sha256(path):
    with path.open("rb") as source:
        return hashlib.file_digest(source, "sha256").hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--additional-pin", action="store_true",
                        help="audit d61443e body reads at the original anchor offsets")
    args = parser.parse_args()
    table_path = FIXTURES / (
        "keyframe-table-corpus-d61443e-v41.json.zlib" if args.additional_pin
        else "keyframe-table-corpus-v41.json.zlib"
    )
    candidate_path = FIXTURES / (
        "keyframe-anchor-bodies-d61443e-v41.jsonl.zlib" if args.additional_pin
        else "keyframe-anchor-bodies-v41.jsonl.zlib"
    )
    tables = json.loads(zlib.decompress(table_path.read_bytes()))
    sequential_stops = collections.Counter()
    sequential_attempts = 0
    sequential_suffix_bits = 0
    for table in tables:
        records = table["records"]
        sequential_attempts += len(records)
        last = records[-1]
        stop = last["DesyncAt"]
        if stop < 0:
            raise ValueError("Sequential oracle changed; review its terminal state")
        component = next(c for c in last["Comps"] if c["Index"] == stop)
        sequential_stops[component["Name"]] += 1
        # This suffix is unwalked by the sequential grammar, not necessarily
        # unread by independent candidate readers elsewhere in the API.
        sequential_suffix_bits += max(0, table["size"] * 8 - last["BitEnd"])

    complete = 0
    stopped = collections.Counter()
    examples = {}
    for row in rows(candidate_path):
        trace = row["trace"]
        stop = trace["DesyncAt"]
        if stop < 0:
            complete += 1
            continue
        component = next(
            (c for c in trace["Comps"] if c["Index"] == stop), None
        )
        name = component["Name"] if component else "<no component trace>"
        reason = (
            "missing_reader"
            if component and not component["Ported"]
            else "reader_stopped"
        )
        if args.additional_pin and reason == "missing_reader":
            # Ported=false also represents a known conditional grammar refused
            # under this profile. It does not prove that no reader exists.
            reason = "reader_declined"
            if name in ("simulation-state", "simulation-state-component"):
                reason = "simulation_width_policy_refusal"
        key = (name, reason)
        stopped[key] += 1
        examples.setdefault(
            key,
            {
                "file": row["file"],
                "packet": row["packet"],
                "payload_offset_byte": row["offset"],
                "payload_size_byte": row["size"],
                "anchor": row["anchor"],
                "component_index": stop,
                "component_start_bit": component["StartBit"] if component else None,
                "reader_end_bit": trace["EndBit"],
            },
        )
    result = {
        "reference_commit": "43a01721e8a02c87c955e175936f0ccf8dd97a81",
        "provenance": {
            path.name: sha256(path) for path in (table_path, candidate_path)
        },
        "interpretation": (
            "Pinned Go reader outcomes, not full v41 grammar coverage. "
            "Candidate reads may overlap and are not canonical records. "
            "Complete means the reader returned without a desynchronization, "
            "not independently proven boundaries or fully interpreted fields."
        ),
        "sequential": {
            "tables": len(tables),
            "record_attempts": sequential_attempts,
            "terminal_components": dict(sorted(sequential_stops.items())),
            "unwalked_suffix_bits": sequential_suffix_bits,
        },
        "candidates": {
            "reads": complete + stopped.total(),
            "complete": complete,
            "stopped": stopped.total(),
            "stopping_components": [
                {
                    "name": name,
                    "reason": reason,
                    "attempts": count,
                    "first_example": examples[(name, reason)],
                }
                for (name, reason), count in sorted(
                    stopped.items(), key=lambda item: (-item[1], item[0])
                )
            ],
        },
    }
    if args.additional_pin:
        # Both outputs are recomputed by the additional pin. Only candidate
        # selection offsets retain their original reference provenance.
        result["reference_commit"] = "d61443ef59268ad734355db8e9974f68db5ca6d0"
        result["sequential"]["reference_commit"] = result["reference_commit"]
        result["candidates"]["reference_commit"] = (
            "d61443ef59268ad734355db8e9974f68db5ca6d0"
        )
        result["candidates"]["selection_reference_commit"] = (
            "43a01721e8a02c87c955e175936f0ccf8dd97a81"
        )
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
