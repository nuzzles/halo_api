#!/usr/bin/env python3
"""Summarize pinned native record expectations without treating parity as completeness."""
import argparse
import collections
import hashlib
import json
from pathlib import Path
import re
import zlib


def rows(path):
    decoder = zlib.decompressobj()
    pending = b""
    with path.open("rb") as stream:
        while block := stream.read(1 << 20):
            pending += decoder.decompress(block)
            lines = pending.split(b"\n")
            pending = lines.pop()
            for line in lines:
                if line:
                    yield json.loads(line)
    pending += decoder.flush()
    if pending.strip():
        yield json.loads(pending)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--rust-log", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    theater = Path(__file__).resolve().parents[1]
    fixture = theater / "fixtures/keyframe-anchor-bodies-v41.jsonl.zlib"
    dispatch = set(json.loads((theater / "reference/dispatch-names-v75.json").read_text()))
    files, films = set(), set()
    stops = collections.Counter()
    complete = total = 0
    for row in rows(fixture):
        total += 1
        files.add(row["file"])
        films.add(str(Path(row["file"]).parent))
        trace = row["trace"]
        if trace["DesyncAt"] < 0:
            complete += 1
        else:
            component = next((c for c in trace.get("Comps") or [] if c["Index"] == trace["DesyncAt"]), None)
            stops[component["Name"] if component else "<no component>"] += 1
    rust_sources = [theater / "components.rs", *sorted((theater / "components").glob("*.rs"))]
    rust_names = set()
    for path in rust_sources:
        if "test" not in path.name:
            rust_names.update(re.findall(r'"([^"\n]+)"', path.read_text()))
    current = None
    if args.rust_log:
        text = args.rust_log.read_text()
        reports = re.findall(r'^NATIVE_KEYFRAME_AUDIT (.+)$', text, re.M)
        if reports and "test result: ok." in text:
            current = json.loads(reports[-1])
            if current["candidate_records"] != total or current["complete"] != complete or current["stopped"] != stops:
                raise SystemExit("current Rust coverage differs from pinned expectations")
    report = {
        "reference_commit": "43a01721e8a02c87c955e175936f0ccf8dd97a81",
        "fixture": fixture.name,
        "fixture_sha256": hashlib.sha256(fixture.read_bytes()).hexdigest(),
        "configuration": "ContexteParDefaut with recorded corruption flag; no calibrated width overrides",
        "reference": {"films": len(films), "keyframe_source_files": len(files), "candidate_records": total,
                      "complete_candidate_reads": complete, "stopped_candidate_reads": total-complete,
                      "stops": [{"component": name, "count": count, "named_in_reference_dispatch": name in dispatch} for name, count in stops.most_common()]},
        "dispatch_name_inventory": {"reference_names": len(dispatch), "names_absent_from_rust_reader_sources": sorted(dispatch-rust_names),
                                    "limitation": "Textual presence only; runtime component oracle is separate."},
        "current_rust_corpus_check": current,
        "full_native_parsing_complete": False,
        "limits": ["Recovered anchors are candidates, not an independently proven partition of all records.",
                   "A matching stop proves reference parity at that attempt, not successful decoding beyond it.",
                   "A zero-width calibrated skip must not be reported as decoded component fields.",
                   "This audit covers keyframe candidates only, not every delta, packet kind, or bootstrap/summary byte."],
    }
    args.output.write_text(json.dumps(report, indent=2)+"\n")
    print(json.dumps({"records": total, "complete": complete, "stopped": total-complete, "current_rust_checked": current is not None}))


if __name__ == "__main__":
    main()
