//! Native Theater recording and resolved query/playback API.
//!
//! ```no_run
//! use halo_api::theater::{Film, film::{FilmChunk, ChunkKind}};
//! # fn inspect(registry: Vec<u8>, replication: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
//! let film = Film::parse([
//!     FilmChunk { kind: ChunkKind::Registry, index: None, start_ms: None, data: registry },
//!     FilmChunk { kind: ChunkKind::Replication, index: None, start_ms: None, data: replication },
//! ])?;
//! let mut resolved = film.resolve();
//! let _: &[halo_api::theater::film::ReplicationStreamChunk] = &film.replication.chunks;
//! let _: &halo_api::theater::film::chunks::registry::FilmRegistry = &film.registry.definition.registry;
//! let _: Vec<&halo_api::theater::film::chunks::summary::SummaryEvent> = film.summaries.events().collect();
//! let _: Vec<halo_api::theater::film::chunks::replication::components::ComponentField> = Vec::new();
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
