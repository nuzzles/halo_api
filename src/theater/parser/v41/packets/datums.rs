//! Packet type 1: datum/reference table entries.
use super::*;

pub(super) fn decode(payload: &[u8]) -> Result<ReplicationStreamPacketBody, PacketDecodeError> {
    decode_datum_table(payload)
        .map(ReplicationStreamPacketBody::Datums)
        .map_err(|error| PacketDecodeError::InvalidDatumTable {
            packet_type: 1,
            reason: error,
        })
}
