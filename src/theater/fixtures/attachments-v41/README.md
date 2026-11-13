# Parent-state source fixtures

Reference: libfilmshell `8f0f8d078e513c800baa21fac416fd4c0c127d03`, components.rs
blob `6acd5fbb992f7d65ff8fdbd88a6b7b739a88a1a4`. The exporter archives that exact
revision and appends a test calling the unchanged ParentState block. The
original reader is not instrumented or patched. Only the disposable harness
lockfile is regenerated against cached offline dependencies. Build output uses
`/private/tmp/halo-attachment-oracle-target` separately from halo_api.

150 independently authored bodies cover parameters 0–4, archetypes 35/40/47,
old detached free-reference branches, unit/object handles, all four generation
values, boundary indices/node values, optional secondary nodes, local codes,
orientation gates and seat values. The native output preserves parent class and
index, generation, node, seat and endpoint. Rust verifies every body at all eight
alignments, every-bit truncation of the longest body, and malformed-field refusal.

```sh
python3 cases.py
python3 export_reference.py /path/to/libfilmshell-8f0f8d0 > /tmp/parent-reference.json
cmp reference.json /tmp/parent-reference.json
```

Actual Film fixtures exercise a ten-node chain, cycle, detach, missing target,
exact-event queries, runtime/keyframe separation, unread suffix/chunk boundaries,
source discontinuities and object-handle mapping refusal. Explicit model mutations
separately exercise deletes, missing NEW baselines, generation/reuse boundaries,
schema/duplicate/partial refusals and unbound DELTAs. These policies do not import
upstream's four-level root cutoff, frame-minus-one closure or film-end ride span.

Captured fields are exported by the existing scoreboard/engine/HUD corpus test:

```sh
HALO_ATTACHMENT_EXPORT=/tmp/attachments cargo test --offline --config build.incremental=false captured_scoreboard_values_keep_canonical_sources_and_expose_gaps -- --ignored --nocapture --test-threads=1
python3 export_reference.py /path/to/libfilmshell-8f0f8d0 /tmp/attachments/octagon/03-first-to-50.json > /tmp/octagon-parent-native.json
python3 verify_captured.py /tmp/attachments/octagon/03-first-to-50.json /tmp/octagon-parent-native.json
```

Repeat for ranked-arena/02-oddball.json. Separate `*.refusals.json` files retain
unbound DELTA fields and source positions. These are not fed to an original
reader with a guessed archetype. Their raw fields still match source payloads.
The captured test also samples root queries, but authored chain/lifecycle tests
supply the comprehensive root-policy evidence. See ATTACHMENTS.md for identity,
continuity, object-handle and ride/pose limitations.
