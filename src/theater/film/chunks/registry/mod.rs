//! Registry chunk container and registry payload models.

pub mod models;
pub use models::*;

pub type RegistryChunk = super::Chunk<FilmRegistryRead>;
