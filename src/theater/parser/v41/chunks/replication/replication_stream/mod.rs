//! Packet-type dispatch for v41 replication streams.
use crate::theater::parser::packet::PacketDecodeError;
use crate::theater::parser::v41::chunks::registry::source::FilmRegistry;
#[cfg(test)]
use crate::theater::parser::v41::chunks::replication::source::EventListRead;
use crate::theater::parser::v41::chunks::replication::source::ReplicationStreamPacketBody;
use crate::theater::parser::v41::chunks::replication::state::ReplicationDecodeState;
use crate::theater::parser::v41::context::V41DecodeConfig;

pub(crate) mod datums;
pub(crate) mod frame;
#[cfg(test)]
pub(crate) use frame::{ContinuationStatePolicy, EventContinuation};
pub(crate) mod keyframes;
mod roster;

pub(crate) struct DecodeContext<'a> {
    pub config: &'a V41DecodeConfig,
    pub registry: &'a FilmRegistry,
    pub state: &'a mut ReplicationDecodeState,
    pub event_gate15: Option<bool>,
}

pub(crate) fn decode(
    packet_type: u16,
    payload: &[u8],
    context: &mut DecodeContext<'_>,
) -> Result<ReplicationStreamPacketBody, PacketDecodeError> {
    decode_with_vehicle_gates(packet_type, payload, context, |_| None)
}

pub(crate) fn decode_with_vehicle_gates(
    packet_type: u16,
    payload: &[u8],
    context: &mut DecodeContext<'_>,
    vehicle_gate: impl Fn(u32) -> Option<bool>,
) -> Result<ReplicationStreamPacketBody, PacketDecodeError> {
    decode_with_context(packet_type, payload, context, vehicle_gate, |_, _| None)
}

pub(crate) fn decode_with_context(
    packet_type: u16,
    payload: &[u8],
    context: &mut DecodeContext<'_>,
    vehicle_gate: impl Fn(u32) -> Option<bool>,
    delta_schema: impl Fn(i64, u32) -> Option<u32>,
) -> Result<ReplicationStreamPacketBody, PacketDecodeError> {
    match packet_type {
        0 => frame::decode(payload, context, &delta_schema),
        1 => datums::decode(payload),
        2 => keyframes::decode(payload, context, vehicle_gate),
        // Only the empty end marker has an established body layout. A framed
        // payload remains source evidence, not a successfully decoded marker.
        7 if payload.is_empty() => Ok(ReplicationStreamPacketBody::EndPacketBody),
        8 => Ok(ReplicationStreamPacketBody::RosterPacketBody(Box::new(
            roster::decode(payload, false, &context.config.players),
        ))),
        12 => Ok(ReplicationStreamPacketBody::BotsPacketBody(Box::new(
            roster::decode(payload, true, &context.config.players),
        ))),
        _ => Err(PacketDecodeError::UnsupportedLayout { packet_type }),
    }
}

#[cfg(test)]
pub(crate) fn continue_event_views(
    payload: &[u8],
    events: &EventListRead,
    config: &V41DecodeConfig,
    registry: &FilmRegistry,
    world: &mut ReplicationDecodeState,
) -> Option<EventContinuation> {
    let mut context = DecodeContext {
        config,
        registry,
        state: world,
        event_gate15: None,
    };
    frame::continue_event_views(payload, events, &mut context)
}
