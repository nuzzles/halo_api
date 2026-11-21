# Film Option audit

Audit date: 2026-11-21. Scope: all public film data types, including registry settings,
players, replication, and catalog types. Settings decisions combine a registry corpus
audit with source inspection; replication and catalog findings come from their owning
parsers and bundled reference data. Parser failure, recorded absence, and missing
catalog knowledge remain distinct. Internal parser/lookup `Option` return types are
not public data-model nullability and remain where they signal failure or absence.

## Required catalog field

`LevelCalibration.api_id` is now `Uuid`: all 33 entries in
[`levels/data.rs`](../src/theater/film/catalog/maps/levels/data.rs) supplied an ID.
[`levels/mod.rs`](../src/theater/film/catalog/maps/levels/mod.rs) compares IDs
directly and tests non-nil, unique IDs, round-trip lookup, and lookup misses.
The lookup functions still return `Option` for unknown IDs/tags or mismatched
builds. This is a bundled-catalog completeness contract, not a claim that all
possible game levels have service IDs. `display_name` remains optional: 24 of
the 33 entries have no name.

## True recorded absence: retain

The public types live in
[`replication/event.rs`](../src/theater/film/replication/event.rs) and
[`replication/mod.rs`](../src/theater/film/replication/mod.rs).

| Fields | Owning parser evidence |
| --- | --- |
| Event handle arrays and damage victim/attacker/third references | `handles()` in `parser/v41/replication/event.rs` reads independent presence bits. |
| Event player slots, including kill victim/killer/assistant | `player()` reads an absence bit before the five-bit slot; these are not identity lookup failures. |
| Event weapon/item/effect tags, teleport tags, throw tag, projectile tags | `optional_tag()` reads a presence bit before each tag, including inside the tagged throw branch. |
| `Melee.extra` | Read only for `kind == 1`. |
| Projectile detonation/impact `variant`; detonation `optional18` | Separate wire gates, also preserved by `stream/frame/events/projectile.rs`. |
| Control input `short`, `analog`, `third_analog`, `extra`, `flags` | `stream/frame/controls.rs` gates the input group and its individual fields. Analog values are required only inside the present group. |
| Control `command_count` | `stream/frame/controls/commands.rs` distinguishes an absent block from a present block with count zero. |
| Control kind-2 `value` | Explicitly gated 13-bit value. |
| `RosterPlayer.option` in `film/players.rs` | `stream/roster.rs` admits formats 20/21/24/25/27. Format 20 omits the field; later admitted formats read five bits only when `OptionAbsent == 0`. |

Paths in the evidence column are relative to `src/theater/`; control and roster
paths beginning with `stream/` are relative to `src/theater/parser/v41/replication/`.
Event projection requires a complete layout and checks the body boundary.
Roster projection rejects incomplete snapshots. The inner `Option` returned by
wire helpers represents recorded absence; their outer `Option` represents read
failure. Requiring these public fields would discard valid wire states.

## Catalog absence and incomplete knowledge: retain

[`tag.rs`](../src/theater/parser/v41/replication/tag.rs) projects catalog values
into [`tag_reference.rs`](../src/theater/film/replication/tag_reference.rs).

| Fields | Meaning of absence |
| --- | --- |
| `ReplicationValue::Tag.reference` | None of the firing/projectile/damage/vehicle-bounds tables matches; the recorded ID remains available. |
| `FiringReference.magazine/projectile`, `FireStats.magazine/projectile` | No magazine or projectile relationship in the catalog; `physics/data.rs` includes tag `0x0000a4bc` with both absent. |
| `ProjectileReference.impact`, `ProjectileStats.impact` | No direct-hit effect link; `physics/data.rs` includes `0x003f48d6` with no impact. |
| `DamageReference.amount`, `DamageEffect.amount` | No damage numbers defined for that catalog effect, not a measured zero-damage film observation. |
| `DamageEffect.weapon`, replication `DamageCause::{Weapon,Melee}.weapon` | No single associated weapon; `damage/data.rs` includes melee `0x000027cc` and weapon effect `0x0000563f` without weapon IDs. |
| `TagInfo.name/item/parent/part_of` | Unnamed or unclassified entries, absent inheritance, or objects that are not vehicle parts. Concrete absent entries exist in `items/data.rs`. |
| `ModeOption.default` | No represented literal default; this includes nonliteral script defaults and does not prove that the script lacks a default. |
| `AmmoKind::Magazine.rounds_per_shot` | Uncertain interpretation of burst/event cadence; BR75 and Shock Rifle deliberately have no value. |
| `LevelCalibration.display_name` | Missing English reference metadata. |

