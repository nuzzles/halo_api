//! Packet type 0: incremental frame, event, entity, and control records.
#[cfg(test)]
use crate::theater::film::EventListRead;
pub(crate) mod controls;
pub(crate) mod entities;
pub(crate) mod events;
pub(crate) mod header;
pub(crate) mod records;
use crate::theater::film::{
    EventListStop, FrameDecodeError, FramePacket, FrameRead, PacketDecodeError,
    ReplicationStreamPacketBody,
};
use crate::theater::parser::v41::chunks::replication::replication_stream::DecodeContext;
use crate::theater::parser::v41::chunks::replication::replication_stream::frame::events::read_reference_event_list;

pub(crate) fn decode(
    payload: &[u8],
    context: &mut DecodeContext<'_>,
) -> Result<ReplicationStreamPacketBody, PacketDecodeError> {
    let events = read_reference_event_list(payload, 1, context.event_gate15, payload.len());
    let configuration = payload.first().map(|byte| byte & 0x80 != 0);
    let frame = if events.stop != EventListStop::Terminator {
        FrameRead::Refused(FrameDecodeError::IncompleteMessageList)
    } else if events.records.iter().any(|record| record.code == Some(0)) {
        FrameRead::Refused(FrameDecodeError::ConflictingMessageLayout)
    } else {
        context
            .config
            .decode_entity_control_views(payload, events.end_bit, context.registry, context.state)
            .map(|frame| FrameRead::Decoded(Box::new(frame)))
            .unwrap_or_else(|error| FrameRead::Refused(error.into()))
    };
    Ok(ReplicationStreamPacketBody::FramePacketBody(Box::new(
        FramePacket {
            configuration,
            events,
            frame,
        },
    )))
}

// Reference-oracle adapter; never part of the canonical recording.
#[cfg(test)]
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub(crate) struct EventContinuation {
    pub start_bit: usize,
    pub frame: FrameRead,
    #[serde(default)]
    pub state_policy: ContinuationStatePolicy,
}
#[cfg(test)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) enum ContinuationStatePolicy {
    #[default]
    Unknown,
    Applied,
    IsolatedConflictingDamageGrammar,
}
#[cfg(test)]
pub(crate) fn continue_event_views(
    payload: &[u8],
    events: &EventListRead,
    context: &mut DecodeContext<'_>,
) -> Option<EventContinuation> {
    if events.stop != EventListStop::Terminator || events.records.is_empty() {
        return None;
    }
    let state_policy = if events.records.iter().any(|record| record.code == Some(0)) {
        ContinuationStatePolicy::IsolatedConflictingDamageGrammar
    } else {
        ContinuationStatePolicy::Applied
    };
    let frame = if state_policy == ContinuationStatePolicy::Applied {
        context.config.decode_production_views(
            payload,
            events.end_bit,
            context.registry,
            context.state,
        )
    } else {
        context.config.decode_production_views(
            payload,
            events.end_bit,
            context.registry,
            &mut context.state.clone(),
        )
    };
    Some(EventContinuation {
        start_bit: events.end_bit,
        frame: frame
            .map(|frame| FrameRead::Decoded(Box::new(frame)))
            .unwrap_or_else(|error| FrameRead::Refused(error.into())),
        state_policy,
    })
}
