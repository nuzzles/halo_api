# Raw-resync loop guard boundaries

Reference: `43a01721e8a02c87c955e175936f0ccf8dd97a81`.

TestHaloRustResyncGuard emits six complete native runs: 4,095, 4,096 and 4,097
clean Delta records, with/without 32-bit record prefixes. Each record alternates
between two bound slots. An End marker follows the requested records. The
native test asserts min(requested,4096) returned records and no acceptance calls.

The Rust comparison checks all 24,574 ordered returned IDs, record endpoints and
successful decode states, final binding cardinality, and the absence of recovery
landings/callbacks. The same test runs on WASM. At 4,095 records the normal End
path runs; at 4,096 or more the iteration guard stops before the End marker.
Rust's stop classification is checked against this native control flow; native
DecodeFrameResync does not expose its final loop cursor, which is separately checked
through pinned AST instrumentation in resync-cursor-v41.md.

This validates the existing positive 4,096-record limit. No traversal logic or
limit changed. It is not a full-film or semantic action fixture.
