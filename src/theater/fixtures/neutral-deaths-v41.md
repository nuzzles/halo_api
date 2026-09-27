# Neutral-death publication and filtering log oracle

Pinned LevelUp feat/v75: 43a01721e8a02c87c955e175936f0ccf8dd97a81.
Harness: ../reference/halo_rust_neutral_deaths_test.go.txt in the replay package;
run TestHaloRustNeutralDeaths. The fixture is /private/tmp/halo-neutral-deaths.json
compressed with zlib level 9.

All 256 original inputs and output fields were verified unchanged. The extension
captures actual native logs and adds one nonempty fully retained control with a
track XUID overriding a conflicting slot bridge. There are 257 cases, 245 INFO
records and 12 silent controls. Rust captures the actual event list and compares
level, message and both counts, along with complete filtered output. Only native
wall-clock log time is removed; framework metadata is not part of the contract.

Filtering preserves input order, excludes unknown kinds and unpublished actors,
and logs only if at least one record was removed. This describes publication
loss, not extra recorded deaths. These synthetic cases do not establish whether
an externally supplied neutral-death classification is physically correct.
