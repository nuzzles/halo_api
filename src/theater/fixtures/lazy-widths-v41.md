# Lazy native direct-reader width conversion

Reference: 43a01721e8a02c87c955e175936f0ccf8dd97a81.
`halo_rust_lazy_widths_test.go.txt::TestHaloRustLazyWidths` invokes the actual Go
dispatcher with widths 2^32, 2^32+1, and uint64::MAX. Its 648 cases cover fixed
components, saved-position branches, high/full-precision branches, default-region
vectors, and absent/rejected ability bodies, with varied offsets and short padded
payloads. None consumes an oversized field. Status, endpoint and subsequent
seven-bit reads come from native output, not from Rust. Generation passed (0.337s).

Previously NativeFilmReader::read_component converted every profile width to
usize up front. On wasm32 it rejected every oracle case before reading anything.
The direct path now carries raw movement widths separately from body switches;
world, traversal, delta, and ability-axis widths are converted only at the actual
field read, after gates. Raw profile metadata stays unchanged. A reached width
outside the reader's address domain reports Width after preserving prior cursor
advancement; that is still an explicit unsupported address-domain boundary.

All 648 native cases pass both host and simulated 32-bit domains. A reached
second-axis test verifies refusal at bit 11 after the selectors and first axis.
The actual wasm32 Node runtime also passes all 648 cases and asserts that the old
checked encoding adapter rejects those same profiles on wasm32. Reproduce with:

    python3 src/theater/reference/verify_lazy_widths_wasm.py

The runner builds a temporary cdylib offline from this checkout and the native
fixture, then executes it with Node. Unexpected host imports throw if invoked.
It requires the wasm32-unknown-unknown Rust target, Node, and cached dependencies.
It is a focused runtime test, not a browser integration test.

Validation: six focused reader tests passed (0.32s); full Theater 563 passed,
46 ignored (100.72s); Clippy passed; all six captured 44-input/objective-control
and complete-document checks passed (98.69s); formatting, diff checks, generator
syntax and all 485 pinned source hashes passed. Existing zero/wide position
oracles continue to pass, including nonfinite value-bit checks.

This closes eager width conversion in the direct component entry point. The
public component_encoding adapter and frame_encoding still validate widths
eagerly; frame-level laziness, actually consuming unrepresentable widths, and
signed/overflowed cursor continuation remain separate work. No architecture
refactor or semantic-golden completion is claimed.
