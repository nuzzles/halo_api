# Native kill decoder input error precedence

Reference: 43a01721e8a02c87c955e175936f0ccf8dd97a81.
Harness: ../reference/halo_rust_kill_chunks_test.go.txt.
Fixture: kill-chunks-v41.json.zlib, extended with decode_error on 128 cases.

The actual exported native Decode operation runs loadFilm before loadKillFeed.
No chunks must return ErrNoChunk; chunks without any type-zero replication
packet must return ErrNoPacket; only then can a missing usable kill feed return
ErrNoKillFeed. Rust's full decoder previously attempted the feed first, merging
these failures into its generic missing-feed error.

The v41 Rust full-decoder entry now validates source presence and replication
packets first. DecodeError::KillSource retains the typed KillSourceFilmError:
NoChunk, NoReplicationPacket or NoKillFeed. Display strings match the native
sentinels. NO_KILL_FEED now denotes the typed native error rather than a generic
Missing string. Metadata numbering does not select or exclude input bytes.
The explicit unsupported-version check remains the Rust v41 scope boundary.

Native generation passed in 0.339s. The new field comes from a direct call to
Decode(context.Background(), name, source, nil), not a hand-written model of
error precedence. There are 10 no-chunk, 24 no-replication and 94 no-kill-feed
outcomes. Every pre-existing fixture key/value was checked unchanged before the
new fixture was installed. Existing loaded-source packet checks remain intact.
The focused full-decoder comparison passes all 128 cases (0.20s), checking exact
error text and typed category. It does not claim cancellation or native logging
parity. Successful decode paths incur an initial packet-presence scan; no source
bytes are copied by that guard. Positive full-decoder corpus validation remains
separate from these deliberately unsuccessful inputs.

Full Theater: 580 passed, zero failed, 48 ignored (98.82s). Clippy passes
(50.08s); actual WASM passes all 128 new cases in its 14,474 native-comparison
runner (plus 80 prior domain cases and supplementary controls; build 47.27s).
All 485 pinned hashes and formatting/diff/Python syntax checks pass. The two
captured full kill-source results pass (369 kills, 10.71s after 3m 06s release
build), including complete result and publication-gate assertions.
