# Death-clock ambiguity warning oracle

Reference: LevelUp feat/v75, 43a01721e8a02c87c955e175936f0ccf8dd97a81.
Harness: ../reference/halo_rust_bridge_warning_test.go.txt, copied into the native
internal/games/halo_infinite/film/replay package. Run TestHaloRustBridgeWarning;
compress /private/tmp/halo-bridge-warning.json with zlib level 9.

512 cases call actual BridgeHealth.warnIfCalageEtroit with matched counts 0..31
and runner-up counts 0..15. Capture uses slog JSON; only wall-clock time is
removed. The native output contains 225 warnings and 287 silent controls. Each
Rust call compares the entire ordered event list, including level, message and
all five attributes. Zero matches, zero runner-up, exact two-to-one margin and
one-below-margin controls are included. The warning is a decoder alignment
confidence diagnostic, not evidence of a recorded gameplay event.

Native buildCoverage invokes the warning after SanteDuPont and before verdict
assembly. Rust ReplayCoverage::new invokes it at the equivalent point; simply
reading IdentityRegistryOutput::bridge_health remains free of this side effect.
These synthetic confidence controls do not validate the alignment algorithm or
independently observed deaths, which have separate fixtures and captured gates.
