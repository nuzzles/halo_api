//! Shared packet envelope and source models.

pub mod models;
pub use models::{
    BitRange, ByteRange, FilmPacketHeader, Packet, PacketBody, PacketDecodeError, PacketRead,
    PacketSource, PacketStream, SourceRegion, SourceRegionKind,
};

pub(crate) mod coverage;
