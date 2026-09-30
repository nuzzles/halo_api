//! Packet-type dispatch for v41 replication streams.
#[cfg(test)]
use crate::theater::film::EventListRead;
use crate::theater::film::{FilmRegistry, PacketDecodeError, ReplicationStreamPacketBody};
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
    match packet_type {
        0 => frame::decode(payload, context),
        1 => datums::decode(payload),
        2 => keyframes::decode(payload, context),
        7 => Ok(ReplicationStreamPacketBody::EndPacketBody),
        8 => Ok(roster::decode()),
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
