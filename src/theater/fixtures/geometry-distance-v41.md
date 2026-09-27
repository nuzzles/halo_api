# Native geometry distance oracle

Pin: LevelUp feat/v75 at 43a01721e8a02c87c955e175936f0ccf8dd97a81.
Generator: ../reference/halo_rust_geometry_distance_test.go.txt.
Registered oracle tests: TestHaloRustGeometryDistance and
TestHaloRustGeometryDistanceGrenades.

The first test invokes production dist3 and planDist for 4,661 point pairs:
4,096 deterministic finite pairs, 100 special-value pairs, and 465 pairs around
the 0.5m, 4m and 10m boundaries. Input f32 and result f64 values are stored as
IEEE bits. Native geometry_plan_distance tests compare all planar result bits,
including native NaN payloads and infinity precedence. The dist3 results are
retained for the continuing audit; they are not part of that Rust test.

A standalone expression probe on the original 4,096 finite pairs found 1,509
mismatches using Rust std hypot, 26 with the unfused scaled formula, and zero
with the native scaled formula and explicit fused multiply-add. Native results
were generated on darwin/arm64; Rust preserves those results explicitly across
host and WASM, rather than depending on each target's hypot implementation.
The f32-subtraction/f64-squares 3D expression matched all 4,096 finite cases.

The second test invokes production buildGrenades in four cases. They establish:
first-candidate retention when nearly equidistant births round to the same native
distance; the same ordering at 4m; acceptance of one birth at the rounded 4m
boundary; and rejection of a farther birth with biped fallback. The entire
publication and coverage are compared, including projectile links. Expectations
come from native production, not the new Rust helper.

The correction routes all four native planDist callers through the shared Rust
replay_plan_distance: grenade matching, vehicle occupancy matching, projectile
step clipping, and vehicle relay merging. These tests are not an independently
annotated action suite or a claim that all film parsing is complete.

Follow-up: ../reference/geometry-optimization-audit.json resolves the two
3D NaN payload differences as compiler-dependent native arithmetic. The same
pinned Go source produces both bit patterns with optimization enabled/disabled;
all classifications and non-NaN results agree. Recorded float bits are unaffected.
