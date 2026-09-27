# Live frame lazy position widths

Pinned reference: LevelUp `43a01721e8a02c87c955e175936f0ccf8dd97a81`.
The source tree still passes all 485 pinned hashes. These are generated native
reader comparisons, not captured actions or independent semantic annotations.

`context-frame-lazy-widths-v41.json.zlib` contains 128 native DecodeFrameRecords
cases with position dimensions 0, 65, 2^32, 2^32+1 and uint64::MAX. Components
use fixed widths; cases include immediate End, creation/delta/deletion,
truncation, extra fields, generation policies, observer replacement, callback-time
calibration mutation and world updates. Oversized fields are deliberately unused.
All original 128 context-frame expectations remain unchanged.

`world-position-widths-v41.json.zlib` contains 144 native direct and frame reads
of object-position-component. Consumed dimensions include zero and widths above
64, with unequal axis widths. Oversized dimensions occur only on the high-precision
branch which does not use them. Native numeric axis values, ordered component
field boundaries, direct end/following read, frame records/end and raw profile
retention are checked. Native numeric axes come from separate reference bit reads;
Rust does not generate the expectations. Discarded prefix bits remain in fields.

The fix passes raw movement dimensions to the live Reader and converts each width
when its field is reached. A private policy adapter supplies representable widths
only to carry switches through the checked legacy FrameEncoding adapter. Actual
reads must never consume these placeholders. This also repairs the previously
missed object-position-component path in the direct reader. Width errors preserve
the reached cursor and committed preceding world state.

Reproduce native generation using the saved Go harnesses:
`halo_rust_context_frame_test.go.txt` (TestHaloRustContextFrameLazyWidths) and
`halo_rust_world_position_widths_test.go.txt` (TestHaloRustWorldPositionWidths).
Both are registered in generate_oracles.py. Fixture hashes and counts are in
`../reference/frame-lazy-width-validation.json`.

Host comparison: `cargo test --lib native_context_frame`.
Actual 32-bit comparison: `python3 src/theater/reference/verify_frame_lazy_widths_wasm.py`.
The latter runs 1,424 native cases: both context fixtures, world-object cases and
both existing 512-case position-accumulator fixtures. A separate address-domain
control puts an unrepresentable width on the second axis and verifies a Width
error at bit 35, after the record header, region and first axis. It is not a native
billions-of-bits comparison. Unexpected JavaScript host calls fail the runner.

Remaining scope: other NativeFrameConfig adapters still validate eagerly, as do
signed frame scalar settings. Signed/overflowed cursor continuation and broader
whole-Film/source audits remain open. This does not complete v41 parity or begin
the deferred Film / ResolvedFilm / playback architecture.
