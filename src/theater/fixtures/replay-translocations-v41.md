# Translocation publication and coverage logging oracle

Reference: JGtm/LevelUp feat/v75 at
43a01721e8a02c87c955e175936f0ccf8dd97a81.

Harness: ../reference/halo_rust_replay_translocations_test.go.txt, installed in
internal/games/halo_infinite/film/replay as a Go test. Run
TestHaloRustReplayTranslocations to write /private/tmp/halo-replay-translocations.json;
the fixture is that JSON compressed with zlib level 9.

The 1024 deterministic cases call native buildTranslocations and capture actual
logTranslocationCoverage output through slog's JSON handler. Only wall-clock
log time is removed; level, message and all six attributes remain. Existing
input and publication fields were compared before replacement and are unchanged.
Rust captures its actual tracing event, checks exactly one event per call, and
compares the complete observation as well as the original publication oracle.

Controls include empty events, zero step, unpublished slots, events before the
origin, unavailable paired positions and recorded zero coordinates. This is
reference publication/logging evidence, not independently annotated gameplay.
Rust uses tracing's message field and maps it to slog's msg only in the test;
wall-clock timestamps and logging framework metadata are outside this contract.
The Film document constructor emits the coverage event once when the layer is
assembled; standalone data builders remain free of logging side effects.
