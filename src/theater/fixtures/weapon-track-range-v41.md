# Weapon track chunk-range oracle (v41)

Reference: LevelUp feat/v75 at 43a01721e8a02c87c955e175936f0ccf8dd97a81.
Harness: ../reference/halo_rust_weapon_track_range_test.go.txt.

The fixture embeds captured bytes from maps/01-bazaar-idle/chunk-002-type-2.bin
and independently computed native BuildBipedTracks results. The same captured
chunk is deliberately copied to different numbered files; these are controlled
range-selection cases, not separate recordings or independent action annotations.

Source SHA-256: e3c9a688c5a7f0b82b699c0d0f8dd4c1c462016c3838071e652bc460c82dceb9.

Four directory layouts ([1], [2], [1,3], and files created in order [3,1,2])
are each tested with limits 0, 1, 2, 3. Zero means the contiguous numbered prefix;
positive limits explicitly request 1..n and tolerate missing chunks. The native
source sorts directory filenames numerically. A following keyframe may supply
the slot band but does not supply position samples. Complete ordered tracks and
error presence are compared, not just sample counts.

The source yields 250 position samples per included copy. The oracle includes
14 successful cases and two errors (no automatic prefix / no readable requested
chunk). Negative limits are not native oracle cases: the reference panics when
allocating a slice with negative capacity. Rust continues treating negative
limits as automatic selection, without reproducing that panic.
