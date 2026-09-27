# Kill-position and opening-proxy oracle

Pinned reference: JGtm/LevelUp feat/v75 at
43a01721e8a02c87c955e175936f0ccf8dd97a81.
Generator: ../reference/halo_rust_kill_positions_test.go.txt.
Native functions: BuildKillPositions, BuildKillOpenings, ShiftKillRefs in
replay/killpos.go and replay/killpos_opening.go.

512 deterministic cases include 7,168 kill references, clock offsets, recycled
slots, multiple slots for one identity, anonymous lives, missing world positions,
empty positions/identity bridges, equal position timestamps and replication gaps.
Native ownersFromLives constructs the test registry. Rust constructs its registry
from the same lives and identity indices. Every ordered position, optional side,
shifted reference and report field is compared; complete outputs roundtrip JSON.

Native placement totals: 605 both, 680 killer-only, 1,280 victim-only and 4,603
dropped; 3,426 missing-bridge diagnostics. Opening totals: 399 both, 452
killer-only, 882 victim-only and 5,435 dropped; 3,435 missing-bridge diagnostics
and 512 sides rejected by the same-life check. Missing-bridge diagnostics overlap
the publication categories and are not added into their denominator.

Opening positions are a 1,500ms time-shift proxy, not recorded engagement starts
or first damage. Results retain the original kill time. The life filter uses
replication stays with the native five-second gap split and 120ms end tolerance.
This is reference algorithm parity, not independent annotation of actual fights.
