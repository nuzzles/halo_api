# Crate orientation evidence

`wire_cases.py` independently authors 23 class38/parameter2 component bodies:
raw vectors, config-mode default/nondefault up, config overriding either raw or
delta selectors, nominal absolute forms, every direction/roll-delta gate branch,
and invalid/degenerate raw vectors. Native runtime tests also reuse 29 existing
parameter2 component cases (including the instruction-backed disputed branches)
at all eight bit alignments. They validate every-bit truncation, original source
bits, corruption trailers, native packets, namespaces and schema refusal.

`export_reference.py` checks libfilmshell `8f0f8d078e513c800baa21fac416fd4c0c127d03`
blob hashes for zones.rs and bits.rs, then executes its unchanged Bits and angle_at
source on every authored body. `reference.json` retains even the discrepant raw
and config-overridden results. Equality is asserted only for the compatible
`raw=0, delta=0, config=1, default-up=1` form. That projection uses an unshifted
turn, not the engine's midpoint roll convention.

`export_engine_reference.py` separately checks the original LevelUp sources at
`413971bcc485be53a2fa8cf229e170539a5146f8`: orientation_frame.go and
components_managed_object.go under
apps/go-api/internal/games/halo_infinite/film/internal/grammar/. It executes
unchanged RollAngleFromRaw and dequantMidpoint. The fixture stores f32 bits;
tests compare the local explicitly f64 derived roll within 1e-6 radians. This is
source-backed projection evidence, not bit-exact executable reconstruction.
The native raw-vector tests use authored little-endian engine-memory vectors
and independently known cardinal headings; invalid/parallel/vertical vectors
remain source words but do not publish a heading.

Reproduce from the repository root:

```sh
python3 src/theater/fixtures/orientation-v41/wire_cases.py > /tmp/orientation-cases.json
cmp src/theater/fixtures/orientation-v41/cases.json /tmp/orientation-cases.json
python3 src/theater/fixtures/orientation-v41/export_reference.py /path/to/pinned/libfilmshell > /tmp/orientation-reference.json
cmp src/theater/fixtures/orientation-v41/reference.json /tmp/orientation-reference.json
python3 src/theater/fixtures/orientation-v41/export_engine_reference.py /path/to/pinned/grammar > /tmp/orientation-engine-reference.json
cmp src/theater/fixtures/orientation-v41/engine-reference.json /tmp/orientation-engine-reference.json
```

The engine exporter accepts GO_BIN and GOCACHE overrides; otherwise its build
cache stays in its temporary directory. No fixture reads halo_api output.

The raw branch of upstream angle_at reads immediately after the raw bit, skipping
the actual parameter2 config bit; it also treats the copied memory words as
MSB-packed float bits. Its early raw branch ignores config overrides. The local
view uses the existing instruction-backed canonical fields and never copies
those offsets. Raw reference-angle projection is explicitly unsupported, while
the engine-heading helper reads all six canonical little-endian vector words.
No absent component becomes a default heading, no first/later heading is selected,
and no geometry/lifecycle association is made. Nominal-mode, nondefault-up and
incremental heading reconstruction remain unresolved here.
