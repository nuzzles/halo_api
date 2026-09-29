//! Registry chunk container and registry payload models.

use super::FilmChunk;
use serde::{Deserialize, Serialize};

pub mod models;
pub use models::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegistryChunk {
    pub definition: FilmRegistryRead,
    /// Original input, including exact transport bytes and supplied metadata.
    pub source: FilmChunk,
    /// Always zero for a valid film because the registry must be first.
    pub source_position: usize,
    /// Decompressed registry and bootstrap bytes, including opaque trailing data.
    pub data: Vec<u8>,
}
