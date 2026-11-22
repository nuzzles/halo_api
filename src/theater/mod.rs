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
//! let mut runtime = halo_api::theater::FilmRuntime::load(film);
//! let _: &[halo_api::theater::film::ReplicationObservation] = &runtime.film().replication_stream.observations;
//! let _: &[halo_api::theater::film::SummaryHighlight] = &runtime.film().summary.highlights;
//!
//! let world = runtime.seek(10_000_000);
//! # Ok(())
//! # }
//! ```
//! The recording has one registry, an ordered replication stream, and a
//! required terminal summary/highlights chunk.
//! Use [`Film::parse_replication`] or [`Film::parse_summary`] to decode only
//! the selected section and registry. The other chunk type need not be supplied.
//! Finished models are available under `film`; packet and structural read
//! models remain private to parsing.
//! Playback owns only typed state and an independent time cursor. The world
//! retains complete recorded field bodies; delta-encoded fields are not
//! reconstructed into absolute gameplay values. Summary highlights remain
//! guarded candidates until the full summary grammar is established.
pub mod film;
// The private decoder retains source-backed readers and diagnostic helpers for
// format work even when the finished Film projection does not consume them.
#[allow(dead_code)]
pub(crate) mod parser;
pub mod runtime;
pub use film::{Film, FilmReplication, FilmSummary};
pub use runtime::FilmRuntime;
