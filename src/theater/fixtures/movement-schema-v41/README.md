# Invalid schema declaration

Match `26903803-3f4d-4ff3-8d26-c90ea5699ea7` (Oddball, Live Fire),
replication chunk 13, packet 2254, decompressed payload byte 833439,
relative timestamp 258775387 us. This file is the complete 196-byte frame payload.

A misaligned walk encounters a supposed NEW at bit 1061 for full ID
1073742495, archetype 21. Its mask is 25770065923, but that registry
archetype has only 18 components. Although the permissive component reader
can finish the two in-range components, this cannot establish a schema.
The actual biped NEW for that ID is recorded at 728678769 us.

Payload SHA-256: `1ba355a65297c633256f52abedac8ece13f6eed37ff3b40eead6048fbc7d7970`.

## Bounded position with a reference tail

`recharge-attached-position.bin` is the complete 307-byte frame payload from
Strongholds on Recharge (match `fbb44479-2188-4170-af20-d51b4bfff646`), replication chunk 6, packet 1516, decompressed payload
byte 725434, relative timestamp 112593277 us. The biped record for full ID
1073742356 spans bits 822..1012. Its position spans bits 867..942: a bounded
14-bit vector per axis, handle 768/generation 1, region 1, and a two-bit tail.
The following player record starts at bit 1012. Omitting either the vector or
its reference tail misaligns the remaining player records.

Payload SHA-256: `c9b56927fd8b15bd82d24cdee9988bbcdb250e4cefb3a31025fc182d983b64d2`.

## Referenced device transform

`recharge-device-keyframe.bin` captures 149 bytes starting at bit 265048 of
the same match's replication chunk 1, packet 1 (type 2), decompressed payload
byte 343051. The device record is 1123 bits, followed by the next record's
64-bit header and five source bits. Its full ID is 1073743104, archetype 43.
`recharge-device-registry.json` preserves that archetype's 41 component names
and precision levels.

Category-0 references add 0x200 to their slot value, so handle 768/generation 1
refers to this device. Its full-state position is three little-endian f32 values;
the recorded orientation supplies the basis for bounded player positions.
The complete component walk must end exactly at the next device header,
full ID 1073743105. Field grammar follows LevelUp commit
`413971bcc485be53a2fa8cf229e170539a5146f8`, including `components_device_ti43.go`,
`orientation_frame.go`, and `equipement_porte_lecteurs.go`.

Payload SHA-256: `27d0a97b57e3f75e6c1cacdd11face12e0838b0717124a41619e66a72de96058`.
Registry SHA-256: `2fcbd3660b359b7d2b7e6dfdb1a441167cd48dd1c6f5e51f0f08347abcd87f90`.

## False successor during schema inference

`empyrean-false-successor.bin` is the complete 226-byte frame payload from the
ranked CTF Empyrean fixture, replication chunk 3, payload byte 650454, relative
timestamp 47055138 us. The accompanying registry JSON preserves archetypes 13
and 35 from that film.

Trying archetype 13 for the unbound slot 1604 at bit 1252 produces a complete
record ending at bit 1341. The following bits also decode as a complete biped
delta for slot 515. Both bodies fit their component masks, but this cannot be a
valid continuation: `FUN_142f2e174` emits deltas in increasing slot order, as
documented in the pinned LevelUp `ecrivain_invariants.go`. The regression rejects
this apparent successor instead of using it to establish a schema and fabricate
a second player update from an unrelated payload.

Payload SHA-256: `42f9542776c5e6ab3bd9c4d453540a4af19fe59167710561a7ed92e63724c263`.
Registry SHA-256: `76cd011387e3407b46f64c82566d30a177235be08178e8af74236b2586adda21`.

## Detonation before a player death

`argyle-detonation.bin` is the complete 578-byte frame payload from CTF Argyle,
match `28a419d8-3e13-431f-b8f8-f766546f2d7b`, replication chunk 41, payload byte
724626, relative timestamp 811025027 us. The projectile-detonation event starts
at bit 1 and ends at bit 140. Its position uses precision level 15 and map widths
14/14/16. A fixed 111-bit body incorrectly ends at bit 133.

The complete event list is types 5, 1, 85, 0 and ends at bit 535. The kill event
names victim 2, killer 4, and assist 1. The following entity/control frame ends
at bit 4623, with one padding bit. The victim's biped record (1073742500) contains
both a dead-state and control detachment; skipping it holds the previous life
until the respawn at 818938234 us.

Event grammar follows `vue_a_charges_armes.go` at LevelUp commit
`413971bcc485be53a2fa8cf229e170539a5146f8` (`FUN_1408096f8`, `FUN_1410f03b4`).
Detonation and impact positions use levels 15 and 12 respectively. Their optional
weapon, effect and direction fields also make fixed payload lengths invalid.

Payload SHA-256: `5add23630a8580e6678e04c75f97204be0ae5c0856580a164186ceb8e331ffb2`.
