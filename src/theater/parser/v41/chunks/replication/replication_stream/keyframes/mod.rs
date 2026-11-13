//! Packet type 2: keyframe entity baselines.
pub(crate) mod record;
mod slots;
#[cfg(test)]
pub(crate) mod table;
use crate::theater::parser::packet::PacketDecodeError;
use crate::theater::parser::v41::chunks::replication::replication_stream::DecodeContext;
use crate::theater::parser::v41::chunks::replication::source::ReplicationStreamPacketBody;

pub(crate) fn decode(
    payload: &[u8],
    context: &mut DecodeContext<'_>,
    vehicle_gate: impl Fn(u32) -> Option<bool>,
) -> Result<ReplicationStreamPacketBody, PacketDecodeError> {
    let table = slots::decode(payload, context.registry, context.config, vehicle_gate);
    for attempt in &table.records {
        // Preserve every raw ID in Film; only in-domain IDs can declare a
        // subsequent replication schema. The physical ordinal is not a new ID.
        if attempt.record.is_some() && attempt.id & 0x3fff_ffff < 0x1fff {
            context.state.bind_keyframe(
                attempt.id >> 30,
                attempt.id & 0x3fff_ffff,
                attempt.archetype,
            );
        }
    }
    Ok(ReplicationStreamPacketBody::KeyframesPacketBody(Box::new(
        table,
    )))
}
