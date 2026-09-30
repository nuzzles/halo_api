//! Packet type 8: roster payloads whose v41 layout is not structurally established.
use crate::theater::film::ReplicationStreamPacketBody;

pub(crate) fn decode() -> ReplicationStreamPacketBody {
    ReplicationStreamPacketBody::RosterPacketBody
}
