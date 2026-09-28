//! Native Theater recording and resolved query/playback API.
//!
//! ```no_run
//! use halo_api::theater::{Film, film::{FilmChunk, ChunkKind}};
//! # fn inspect(registry: Vec<u8>, replication: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
//! let film = Film::parse([
//!     FilmChunk::new(ChunkKind::Registry, registry),
//!     FilmChunk::new(ChunkKind::Replication, replication),
//! ])?;
//! let mut resolved = film.resolve();
//! let _: &[halo_api::theater::film::ParsedChunk] = &film.replication.chunks;
//! let _: &[halo_api::theater::resolved::Event] = resolved.events();
//! assert!(std::ptr::eq(resolved.film(), &film));
//! let world = resolved.seek(10_000_000);
//! # Ok(())
//! # }
//! ```
//! The recording has exactly three sections: registry, replication, summaries.
//! Supporting models are public in their defining modules, not flattened here.
//! ```compile_fail
//! use halo_api::theater::FilmChunk;
//! ```
//! The former source/options entry point is removed:
//! ```compile_fail
//! use halo_api::theater::film::ParseOptions;
//! ```
//! ```compile_fail
//! use halo_api::theater::parser::FilmSource;
//! ```
//! See this module's `docs/` directory for the fidelity and validation contract.
pub mod film;
pub mod parser;
pub mod resolved;
pub use film::Film;
pub use resolved::ResolvedFilm;
