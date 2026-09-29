//! Reference film chunk containers and their decoded payload models.

use super::ParseError;
use serde::{Deserialize, Serialize};

pub mod models;
pub use models::{ChunkKind, FilmChunk};

pub mod packet;
pub use packet::{FilmPacketHeader, SourceSpan};

pub mod registry;
pub use registry::*;

pub mod replication;
pub use replication::*;

pub mod summary;
pub use summary::*;
