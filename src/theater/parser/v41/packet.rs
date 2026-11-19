//! v41 packet body results and format-specific decode failures.
pub(crate) use crate::theater::parser::wire::packet::{
    BitRange, ByteRange, PacketBody, SourceRegion, SourceRegionKind,
};

pub(crate) type Packet<T> = crate::theater::parser::wire::packet::Packet<T, PacketDecodeError>;
pub(crate) type PacketRead<T> =
    crate::theater::parser::wire::packet::PacketRead<T, PacketDecodeError>;
pub(crate) type PacketStream<T> =
    crate::theater::parser::wire::packet::PacketStream<T, PacketDecodeError>;

use crate::theater::parser::v41::replication::source::{DatumDecodeError, FrameDecodeError};

/// Why a structurally recognized v41 packet could not be decoded.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub(crate) enum PacketDecodeError {
    #[error("packet type {packet_type} has no established v41 layout")]
    UnsupportedLayout { packet_type: u16 },
    #[error("packet type {packet_type} has an invalid datum table: {reason}")]
    InvalidDatumTable {
        packet_type: u16,
        reason: DatumDecodeError,
    },
    #[error("packet type {packet_type} has an invalid keyframe profile: {reason}")]
    InvalidKeyframeProfile {
        packet_type: u16,
        reason: FrameDecodeError,
    },
    #[error("packet type {packet_type} has a truncated summary event count")]
    TruncatedSummaryCount { packet_type: u16 },
}
