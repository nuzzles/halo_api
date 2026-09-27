# Native keyframe inventory source traversal

Pinned reference: `43a01721e8a02c87c955e175936f0ccf8dd97a81`.
Harness: `../reference/halo_rust_inventory_packet_source_test.go.txt`, registered
after its seed generator `TestHaloRustKeyframeInventory` in `generate_oracles.py`.
The seed is the existing native inventory payload; source outputs are independently
computed by `ScanKeyframeInventory`, including all stats and fallback counts.

120 cases combine five packet shapes, six metadata layouts, empty/populated
family catalogs, and default/explicit grenade caps. Shapes include normal
keyframes, End before a keyframe, malformed trailing bytes, zero-size non-End
before a keyframe, and an unrelated preceding packet. Layouts include absent
Film, bootstrap only, chunk 1, a missing chunk 1, duplicates and a duplicate after
a valid prefix. The absent Film case passes nil to the native scanner; it does
not claim the native loader accepts empty input.

18 cases contain inventory records; 30 have no readable source prefix. Empty
catalogs return before source validation/fallback. Rust compares ordered complete
inventory records, exact ammo gauge bits, native packet ordinals, every scan
counter and fallback flags, including diagnostics on errors. The Result wrapper
still reports failures; the diagnostic API and Film retain the partial report.
A constructor regression covers the missing numbered prefix, retained default
cap and error, and JSON. Replay consumers suppress errored inventory data.

These are synthetic source-boundary cases, not a new captured gameplay recording.
