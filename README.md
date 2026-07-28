# halo_api

[![CI](https://github.com/nuzzles/halo_api/actions/workflows/ci.yml/badge.svg)](https://github.com/nuzzles/halo_api/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/halo_api.svg)](https://crates.io/crates/halo_api)
[![docs.rs](https://docs.rs/halo_api/badge.svg)](https://docs.rs/halo_api)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

Unofficial Halo Infinite REST API client for Rust: CSR/rank lookups, service records, match history, and more.

> [!IMPORTANT]
> This is an unofficial, community-maintained library. It is not affiliated with, endorsed by Microsoft.

## What this crate does

- Separates authentication (`AuthClient`) from Halo API operations (`HaloInfiniteClient`).
- Acquires and caches both the Spartan token and Waypoint flight clearance.
- Covers stats, skill, profile, UGC, progression, ban, and privacy endpoints.
- Automatically invalidates and retries once on an expired/unauthorized (401) response, instead of surfacing a
  hard failure the caller has to handle manually.

This crate depends on the [`xbox`](https://crates.io/crates/xbox) crate for Xbox Live authentication (XSTS
tickets, XUID resolution) but does not require it directly — anything implementing `SpartanTokenSource` can
supply a spartan token from elsewhere.

## Quick start

```rust,no_run
use std::sync::Arc;

use halo_api::clients::hi::models::PlaylistId;
use halo_api::auth::AuthClient;
use halo_api::clients::hi::HaloInfiniteClient;
use xbox::auth::LegacyPasswordProvider;
use xbox::XboxClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let xbox_client = Arc::new(XboxClient::new(LegacyPasswordProvider::new(
        "my-username",
        "my-password",
    )));
    let auth = AuthClient::from_xbox_client(xbox_client.clone());
    let halo = HaloInfiniteClient::new(auth);

    let xuid = xbox_client.gamertag_to_xuid("Some Gamertag").await?;
    let csr = halo
        .playlist_csr(PlaylistId::RANKED_ARENA, &xuid)
        .await?;

    println!("{csr:?}");
    Ok(())
}
```

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
