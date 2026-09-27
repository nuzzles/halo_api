# Accepted resync candidate invalidated by callback mutation

Pinned reference: `43a01721e8a02c87c955e175936f0ccf8dd97a81`.
Native harness: halo_rust_resync_mutation_test.go.txt, registered in the generator.

Sixteen cases cross calibrated/stub width maps, optional 32-bit record prefixes,
retained/deleted overrides, and two payload values per combination. An unbound
Delta causes recovery to scan forward to a bound slot with one component named
`not-ported`, initially made readable by an eight-bit override. The acceptance
callback either retains that override or deletes it from the shared map.

The native harness asserts that every candidate initially decodes cleanly, exactly
one acceptance callback runs, and exactly one record is returned. Eight deletion
cases return the failed re-read (DesyncAt=0), ending before the component payload;
eight controls return a clean record (DesyncAt=-1) after consuming those eight bits.
This is native behavior: DecodeFrameResync ignores TryDeltaAt's success boolean
when appending the accepted candidate's second decode.

Rust matches the ordered callback arguments, shared map contents, landing,
record identity/archetype/mask, component ordering/status/start, returned record
endpoint and final world binding. Failed component attempts remain present with
no invented payload fields. Controls retain the source byte as a raw eight-bit
field. Frame JSON roundtrip is checked. The full comparison also runs on WASM.

The existing resync implementation already matched this path; no traversal logic
changed. NativeSharedWidths::remove is now public so an external callback can
perform the native map deletion, preserving aliases and the allocated-empty map.

This closes the failed-re-read callback case previously listed in
context-resync-frame-v41.md. It does not provide a native final-loop cursor oracle,
or arbitrary callback-mutation coverage. The positive guard case is separately
covered by resync-guard-v41.md.
