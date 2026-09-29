//! Packet-type dispatch for v41 replication streams.
use super::*;

mod datums;
mod frame;
mod keyframes;
mod roster;

pub(super) struct DecodeContext<'a> {
    pub config: &'a FrameConfig,
    pub registry: &'a FilmRegistry,
    pub world: &'a mut FilmWorld,
    pub event_gate15: Option<bool>,
}

pub(super) fn decode(
    packet_type: u16,
    payload: &[u8],
    context: &mut DecodeContext<'_>,
) -> Result<ReplicationStreamPacketBody, PacketDecodeError> {
    match packet_type {
        0 => frame::decode(payload, context),
        1 => datums::decode(payload),
        2 => keyframes::decode(payload, context),
        8 => Ok(roster::decode()),
        _ => Ok(ReplicationStreamPacketBody::UnknownPacketBody),
    }
}

#[cfg(test)]
pub(super) fn continue_event_views(
    payload: &[u8],
    events: &EventListRead,
    config: &FrameConfig,
    registry: &FilmRegistry,
    world: &mut FilmWorld,
) -> Option<EventContinuation> {
    let mut context = DecodeContext {
        config,
        registry,
        world,
        event_gate15: None,
    };
    frame::continue_event_views(payload, events, &mut context)
}
