//! Packet type 2: keyframe entity baselines.
pub(crate) mod record;
mod slots;

use crate::theater::parser::v41::context::V41DecodeConfig;
use crate::theater::parser::v41::packet::PacketDecodeError;
use crate::theater::parser::v41::registry::source::FilmRegistry;
use crate::theater::parser::v41::replication::source::{
    KeyframeTable, ReplicationStreamPacketBody,
};
use crate::theater::parser::v41::replication::stream::DecodeContext;

pub(crate) fn decode(
    payload: &[u8],
    context: &mut DecodeContext<'_>,
    vehicle_gate: impl Fn(u32) -> Option<bool>,
) -> Result<ReplicationStreamPacketBody, PacketDecodeError> {
    let table = slots::decode(payload, context.registry, context.config, vehicle_gate);
    Ok(publish_table(table, context))
}

pub(crate) fn predecode(
    payload: &[u8],
    registry: &FilmRegistry,
    config: &V41DecodeConfig,
    vehicle_gate: impl Fn(u32) -> Option<bool>,
) -> KeyframeTable {
    slots::decode(payload, registry, config, vehicle_gate)
}

pub(crate) fn publish_table(
    table: KeyframeTable,
    context: &mut DecodeContext<'_>,
) -> ReplicationStreamPacketBody {
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
    ReplicationStreamPacketBody::KeyframesPacketBody(Box::new(table))
}
