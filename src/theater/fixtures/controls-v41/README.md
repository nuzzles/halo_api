# Control-view wire fixtures

Historical update: the three `secondary-full-*` bodies below encode branches
corrected at f050371. Their source and original traces remain unchanged, but
current full-body expectations come from `../controls-f050371-v41/`. The other
14 cases still exercise the current reader. The original exporter continues to
reproduce this fixture's original oracle exactly.

`cases.json` contains 17 synthetic v41 control views authored as explicit wire
layouts in `wire_cases.py`. Nonzero values exercise optional fields, raw float
bit patterns (including a NaN payload and negative zero), generation zero,
baseline zero, list absence versus zero count, and the automatic relative
layout for subsequent commands in one list. Secondary blocks include both
handle categories, default/world/region-3 positions, direction/magnitude
branches, and object versus non-object targets. Mixed kinds verify wire order.

`reference.json` was exported from OpenSpartan/libfilmshell commit
`18067987ebc3628ad499c819c6aee814692df43b`. `export_reference.py` obtains that exact
commit with `git archive`, instruments only the bit-reader read/skip operations
in a disposable copy, and invokes its original `control_view`. It never loads
halo_api. Every independently authored view must end at exactly the upstream
endpoint before the export succeeds. Reader parameters are explicit: axis
widths `[15,15,17]`, with a separate region-3 case using `[19,19,16]` and a
two-bit region index.

The Rust tests compare each upstream primitive boundary against retained source
fields and verify complete, contiguous bit preservation. The canonical fields
split several upstream aggregate skips into smaller fields. Tests additionally
truncate every view at every bit, aligning the retained end to a byte boundary
so no artificial zero suffix is readable. Closed gates, missing position
context, and the disputed kind-3 prefix have separate assertions. A contextual
case also verifies the caller's profile widths through the production adapter.
Partial records do not enter the complete-record list; their read fields remain visible.
A valid mixed control view appended after a rejected entity header must remain
unparsed: only an entity terminator establishes the canonical control boundary.

This evidence verifies the port against the pinned wire readers on synthetic
branches. It does not establish gameplay meanings or real-film occurrence of
the new branches. Kind 3 remains explicitly unsupported because the references
disagree even about whether a baseline follows it. No kind-3 baseline or body
is published. Kind 2's 13-bit value has no assigned player identity or inferred
meaning.

Regenerate authored inputs only when intentionally extending the fixture:

```sh
python3 src/theater/fixtures/controls-v41/wire_cases.py
python3 src/theater/fixtures/controls-v41/export_reference.py /path/to/libfilmshell > /tmp/control-reference.json
```

Review the new oracle output before replacing `reference.json`. The exporter
requires cached Cargo dependencies for offline execution and uses the installed
stable Rust toolchain. The fixture pins its own upstream repository and commit;
existing LevelUp golden files are unchanged.
