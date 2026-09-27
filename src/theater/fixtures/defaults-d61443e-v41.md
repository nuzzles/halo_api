# Native v41 default-state correction

Reference: LevelUp `d61443ef59268ad734355db8e9974f68db5ca6d0`, additional to the original `43a01721e8a02c87c955e175936f0ccf8dd97a81` pin. Original fixtures are retained as historical evidence.

`halo_rust_defaults_d61443e_test.go.txt` independently invokes the Go production default-state readers for 3,008 cases (47 archetypes, 64 samples each, all eight alignments). The deterministic input sequence matches the original generator. Exactly 31 biped endpoints change, each by +32 bits. Every other endpoint is unchanged. The Rust test checks endpoints, contiguous raw source fields and truncated payload refusal, including both biped gate branches.

`halo_rust_default_references_d61443e_test.go.txt` independently captures the same readers' native reference publications. Version >=12 publishes the final guarded reference even when absent; 63 publication lists change. The final gate and optional 32-bit word are film data, not an external-reader side effect.

To regenerate, copy both generator files into the additional reference grammar package as `_test.go`, then run `go test ./internal/games/halo_infinite/film/internal/grammar -run 'TestHaloRust(DefaultReferencesD61443e|DefaultsD61443eParity)' -count=1` from `apps/go-api`. Zlib-compress `/private/tmp/halo-levelup-defaults-d61443e-oracle.json` and `/private/tmp/halo-default-references-d61443e.json` into the correspondingly named fixtures. Do not overwrite the original pin or fixtures.

`halo_rust_biped_default_boundary_d61443e_test.go.txt` adds 16 independent NEW-record cases: both final gates at all eight alignments. It uses the upstream independently written biped layout, then a component gate, empty sparse mask and list terminator. Rust checks the record and list endpoints, mask, native reference word and explicit absent reference. This verifies that the final word cannot become the following mask.

Projectile NEW defaults remain a separate pending migration. This fixture does not establish full film parsing or updated corpus parity.
