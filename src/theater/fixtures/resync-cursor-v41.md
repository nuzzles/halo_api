# Final cursor of native raw resync

Reference: `43a01721e8a02c87c955e175936f0ccf8dd97a81`.
The native public DecodeFrameResync returns records but not its local cursor.
`instrument_resync_cursor.go.txt` verifies frame_harvest.go's pinned SHA-256 and
uses Go's AST to copy the function under a test-only name. It changes only the
name, result type, and four return expressions, adding `br.BitPos()` to each.
No reads, branches, limits, callbacks or mutations are inserted or replaced.
The normal fixture generator regenerates this helper before running the oracle.

TestHaloRustResyncCursor runs both original and instrumented functions from fresh
state for every input, asserting byte-identical JSON for records, callbacks,
shared width maps and world bindings. Only the instrumented run supplies the
additional final cursor. This supplies independent native runtime evidence,
not an expected cursor computed by Rust or from an expected record count.

Seventy cases cover optional record prefixes; bound/unbound slots; absent and
present baseline selectors; calibrated/stub widths; acceptance/rejection and
callback deletion; truncated sources; and all six 4,095/4,096/4,097-record guard
cases. The fixture contains 24,582 returned records and 21 acceptance calls.

The comparison found a real Rust mismatch: native decodeDelta reads the baseline
selector before looking up an unbound slot, whereas the contextual Rust adapter
stopped at the ID. The missing consumption was one or eight bits. Native
slot-based contextual decoding now uses the policy that reads the selector
before lookup. The strict-generation entry point keeps its existing policy.
This corrects the final cursor and retains the baseline fields on the failed
record read without changing the native recovery search's starting position.

This closes the final-loop cursor evidence gap for these inputs. It does not
claim all signed cursor domains, arbitrary callback mutations, or complete
captured-film parser parity. The still-running captured march process was
started before this fix and validates the earlier signed-reader checkpoint.
