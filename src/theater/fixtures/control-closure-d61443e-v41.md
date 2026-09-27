# Native control-view closure oracle

Pin: LevelUp d61443ef59268ad734355db8e9974f68db5ca6d0.
Generator: ../reference/halo_rust_control_closure_d61443e_test.go.txt.
Run TestHaloRustControlClosureD61443e from the newer pin's grammar package.
Output: /private/tmp/halo-control-closure-d61443e.json.zlib.

3,427 direct cases compare native consumeVueC endpoints and LectureVueC
completion/refusal outcomes, including all five stop causes. There are 160
frame cases, each exercised through Rust's production and inference-view APIs.
Checks cover one publication per class-mode frame, preceding-view failures,
closed entries, extra trailing bytes, nonzero padding, truncations and loop limits.
A first generator attempt used NewWorld(nil) for an intentionally malformed NEW;
that test setup panicked. The final generator supplies an empty Registry, matching
Rust's empty-registry test setup, without changing production Go sources.

NativeControlVerdict is a decoder diagnostic, not an inferred action. Entries
are published only when the final view terminates with 0..7 zero bits remaining.
Original DecodedFrameView entries and fields remain available even on refusal.
Closed status alone cannot prove the source offset was a canonical record boundary.
Existing action/control fixtures independently compare all typed entry fields;
this fixture focuses on closure and observer orchestration.
