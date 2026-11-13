# Secondary control corrections at f050371

`wire_cases.py` independently authors 336 complete control views: 48 short
secondary blocks and 288 full blocks. Every view carries a following kind-2
record with value `0x15ab` and an explicit list terminator. Full blocks cover both
category-1 handle forms and absence, all four generations, object-list counts
0/1/3/7, attached/default/world/region-3 positions, optional target presence,
direction/magnitude gates, and absent/present-zero baselines.

`reference.json` executes the original OpenSpartan/libfilmshell `control_view`
at commit `f050371a6cda83b9887eeb83a1884336fce28e53`. `export_reference.py`
archives that exact revision into a disposable directory and instruments only
the primitive read/skip methods. It does not import halo_api, change upstream
branches or infer endpoints. Each authored endpoint must equal the original
reader's endpoint before export succeeds. External position widths are explicit:
world `[15,15,17]`, region 3 `[19,19,16]`, index width one/two respectively.

The corrected full secondary grammar (`unit_block`, annotated upstream with
game reader `141fdae44`) uses category 1 for the second optional reference;
its local-position flag selects exactly three 14-bit fields instead of a point;
and its present target carries both the category-0 handle and the 11-bit word.
The 11-bit word remains raw. These changes do not establish new actor roles,
coordinate transforms or gameplay effects.

Rust checks compare primitive boundaries and raw scalars at every bit alignment,
truncate every active control fixture at every bit, and validate following-record
boundaries, serialization, explicit unknown reference domains, caller position
widths, missing/unknown indexed context, and public Film/runtime source order.
Attached and default-box positions need no map context. Present handles still
require configuration-one reference domains; a profile cannot supply them.

The old `controls-v41` 17-case fixture remains byte-for-byte unchanged. Its three
`secondary-full-*` cases were authored against the old conflicting branches;
their original primitive traces remain historical evidence. The other 14 cases
still validate current behavior. The original exporter reproduces the original
oracle exactly; it is not retargeted to the new source revision.

Reproduce:

```sh
python3 src/theater/fixtures/controls-f050371-v41/wire_cases.py
python3 src/theater/fixtures/controls-f050371-v41/export_reference.py /path/to/libfilmshell > /tmp/controls-f050371-reference.json
cmp /tmp/controls-f050371-reference.json src/theater/fixtures/controls-f050371-v41/reference.json
```

The exporter resolves cached dependencies only inside its disposable archive and
disables incremental compilation. Existing source trees and lockfiles are not
modified. These fixtures
prove agreement with the pinned reader, not that every branch occurs in captures.

## Captured differential evidence

`captured-cases.json` retains 72 exact control-start slices from the 32-film
corpus. All selected views occur in `raids/01-hour-long-raid`, with recorded
configuration one and registry-derived world boxes `[15,15,17]` and `[18,18,18]`.
Selection requires an observed full-secondary state bit, including partial views.
Names retain manifest, chunk source position, packet index and original start bit.
The recorded bit length excludes the padding introduced by copying an unaligned
source slice. Rust shifts each slice so that this logical end is byte-aligned,
preserving the original physical truncation boundary.

`captured-reference.json` executes f050371; `captured-original-reference.json`
executes the original `18067987ebc3628ad499c819c6aee814692df43b` reader. The exporter
instruments primitive reads/skips only and supplies the recorded box widths.
All **47 complete views** end exactly at the corrected reader's endpoint. The
old reader agrees on only one of those views; **46 endpoints differ**. Complete
records across the selected views contain **70 full secondary bodies**, including
**28 local positions** and **27 present targets**. The remaining **19 kind-3
refusals and six truncated views** stay partial. Upstream reads the kind-3
baseline before refusing, whereas our parser stops before choosing that layout.
The external reader receives the padded bytes; its partial-view endpoints are
diagnostic evidence, not authority to extend the original logical source.

The normal captured regression verifies the stored observations, current oracle
boundaries within the consumed prefix, exact source fields and complete-body
branch counts. The captured canonical observations alone are not an independent
oracle. The two external reader traces supply the independent comparison.
This establishes endpoint agreement for these excerpts; it does not attribute
each of the 17 fewer complete records in the aggregate corpus to a specific
previous local decode.

Reproduce the extraction and original-reader comparisons:

```sh
HALO_CONTROL_CAPTURE_EXPORT=/tmp/control-captures.json cargo test --offline --config build.incremental=false --all-features captured_full_secondary_controls_for_upstream_comparison -- --ignored --nocapture
python3 src/theater/fixtures/controls-f050371-v41/export_captured.py /path/to/libfilmshell /tmp/control-captures.json f050371a6cda83b9887eeb83a1884336fce28e53 > /tmp/captured-reference.json
python3 src/theater/fixtures/controls-f050371-v41/export_captured.py /path/to/libfilmshell /tmp/control-captures.json 18067987ebc3628ad499c819c6aee814692df43b > /tmp/captured-original-reference.json
```

The corpus defaults to `experiments/films`; `HALO_FILM_CORPUS` may select another
copy. Both upstream revisions must exist in the supplied checkout's object store.
The old revision receives a serialization-only dev dependency in the disposable
archive; no production grammar is changed.
