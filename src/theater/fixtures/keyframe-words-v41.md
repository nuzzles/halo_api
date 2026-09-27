# Native keyframe size words

Pinned LevelUp: 43a01721e8a02c87c955e175936f0ccf8dd97a81.
194 native cases: 134 zero-source layout controls and 60 constructed guard
cases; 32 panic and 162 return normally. The constructed cases cross widths
1, 31, 32, 33, 64 and 65 with corruption checking on/off and five word pairs.
36 cases publish EMP callbacks. Rust compares endpoints, events, retained guard
values, component boundaries and payloads. The 65-bit words retain the leading
source bit discarded by the native numeric accumulator.

Negative and huge consumed widths are tested only at negative body positions,
where native panics immediately. Positive consumed 2^32-bit reads are not run
here; generic native bit-reader tests supply complementary evidence. Huge
header skips use tiny buffers, not multi-gigabit recordings.

Generator: reference/halo_rust_keyframe_words_test.go.txt. Validation status:
reference/keyframe-layout-validation.json.
