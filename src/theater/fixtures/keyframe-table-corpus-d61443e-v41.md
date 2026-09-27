# Sequential keyframe table oracle at d61443e

Reference: `d61443ef59268ad734355db8e9974f68db5ca6d0`.
Generator: `../reference/halo_rust_keyframe_table_corpus_d61443e_test.go.txt`.
Fixture: `keyframe-table-corpus-d61443e-v41.json.zlib`.
SHA-256: `86a16e303d3596f807660b419861df876f440c41dcad54665e2db9b481d84b31`.

The generator independently runs `WalkKeyframeRecords` at the additional pin on
all 451 keyframe packets from 32 local v41 films, using their recorded corruption
control. It does not select recovered anchors or use Rust outputs. Original Go
production files remain unchanged.

All 451 result objects are exactly equal, after JSON decoding, to the original
`keyframe-table-corpus-v41.json.zlib` oracle. This comparison includes ordered
record and component traces, source locations, values, endpoints and stop strings;
it is stronger than matching counts alone. The existing Rust tests of that
original oracle therefore have unchanged expectations, without replacing history.

There are 902 record attempts. Every table still stops at
`tacmap-mapdismissallock`. This is evidence of the shared first missing grammar,
not full native parsing parity. More candidate body reads elsewhere do not turn
these incomplete sequential tables into complete partitions.

`audit_native_keyframe_blockers.py --additional-pin` now uses this fixture for
sequential results. Candidate-body selection still has the original pin's offsets;
those outputs are independently recomputed at d61443e, as recorded separately in
the report. The default audit keeps its original historical inputs.
