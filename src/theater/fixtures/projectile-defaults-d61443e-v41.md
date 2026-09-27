# Projectile default-state v41 oracle

Additional reference commit: `d61443ef59268ad734355db8e9974f68db5ca6d0`.
Production readers: `default_state_ti41.go`, `TraverseEntity`, `DecodeFrameRecords` and `WalkKeyframeFullState`.

`halo_rust_projectile_defaults_d61443e_test.go.txt` produces 1,536 direct default-state cases. Both native param5 forms, three precision/scope modes, every alignment, varied MPP/world widths, all-zero/all-one controls, and deterministic random bits are represented. The 768 param5=true cases also become complete NEW records, with an independently checked component-mask boundary and list terminator. Native observer output is captured directly from the Go reader. Rust checks exact cursor/next byte, every contiguous raw source field, component observations, JSON round trip and rejection when the final source byte is removed. Every target kind and both position forms are exercised. Absolute-region observations are compared explicitly: the shared absAxisWFor helper records each region once, including for projectile positions. The earlier no-observation assumption was incorrect and has been removed.

`halo_rust_projectile_default_routing_d61443e_test.go.txt` supplies eight alignment cases each for configured NEW-default skipping and keyframe defaults. The former reads a seven-bit configured fallback; the latter reads no projectile default between its two positive/zero guards. Rust must preserve both routes. Param5=false is supported by the isolated grammar helper; it is deliberately not installed as the full-keyframe default deserializer because the pinned reference does not install it there.

Regeneration: copy these `.go.txt` files as `_test.go` into the additional reference grammar package, with `halo_rust_component_hooks_test.go.txt`. Run `go test ./internal/games/halo_infinite/film/internal/grammar -run 'TestHaloRustProjectileDefault.*D61443e' -count=1` from `apps/go-api`. Copy `/private/tmp/halo-projectile-defaults-d61443e.json.zlib` as the defaults fixture, and zlib-compress `/private/tmp/halo-projectile-default-routing-d61443e.json` as the routing fixture. Preserve the original reference pin and historical fixtures.

This covers native recording fields and routing, not resolved projectile trajectories or inferred actions. Full corpus migration and remaining native grammars are separate outstanding work.
