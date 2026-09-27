# Live signed variable-width reader

Reference: LevelUp `43a01721e8a02c87c955e175936f0ccf8dd97a81`.
`TestHaloRustSignedVariableReader` calls actual `Lecteur.ReadSignedVarWidth`
three times on each input, followed by `ReadBits(7)`. Its harness is retained
in `reference/halo_rust_context_reader_sequence_test.go.txt` and registered in
`reference/generate_oracles.py`.

The 3,784 sequences cover all four selectors, every starting bit alignment,
eleven boundary/mixed-bit values and every byte prefix of two encoded values
plus a trailing field. The second field uses another selector and complemented
value. The third read deliberately continues into the remaining source/padding.
The fixture contains 11,352 native signed results and endpoints, including 2,694
negative values and 8,624 endpoints past the supplied bytes, plus 3,784 following
primitive results/endpoints. Empty inputs and starts beyond source are included.

Rust's live `NativeFilmReader.read_signed_variable` exposes the existing padded
cursor codec: sign extension for widths 8/16, signed conversion at 32 and low
32-bit truncation at 64. The test compares each native result and cursor before
continuing, while preserving the reader's capture slot and profile. No callback
or world-state mutation is part of this primitive operation.

This closes the missing live codec entry point. It does not establish signed
negative-cursor continuation or address-overflow parity for the unsigned live
reader, nor complete parser/Film acceptance. Those remain separate contracts.