Catalog data paths above are relative to `src/theater/film/catalog/`.
Keep `Option<TagLink<T>>`: `None` means no link, while
`Some(TagLink::Unknown(id))` preserves an existing link whose target details are
unavailable. Existing tag-projection tests cover this distinction. Optional
catalog lookup returns and unknown-weapon names likewise represent legitimate
lookup states, not unnecessary field wrappers.

## Settings and customization corpus counts

The registry audit covered 29 local `film.json` captures under
`halomagick_twin/apps/theater/fixtures`, plus the checked-in `bazaar_idle` and
`octagon_ar_kill` fixtures: **31 films**, all v41/format 27, HI_1_13_0. These
counts were collected before changing settings types. They establish the tested
admission contract, not universal guarantees about other builds or variants.

| Fields | Recorded absence | Decision |
| --- | --- | --- |
| Build, start time, variant name/description, engine type, minimum respawn time, play-of-game flag | 0/31 each | Required; parse error when missing or invalid. |
| Starting-loadout list; each loadout's weapon list | 0/31 each | Required vectors; explicitly empty lists remain valid. |
| Option list | 0/31 | Required vector. |
| Option name, eight-byte payload, decoded value | 0/363 each | Required fields; unknown discriminators get an explicit `Unknown` value, not `None`. |
| Starting-weapon tag | 59/93 | Enum retains 34 tagged and 59 unspecified slots. |
| Starting-weapon random flag | 93/93 | Enum selects `Random` only for an explicitly enabled flag; absent/false is not random. |
| Loadout equipment | 31/31 | Keep `Option<TagId>`: no equipment tag recorded. |
| Killcam, last-chance duration | 31/31 each | Keep optional; do not infer false or zero. |
| Score limit | 1/31 | Keep optional; absent in `octagon_ar_kill`. |
| Time limit | 4/31 | Keep optional. |
| Overtime duration | 12/31 | Keep optional. |
| Last-chance enabled | 24/31 | Keep optional. |

`StartingWeapon` now distinguishes `Real(TagId)`, `Random`, `RandomWithTag(TagId)`,
and `Unspecified`. The combination variant avoids discarding a tag if both independent
Bond fields are supplied; it is covered synthetically, not observed in this corpus.
`Unspecified` does not assert an empty runtime inventory. Missing and explicitly false
random flags both select the non-random branch; their encoding distinction is not exposed.

Removing `variant_decoded` follows from requiring complete, unambiguous variant
projection. `MatchSettings` no longer defaults during registry deserialization.
Missing/null required fields are errors; absent optional fields remain absent.
Bond writers can omit defaults, so requiring the core fields deliberately rejects
previously accepted films whose core metadata cannot be projected. No game defaults
are substituted. Optional paths distinguish missing data from malformed/duplicate fields.

Every initial table in the registry audit decoded all 32 customization blocks
(992 blocks, including vacant slots). `FilmCustomization.spartan` is required and
`raw` is removed. The supported appearance layouts remain HI_1_12_0/HI_1_13_0;
unrecognized layouts and invalid counts fail film parsing, including later roster/bot
projections and both threading paths. All-zero appearances remain valid typed records.

Reproduce the registry audit offline with:

```sh
HALO_FILM_CORPUS=/path/to/fixtures cargo test --offline --lib local_corpus_settings_audit -- --ignored --nocapture
```

The test includes the two checked-in fixtures automatically and handles both compressed
and already-inflated registry chunks. After the refactor, it validates the required
contract and reports remaining nulls rather than reconstructing the removed wrappers.
