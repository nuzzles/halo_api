# Native replay input retention audit

Reference: LevelUp `43a01721e8a02c87c955e175936f0ccf8dd97a81`,
`replay/film_inputs.go`, audited 2026-09-24.

This maps all 44 native `FilmInputs` fields to the current Rust storage and
assembly paths. It is a wiring inventory, not a claim that every branch has
passed complete captured-film parity. The native structure itself contains
derived scanner results; this inventory does not establish the future faithful
`Film` contract. That refactor remains queued in [NEXT_PHASE.md](NEXT_PHASE.md).

Paths below are relative to `src/theater`. Fields listed without a type prefix
are fields of the current `Film`. Original chunks remain an explicit input to
the player/document builders for identity evidence. They cannot currently be
replaced by a serialized `Film` alone.

| Native fields | Rust retained input | Assembly path |
| --- | --- | --- |
| FilmMajorVersion | `major_version` | `replay_document_film.rs`, version guard and coverage |
| Translocations | `translocations` | `replay_translocations.rs` |
| Positions | `biped_positions`, accepted records and companions | `replay_from_film.rs`, `replay_combat.rs` |
| BipedCreations | `biped_creations.records` | `replay_from_film.rs`, identity registry |
| Fire | `fire_events` | `replay_from_film.rs`, `replay_combat.rs` |
| Loadouts | `keyframe_loadouts` | `replay_combat.rs` |
| WeaponChanges | `weapon_changes`, independent `weapon_changes_error` | `replay_weapon_changes.rs`, objective carry consumers |
| Pickups, PickupStats | `pickups` records and stats | `replay_pickups_film.rs` |
| Inventory | `keyframe_inventory.records` | `replay_inventory.rs`, `replay_combat.rs`, `replay_abilities_film.rs` |
| InventoryDeltas, InventoryDeltaAmmoRefused | `inventory_deltas.records`, `.stats.ammo_refused`, independent `inventory_deltas_error` | `replay_combat.rs`, inventory publication |
| AbilityRanks | `biped_channels`, `ability_ranks()` projection | `replay_abilities_film.rs` |
| EquipmentChanges, EquipmentChangeStats | `equipment_changes.assembly` and scanner diagnostics | `replay_equipment_changes.rs`, `replay_equipment_placements.rs` |
| CamoStates | `biped_channels` camouflage reads and independent `camo_error` | `replay_equipment_episodes.rs` |
| GrappleReads | `ability_states.grapple` | `replay_grapple.rs` |
| AbilityImpulses, AbilityImpulseStats | `ability_states.impulses`, `.impulse_stats` | `replay_abilities_film.rs` |
| AbilityCharges, AbilityChargeStats | `ability_charges.records`, `.stats` | `replay_abilities_film.rs` |
| MovementStates, MovementStateStats | `movement_states.records`, `.stats` | `replay_stances.rs` |
| ZoomEvents | `zoom_events` | `replay_from_film.rs`, scope lookup on raw life spans |
| Placements, PlacementStats | `equipment_placements.placements`, `.stats` | `replay_equipment_placements.rs` |
| SpawnEvents, SpawnStats | `equipment_spawns.records`, `.stats` | `replay_equipment_placements.rs` |
| Pads | `ground_weapon_creations`, `equipment_pad_creations`, `ground_object_tracks`, `world_object_keyframes` | `replay_ground_film.rs`; power-up creation scan is independent of confirmed placements |
| Vehicles | `native_vehicles` | `replay_vehicles.rs`, `replay_combat.rs` |
| FlagMarks | `carrier_marks` | `replay_flags_film.rs`, flag mode gate |
| FlagGauge, FlagGaugeScanned | `managed_properties.reads`; recognized flag consumer supplies scan flag | `replay_flags_film.rs` |
| ZoneReads, ZoneScanned | `managed_properties.reads`; nonempty zone catalog supplies scan flag | `replay_zones_film.rs` |
| BombReads | `navpoint_radial` | `replay_bomb_armings.rs`, `replay_bomb_film.rs` |
| Grenades | `grenade_throws.records` | `replay_combat.rs` |
| Projectiles | `native_projectiles.tracks` | `replay_combat.rs` |
| Deaths | `ReplayIdentityEvidence.deaths`, scanned from original chunks | `replay_evidence.rs`, `replay_from_film.rs` |
| PlayerIndices | `ReplayIdentityEvidence.replication_indices` | `replay_evidence.rs`, guarded by nonempty deaths, roster union and injectivity check |
| FilmTable | `player_table` projected to `ReplayFilmPlayerTable` | `replay_from_film.rs`; raw decoded table remains in `Film` |
| PlayerTeams, TeamScan | `player_teams` | `replay_from_film.rs`, `replay_players.rs` |
| FilmClockOriginUS | `ReplayIdentityEvidence.film_clock_us` | `replay_clock.rs`, `replay_from_film.rs`, `replay_bomb_film.rs` |

