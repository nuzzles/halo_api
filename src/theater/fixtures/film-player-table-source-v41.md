# Source player-table oracle

Pinned LevelUp feat/v75 commit: 43a01721e8a02c87c955e175936f0ccf8dd97a81.
Harness: ../reference/halo_rust_film_player_table_source_test.go.txt, copied into
internal/games/halo_infinite/film/replay; run TestHaloRustFilmPlayerTableSource.
Expected JSON comes from the actual ScanFilmPlayerTable and FilmPlayerTable.Lue.
Only nil seat slices are normalized to empty arrays.

Input is the captured v41 Bazaar bootstrap from
experiments/films/maps/01-bazaar-idle/chunk-000-type-1.bin, retained as
film-player-table-source-input-v41.zlib (zlib level 9).
Decompressed SHA-256: 9895f10b150fdb75bfdb0ab8f1976e4302bf1f4c1ab7b8703c8bde8dfc49edaf.

Nineteen cases compare the complete table and read predicate: three successful
one-player outputs plus every named refusal (missing registry, missing identity,
unknown build, truncated input and table not found). Metadata cases prove that
number zero can select the second loaded chunk, that no zero means no registry,
and that the first duplicate zero wins. Byte cuts are explicit native offsets;
build mutations preserve the rest of the recording. Every Rust result also
roundtrips through JSON. Synthetic mutations are not new gameplay recordings.

## Native refusal diagnostics and metric increments

The oracle now captures native log observations with their exact JSON numbers
and per-call deltas from the native `levelup` expvar map. All original fields of
the nineteen cases were checked unchanged. A twentieth case wraps the captured
bootstrap in two zlib layers; source loading removes one, leaving an explicitly
still-compressed registry. The first metric probe incorrectly inspected root
expvars; it was corrected to the native `levelup` namespace before saving the
fixture. The unknown-build case increments its named counter exactly once.

`scan_replay_film_player_table_with_diagnostics` retains the projected table,
typed native refusal error (including the unknown build), and metric increments.
Missing registry has no underlying decoder error; missing identity, truncation,
unknown build, table not found and still-compressed source stay distinct. The
error's rendered message is compared with actual native observations. The older
table-only function delegates and keeps its existing result shape.

Film.player_table_diagnostics uses the same conversion from the already decoded
table, avoiding a second table scan. Missing identity is explicit; older exports
without the field deserialize it as unavailable. Public Film construction and
JSON roundtrips cover success, unknown build and absent identity using the
captured bootstrap plus an independently framed empty replication packet.
Constructor-level bootstrap errors still fail, as before.

These APIs retain per-call metric publication data without incrementing a global
counter. Versions other than 41 remain unsupported.

## Source-table publication

ReplayFilmPlayerTable.log now publishes the native success/refusal observations,
including the separate interleaved-vacancy warning after successful table counts.
Film construction invokes it with the supplied match ID and retained diagnostics.
The source-only adapter remains usable without logging; callers can publish its
returned table and diagnostics explicitly. This supersedes the earlier lack of
source-table log delivery, not global metric accumulation or registry warnings.

The twenty-first case splices an existing generated interleaved slot body from
player-table-v41.json.zlib after the captured bootstrap's native identity endpoint.
The selected input row and native endpoint are retained in the oracle. Actual
ScanFilmPlayerTable reads seven occupied seats and rejects the direct identity
mapping due to interleaved vacancies. This is synthetic record composition, not
a newly captured film. Every field in the preceding twenty cases was checked
unchanged before saving the extension.

The test compares all source-table INFO/WARN records with native output: fields,
values, match identity and ordering. Tracing omits absent Option fields, so only
the absent refusal error is normalized to native JSON null; its typed retained
value is already checked independently. Registry warnings are excluded from this
source-table comparison. Film-level checks exercise success, missing identity,
unknown build and positive interleaving, together with JSON and old-field absence.
The generator now explicitly registers this harness after its slot-body input
fixture, including relocatable input/output paths.
