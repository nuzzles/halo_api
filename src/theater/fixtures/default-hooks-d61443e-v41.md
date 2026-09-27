# Biped default-hook fixture migration

Reference: `d61443ef59268ad734355db8e9974f68db5ca6d0`. The five `halo_rust_default_hook_*_d61443e_test.go.txt` generators retain their previous deterministic seeds/inputs and invoke the new production Go readers. Original fixtures are preserved. New fixtures replace the active Rust expectations; no expectation is generated from Rust.

| Fixture | Cases | Changed rows |
| --- | ---: | ---: |
| reads | 3008 | 63 |
| frames | 512 | 58 |
| keyframes | 256 | 23 |
| chain | 512 | 61 |
| repair | 512 | 266 |

Changes include the final biped gated word and its explicit present/absent reference, plus downstream read/repair effects. Row counts alone do not prove agreement.

Run the five generators with `go test ./internal/games/halo_infinite/film/internal/grammar -run 'TestHaloRustDefaultHook.*D61443e' -count=1`. Helpers: `halo_rust_component_hooks_test.go.txt`, `halo_rust_components_test.go.txt`, `halo_rust_movement_recovery_hooks_test.go.txt`. Output files are `/private/tmp/halo-default-hook-{reads,frames,keyframes,chain,repair}-d61443e.json`; zlib-compress them into the matching fixtures. Extra `_test.go` files never modify pinned production sources.
