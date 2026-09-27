# Raw biped quanta without map bounds

Pinned LevelUp commit: `43a01721e8a02c87c955e175936f0ccf8dd97a81`.
Oracle extension: `TestHaloRustRecordMaskHook` in
`reference/halo_rust_record_mask_hook_test.go.txt`, stored in the existing
`record-mask-hook-v41.json.zlib` and already registered in the generator.

The 256 scan configurations contain 6144 payloads, with 5294 accepted native
records after saturation filtering. The extra pass sets WorldRange=nil and
QuantaOnly=true on native ScanBipedRecords; its observer is nil so it cannot
change the pre-existing mask callback expectations. Every old fixture field was
verified unchanged. Rust compares ordered slots and Q, verifies native
HasWorld=false and zero native coordinate placeholders, and roundtrips its raw
record type while asserting no world coordinate field is serialized.

`scan_quantized_biped_position_records` requires only source bytes, a slot band,
wire layout and scan options. It shares record walking and companion extraction
with the existing world-coordinate scanner; dequantization is a subsequent step
only for the latter. No dummy bounds or fabricated world positions are used.
Saturation filtering and optional companions apply. This payload API has no
packet timing, so isolation filtering is a higher-level concern; spatial speed
filtering has no meaning without world coordinates.

This closes the raw payload scan gap, not the complete native loaded-source
QuantaOnly/selected-chunk option contract. Native companion fixtures remain
separate evidence for direction/vitality decoding. This is parser regression
evidence, not an independently annotated player-action golden film.

## Loaded-source explicit-layout scan

`scan_source_quantized_positions_for_band` scans a loaded FilmSource using an
explicit layout and sparse slot band. Empty chunk selection uses native prefix
selection; explicit lists preserve order/duplicates and skip missing buffers.
Chunk numbers, source buffer positions and packet ordinals are distinct fields.
It applies shared temporal isolation using native wrapping neighbor differences,
without sorting samples or applying a world-speed filter. No source bytes copy.
No chunks, empty band and no readable chunks have distinct failure categories.

The source extension has 1280 calls, 19484 returned records and 429 errors.
It varies metadata order/gaps, empty/sparse bands, missing and duplicate chunk
requests, and temporal/spatial filter settings. A non-delta packet with a one-byte
payload precedes all position packets to verify ordinals. The initial zero-size
version stopped native packet indexing and yielded no positive records; it was
corrected before accepting the source comparison. The Rust test explicitly
asserts the positive record total to prevent a vacuous pass.

Compare ordered source numbers, ordinals, timestamps, slots, Q, world absence,
source range accessibility and portable roundtrip. All older mask/payload fixture
fields were checked unchanged. Automatic layout detection and automatic biped
band discovery remain separate loaded-source work; callers currently provide
both explicitly. Existing captured corpus processes predate this implementation.

## Loaded-source mask publications

`scan_source_quantized_position_report_for_band` retains positions and native
RecordMaskHook publications separately. Masks are emitted after saturation
rejection and before temporal isolation, only with CaptureDirs enabled. Each
observation retains requested chunk number, packet ordinal, source packet and
payload-relative end-of-position bit offset; payload bytes remain in FilmSource.
The original Vec-returning API delegates to the report API.

The same 1280 native source calls now capture 9953 ordered hook publications;
155 calls have more publications than returned positions. Rust compares every
mask, offset and referenced payload against those callbacks, checks source chunk
and packet references, and roundtrips the complete report. Previous fixture
fields were verified unchanged before replacement. These are native observer
expectations, not independently annotated action events.

## Automatic loaded-source setup

`source_biped_slot_band` reads the first keyframe in each requested chunk, plus
one after the last requested number, and exposes the native filled band as
inclusive bounds. `detect_source_i0_layout` borrows up to six native prefix
chunks and uses that band's following keyframe. It retains inconclusive reports
and implausible candidates. No world bounds are inferred from this measurement.

`scan_source_quantized_positions` discovers the biped band; the explicit-band
`scan_source_quantized_position_report_with_layout` accepts an optional layout.
With no supplied layout both use prefix-based detection independently of the
selected scan chunks. Setup error precedence is chunks, band, layout, readable
chunks. Native callback publications use the existing report path.

`halo_rust_i0_loaded_test.go.txt` extends `i0-loaded-v41.json.zlib` to 144 cases,
1008 selected-band comparisons and 4608 scan calls (21819 returned records).
All 128 original case fields remain unchanged. Sixteen added controlled bit
patterns supply positive detection: nine successful 14/13/13-bit layouts and
108 successful measured-layout scan calls, alongside refusals. The original
fixture had no positive detection; it could not alone establish successful
measured scans. Rust compares every diagnostic field and accepted position's
chunk, packet ordinal, timestamp, slot and Q. The test recreates the native
registry header even for sources with no data chunks.

## Loaded world scans and speed-filter overflow

`scan_source_world_positions_for_band` applies the same explicit-layout source
scan, converts accepted quanta with independently supplied minimum/maximum world
bounds, then runs native speed filtering. Position reports keep the raw record
and source identity alongside each world vector. Pre-filter masks survive both
isolation and speed rejection. `SourceWorldScanOptions` carries native scalar
scan settings, dynamic orientation and a direct per-slot exemption map. Exemption
arrays preserve caller order; event-derived maps are sorted by the existing path.

The source oracle now also compares 1280 world calls with 16914 returned records,
including empty and unsorted exemption arrays, sparse bands, duplicate selections,
source errors, world coordinates and portable report roundtrips. All older fixture
fields were checked unchanged. Shared speed filtering now uses native wrapping
arithmetic for the +/-200000us exemption window. In 256 boundary cases (3072
positions), the former saturating arithmetic disagreed with native accepted
indices in 204 cases. Ordinary native filter fixtures exercise the same helper.
Automatic world setup and full caller-context composition remain separate work.

## Automatic world setup

`scan_source_world_positions` accepts optional band and layout arguments. A
missing band performs native biped discovery; an explicitly empty band remains
an error. Measured layout selection shares the quantized path's setup checks,
refusal handling and error precedence. Discovery is performed once; world bounds
remain independently supplied and never inferred from the measured widths.

The existing 144-source i0 oracle now also checks all 4608 calls with world bounds,
variable speed limits and per-slot teleport exemptions. It contains 18911 world
records, including 1527 using measured layouts. Rust asserts both positive totals
and compares all ordered source identities, timestamps, quanta and float32 world
coordinate bits. All previous fixture fields remain unchanged. This closes the
previous automatic-world-setup note; broader caller-context coverage remains open.

## Context independence and complete companions

The source fixture adds 2560 native context experiments: zero profile and a
modified default profile, with all 30 observer function fields instrumented.
Complete native records/errors/masks remain equal to each prior source case;
only RecordMaskHook fires (19906 publications across both variants). Old fixture
fields were checked unchanged. Rust additionally compares all returned companion
values, including direction, roll/mode, aim flags and secondary aim, body health
and flags, and shield regeneration/flags. See READER_CONTEXT_PORT.md for the
reachable-code audit and the remaining general reader-context gaps.
