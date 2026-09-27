# halo_api

[![CI](https://github.com/nuzzles/halo_api/actions/workflows/ci.yml/badge.svg)](https://github.com/nuzzles/halo_api/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/halo_api.svg)](https://crates.io/crates/halo_api)
[![docs.rs](https://docs.rs/halo_api/badge.svg)](https://docs.rs/halo_api)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

Unofficial Halo Infinite REST API client for Rust: CSR/rank lookups, service records, match history, and more.

> [!IMPORTANT]
> This is an unofficial, community-maintained library. It is not affiliated with, endorsed by Microsoft.

## What this crate does

- Decodes supported v41 Theater films into typed, serializable player/life/event streams with `theater::Film`.
- Separates authentication (`HaloAuthClient`) from Halo API operations (`HaloInfiniteClient`).
- Acquires and caches both the Spartan token and Waypoint flight clearance.
- Covers stats, skill, profile, UGC, progression, career rank, reward tracks, ban, and privacy endpoints.
- Filters service records by season, playlist, mode, and ranked/social via `ServiceRecordFilter`.
- Paces requests per Halo Waypoint origin so bursts don't trip throttling; configure via
  `HaloInfiniteClient::builder()`.
- Automatically invalidates and retries once on an expired/unauthorized (401) response, instead of surfacing a
  hard failure the caller has to handle manually.

This crate depends on the [`xbox`](https://crates.io/crates/xbox) crate for Xbox Live authentication (XSTS
tickets and XUID resolution). [`HaloAuthClient`] acquires and refreshes all Halo Waypoint credentials.

If your application already obtains Halo Waypoint credentials, use
`HaloAuthClient::from_tokens(spartan_token, clearance_token)` instead. Those values remain private to the
client, but the caller is responsible for replacing the client when they expire.

## Quick start

```rust,no_run
use std::sync::Arc;

use halo_api::clients::hi::models::PlaylistId;
use halo_api::auth::HaloAuthClient;
use halo_api::clients::hi::{HaloInfiniteClient, Player};
use xbox::auth::LegacyPasswordProvider;
use xbox::XboxClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let xbox_client = Arc::new(XboxClient::new(LegacyPasswordProvider::new(
        "my-username",
        "my-password",
    )));
    let auth = HaloAuthClient::from_xbox_client(xbox_client.clone());
    let halo = HaloInfiniteClient::new(auth);

    let xuid = xbox_client.gamertag_to_xuid("Some Gamertag").await?;
    let csr = halo
        .playlist_csr(PlaylistId::RANKED_ARENA, &Player::from(&xuid))
        .await?;

    println!("{csr:?}");
    Ok(())
}
```

## Theater films

`halo_api::theater::Film::try_from_chunks(&chunks, options)` decodes decompressed
Theater chunks without network or filesystem access. It consolidates the motion,
aim, input, roster/lives, recorded armor appearance, combat, vitality, weapons,
scope, velocity, projectile motion, and named summary-event experiments. This remains a partial v41 decoder;
unknown data is not guessed.
`clients::hi::film` reexports the current theater API. The highlight client methods
remain available as adapters over the v41 summary decoder.

For kills, deaths, mode highlights, and medal awards alone, use
`theater::decode_summary_events(&chunks, 41)` or
`halo.match_summary_events(match_id).await`: only footer chunks are needed.
`match_summary_events_with_validation` additionally compares each player's
kills, deaths, and individual medal `NameId` counts against match stats.
The catalog covers 155 published film codes, including all 151 current CMS
medals. Unknown codes stay inspectable; assists and objective subtypes remain
unresolved. See [event format, evidence, and examples](experiments/FILM_EVENTS.md).

See the [decoder API, folder example, JSON replay export, and measurements](experiments/THEATER_DECODER.md).
The library is checked on `wasm32-unknown-unknown`; file loading and replay export
live in the experiment example.

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
