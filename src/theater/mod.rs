//! Canonical Theater recording and owned runtime API.
//!
//! ```no_run
//! use halo_api::theater::{Film, film::{FilmChunk, ChunkKind}};
//! # fn inspect(registry: Vec<u8>, replication: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
//! let film = Film::parse([
//!     FilmChunk { kind: ChunkKind::Registry, index: None, start_ms: None, data: registry },
//!     FilmChunk { kind: ChunkKind::Replication, index: None, start_ms: None, data: replication },
//! ])?;
//! let mut runtime = halo_api::theater::TheaterRuntime::load(film);
//! let _: Vec<&halo_api::theater::film::ReplicationStreamChunk> = runtime.film().replication_chunks().collect();
//! let _: &halo_api::theater::film::chunks::registry::FilmRegistry = &runtime.film().registry.body.registry;
//! let _: &[halo_api::theater::runtime::SummaryEvent] = runtime.summary_events();
//! let _: Vec<halo_api::theater::film::chunks::replication::components::ComponentField> = Vec::new();
//! let _: &[halo_api::theater::runtime::Event] = runtime.events();
//!
//! let world = runtime.seek(10_000_000);
//! # Ok(())
//! # }
//! ```
//! The recording has a registry and one ordered sequence of all later chunks.
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
//! The resolved storage is deliberately private to the runtime.
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
pub mod parser;
pub mod runtime;
pub use film::Film;
pub use runtime::TheaterRuntime;
