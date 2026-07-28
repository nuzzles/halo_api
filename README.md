# halo_api

[![CI](https://github.com/nuzzles/halo_api/actions/workflows/ci.yml/badge.svg)](https://github.com/nuzzles/halo_api/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/halo_api.svg)](https://crates.io/crates/halo_api)
[![docs.rs](https://docs.rs/halo_api/badge.svg)](https://docs.rs/halo_api)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

Unofficial Halo Infinite REST API client for Rust: CSR/rank lookups, service records, match history, and more.

> [!IMPORTANT]
> This is an unofficial, community-maintained library. It is not affiliated with, endorsed by Microsoft.

## What this crate does

- Acquires and caches a Halo "spartan token" (the bearer credential every Halo Waypoint API call requires) from
  an Xbox Live XSTS ticket, via the [`SpartanTokenSource`](src/auth/spartan.rs) trait.
- Wraps Halo Infinite's stats endpoints — Competitive Skill Rank (CSR) by playlist, service records, and match
  history — behind a single typed `HaloClient`.
- Automatically invalidates and retries once on an expired/unauthorized (401) response, instead of surfacing a
  hard failure the caller has to handle manually.

This crate depends on the [`xbox`](https://crates.io/crates/xbox) crate for Xbox Live authentication (XSTS
tickets, XUID resolution) but does not require it directly — anything implementing `SpartanTokenSource` can
supply a spartan token from elsewhere.

## Quick start

```rust,no_run
use std::sync::Arc;

use halo_api::constants::PlaylistId;
use halo_api::HaloClient;
use xbox::auth::LegacyPasswordProvider;
use xbox::XboxClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let xbox_client = Arc::new(XboxClient::new(LegacyPasswordProvider::new(
        "my-username",
        "my-password",
    )));
    let halo = HaloClient::from_xbox_client(xbox_client.clone());

    let xuid = xbox_client.gamertag_to_xuid("Some Gamertag").await?;
    let csr = halo.playlist_csr(PlaylistId::Arena, &xuid).await?;

    println!("{csr:?}");
    Ok(())
}
```

## Architecture

- `auth` — `SpartanTokenSource` trait plus the built-in `XboxSpartanTokenProvider` (backed by any
  `xbox::auth::XblAuthProvider`).
- `client` — `HaloClient`, the top-level entry point: spartan-token caching, request execution, and
  401-invalidate-and-retry-once behavior.
- `endpoints` — one module per Halo Waypoint API surface (`csr`, `service_record`, `match_history`, ...).
- `models` — request/response types for each endpoint.
- `constants` — playlist IDs and Halo Waypoint origin/domain constants.

## Endpoint coverage

Covered today: CSR by playlist, service record, player match history.

Planned: user/profile lookup, per-match stats and skill, UGC assets, playlist metadata, progression files
(season calendars, medals), ban info, and matches-privacy settings.

## MSRV

This crate targets the latest stable Rust toolchain. No specific MSRV is guaranteed yet.

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
