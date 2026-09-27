# Bomb arming start-zero fallback boundary oracle

Pinned LevelUp commit: `43a01721e8a02c87c955e175936f0ccf8dd97a81`.
Generator: `../reference/halo_rust_bomb_start_fallback_test.go.txt`, registered in
`../reference/generate_oracles.py`. Expected outputs come from the native Go
`buildBombArmings` implementation, not Rust.

288 cases vary replay origin (-1, 0, 1, 300, 899, 900, 901, 2000 ms),
interval (0, 100, 300 ms), frame count (0, 1, 20), detonation suppression,
and a duplicate mirrored slot. Radial values 130, 160, 200, 254 occur at
0, 300, 600, 900 ms. Exactly 24 cases produce one fallback each; 264 produce
none. Compare complete armings, coverage, verdict (CV restored from IEEE bits),
and fallback counts. Controls include starts within range, ends outside range,
invalid clocks, suppressed fuse validation, and mirrored-slot deduplication.

This is synthetic direct-layer parity evidence. It does not establish a captured
positive bomb film, complete document publication, or independently observed
semantic action timing.
