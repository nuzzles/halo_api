//! Packet type 8: roster payloads whose v41 layout is not structurally established.
use super::*;

pub(super) fn decode() -> ReplicationStreamPacketBody {
    ReplicationStreamPacketBody::RosterPacketBody
}
