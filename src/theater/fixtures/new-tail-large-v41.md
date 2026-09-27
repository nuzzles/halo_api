# Large and unused native New-record terminal tails

Reference: LevelUp 43a01721e8a02c87c955e175936f0ccf8dd97a81.
The saved TestHaloRustNewTailLarge harness generates 224 cases through the native
DecodeFrameRecords loop. All 384 earlier tail-policy expectations are unchanged.

Widths: 4097, 2^32, 2^32+1, i64::MAX-1024, and i64::MAX. The final width is only
used on record paths that never consume the tail, avoiding a native negative
cursor after signed overflow. Six record paths and all eight start alignments
exercise successful/failed New records and Delta/Delete/End. Consumed large tails
contain a short actual source prefix (including 0xdead); the rest is synthetic
zero padding. Native Skip advances without looping over those padding bits.

Contextual frame encoding now postpones terminal-tail conversion until successful
New completion. The component reader takes the raw signed setting from its native
grammar. Nonpositive settings remain disabled. Consumed large tails retain every
available source bit and compact the padding into a NativeWidthAdjustment tagged
NewRecordTail. Ordinary tails retain their existing scalar field layout.

Tests compare native endpoints, record success, callbacks and world occupancy;
check retained source bits independently; and assert raw metadata preservation.
On wasm32, 48 consumed widths exceed the address domain: they explicitly return
Width at the tail start. They are not claimed as successful native comparisons.
The shared runtime runner now passes 3,392 native comparisons and these 48
refusal checks, plus its existing boundary controls. An additional control uses
a representable positive width whose resulting cursor is 2^32: the adjustment
preserves that native target while refusing the unrepresentable cursor at bit 21.
Signed-overflow targets are likewise retained, but continued decoding from a
negative/wrapped cursor is not supported by this change.

Reproduce with `cargo test --lib new_record` and
`python3 src/theater/reference/verify_frame_lazy_widths_wasm.py`.
Go generation is registered in generate_oracles.py. Hash/count evidence is in
new-tail-large-validation.json. Logs: /private/tmp/halo-new-tail-large-*.log.
Other scalar settings and broader full-Film acceptance remain open. This does
not begin the deferred architecture.
