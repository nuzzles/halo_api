# Captured v41 vitality components

These six byte slices come from the Twin Octagon fixture, match
`9a875c4e-03bc-4fff-a688-e21216d1618b`. `evidence.json` records the original
chunk index, decompressed chunk SHA-256, packet payload offset, component bit
range, entity, time, fields and slice hash. Slices preserve surrounding byte
alignment, without repacking or synthesizing component bits. `start_bit` is
relative to the slice; source offsets are relative to the original packet.

Selected by complete sequential component reads from an owned biped. Expected
quanta/flags are reread from captured bytes; decoded scalar expectations follow
LevelUp `413971bcc485be53a2fa8cf229e170539a5146f8` vitality.go and
quantize_endpoint.go. These are source regression fixtures, not independent
video annotations or proof of complete decoding. The recovery sample retains
its flag values without claiming their independent semantic validation.

Run `cargo test --locked --offline theater::runtime::replay::vitality`.
