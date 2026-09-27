# Large signed mobility-extra widths

Pinned reference: `43a01721e8a02c87c955e175936f0ccf8dd97a81`.
Harness: `reference/halo_rust_mobility_extra_large_test.go.txt`, registered in
`generate_oracles.py`. Actual native generation passed (0.393s).

208 cases vary full-body policy, active/flag bits and component names with widths
through i64::MAX, including i64::MIN, 4096/4097, 2^31, 2^40 and cursor boundaries.
Source includes nonzero 0xdeadbeef bits. Native produces 156 component callbacks.
Four active skips wrap the signed native cursor negative; Rust refuses these
with NativeWidthPurpose::MobilityExtra, retains prior callbacks and the pre-skip
cursor, and does not claim negative-cursor execution parity.

ComponentBodyPolicy now retains signed i64 extra widths. Policy is evaluated at
the mobility branch, so inactive, unrelated and full-body reads do not reject an
unused large value. Nonpositive values remain ignored. Positive nonnegative cursor
outcomes match native on a 64-bit host. On narrower hosts, unrepresentable source
offsets remain explicit refusals.

For skips over 4096 bits that extend beyond source, scalar fields retain every
available source bit and one width adjustment describes the synthetic zero tail.
Ordinary skip field layouts are unchanged. Tests assert callback and cursor parity,
raw profile preservation, source field bits, bounded field count, and decoded
component JSON roundtrip. The diagnostic purpose distinguishes this setting from
calibrated/stub replacement; old diagnostic JSON defaults that field to None.
