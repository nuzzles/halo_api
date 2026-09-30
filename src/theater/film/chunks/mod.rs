//! Reference film chunk containers and their decoded payload models.

use super::ParseError;
use serde::{Deserialize, Serialize};

pub mod models;
pub use models::{Chunk, ChunkKind, ChunkTransport, FilmChunk};

pub mod packet;
pub use packet::{
    BitRange, ByteRange, FilmPacketHeader, Packet, PacketBody, PacketDecodeError, PacketRead,
    PacketSource, PacketStream, SourceRegion, SourceRegionKind,
};

pub mod registry;
pub use registry::*;

pub mod replication;
pub use replication::*;

pub mod summary;
pub use summary::*;
