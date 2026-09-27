# Ignored mobility-extra metadata

Pinned reference: `43a01721e8a02c87c955e175936f0ccf8dd97a81`.
Harness: `reference/halo_rust_mobility_extra_policy_test.go.txt`; registered in
`generate_oracles.py`. Native generation passed (0.479s).

The 128 actual-native cases vary negative widths through i64::MIN, small positive
widths, and large positive widths with the full mobility body enabled. They read
mobility and simulation components with all four initial active/flag combinations.
There are 96 native component callbacks (64 movement, 32 reference).

Native uses extra bits only when the full body is disabled and the value is
positive. Rust's profile adapter previously converted the signed value to i32
before consulting that policy, refusing ignored values outside i32 range.
The subsequent large-width follow-up retains signed i64 metadata in both the
profile and component encoding and applies native policy at the read branch.
See mobility-extra-large-v41.md for that implementation and its wider oracle.

The Rust regression compares dispatch, cursor, live callbacks, retained raw
profile and decoded-component JSON roundtrip. Active positive widths outside i32 are now covered separately by the large-width
fixture. Oversized unused axis metadata and negative native cursor execution
remain separate gaps. This fixture does not claim those cases.
