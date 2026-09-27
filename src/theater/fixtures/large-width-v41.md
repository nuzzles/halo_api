# Large live component overrides

Pinned reference: LevelUp feat/v75 `43a01721e8a02c87c955e175936f0ccf8dd97a81`.
Harness: `../reference/halo_rust_large_width_test.go.txt`, registered in the oracle
generator. Go execution succeeded before fixture compression.

18 cases cover calibration, unknown-component stubs, and ignored stubs on a
supported EMP component. Widths are 4096, 4097, 2^20, 2^40, i64::MAX-25 and
that value plus one. A nonzero 0xdeadbeef payload tests source-bit preservation.
On a 64-bit host, 16 native final offsets are nonnegative and match Rust; two
native signed-addition overflows produce negative offsets and are explicitly
refused by Rust. Smaller usize targets also refuse offsets they cannot represent.

Live positive overrides larger than 4096 bits that exceed available source retain
all source bits as scalar fields and describe the synthetic zero tail through
NativeWidthAdjustment.retained_bits and its end offset. This bounds work by
available source, not skipped padding. The test checks actual scalar bits and a
small field-count bound for terabit and near-i64::MAX jumps. Ordinary overrides
keep their existing scalar representation. No source information is discarded.

This verifies single-record override endpoints. It does not establish native
negative/wrapped-cursor continuation, arbitrary primitive reads near integer
limits, legacy static usize override compaction, or oversized/unused profile
metadata. These remain explicit scope limits rather than full parser parity.
