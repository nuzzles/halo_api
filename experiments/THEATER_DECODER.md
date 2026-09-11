# Upstream Theater decoder

> Active lab cleanup: 32 films are retained (31 replayable; Aquarius unresolved),
> including natural-end and later controls, Octagon gameplay, and the two Arena
> games, plus the hour-long raid. Forced-end recordings and standalone probes are
> archived under `archive/cleanup-2026-09-10/` at the experiments root. Historical
> 36-film/30-clip counts below describe earlier validation runs. See the main
> experiments README for current commands. Python field readers used by the
> byte inspector now live under `examples/legacy/`.


The supported parsing findings now live in [`halo_api::theater`](../src/theater/mod.rs).
The production path uses bounded byte reads and shared candidate scans, with no
Python subprocesses, expanded bit strings, filesystem calls, or corpus-specific
match IDs in the library. The original `clients::hi::film` imports remain compatible.

```rust
use halo_api::theater::{DecodeOptions, Film};

// chunks: &[halo_api::clients::hi::models::FilmChunkData], already decompressed
let film = Film::try_from_chunks(chunks, DecodeOptions {
    major_version: manifest.film_major_version,
    match_id: Some(match_id.to_string()),
    duration_us: None, // Or the manifest's duration in microseconds.
    retain_coverage: true,
})?;
let json = serde_json::to_vec(&film)?;
```

`DecodeOptions::v41()` supplies defaults. Use the actual manifest version when
available. Only v41 and its checked registry layout are supported. Unsupported
versions, duplicate chunk indices, truncated packet framing, inconsistent life
identities, and mismatched registries return typed `DecodeError` values.
Unsupported record forms stay unparsed, rather than causing guessed skips.

## Decode and replay one film

From the repository root:

```sh
cargo run --release --manifest-path experiments/Cargo.toml \
  --example decode_theater_film -- \
  experiments/films/weapons/03-br-sniper-zoom

node experiments/examples/build_decoded_replay.cjs \
  experiments/films/weapons/03-br-sniper-zoom/decoded-film.json

open experiments/films/analysis/theater_viewer.html
```

The Rust example reads `film.json` and its decompressed chunk files from the
selected folder. It writes `decoded-film.json` there by default. An optional
second path chooses another output file; `--compact` omits packet indexes and
aggregate checked-region diagnostics, while retaining each sample's source.
It reports loading, decoding, and JSON-writing times separately.

The portable file is **typed JSON** rather than CSV: player lives, independent
sample streams, source locations, optional values, and projectile tracks survive
without flattening or inventing defaults. Theater Lab's **Open decoded film**
file picker can open that JSON directly. The Node builder uses the same
presentation adapter and embeds the replay into a standalone offline HTML file.
It does not parse recording bytes or consume the historical Python exports.

Build a replay from all existing `decoded-film.json` files:

```sh
node experiments/examples/build_decoded_replay.cjs --corpus experiments/films
```

To decode all cached films first, run from `experiments/`:

```sh
cargo build --release --example decode_theater_film
for metadata in films/*/*/film.json; do
  target/release/examples/decode_theater_film "${metadata%/film.json}" --compact
done
node examples/build_decoded_replay.cjs --corpus films
```

The existing `python3 examples/theater_lab.py` server also serves the generated
HTML at `/replay`. Its detailed bit inspector remains a research tool using the
historical field annotations. Rebuilding those annotations is unnecessary for
the new decode/export/replay workflow.

## Typed observations and limits

`Film` contains the component registry, roster, lives, summary events, clocks,
player streams, projectile tracks, and diagnostics. XUIDs serialize as strings.
Sample timestamps are integer microseconds since the earliest nonzero packet
timestamp; summary timestamps use the recorded millisecond timeline. Each
sample is associated with a life ID: `wire + 256 * (generation - 1)`.

The decoder consolidates:

