# Native identity input retention

`Film.native_identity_inputs` retains the native death feed, its read failure,
the sorted death roster, and the raw replication index table before collision
rejection. An absent outer field means an older export never retained the scan.
An absent raw table with no index error means native scanning was skipped because
there were no deaths. A successful empty table and a failed scan remain distinct.
The native highlight observations underlying the death feed remain in
`Film.native_highlights`.

`FilmIdentityInputs::player_indices` applies the native injectivity guard to a
copy. Conflicting raw entries remain inspectable in the original. These are
native scan results, not the later bootstrap-composed identity table.

Constructors retain the default death-derived roster. Replay reuses these inputs
when its requested roster matches, and rescans source for a different external
roster. Older Film exports use the existing source fallback. With no deaths the
native scan is skipped regardless of additional roster entries. This change does
not introduce the deferred resolved-model architecture or claim lossless archival
of source bytes in Film JSON.

Validation uses the pinned native `replay-evidence-v41.json.zlib` oracle's 1,024
cases: 92 with collisions, 282 death-read failures, and 26 index-read failures.
Each case compares retained evidence with no source chunks against the native
expectation, verifies raw table preservation, checks serialization, and compares
changed-roster fallback with the source scan. Constructor checks cover compact
Film exports and older JSON without the new field. Captured full-document input
checks additionally compare all ordered Deaths and published PlayerIndices fields.
