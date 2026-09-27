# Captured kill-walk calibration acceptance

Reference: LevelUp 43a01721e8a02c87c955e175936f0ccf8dd97a81.

The broad captured acceptance run exposed a stale comparison in
kill_walk::tests::local_kill_record_walk. Both Bandit and Oddball differed only
because Rust now serializes KillCalibration.native_profile, while the older
native fixture did not publish that field. Every existing calibration value,
display string, death record, credible record and candidate matched.

halo_rust_kill_walk_test.go.txt now exports the actual native c.Profil under
calibration.native_profile. Native regeneration passed in 14.159s. A comparison
after removing only the added field proved every previous expectation unchanged
in both rows. No expectations were generated from Rust.

The Rust comparison now converts the complete native profile through the
existing native profile fixture adapter and compares it with the retained
NativeScanProfile. It also checks a complete KillCalibration JSON roundtrip.
Only after those assertions does it normalize the field's naming for the
existing whole-row comparison. Production parser behavior is unchanged.

Execution results and logs are tracked in corpus-acceptance-audit.json. The
original broad run retains its failure even if the focused rerun passes; do not
report it as an entirely green run. These two recordings do not establish
complete parser parity or coverage of every calibration/runtime configuration.

The focused release rerun passed both recordings in 5.31s, including the complete
profile assertions and JSON roundtrips. Native output and the retained fixture
agree, and formatting/diff checks pass. Logs:
/private/tmp/halo-kill-walk-refresh-native.log and
/private/tmp/halo-kill-walk-refresh-rust.log.
