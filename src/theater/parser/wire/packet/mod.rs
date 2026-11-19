//! Packet envelopes, framing, and source coverage.

pub mod models;
pub use models::{
    BitRange, ByteRange, FilmPacketHeader, Packet, PacketBody, PacketRead, PacketSource,
    PacketStream, SourceRegion, SourceRegionKind,
};

pub(crate) mod coverage;
mod framing;
