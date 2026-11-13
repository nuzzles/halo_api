# Packet configuration and shared handles

`reference.json` contains 49 cases through unchanged LevelUp functions at
`413971bcc485be53a2fa8cf229e170539a5146f8`. The exporter verifies five Git blobs
and extracts `readVarWidthInt`, `consume1408f0ac4Probe`, `varWidthRange`,
`varWidthBits`, `bitLen` and the three native constants. It uses the original
bit reader. The observation callback only collects the original published
reference structs; it does not change reads or return values.

Cases cover all nine categories, category 1's two probe values, absent/present
optional handles, direct handles, and minimum/maximum value/generation words.
Prefix padding makes each source end exact. Rust compares native values,
endpoints and optional-reference observations under configuration one, then
checks every source truncation under configuration zero.

The native Go shared reader assumes the active category table. It is **not** an
oracle for configuration-zero body widths. The pinned `varwidth.go` documents
the native `FUN_1406d3140` prologue: category 1 consumes its probe before testing
`DAT_144706104`; a disabled table uses `DAT_144706100` for every category. The
global range is not established by the recorded context here. Local parsing
therefore stops after an optional presence gate and category-1 probe, before
the unknown-width value/generation. These negative cases prove retained native
prefixes and refusal behavior, not successful configuration-zero handle decoding.

Public Film/replay tests carry the packet flag through NEW, DELTA, snapshot
defaults/components and control actions. They retain vacant slots, absent
handles, empty masks and complete records without affected fields. Unknown
widths produce an explicit stop and ContextUnavailable section status, prevent
guard/successor consumption, and cannot be repaired with opaque stubs. A known
NEW schema survives an incomplete body without becoming complete entity state.
The position reader's legacy precision-profile handle tail has a separate
regression, and fixed R15 posture words are verified to remain independent of
the category-table flag. Replay diagnostics retain incomplete control-view
reasons as well as record stops.

All 403,465 captured frame packets and 451 snapshot packets record configuration
one. Configuration-zero coverage is authored, not captured-film evidence.

Reproduce from the repository root:

```sh
GO_BIN=/path/to/go GOCACHE=/tmp/halo-go-cache python3 src/theater/fixtures/reference-configuration-v41/export_source.py /path/to/pinned/grammar > /tmp/reference-configuration.json
cmp src/theater/fixtures/reference-configuration-v41/reference.json /tmp/reference-configuration.json
```
