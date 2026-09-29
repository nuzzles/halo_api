//! Packet type 2: keyframe entity baselines.
use super::*;

pub(super) fn decode(
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
            context.world.bind_keyframe(
                attempt.id >> 30,
                attempt.id & 0x3fff_ffff,
                attempt.archetype,
            );
        }
    }
    Ok(ReplicationStreamPacketBody::Keyframes(table))
}
