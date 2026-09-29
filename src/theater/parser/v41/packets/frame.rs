//! Packet type 0: incremental frame, event, entity, and control records.
use super::*;

pub(super) fn decode(
    payload: &[u8],
    context: &mut DecodeContext<'_>,
) -> Result<ReplicationStreamPacketBody, PacketDecodeError> {
    let events = read_reference_event_list(payload, 1, context.event_gate15, payload.len());
    let frame = context
        .config
        .decode_production_views(payload, 2, context.registry, context.world)
        .map(|frame| FrameRead::Decoded(Box::new(frame)))
        .unwrap_or_else(|error| FrameRead::Refused(error.into()));
    let continuation = continue_event_views(payload, &events, context);
    Ok(ReplicationStreamPacketBody::Frame(Box::new(FramePacket {
        frame,
        events,
        continuation,
    })))
}

pub(super) fn continue_event_views(
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
            context.world,
        )
    } else {
        context.config.decode_production_views(
            payload,
            events.end_bit,
            context.registry,
            &mut context.world.clone(),
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
