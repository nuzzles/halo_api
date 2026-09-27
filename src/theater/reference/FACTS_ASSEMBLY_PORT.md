# Complete FilmFacts blob and final body sections

Pinned native sources: `replay/filmfacts_encode.go` and `filmfacts_decode.go` at
`43a01721e8a02c87c955e175936f0ccf8dd97a81`.

`facts_world_section.rs` adds equipment placements, spawn records/statistics and
composition with shared placement statistics and both world-object scans. T0US
is delta-coded; T1US is absolute. All float32 bits and native signed Points are
preserved. Both life references remain populated even when their validity flags
are false. Spawn Chunk/PacketIndex use signed varints, and all seven counters use
unsigned transport interpreted as native signed ints.

`facts_queue.rs` adds deaths, player indices, film seats and team scan results.
All names, build/refusal labels and component names retain raw bytes. Native ints
remain i64 on WASM. ByXUID sorts unsigned; team indices sort signed. Duplicate
entries replace earlier values, including partial entries appended before a
transport failure. Decoded ByXUID is always a non-nil map. Empty team maps and
empty seat lists normalize to nil, while positive counts can produce allocated
empty containers. Negative map counts skip their loops; negative slice/string
capacities are refused safely at the native panic cursor. Counts never request
eager untrusted allocations. The independent native field inventory is
`facts-tail-field-inventory.json`; these final body sections carry all their
listed fields, with shared codec projections documented separately.

`facts.rs` provides the complete `NativeFilmFacts` blob projection, composed in
exactly the native sequence. `encode_film_facts` returns `NativeFactsWriter`,
retaining both byte output and the optional embedded JSON error (the two results
of native EncodeFilmFactsAvecErreur). The caller can obtain the native bytes-only
result through bytes(), but persistence must inspect error(). In particular an
object-death JSON failure does not erase bytes already written or stop subsequent
sections from being written.

`decode_film_facts` performs typed header admission, decodes every section in
order, then rejects sticky transport errors and trailing bytes. It returns no
partial facts on failure; individual section APIs retain native partial results.
Catalog bounds are transposed from axis ranges into the shared position reader's
endpoint arrays for both biped and vehicle reconstruction. Header layout gate and
region remain zero, matching native header decoding.

## Validation

- 28,713 final-body cases: world composition 3,413; spawn 5,185; queue 11,232;
  film table 2,502; teams 6,381. Four smaller sections use 48 states each and every
  prefix. World composition uses 24 populated states, every prefix for the first
  three, and complete/seeded sampled prefixes for the remainder. Extra trailing
  bytes, duplicate map keys, signed values, raw strings and ten native panic
  controls are included. Shared world/statistics oracle adapters are reused to
  compare all decoded fields and float bits, not only re-encoded bytes.
- 2,538 complete-blob cases from 24 synthetic v41 inputs: every prefix of the empty
  source, 48 seeded prefixes of each other input, wrong-map admission and trailing
  bytes. Successful decoding is compared to native normalized re-encoding. Biped
  and vehicle reconstructed coordinates are independently compared by float bits,
  since those coordinates are recomputed and cannot be verified by re-encoding
  the quantized source alone. Detailed per-section field tests complement this
  composition check.
- Three complete-encoder failures (NaN and both infinities in the embedded death
  profile) compare the full partial bytes and error text to native.
- Host/static/WASM results: `facts-assembly-validation.json`.

This completes the REPLAYINPUTS25 blob codecs, not the full v41 parser goal.
FilmFactsFile has a separate container header and five sections, including data
this blob does not carry. File framing/freshness, scan/cache conversion,
BuildFromFacts and final source/field/runtime/export reconciliation remain open.
The existing captured-film evidence gaps remain open too. The native cache is
lossy relative to the recording; this checkpoint is not a lossless Film fidelity
claim and does not begin the deferred Film/ResolvedFilm/playback refactor.
