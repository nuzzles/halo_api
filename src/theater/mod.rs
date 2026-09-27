//! Native v41 Theater parsing and indexed resolution.
//!
//! Parse once, then borrow the recording for queries and playback:
//! ```no_run
//! use halo_api::theater::{Film, FilmSource, FilmSourceMetadata, film::ParseOptions};
//! # fn example(chunks: &[Vec<u8>], metadata: &[FilmSourceMetadata]) -> Result<(), Box<dyn std::error::Error>> {
//! let source = FilmSource::load(chunks, metadata)?;
//! let film = Film::parse(&source, ParseOptions::default())?;
//! let mut resolved = film.resolve();
//! assert!(std::ptr::eq(resolved.film(), &film));
//! let world = resolved.seek(10_000_000);
//! # Ok(())
//! # }
//! ```
//! The former mixed parser and client reporting API are removed:
//! ```compile_fail
//! use halo_api::theater::Film;
//! let _ = Film::try_from_chunks;
//! ```
//! ```compile_fail
//! use halo_api::clients::hi::film::FilmEventReport;
//! ```
//! See the module's `docs/` directory for format, fidelity and validation.
pub mod film;
mod parser;
pub mod resolved;
pub use film::*;
pub(crate) use parser::*;
pub use resolved::ResolvedFilm;
