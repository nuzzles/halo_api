# Native death-context oracle

Reference: JGtm/LevelUp feat/v75, commit
43a01721e8a02c87c955e175936f0ccf8dd97a81, replay/death_context.go.
Generator: ../reference/halo_rust_death_context_test.go.txt, installed in the
reference replay package and run with TestHaloRustDeathContext.

The fixture calls native ContextesDesMorts on 512 deterministic cases and retains
all input positions, ordered named/anonymous lives, bridge availability, index
disagreements, death-clock offset, external teams and journal deaths. Expected
outputs come from the pinned Go implementation, not Rust. There are 5,472
published contexts: 8,552 visible, 6,086 waiting and 324 out-of-sight teammate
classifications; 3,958 measured nearest distances and 56 empty-result cases.

Inputs include recycled slots, positions outside lives, anonymous lives, unknown
world coordinates, missing victim teams, stale positions, life endpoints, signed
clock offsets and repeated/equal timestamps. Full result order and every output
field are compared, including unavailable distance versus measured zero.

This is algorithm parity over constructed inputs. Journal deaths and team IDs
are external inputs; these results are derived analysis, not directly recorded
film facts. It is not an independently annotated captured action golden or
proof of the deferred Film/ResolvedFilm/playback design.
