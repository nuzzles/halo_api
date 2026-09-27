#!/usr/bin/env python3
"""Compare pinned Go geometry arithmetic with and without compiler optimization.

This audits derived NaN behavior without changing Rust fixtures. The temporary
native harness edit is restored even when a subprocess or comparison fails.
"""
import argparse
import json
import os
from pathlib import Path
import subprocess
import tempfile
import zlib


def is_nan(bits):
    return bits & 0x7FF0000000000000 == 0x7FF0000000000000 and bits & 0xFFFFFFFFFFFFF != 0


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("checkout", type=Path)
    parser.add_argument("--go", default="go")
    parser.add_argument("--report", type=Path, required=True)
    args = parser.parse_args()
    reference = Path(__file__).resolve().parent
    native = args.checkout.resolve() / "apps/go-api/internal/games/halo_infinite/film"
    subprocess.run([os.environ.get("PYTHON", "python3"), str(reference / "verify_reference.py"), str(native)], check=True)
    harness = native / "replay/halo_rust_geometry_distance_test.go"
    previous = harness.read_bytes() if harness.exists() else None
    source = (reference / "halo_rust_geometry_distance_test.go.txt").read_text()
    package = "levelup/go-api/internal/games/halo_infinite/film/replay"
    versions = {}
    try:
        with tempfile.TemporaryDirectory(prefix="halo-geometry-optimization-") as tmp:
            for variant, flags in (("optimized", []), ("unoptimized", [f"-gcflags={package}=-N -l"])):
                output = Path(tmp) / (variant + ".json")
                harness.write_text(source.replace("/private/tmp/halo-geometry-distance.json", str(output)))
                subprocess.run([args.go, "test", "./internal/games/halo_infinite/film/replay", *flags,
                                "-run", "^TestHaloRustGeometryDistance$", "-count=1", "-timeout=60s"],
                               cwd=args.checkout / "apps/go-api", check=True)
                versions[variant] = json.loads(output.read_bytes())
    finally:
        if previous is None:
            harness.unlink(missing_ok=True)
        else:
            harness.write_bytes(previous)
    expected = json.loads(zlib.decompress((reference.parent / "fixtures/geometry-distance-v41.json.zlib").read_bytes()))
    assert versions["optimized"] == expected, "optimized native result differs from retained oracle"
    assert len(versions["optimized"]) == len(versions["unoptimized"]) == 4661
    differences = []
    for i, (a, b) in enumerate(zip(versions["optimized"], versions["unoptimized"])):
        assert a["a"] == b["a"] and a["b"] == b["b"] and a["plan"] == b["plan"], i
        if a["dist3"] != b["dist3"]:
            assert is_nan(a["dist3"]) and is_nan(b["dist3"]), (i, a, b)
            differences.append({"index": i, "a_bits": a["a"], "b_bits": a["b"],
                                "optimized_dist3_bits": a["dist3"], "unoptimized_dist3_bits": b["dist3"]})
    report = {"reference_commit": "43a01721e8a02c87c955e175936f0ccf8dd97a81",
              "go_version": subprocess.check_output([args.go, "version"], text=True).strip(),
              "cases": 4661, "optimized_fixture_agrees": True,
              "planar_bits_identical": True, "non_nan_dist3_bits_identical": True,
              "nan_classification_identical": True, "derived_nan_bit_differences": differences,
              "scope": "Derived arithmetic NaN payloads are compiler-dependent. This does not relax preservation of recorded float bits or other native source data."}
    args.report.write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
