//! Packet type 8: roster payloads whose v41 layout is not structurally established.
use super::*;

pub(super) fn decode() -> Result<ReplicationStreamPacketBody, PacketDecodeError> {
    Err(PacketDecodeError::UnsupportedLayout { packet_type: 8 })
}
