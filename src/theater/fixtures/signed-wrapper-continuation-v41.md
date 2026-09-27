# Native signed march continuation

Pinned LevelUp: 43a01721e8a02c87c955e175936f0ccf8dd97a81.
Ten cases cross calibrated/stub overrides and widths -84, -52, -20, 0 and 2^32.
The native march begins at bit 1. A -84 width returns to -32; -52 returns to zero
and exposes a following Delete before completion. The large positive width uses
a tiny input to test padded continuation, not a multi-gigabit recording.
Native source: reference/halo_rust_signed_wrapper_continuation_test.go.txt.
All contextual cases are compared, plus the represented static-width domain.
See reference/signed-wrappers-validation.json for checkpoint status.
