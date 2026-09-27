# Signed native component width overrides

Pinned source: LevelUp feat/v75 `43a01721e8a02c87c955e175936f0ccf8dd97a81`.
Harness: `../reference/halo_rust_signed_width_test.go.txt`, registered in the
oracle generator. Go execution succeeded before fixture compression.

24 cases apply widths -26, -25, -8, -1, 0, 1, 7 and 64 to a calibration override,
an unknown-component stub, and a stub on a supported EMP component. Native Skip
adds a signed int, so negative widths rewind. Supported component readers ignore
stub overrides, including invalid ones. Calibration bypasses the reader.

For 22 nonnegative final native offsets, Rust compares record ends and completion.
Six actual rewinds retain NativeWidthAdjustment (component, override kind, source
start, original signed width and target). The two -26 cases move from bit 25 to
-1 natively. Rust source offsets are usize, so these are explicitly refused as
InvalidWidthOverride with end_bit=None, retaining the original width and start;
they are not mislabeled as truncation. This is an explicit compatibility limit,
not native negative-cursor parity. Diagnostics survive merging and JSON roundtrip.

Huge positive skips, native signed-integer overflow, negative cursors and
platform-dependent metadata widths are not established by these small-width
cases. They remain separate work. This is harness metadata behavior rather than
an independently observed film action or semantic golden-film fixture.

A rewound component can have an end offset before its start. This is cursor
movement caused by an explicit override, not a forward payload range; consumers
must use the signed adjustment rather than subtracting unsigned offsets.
