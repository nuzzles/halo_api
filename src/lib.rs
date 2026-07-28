//! Unofficial Halo Infinite REST API client for Rust: CSR/rank lookups, service records, match
//! history, and more.
//!
//! This crate is not affiliated with, endorsed by or supported by Microsoft.
//!
//! # Quick start
//!
//! ```no_run
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! use std::sync::Arc;
//!
//! use halo_api::clients::hi::constants::PlaylistId;
//! use halo_api::auth::AuthClient;
//! use halo_api::clients::hi::HaloInfiniteClient;
//! use xbox::auth::LegacyPasswordProvider;
//! use xbox::XboxClient;
//!
//! let xbox_client = Arc::new(XboxClient::new(LegacyPasswordProvider::new(
//!     "my-username",
//!     "my-password",
//! )));
//! let auth = AuthClient::from_xbox_client(xbox_client.clone());
//! let halo = HaloInfiniteClient::new(auth);
//!
//! let xuid = xbox_client.gamertag_to_xuid("Some Gamertag").await?;
//! let csr = halo.playlist_csr(PlaylistId::Arena, &xuid).await?;
//!
//! println!("{csr:?}");
//! # Ok(())
//! # }
//! ```

pub mod auth;
pub mod clients;

pub use clients::hi::{HaloInfiniteClient, InfiniteClientError};

#[cfg(test)]
mod tests;
