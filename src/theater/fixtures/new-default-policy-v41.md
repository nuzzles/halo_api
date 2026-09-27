# Native New default-state width admission

Reference: 43a01721e8a02c87c955e175936f0ccf8dd97a81.
Harness: ../reference/halo_rust_new_default_policy_test.go.txt.
Fixture: new-default-policy-v41.json.zlib (624 cases).

The native TraverseEntity reads NewDefaultStateBits only in its fallback branch:
non-biped New records without a selected archetype deserializer. The same option
is unused by Delta, Delete, End, invalid New archetypes, a selected archetype
reader, and the always-selected biped default reader. It is a signed Skip, not a
strictly positive tail policy.

The oracle crosses eight bit alignments with stub/default-disabled New,
archetype-decoded New, biped New, Delta, Delete, End and invalid New. Widths are
-128, -32, -1, 0, 5, 65, 4097, 2^32, 2^32+1 and i64::MAX (last only unused).
There are 480 unused-width cases, 48 negative consumed skips, 48 large consumed
skips, and 48 ordinary consumed skips. A zero prefix ensures negative skips land
at representable nonnegative offsets; some records end before their header.
The fixture independently records ordered native records, masks, endpoints,
completion and complete world slots after DecodeFrameRecords.

Rust retains the raw native width in the contextual NewRecordEncoding and
converts it only if fallback is selected. Negative skips with nonnegative targets
record a NewRecordDefault width adjustment and continue at that target. Large
positive skips retain all actual source bits and compact the padded remainder;
ordinary skips preserve the existing scalar layout. Skipped source fields,
world identity/generation/view/position state and JSON roundtrips are checked.
The explicit checked legacy encoding adapter remains checked.

On wasm32, 32 consumed 2^32/2^32+1 cases exceed the address domain and must refuse
at the reached default-state field, after the header and archetype. Unused widths
must not reject frame setup. verify_frame_lazy_widths_wasm.py runs this helper in
an actual Node WASM runtime along with existing frame/observer/width controls.

Negative targets and signed-overflow cursor continuation remain unsupported;
NativeWidthAdjustment retains the inputs and recoverable native wrapping target.
This fixture does not claim those cases, arbitrary-width MPP/ID parity, or a new
source-lossless architecture. Native New terminal-tail positive-gate tests remain
separate and protect their different policy despite sharing skip retention code.

## Returned trace metadata

EntityRecord.default_state_bits now preserves the native New Trace.DefaultBits
value at trace entry, before archetype validation. It is configured decoder
context, not an assertion about how many wire bits the record consumed. The
fixture checks all 384 returned New records: 120 negative values, 40 zeros and
24 i64::MAX values, including dedicated/biped readers that ignore the setting.
Non-New records do not acquire this New-specific metadata. On wasm32, 352 New
records return this field; 32 consumed out-of-domain widths refuse before a
successful returned view, as documented above.

The helper checks complete view JSON roundtrips and removes the field to simulate
old exports: absence remains None, never an invented zero. Source widths, cursor
outcomes, components and world-state comparisons from the original fixture remain
unchanged. No native expectation was regenerated for this added assertion.
