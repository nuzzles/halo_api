# halo_api

[![CI](https://github.com/nuzzles/halo_api/actions/workflows/ci.yml/badge.svg)](https://github.com/nuzzles/halo_api/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/halo_api.svg)](https://crates.io/crates/halo_api)
[![docs.rs](https://docs.rs/halo_api/badge.svg)](https://docs.rs/halo_api)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

Unofficial Halo Infinite REST API client for Rust: CSR/rank lookups, service records, match history, and more.

> [!IMPORTANT]
> This is an unofficial, community-maintained library. It is not affiliated with, endorsed by Microsoft.

## What this crate does

- Separates authentication (`HaloAuthClient`) from Halo API operations (`HaloInfiniteClient`).
- Acquires and caches both the Spartan token and Waypoint flight clearance.
- Covers stats, skill, profile, UGC, progression, career rank, reward tracks, ban, and privacy endpoints.
- Reads public HCS leaderboard gamertags and points through `WaypointClient::hcs_leaderboard`, without Xbox authentication.
- Filters service records by season, playlist, mode, and ranked/social via `ServiceRecordFilter`.
- Paces requests per Halo Waypoint origin so bursts don't trip throttling; configure via
  `HaloInfiniteClient::builder()`.
- Automatically invalidates and retries once on an expired/unauthorized (401) response, instead of surfacing a
  hard failure the caller has to handle manually.

## Feature flags

| Feature | Default | Description |
| --- | --- | --- |
| `xbox` | No | Adds `from_xbox_xsts_source` and `XboxXstsTicketSource` auth convenience from the [`xbox`](https://crates.io/crates/xbox) crate. |

If your application already obtains Halo Waypoint credentials, use
`HaloAuthClient::from_tokens(spartan_token, clearance_token)` instead. Those values remain private to the
client, but the caller is responsible for replacing the client when they expire.

## Quick start

```rust,no_run
use halo_api::clients::hi::models::PlaylistId;
use halo_api::auth::HaloAuthClient;
use halo_api::clients::hi::{HaloInfiniteClient, Player};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let auth = HaloAuthClient::from_tokens("spartan-token", "clearance-token");
    let halo = HaloInfiniteClient::new(auth);

    let csr = halo
        .playlist_csr(PlaylistId::RANKED_ARENA, &Player::gamertag("Some Gamertag"))
        .await?;

    println!("{csr:?}");
    Ok(())
}
```

### Offline Theater data

`Film::parse` reads downloaded chunks without API calls. `film.registry.settings` exposes
recording/variant metadata and a required `mode_options: ModeOptions` containing the mode rules,
starting loadouts, and named script options. `film.registry.players` is a `Vec<InitialPlayer>`
containing the complete initial 32-slot player table, including vacant slots. An unreadable table returns
`ParseError::IncompleteRegistryPlayers`. Core settings and their lists are required; missing or
malformed required fields return `ParseError::InvalidMatchSettings`. Genuinely omitted settings
such as score/time limits remain `None`, rather than acquiring game defaults.
Round limit, rounds to win, rounds to draw, and machinima controls stay optional when omitted.
Self-destruction and betrayal respawn penalties are required by the supported-capture
contract; explicit zero penalties and disabled controls are preserved.
`recording_start_time` is a `chrono::DateTime<Utc>` decoded from the recorder's timestamp
(not necessarily the exact match start), serialized as RFC 3339 on native and WASM.

Starting weapons are `StartingWeapon::{Real(WeaponTagId), Random, Unknown}`.
Starting equipment is `StartingEquipment::Real(EquipmentTagId)` or `StartingEquipment::None`
when no tag is recorded. Unfamiliar equipment tags remain `Real`; malformed records fail parsing.
`Unknown` preserves a recorded slot with no tag or enabled random selection; it does not
resolve the runtime's default weapon. Match options have required typed names:
`MatchOptionName::Known(KnownMatchOption)` from the catalogue or
`MatchOptionName::Unknown { name_hash }`, preserving an unmatched hash. Known names serialize
using their game script spelling (for example, `SlayerOptions_scorePerKill`). Values remain
recorded booleans or exact float bits, with `MatchOptionValue::Unknown` for unfamiliar
discriminators; catalogue defaults are never substituted.

Player appearance is also recorded in timestamped roster packets. Initial player-table
appearance blocks can be empty even when later roster blocks are populated:

```rust
use halo_api::theater::{Film, film::ReplicationChange};

fn inspect_appearances(film: &Film) {
    for observation in &film.replication_stream.observations {
        if let ReplicationChange::Roster { players } = &observation.change {
            for entry in players {
                let spartan = &entry.player.customization.spartan;
                println!("{} / {} at {}us: coating hash {:08x}",
                    entry.player.gamertag, entry.xuid, observation.timestamp_us,
                    spartan.coating_name);
            }
        }
    }
}
```

Appearance fields include region/permutation pairs, variant/style, theme/coating, attachments,
objects, and action pose. They are recorded hashes—not endpoint inventory paths, named armor
slots, meshes, textures, or animation clips. Typed appearance currently recognizes the
1,852-byte HI_1_12_0/HI_1_13_0 layout. `customization.spartan` is required; unsupported or
invalid appearance layouts fail film parsing with `ParseError::InvalidCustomization`.
Raw appearance bytes are not retained in the parsed film.

`ReplicationChange::Bots` exposes recorded bot metadata. `ReplicationChange::Event` includes
typed reload, throw, melee, pickup, zoom, impact, and detonation payloads. Event player
slots use `EventPlayerSlot`, and weapon asset tags use `WeaponTagId` (distinct from the
family/variant `WeaponId`). Only damage attacker references and attacker-player slots remain
optional in these payloads. Meaningful victim/attacker/shooter references are retained without
resolving them to entity IDs; unexplained references stay internal. Projectile payloads retain
only world-space `position: [f64; 3]` in native engine XYZ axes and units. Both default-box
and map-region encodings decode to quantization-bin centers; map regions use the exact
recorded build/level's catalogued bounds. Unobserved replication kill/teleport payloads and other unsupported payloads
retain only their event kind in `Other`; summary kill highlights are unaffected.
Uninterpreted event fields, opaque player blocks, and duplicate raw metadata are not exposed.
Slots are not resolved to roster XUIDs. Position encoding details remain internal.
To advance recorded observations to a time, use `FilmRuntime`; the parser does not construct
resolved gameplay timelines or interpret projectile attachment geometry.

### Alternatives

Other projects cover Halo API reverse engineering, film decoding, and replay:

- [OpenSpartan](https://github.com/openspartan) has many Halo API reverse-engineering examples.
- [Grunt](https://github.com/PlayGrifball/grunt) is a C#/.NET client and HTTP gateway for the
  Halo Infinite APIs.
- [SPNKr](https://github.com/acurtis166/SPNKr) is a Python SDK for the Halo Infinite web API,
  including film highlights.
- [libfilmshell](https://github.com/OpenSpartan/libfilmshell) is a Rust library for extracting data
  from Theater films.
- [halo-infinite-film-decoder](https://github.com/JGtm/halo-infinite-film-decoder) turns film chunks
  into a 2D replay document with players, positions, kills, weapons, and objectives; it does not
  download, store, or render films.
- [installation-08's infinite-theater-3d](https://github.com/llampwall/installation-08/tree/main/cmd/infinite-theater-3d)
  is a 3D Theater replay viewer.

## Benchmarks

See the [film parsing benchmarks](benchmarks/README.md) for the methodology and
the [published results](https://nuzzles.github.io/halo_api/) for the comparison chart.

## MSRV

This crate has a [Minimum Supported Rust Version (MSRV)][MSRV] of 1.96.

[MSRV]: CHANGELOG.md

## License

Licensed under either of

- Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license
   ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option

## Contribution

See [CONTRIBUTING.md](CONTRIBUTING.md).

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
