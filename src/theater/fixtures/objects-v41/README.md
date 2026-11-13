# Snapshot-object reference cases

Reference commit: OpenSpartan/libfilmshell
`8f0f8d078e513c800baa21fac416fd4c0c127d03`.

`export_reference.py` verifies two Git blobs and executes unchanged source:

- `src/world.rs`: `6aac5301bee54d94cf150fe2ee8abd15e20dd526` (the complete file).
- `src/snapshot.rs`: `cd8971c6b1e53e593361fe29026324a98a861728`
  (`raw_position`, unchanged).

The harness supplies minimal container, entity/default and catalogue adapters.
Each chunk carries explicit authored object inputs and a tick/snapshot envelope.
The tag adapter returns unknown metadata, leaving catalogue-part exclusion to
separate local tests. These adapters isolate the original comparison and position
arithmetic; they are not native packet readers or proof of snapshot identity.

Nine scenarios cover singleton/open runs, first-anchor drift, absence/reappearance,
tag changes, flags and zero positions, all four kept types, namespace changes and
nonfinite positions. JSON retains input float32 bit patterns and output float64
coordinate bits. Original frame indices map to authored snapshot ordinals; local
runtime tests use explicit recorded packet timestamps instead of invented frames.
The original nonfinite scenario remains in the oracle, but its prior runtime
comparison is historical: the canonical component reader now refuses that NaN
before the two-bit footer, so the parsed film records an incomplete table at
that source rather than a completed stationary observation. The test checks
that refusal separately and continues to compare the other eight scenarios.
Results are compared as multisets: the original sort leaves equal-position/tag
ties unordered across its HashMap iteration. The exporter sorts those output
records deterministically without changing the reader or any result value.

Regenerate with a checkout of the pinned source and rustc:

```sh
python3 src/theater/fixtures/objects-v41/export_reference.py /path/to/libfilmshell > /tmp/objects-reference.json
cmp src/theater/fixtures/objects-v41/reference.json /tmp/objects-reference.json
```

Native local fixtures use the original creation-v41 bodies with only their object
tag word replaced, followed by authored raw-position components and all 8,191
physical keyframe slots. Synthetic registry/configuration admission is explicit.
The original source suites separately establish creation/component bit grammar
and truncation behavior; the stationary oracle establishes only the original
presentation algorithm on these input sequences.

Local safeguards additionally refuse duplicate identities, class changes, source
mismatches and incomplete table/record evidence. Missing source or snapshot data
breaks runs without implying absence. Unread framed tick-entity tails remain
attached warnings because they cannot hide another snapshot packet. Two matching
snapshot positions never prove continuous immobility or a physical lifetime.
See ../../docs/OBJECTS.md for API semantics and remaining scope.