## Clock findings

`Film.origin_timestamp_us` is not used as the replay highlight-clock origin.
`scan_replay_clock_origin` selects manifest chunk **number** 1 and reads its
first packet. `scan_loaded_replay_clock_origin` supplies the same operation for
`FilmSource`, including native metadata-free positional lookup. Neither chooses
the minimum timestamp across the film, skips a zero timestamp, nor falls through
an empty duplicate entry to a later chunk.

`resolve_replay_origin_ms` subtracts that packet timestamp from the first sorted
position timestamp. Zero clock or position before clock withholds the origin.
With at least five matched deaths, disagreement greater than 1000 ms with the
death-clock control also withholds it; equality at the tolerance is accepted.
Missing origins remain `None` in the document and false in origin coverage.
Layer-specific zero fallbacks do not establish an origin.

Evidence: `replay-clock-v41.json.zlib` compares native grid/origin arithmetic;
`replay-evidence-v41.json.zlib` compares automatic identity/clock scans;
`chunk-bridge-v41.json.zlib` compares first-packet lookup for 512 sources with
missing, short, long, reordered and duplicate metadata. The loaded clock check
uses the native lookup result as its expectation, not Rust-generated output.

## Acceptance still open

### Identity evidence ownership

`build_film_replay_players` returns the complete `ReplayIdentityEvidence` in
`FilmReplayPlayers.evidence`, including deaths, injective replication indices,
collision counts, clock origin, and the three named scan errors. Both structures
support serialization. The document builder consumes these values but its
native document schema does not archive this intermediate evidence. Callers
needing it should retain the player-layer output separately.

The scanner depends on `FilmReplayPlayerOptions.roster_xuids`: it scans indices
using the union of that roster and death identities, only when deaths exist.
Consequently an unconditional evidence cache on `Film` would conflate distinct
caller inputs. No such cache was introduced during this audit. This is current
parity ownership, not the deferred canonical Film source-preservation contract.

The automatic identity oracle contains 1,024 cases: 742 with deaths, 682 with
additional rosters, 92 total index collisions, and 282/26/174 death/index/clock
errors. Its Rust comparison now also checks complete evidence JSON roundtrips;
native field/error comparisons remain the independent decoding expectation.

All native input names have a retained source and consumer, but this alone does
not prove exact scanner invocation/error/profile behavior. In particular,
`FlagGaugeScanned` and `ZoneScanned` describe native consumer enablement even
when the shared scan fails; they are not successful-decode claims. Rust retains
the shared scan error separately.

Continue the source-by-source profile and observer audit, including integrated
`PosCaptureHook` routing, and obtain positive complete captured v41 CTF, VIP and
bomb comparisons. Existing per-field evidence in `gameplay-parity.json` is
historical and must be refreshed individually before upgrading its statuses.
Native cache codecs are derivative artifacts and need separate classification;
this table does not imply byte-for-byte cache or film re-encoding support.
See [FILM_FACTS_CACHE_SCOPE.md](FILM_FACTS_CACHE_SCOPE.md) for the checked cache
omissions, placement-field mapping and still-open full profile snapshot contract.

## Complete captured CTF witness

`fixtures/ctf-document-v41.json.zlib` embeds the complete eight-chunk v41
Husky Raid CTF recording from the reference's API wire integration tests. Its
baseline document matches the native parser with the flag consumer enabled and
explicit API player-total inputs. API team scores are withheld from assembly and
independently verify the capture total. See the adjacent fixture Markdown for
provenance and limitations. This adds positive complete CTF evidence; it does not
close VIP/bomb or map-spawn geometry gates. The decoded-kill companion now also
compares the entire enriched document, all 20 kill records, roster and health/pass
statistics. API totals independently agree with 20 kills and three captures.

The catalog-backed CTF companion additionally compares both flag teams, every
home/carried span and coordinate, two selected spawns and four team births, using
byte-identical native/Rust objective catalogs. This completes that CTF witness's
flag-spawn path alongside its baseline and decoded-kill cases. It does not prove
browser rendering, map meshes, or independent spatial/action-time ground truth.
