//! Canonical Theater recording and owned runtime API.
//!
//! ```no_run
//! use halo_api::clients::hi::models::{RawFilmChunk, RawChunkKind};
//! use halo_api::theater::Film;
//! # fn inspect(registry: Vec<u8>, replication: Vec<u8>, summary: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
//! let film = Film::parse([
//!     RawFilmChunk { kind: RawChunkKind::REGISTRY, index: 0, start_ms: 0, data: registry },
//!     RawFilmChunk { kind: RawChunkKind::REPLICATION, index: 1, start_ms: 0, data: replication },
//!     RawFilmChunk { kind: RawChunkKind::SUMMARY, index: 2, start_ms: 0, data: summary },
//! ])?;
//! let mut runtime = halo_api::theater::TheaterRuntime::load(film);
//! let _: &[halo_api::theater::film::ReplicationObservation] = runtime.observations();
//! let _: &[halo_api::theater::film::SummaryHighlight] = runtime.summary_highlights();
//!
//! let world = runtime.seek(10_000_000);
//! # Ok(())
//! # }
//! ```
//! The recording has one registry, an ordered replication stream, and a
//! required terminal summary/highlights chunk.
//! Finished models are available under `film`; packet and structural read
//! models remain private to parsing.
//! ```compile_fail
//! use halo_api::theater::film::RawFilmChunk;
//! ```
//! The former source/options entry point is removed:
//! ```compile_fail
//! use halo_api::theater::film::ParseOptions;
//! ```
//! ```compile_fail
//! use halo_api::theater::parser::FilmSource;
//! ```
//! See this module's `docs/` directory for the fidelity and validation contract.
//! Playback owns only typed state and an independent time cursor.
//! ```compile_fail
//! use halo_api::theater::runtime::resolved::ResolvedFilm;
//! ```
//! ```compile_fail
//! use halo_api::theater::ResolvedFilm;
//! ```
//! ```compile_fail
//! # fn old_api(film: halo_api::theater::Film) {
//! let resolved = film.resolve();
//! # }
//! ```
pub mod film;
// The private decoder retains source-backed readers and diagnostic helpers for
// format work even when the finished Film projection does not consume them.
#[allow(dead_code)]
pub(crate) mod parser;
pub mod runtime;
pub use film::Film;
pub use runtime::TheaterRuntime;
