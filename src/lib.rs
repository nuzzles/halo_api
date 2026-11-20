//! Unofficial Halo Infinite API client and Theater film parser for Rust.
//! The [`theater`] module parses v41 films and plays their recorded entity state.
//!
//! This crate is not affiliated with, endorsed by or supported by Microsoft.
//!
//! # Quick start
//!
//! ```no_run
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! use halo_api::clients::hi::models::PlaylistId;
//! use halo_api::auth::HaloAuthClient;
//! use halo_api::clients::hi::{HaloInfiniteClient, Player};
//!
//! let auth = HaloAuthClient::from_tokens("spartan-token", "clearance-token");
//! let halo = HaloInfiniteClient::new(auth);
//!
//! let csr = halo.playlist_csr(PlaylistId::RANKED_ARENA, &Player::gamertag("Some Gamertag")).await?;
//!
//! println!("{csr:?}");
//! # Ok(())
//! # }
//! ```

pub mod auth;
pub mod clients;
pub mod theater;

/// The optional Xbox client crate, available with the `xbox` feature.
#[cfg(feature = "xbox")]
pub use xbox;

pub use clients::hi::{
    HaloInfiniteClient, HaloInfiniteClientPool, HaloInfiniteClientPoolError, InfiniteClientError,
};
pub use clients::waypoint::{HcsError, HcsLeaderboardEntry, WaypointClient};

#[cfg(test)]
mod tests;
