# Live validated resync context (v41)

Reference: LevelUp feat/v75 at `43a01721e8a02c87c955e175936f0ccf8dd97a81`.
Harness: `../reference/halo_rust_context_resync_test.go.txt`, registered in the
oracle generator. Native Go generation passed before compressing JSON.

128 synthetic contexts, two scans per context, 28 successful landings and 682
callbacks including post-scan restoration probes. The Rust test compares landing
bits, ordered callback payloads, cumulative validated-resync counters at EMP
publication time, restored receivers, and unchanged world bindings.

Cases include extra prefixes, short sources, hard/soft bindings, non-target trial
reads, alternative archetypes, and mobility callbacks that reinstall movement and
unit-reference hooks during the neutralized scope. Candidate deltas set the
record slot in native decodeDelta; fresh recursive body trials start at slot zero.
Position/reference/movement hooks are temporarily removed, not irreversibly
filtered from contextual diagnostic candidates. The observer remains shared.

These are parser test stimuli, not independently observed player actions. This
fixture does not establish raw-resync shallow-copy semantics, positive budget
exhaustion, or explicit repair parity. Resync is an explicit API; default traversal
is unchanged. The deferred semantic golden-film suite remains separate.
