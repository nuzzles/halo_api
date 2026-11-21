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

### Alternatives

#### API Wrappers

- [Grunt API](https://github.com/PlayGrifball/grunt) is a C#/.NET client and HTTP gateway for the
  Halo Infinite APIs.
- [SPNKr](https://github.com/acurtis166/SPNKr) is a Python SDK for the Halo Infinite web API.

#### Theater Libraries

These projects serve different parts of Halo Infinite Theater replay work:

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