- Player-level armor appearance snapshots: 22 base region/permutation pairs,
  visor/color identifiers, armor variant, coating StyleId, five attachment TagIds,
  and four mythic-effect identifiers. Optional new fields preserve unknown status
  when reading older exports. These bind by
  recorded XUID/name independently of lives. No live-account defaults are used.
  See [FILM_APPEARANCE.md](FILM_APPEARANCE.md) for the controlled coating comparison.
- Position, desired aim, analog axes, and checked input-chain boundaries.
- Held/released crouch input from guarded command tails, including terminal
  roster-0/1 records across respawns, both observed header tags, and combined
  jump/crouch forms in Octagon gameplay. Other button forms and larger rosters
  remain unsupported; physical posture and slide state are not inferred.
  See [FILM_POSTURE.md](FILM_POSTURE.md).
- Sparse position/aim/tick prefixes and independent body/shield windows. Packet
  grammar is checked independently of coordinate bit widths and gameplay category.
- Spawn coordinates, roster identity, generations, death boundaries, and clock
  transitions associated with surviving-player round resets. The alternative
  spawn offset and coordinate widths are selected by guards, without player or
  match ID special cases.
- The hour-long raid checks all four two-bit generation tags: wire values
  `01/10/11/00` map to logical generations `1/2/3/4`. Spawn record flags `00/01/10`
  are separate from the wire ID. Two observed coordinate suffixes preserve the
  same axis boundary. A complete repeat of a wire/tag pair is still rejected.
  See [FILM_RAID.md](FILM_RAID.md).
- Guarded firing, melee with its companion, grenade throws, reload starts,
  magazine observations including the short zero form, selected slot, and scope
  stages. Duplicate firing and events outside a checked life are excluded.
