# Generic entity record chains

Pinned LevelUp commit: `43a01721e8a02c87c955e175936f0ccf8dd97a81`.
Oracle: `TestHaloRustFramesParity` in `reference/halo_rust_frames_test.go.txt`,
already registered in the oracle generator.

The 32 synthetic packets carry creation, delta, deletion, generation reuse,
optional record prefixes/extra fields and component corruption checks. Native
`DecodeFrameRecords` produces expectations under both strict and relaxed
`GenerationStricte`. Sixteen cases stop on a stale generation in strict mode
but complete when relaxed. All previous strict/production fixture fields were
verified unchanged when the relaxed outputs were added.

The Rust comparison checks completion, consumed boundaries, ordered record IDs,
header positions, archetypes and final retained binding. In relaxed mode a delta
uses the slot's bound archetype but retains its recorded ID and does not rewrite
the binding. The original bounded decoder still defaults to strict checking.

This fixture does not establish full native observer/context inheritance or
reader-owned position accumulation. Those remain distinct from the verified
record traversal, generation policy and mutable source-reader behavior. It is a native grammar regression
fixture, not an independent golden recording of player actions.

## Explicit native tail behavior

The fixture now includes 384 complete/truncated buffer cases at six byte lengths
and both generation policies. Native readers start at bit 2, a known record
boundary; 304 cases consume synthetic tail bits. Rust compares ordered records,
header and trace boundaries, completion, and exposes padding on every retained
record (including End). Prior strict, relaxed and production outputs were
verified unchanged.

`decode_native_entity_view` selects zero-tail reads and native component-mask
traversal at an explicit record boundary. It is separate from the existing
bounded API. No source bytes are extended; `padded_bits` is the number of bits by
which each record's end exceeds the source buffer, matching the existing record
provenance convention. This does not assert that synthetic values were recorded.

Preamble handling, signed mutable reader positions, observer inheritance and
native reader-owned position accumulation remains outside this explicit-boundary API.

## Wildcard generations

An additional 256 native cases start on the delta after the first creation,
with an absent, matching, mismatched or wildcard initial binding, under both
generation policies. The comparison checks consumed boundaries, ordered record
IDs, completion and final binding presence/ID/unknown-generation state. Deletion
and successful New records replace the wildcard with a known generation; a
subsequent stale-generation delta must then fail strict admission. Wildcard
inputs contain namespace bits to check native slot masking.

`EntityBinding.generation_any` preserves generation uncertainty explicitly and
roundtrips through portable serialization. Missing fields default to false;
false fields are omitted to preserve existing serialized binding shapes.

Correction to earlier audit notes: pinned native `world.go` explicitly removed
the held-weapon cache. It is not missing parity work. Caller-owned position
accumulation and observation state remain separate contracts to audit.

## Caller-owned reader and packet preamble

`decode_native_frame_records` accepts a mutable `NativeFilmBits` and advances it
to the consumed end on success or desynchronization. Positive preamble widths
apply only when starting at bit zero. The returned view starts after that skip;
its End record and padding remain explicit. Negative reader positions return
None without mutation; this is an intentional input-domain limit.

The 512 native reader scenarios combine four starts (zero, first record,
internal delta, beyond buffer) with four preamble settings (-1, 0, 2, 7), across
the 32 packets. Compare ordered IDs, record and final reader boundaries, and
completion. Earlier oracle fields were verified unchanged. Native private
accumulator state has no public installer/production caller; the optional
component-level accumulator already has its own Rust/native suite. Inherited
observer/context state remains a separate API audit item.

## Native simulation-state completion policy

The 64 simulation cases replace archetype 3's component with simulation-state,
then invoke native DecodeFrameRecords with SimStateComplet false and true.
All 32 packet pairs have different consumed ends. Tests supply the existing
native-default position-width encoding; absent Rust width context is not
interchangeable with native reader construction. Rust compares ordered record
IDs, trace boundaries, final reader position and completion, and separately
asserts that an unspecified native encoding policy equals explicit false.
The original fixture fields were verified unchanged.

Native generic entry points honor `FrameEncoding.keyframe_simulation_complete`
(the shared encoding field also used by direct native component dispatch).
Previously they incorrectly forced true. The bounded convenience decoder keeps
its existing completed-body policy; no native profile default is inferred from
that convenience behavior.
