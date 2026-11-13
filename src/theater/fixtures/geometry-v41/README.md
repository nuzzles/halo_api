# Crate multiplayer-properties geometry oracle

The fixture pins libfilmshell `8f0f8d078e513c800baa21fac416fd4c0c127d03`.
`export_reference.py` verifies Git blob hashes for zones.rs, bits.rs and
calibrate.rs, compiles the unchanged upstream Bits, Bond reader and shape_at
sections, and records their results. Floating-point projections are stored as
exact f64 bits.

`wire_cases.py` authors 17 Compact Binary v2 component bodies independently of
halo_api output. They cover box/cylinder codes, recorded cylinder right extent,
missing dimensions/team, wrapped integers, lists, multiple/empty items, unknown
shape, duplicate root/dimension fields, inheritance, wrong root/nested lengths,
empty integer wrappers, multiple geometry set entries, and explicit absent properties. Rust tests use the native component reader at all
eight alignments, native packets with an explicitly configured synthetic class38
registry, every truncated box prefix, altered retained fields and corruption
trailers. The shared Bond test separately checks every shortened envelope with
valid-looking following bytes still physically present.

Reproduce from the repository root:

```sh
python3 src/theater/fixtures/geometry-v41/wire_cases.py > /tmp/geometry-cases.json
cmp src/theater/fixtures/geometry-v41/cases.json /tmp/geometry-cases.json
python3 src/theater/fixtures/geometry-v41/export_reference.py /path/to/pinned/libfilmshell > /tmp/geometry-reference.json
cmp src/theater/fixtures/geometry-v41/reference.json /tmp/geometry-reference.json
```

The reference is a comparison, not the canonical admission policy. Locally all
items and geometries survive; missing dimensions remain absent; unknown shape
codes and negative/zero dimensions survive. A cylinder's recorded right extent
is not replaced with zero. Team stays a source integer, not a current owner.
No default orientation, first-item choice or world-coordinate calibration is
added. Duplicate requested fields and unresolved inheritance refuse semantic
selection. Root/nested bounds and terminators are enforced by the existing Bond
reader; upstream's tolerated malformed lengths are not admitted.

The canonical TLV component reader still ends at its established terminator and
retains nested structures as opaque bytes. Geometry interprets only that exact
retained body, excluding corruption trailers, with packet-relative source ranges.
The inherited-root case therefore encounters a bounded partial Bond read; a
separate full-body Bond check establishes the inheritance-selection refusal.
These fixtures do not prove arbitrary build support or complete zone lifecycles.
