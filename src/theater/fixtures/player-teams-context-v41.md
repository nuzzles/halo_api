# Player team context parity

Reference: LevelUp feat/v75, commit 43a01721e8a02c87c955e175936f0ccf8dd97a81.
Generator: ../reference/halo_rust_player_teams_test.go.txt.
Fixture: player-teams-v41.json.zlib.

The original 1,024 cases are byte-value identical after JSON parsing. The next
512 cases vary native keyframe header/size widths, calibrated/stub traversal
widths and corruption policy. Seed 41195 continues the original generator's
sequence. Actual lireEquipeDuRecord returns every direct expectation; actual
scanPaquetEquipes and publierEquipes return all aggregate counters/assignments.
There are 2,882 accepted and 1,732 refused new direct reads, including missing
guards, absent/mismatched registry components, domain refusals and truncation.

Rust compares all direct results and aggregate outputs. Native cases 1025,
1029 and 1031 additionally exercise real Film construction and JSON export;
these cases distinguish the effective context from default-profile results.
These are supplied profile contracts, not proof of alternate keyframe layouts
in captured v41 films or complete shared observer API equivalence.

## Complete team-walk attempts

The fixture now contains 2,048 cases. Every previous field in the first 1,536
cases was verified unchanged. The additional 512 place an observed EMP timer
after the team designator and before an unsupported component. Native
scanPaquetEquipes callbacks are captured directly. A separate native replay of
each recovered candidate captures its start/slot, WalkKeyframeFullState end,
independent team reading and callback vector. Its complete concatenated vector
is also checked against the actual public scan's callback stream.

There are 15,079 attempts, 2,914 callbacks in 336 cases, and 217 failed team
readings that still emit callbacks. Rust compares every ordered attempt and
callback, preserving full KeyframeRecord fields, stop reasons and diagnostics.
Raw-field semantics rely additionally on the independent keyframe/codec oracles;
this fixture independently verifies boundaries, team values and callbacks.
Source scanner attempts carry packet metadata and pre-filter packet ordinals;
payload-only attempts explicitly lack them. Film constructor tests verify the
source slice and complete Film serialization. Old exports default to no trace,
which is unavailable history rather than proof that no attempts occurred.

The harness uses ../reference/halo_rust_component_hooks_test.go.txt alongside
the team harness in the native grammar test package. An initial added mode-1
control accidentally appended a component to its absent-archetype registry;
that fixture-only mistake was corrected by retaining its empty component list.
All earlier 1,536 cases and other fields remain unchanged. Constructor cases
1537 and 1545 additionally verify nonempty callback retention through Film.
