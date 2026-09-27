# Native data-contract audit

Pinned LevelUp commit: `43a01721e8a02c87c955e175936f0ccf8dd97a81`.
This audits declarations in native `film/types`, separately from the algorithms
that populate them. It is not a claim of complete parser or event parity.

## Objective contracts

All declarations in `types/objectives.go` are represented:

| Native type | Rust type | Fields retained |
| --- | --- | --- |
| DeathInstant | StatborgDeathInstant | XUID, TimeMS |
| FlagSpan | FlagGrabSpan | State, StartMS, EndMS, XUID |
| FlagTrack | FlagGrabTrack | Team, ordered Spans |
| FlagGrabsNetPlayer | FlagGrabCounts | XUID, Raw, Net |
| PlayerLine | StatborgPlayerLine | XUID, Kills, Deaths, Assists |
| ScorePoint | StatborgScorePoint | TimeMS, Slot, signed Value |
| StatValue | StatborgValue | signed A/B/C/D, HasC, HasD |
| StatRecord | StatborgRecord | TimeMS, Slot, Round, component-keyed Comps |

Definitions live in statborg_identity.rs, flag_grabs_net.rs,
statborg_series.rs and statborg_rounds.rs. Flags remain independent of values;
no absent C/D value is silently treated as present. Native int coordinates and
counters use i64 except the inherently nonnegative Raw/Net counts, which use
usize. Native nil versus allocated-empty container distinctions are not a wire
field or part of this JSON-shape comparison; ordered elements and keyed values
are covered. Algorithm/captured-objective gaps remain separate manifest entries.

## Movement contracts

`types/grammar_mouvement.go` has two structs, five kind labels and four numeric
constants. MovementStateRead retains Slot, Kind, TimestampUS, Chunk, PacketIndex,
On and Progress. MovementStateStats retains all 17 fields, including Absent,
Scanned, publication/locating counters, binding/ambiguity counters, duplication,
velocity/episode/derived-jump counts and all three MapWidths.

The five MOVEMENT_* constants preserve the native labels, especially
`jumpDerived`. SPARTAN_JUMP_* constants preserve height0.85m, tolerance0.10,
rise speed0.5m/s and hold250000us. The native name RiseMinMS denotes metres per
second, not milliseconds. The existing jump derivation now references these
constants without changing its arithmetic or thresholds. These are pinned
reference derivation parameters, not a new claim that every inferred jump is
independently annotated or directly recorded.

## Ability contracts

All six declarations in `types/grammar_abilities.go` have storage mappings:

| Native type | Rust representation | Fields retained |
| --- | --- | --- |
| AbilityCharge | AbilityCharge | Slot, source Chunk/TimestampUS, PacketIndex, Emplacement, Charges, Low |
| AbilityChargeStats | AbilityChargeStats | Records, WithI56, Read, Unread, Armed, Absent, Scanned |
| AbilityImpulse | AbilityImpulse | Slot, source Chunk/TimestampUS, PacketIndex, Predicted |
| AbilityImpulseStats | AbilityImpulseStats | Records, WithI57, WithI59, Read, Unread, Tag1, Absent, Scanned |
| AbilityRank | BipedAbilityEmission through ability_ranks() | Slot, source Chunk/TimestampUS, PacketIndex, Counter, Rank |
| AbilityRankStats | BipedChannels.ability_stats (BipedChannelStats) | Records, WithI48 -> with_component, Read, Unread, Gated |

Loaded scans populate packet ordinals; unframed caller input retains an unknown
ordinal as `None`. `ability_emissions()` additionally retains the no-rank gate;
`ability_ranks()` applies the native publication filter. A missing rank is never
published as rank zero. Predicted and nonpredicted impulse reads stay distinct.
Absent and Scanned are independent booleans for both charge and impulse stats.

The narrower Rust values cover the produced wire domain: three energy slots,
seven-bit energy values split into high/low nibbles, a three-bit counter and a
six-bit rank. This does not promise arbitrary Go struct-literal interoperability
for negative indices/counts or out-of-wire-range integers. These contracts are
represented through typed projections, not identical default JSON layouts.

`corpus_tests::assert_biped_gameplay_parity` compares every field and counter of
all six contracts against native outputs. The previously recorded captured run
includes 33 ranks, 39 charges and four impulses (see DATA_CONTRACT_INVENTORY.md).
The independent channel/source-availability fixtures additionally compare rank
gates, counters and packet ordinals, including absent source. The 12-type
declaration fixture below covers only the two ability stats types; it must not
be cited as evidence for all six. This closes the stale storage-mapping notice,
not all ability algorithms, callback domains or whole-film acceptance gates.

## Evidence

`halo_rust_type_contracts_test.go.txt` fills the actual native types with 64
values apiece and serializes them through Go's JSON runtime. Twelve types / 768
instances and all nine movement constants are compared through Rust typed
round trips. Exact object equality catches omitted, renamed or extra fields;
nested flag spans, component maps and arrays are included. Fixture values are
representable contract samples, not independently annotated gameplay events.
Existing native movement sequence and velocity tests validate the unchanged
jump derivation separately. Both forms of evidence are required for their
respective claims; neither replaces captured objective-mode parity.

`types/doc.go` only documents the upstream Go package boundary. It has no runtime
declarations and does not require or authorize the queued Rust architecture change.
