//! Packet type 2: keyframe entity baselines.
pub(crate) mod record;
pub(crate) mod table;
use crate::theater::film::{PacketDecodeError, ReplicationStreamPacketBody};
use crate::theater::parser::v41::chunks::replication::replication_stream::DecodeContext;

pub(crate) fn decode(
    payload: &[u8],
    context: &mut DecodeContext<'_>,
) -> Result<ReplicationStreamPacketBody, PacketDecodeError> {
    let table = context
        .config
        .read_keyframe_table(payload, context.registry)
        .map_err(|error| PacketDecodeError::InvalidKeyframeProfile {
            packet_type: 2,
            reason: error.into(),
        })?;
    for attempt in &table.records {
        if attempt.record.is_some() {
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
