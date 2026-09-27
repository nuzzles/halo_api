# Native signed component-to-view continuation

Pinned LevelUp: 43a01721e8a02c87c955e175936f0ccf8dd97a81.
16 complete frames cross class/generic views and calibrated/stub overrides with
widths -83, -19, 0 and 2^32. The -83 width rewinds a completed delta from component
bit 51 to -32. Native skips the next 32-bit prefix, reaches the End at zero, and
continues into the control view or subsequent generic views. The large-width case
uses a tiny buffer and tests padded continuation, not a multi-gigabit recording.
Native source: reference/halo_rust_signed_view_continuation_test.go.txt.
See reference/signed-views-validation.json for checkpoint status.
