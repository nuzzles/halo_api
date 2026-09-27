# Signed native record headers

Pinned reference: `43a01721e8a02c87c955e175936f0ccf8dd97a81`.
Generator: `../reference/halo_rust_signed_header_test.go.txt` directly invokes
readRecordType and readRecordID on a native Lecteur, retaining final cursor on
panic and reset/read recovery. Repeated native generation is byte-identical.

411 cases cross four source buffers, signed starting positions (including i64
extrema and the wasm32 address boundary), and signed ID widths. There are 136
native panic outcomes and 275 successful headers. Native widths above 64 use a
per-bit loop: impractically large active reads are excluded, except one 2^32-bit
ID read. Huge unused widths remain covered by End-only cases. No accelerated or
rewritten native reader is used to generate these expectations.

native_signed_record_headers compares header kind, ID, signed start/end,
panic outcome, reached cursor and reset/read recovery. Successful headers also
roundtrip through JSON. WASM checks the 275 nonpanicking outcomes.

native_signed_header_frame_entries compares the 361 cases that also determine
whole-frame behavior (End or header panic), through both NativeFilmReader and
the NativeFilmBits frame adapter. WASM checks the 225 successful End cases,
including End headers whose endpoint wraps negative. Non-End successful headers
are excluded only from this frame test because their bodies are not part of the
header oracle; they remain covered by the direct-header test.

The separate native_record_id_overflow_panics_preserve_cursor test uses analytical
overflow expectations for i64::MAX active widths. It is not counted as an executed
native oracle comparison: the native loop cannot feasibly complete those reads.
