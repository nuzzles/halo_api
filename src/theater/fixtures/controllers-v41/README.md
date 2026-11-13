# Unit-controller values and event-time attribution

Reference: libfilmshell `8f0f8d078e513c800baa21fac416fd4c0c127d03`,
`components.rs` blob `6acd5fbb992f7d65ff8fdbd88a6b7b739a88a1a4`.
The exporter archives that exact revision, checks the blob, and appends a test
that calls the unchanged original `Reader::block(Component::UnitControl)` at
row parameters 1 and 2, requiring equal results before exporting each case. It compares the optional controller and final bit position. Only
the disposable harness lockfile is regenerated against cached offline dependencies;
no original reader or original test body is modified. The target directory is
`/private/tmp/halo-controller-oracle-target` and is separate from halo_api.

130 independently authored bodies cover no controller, every five-bit player ID,
absent/present secondary six-bit values and absent/present 32-bit references.
Those latter fields retain raw values locally; the native projection discards
their meanings. The local test runs every body at all eight bit alignments and
checks every source field against the payload. The longest body is truncated
at every bit, and malformed fields, unexpected suffixes and both corruption-check
trailers are tested independently.

Reproduce from this directory:

```sh
python3 cases.py
python3 export_reference.py /path/to/libfilmshell-8f0f8d0 > /tmp/controller-reference.json
cmp reference.json /tmp/controller-reference.json
```

Runtime tests use actual parsed keyframe/DELTA bodies to check changing and
cleared controllers, repetitions, partial reads, exact event-time holder
resolution, missing populations and separate runtime/keyframe ID domains.
Additional explicitly mutated record models test lifecycle/ambiguity policies;
those are not wire-format fixtures. A controller clear or deletion is never
promoted to an inferred death. A controller value is not proof of a continuous
life owner. Upstream's first-controller/first-end aggregation is not copied.

Captured comparison uses the existing scoreboard/engine/HUD corpus test:

```sh
HALO_CONTROLLER_EXPORT=/tmp/controllers cargo test --offline --config build.incremental=false captured_scoreboard_values_keep_canonical_sources_and_expose_gaps -- --ignored --nocapture --test-threads=1
python3 export_reference.py /path/to/libfilmshell-8f0f8d0 /tmp/controllers/octagon/03-first-to-50.json > /tmp/octagon-native.json
python3 verify_captured.py /tmp/controllers/octagon/03-first-to-50.json /tmp/octagon-native.json
```

Repeat for `ranked-arena/02-oddball.json`. Captured comparisons cover complete
retained controller observations; missed packet regions, unbound schema and
continuous-lifetime ownership remain separate evidence requirements. Canonical
wire readers and prior fixture pins are unchanged.

## Controller-holder continuity

`runtime/controllers/ownership/tests.rs` combines these exact controller bodies
with the independently authored assignment packets. The state-machine tests
explicitly distinguish source-model mutations from wire evidence. They cover
current roster reassignment, source gaps, lifecycle changes and recorded clears;
no native first-controller/first-end policy is treated as a canonical lifetime.

The retained two-film test also checks three selected ownership histories (the
busiest in each identity domain, then the busiest remaining entity). Set
`HALO_OWNERSHIP_EXPORT` to retain compact controller/listing-source attribution
rows. These are local consistency checks; the original native comparison above
remains the independent oracle for direct controller values. See CONTROLLERS.md
and VALIDATION.md for continuity limits and exact checkpoint results.
