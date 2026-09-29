//! Ordered replication stream and packet envelope models.

use super::{FilmPacketHeader, KeyframeTable, ReplicationStreamChunk};
use serde::{Deserialize, Serialize};

pub mod models;
pub use models::*;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReplicationStream {
    pub chunks: Vec<ReplicationStreamChunk>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplicationStreamPacket {
    pub header: FilmPacketHeader,
    pub body: ReplicationStreamPacketBody,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)]
pub enum ReplicationStreamPacketBody {
    Frame(Box<FramePacket>),
    Datums(DatumTable),
    Keyframes(KeyframeTable),
    /// A reader could not start. The complete payload remains in the chunk.
    Refused {
        message: String,
    },
    /// No body reader selected; this does not imply an empty payload.
    Opaque,
}
