# Zero-bit position axes

Reference: LevelUp `413971bcc485be53a2fa8cf229e170539a5146f8`.
The exporter verifies and executes the unchanged width law in
`profile/loi_largeurs.go` (blob `623a55741a4578c57bacb3a7abf47fffe8d5737d`)
and indexed point reader `lireE524Sur` / `largeursDeLaLigne` in
`lecteur_position.go` (blob `6033b0366c28cf899883ab35681ba4eb1f5f8b16`).
Both cite the native position reader and width-table filler. This is separate
source evidence, not a libfilmshell-reader parity claim: its point helper uses a
positive first-axis width as an availability sentinel and can substitute other
widths for this case.

Eight authored boxes cover all-zero axes, mixed zero/nonzero axes, nonzero centers,
the immediate f32 neighbors of the one-bin threshold, and large/capped ranges.
Five precision levels (12, 15, 16, 22, 23) and two code patterns yield 80 cases.
The original code calculates widths and reads an authored indexed point with a
following `0xd3` marker. The harness supplies bounded primitive bit reads and an
observation counter; it does not replace the reader body or width arithmetic.
Reference JSON retains exact f32 bounds, widths, codes, source bytes, primitive
traces including R(0), and the point's end. High-precision cases validate the
width/helper contract, not the presence of a corresponding registered event site.

A positive range smaller than one quantization bin has one code, zero, and needs
zero bits. halo_api previously forced its width to one, which could consume a bit
from the following field. The canonical model now retains the named axis with
width 0, bit length 0 and empty bytes at its exact cursor position. It does not
manufacture a field when an earlier read fails or a region is unknown. Runtime
conversion requires the explicit field and supplied matching bounds; the result
is the bin center, which need not be coordinate zero.

Tests compare every original width and point trace. Native object/dynamic
components cover following fields and every-bit truncation; complete DELTA
records cover a following component and unknown-region refusal. Event helpers
cover all 80 traces and every-bit truncation. Native detonation lists retain
zero-width ActionPoint words and the following zoom event; the known detonation
suffix is structurally validated without consuming its public unnamed tail.
Weapon NEW/DELTA records cover unchecked and both check-trailer forms, source
fields, exact-event queries and Film/playhead immutability. Runtime bounds tests
reject missing/malformed fields and nonzero values for a zero-width axis.

No existing fixture is rewritten. Reproduce with Go 1.23+:

```sh
python3 src/theater/fixtures/zero-position-axes-v41/export_reference.py /path/to/pinned/grammar /path/to/go > /tmp/zero-reference.json
cmp /tmp/zero-reference.json src/theater/fixtures/zero-position-axes-v41/reference.json
```

The exporter works offline once the two verified sources are available.
