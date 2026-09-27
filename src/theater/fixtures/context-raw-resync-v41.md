# Live raw target scan (v41)

Pinned parser: LevelUp feat/v75 `43a01721e8a02c87c955e175936f0ccf8dd97a81`.
Native harness: `../reference/halo_rust_context_raw_resync_test.go.txt`, registered
in the generator. Native execution succeeded before compressing the fixture.

128 contexts, two scans each, 140 successful landings, 172 acceptance callbacks
and 762 component/restoration callbacks. Tests compare landing bits, ordered
acceptance arguments (including float32 bit patterns), observer callback order,
caller-visible absolute-index histograms, hook behavior after scanning and
unchanged World. Cases include short sources, extra prefixes, soft/hard bindings,
alternative archetypes, nil/empty/populated histograms, rejected candidates,
positions and positionless records.

The scan shallow-copies its observer and replaces only its position hook with a
first-position collector. Other copied hook identities remain independent of
later caller hook replacement; already allocated maps remain shared. Mobility
callbacks modify the caller's hooks during the scan, and post-scan probes verify
that those changes persist. This differs from scoped validated-resync restoration.
The vector passed with has_position=false may retain a previous trial's value;
its presence flag determines whether it describes the current candidate.

The configured API retains all diagnostic candidates separately from native
callback delivery. The legacy no-context raw scan is unchanged. This fixture
covers the explicit scan, not DecodeFrameResync's whole loop, capture suppression
around recovery, or accepted-record re-read. Those live integrations remain open.
Synthetic reader probes are not recorded player actions or independent semantic
golden-film annotations.