- Multiplayer magazine deltas can continue into a checked neighboring pawn.
  Literal quantities 0–36 and paired 11-bit inventory fields support recorded
  shots and refills: 2,306 magazine samples in Oddball, 1,071 in Ranked Bandit,
  and 16,288 in the raid. Inventory values and energy-ammo forms remain unexported.
  See [weapon evidence](FILM_WEAPONS.md#current-upstream-multiplayer-support).
- Weapon windows from firing even without magazine data. `Firing::weapon_slot()`
  and `weapon_fingerprint()` split checked slot/guard/key/guard fields; 17,311
  independent same-packet magazine associations agree. BR75 is recognized in
  either carried slot, and both observed component-42 selection directions are
  supported. Spawn references calibrate names but do not export initial inventory.
  See [weapon fingerprints](FILM_WEAPONS.md#weapon-fingerprints-and-both-selection-directions).
  Observed weapon-set updates
  invalidate held identity, including component-42 forms with unknown payloads.
- Stalker control `daf193c7` confirms firing identity independently of ammo. A
  checked nine-bit component-35 `weapon-state-overheated` form allows its combined
  primary-selection record to be read. Upper values 0–21 with trailing `00` are
  supported; other forms remain unknown. The field is not exported as heat or ammo.
  See [Stalker evidence](FILM_WEAPONS.md#stalker-rifle-firing-control).
- Victim-bound weapon-hit prefixes with an immediately preceding guarded firing
  record. These do not supply a damage amount, health value, or descope transition.
- Both controlled grenade paths, including recorded spawn/position/terminal
  locations. Single-player/life, release timing, sample-count, and coordinate
  continuity checks must all pass. Ranked paths remain unsupported.
- Existing registry, roster, packet, and kill/death/medal summary decoding and
  API aggregate-validation helpers, preserved under the compatibility API.

`CoordinateLayout` now names the three observed encodings: `X15Y15Z17`,
`X17Y17Z16` (Bazaar), and `X18Y18Z15`. `axis_bits()` returns the widths; position
offsets derive from them. The two former JSON names, `Controlled` and `Ranked`,
remain accepted as deserialization aliases. New exports use the numeric names.
Spawn prefixes alone do not determine widths: Bazaar shares the Forge-control
prefix but has 50 coordinate bits instead of 47. Supported suffix boundaries
select the observed tuples; general map-bound/precision metadata is still unknown.
See [FILM_COORDINATES.md](FILM_COORDINATES.md) for captured evidence and limits.
The idle Aquarius capture has changed spawn flags and a 36-bit candidate window;
its per-axis split is unresolved. It deliberately does not add another enum
variant or fabricate a spawn position. The replay builder skips films with no
decoded positions, retaining their partial Film JSON and diagnostics for inspection.

`SourceSpan` identifies a **checked window**, not necessarily the exact scalar
bits or the entire record. Offsets are relative to decompressed packet payloads;
bit ranges are MSB-first and half-open. `diagnostics.checked_regions` distinguishes
accepted structure from unparsed data. Windows may overlap and can contain
opaque fields. Their union is not a percentage of semantically decoded bytes.
Raw chunk bytes remain with the caller.

No initial aim, health, ammo, or scope is invented. Spawn coordinates are real
observations, even in idle films. Raw coordinate units, angular quantization,
health scales, full inventory/reserves, complete damage frames, grenade type,
Ranked trajectories, map geometry, and complete record-chain grammar remain open.
Scope stages are not magnification multipliers. Firing events are not an exact
bullet count; reload cause and animation duration are not decoded.
Scope values remain recorded samples; hits never synthesize an unscoped value.
The separate [settings lookup](FILM_SETTINGS.md) retrieves the match's saved
variant and engine revision, without changing the offline decoder or replay.

## Verification and measured performance

Measured locally on 2026-09-10, release mode, after compilation:

| Work | Time |
| --- | ---: |
| Decode all 36 cached films | 6.79 s |
| Write their compact typed JSON files | 0.38 s |
| Decode three-round Ranked Oddball | 2.10 s |
| Decode eight-player Bandit EVO | 1.17 s |
| Decode BR/S7 scope control | 0.11 s |
| Decode the 65:53 raid (20 players, 942 lives) | 9.06 s |

These are local timings, not a cross-machine guarantee or a measured speedup
ratio against Python. The original native replay included **36 clips, 458,516 positions,
362,112 aim samples, 130,112 input samples, 4,986 firing events, 105 melees,
362 throws, 344 reloads, 23 magazine values, five selections, 1,414 scope
observations, 2,010 body values, and 178,472 shield values**. Each controlled
grenade path has 87 positions.

The historical composed replay had 30 clips and 458,491 positions. The additional
25 positions in the native replay are checked controlled spawns that the old
presentation omitted; six formerly absent idle/selection clips now have spawn
positions. Native output does not copy the old Octagon presentation's reported
stationarity or initial facing into decoded observations.

`check_decoded_parity.py` compared **1,139,015 observations/lifetimes** against the
saved independent exports across all 36 films, including exact times, values,
lives, weapons, vitality, and projectile samples. It does not regenerate them.
Small captured records now exercise the Rust decoder without downloading films.
The eleven new focused tests cover captured control/Ranked records, bounded
malformed inputs, and pipeline/JSON round trips. The current suite has **33
Theater tests**, including captured terminal input records, crouch/jump
combinations, raid spawn flags and generation tags, and roster binding across
respawns; it ran in 1.08 s locally.
Compilation is separate.

From the root:

```sh
cargo test --lib theater
cargo check --lib --target wasm32-unknown-unknown
python3 experiments/examples/check_decoded_parity.py
node experiments/examples/check_decoded_replay.cjs
```

The last two checks require the local corpus/exports and generated replay.
The browser check covers JSON file import, all scope stages, empty magazine and
reload, body/shields, recorded grenade movement, death/respawn and reverse
seeking, reused life IDs, and mobile layout. The older broad Python probes and
browser checks remain available to reproduce historical research; they are not
part of the normal Rust decoder test loop.

The upstream library builds for `wasm32-unknown-unknown`, and CI checks that target.
Native Xbox password login is enabled only on native targets; browser futures
use local async trait futures. The offline film decoder itself needs no Tokio
runtime, browser bindings, HTTP client, or filesystem to execute.
