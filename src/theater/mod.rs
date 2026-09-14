//! Typed, offline Halo Infinite Theater decoding.
//!
//! [`Film::try_from_chunks`] consumes decompressed chunk bytes, without filesystem,
//! network, clocks, or native-only APIs. Version 41 support consolidates the
//! captured motion, aim, input, lifecycle, appearance, combat, vitality, weapon, scope and
//! projectile motion experiments. It is a **partial** decoder: unknown
//! components never cause guessed skips or fabricated observations.
//!
//! [`decode_summary_events`] reads stored kills, deaths, mode highlights and named
//! medal awards using only decompressed footer chunks. [`validate_summary_events`]
//! compares per-player counts and individual medal identities with match stats.
//!
//! ```no_run
//! use halo_api::theater::{DecodeOptions, Film};
//! # fn example(chunks: &[halo_api::clients::hi::models::FilmChunkData])
//! # -> Result<(), Box<dyn std::error::Error>> {
//! let film = Film::try_from_chunks(chunks, DecodeOptions::v41())?;
//! let portable_json = serde_json::to_vec(&film)?;
//! # Ok(())
//! # }

mod appearance;
mod bits;
mod combat;
mod coordinates;
mod decode;
mod highlights;
mod input;
mod medals;
mod motion;
mod packets;
mod projectile;
mod registry;
mod roster;
mod summary;
mod types;
mod velocity;

pub use coordinates::*;
pub use highlights::*;
pub use medals::*;
pub use registry::*;
pub use roster::*;
pub use summary::*;
pub use types::*;

#[cfg(test)]
mod tests;
