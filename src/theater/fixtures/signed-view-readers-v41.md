# Native signed message/control readers

Pinned LevelUp: 43a01721e8a02c87c955e175936f0ccf8dd97a81.
120 cases cover both view classes, four sources, and signed start positions through
i64 extremes. 48 panic and 72 return normally. The oracle compares kind order,
completion, and final cursor. Native guards use wrapping position+width; a start at
i64::MAX can read a padded zero and complete with endpoint i64::MIN. Host compares
panic behavior; the aborting WASM runtime executes only nonpanicking cases.
Native source: reference/halo_rust_signed_view_readers_test.go.txt.
See reference/signed-views-validation.json for checkpoint status.
