//! Packet type 0: incremental frame, event, entity, and control records.
#[cfg(test)]
use crate::theater::film::EventListRead;
pub(crate) mod controls;
pub(crate) mod entities;
pub(crate) mod events;
pub(crate) mod header;
pub(crate) mod records;
use crate::theater::film::{
    EventListStop, FrameDecodeError, FramePacket, FrameRead, FrameViewStop, PacketDecodeError,
    ProductionEntityEnd, ReplicationStreamPacketBody,
};
use crate::theater::parser::v41::chunks::replication::replication_stream::DecodeContext;
use crate::theater::parser::v41::chunks::replication::replication_stream::frame::events::{
    read_contextual_event_list, read_reference_event_list,
};

pub(crate) fn decode(
    payload: &[u8],
    context: &mut DecodeContext<'_>,
) -> Result<ReplicationStreamPacketBody, PacketDecodeError> {
    if context.config.guard_rejected_frame_deletes {
        context.state.begin_frame_deletes();
    }
    let events = if context.config.event_quantized_vectors
        || context.config.event_fire_v3
        || context.config.projectile_event_positions.is_some()
    {
        read_contextual_event_list(
            payload,
            1,
            context.event_gate15,
            payload.len(),
            context
                .config
                .event_quantized_vectors
                .then_some(&context.config.profile.movement.world_object),
            context
                .config
                .event_fire_v3
                .then_some(&context.config.profile.movement.world_object),
            context.config.projectile_event_positions.as_ref(),
        )
    } else {
        read_reference_event_list(payload, 1, context.event_gate15, payload.len())
    };
    let configuration = payload.first().map(|byte| byte & 0x80 != 0);
    let frame = if events.stop != EventListStop::Terminator {
        FrameRead::Refused(FrameDecodeError::IncompleteMessageList)
    } else {
        context
            .config
            .decode_entity_control_views(payload, events.end_bit, context.registry, context.state)
            .map(|frame| FrameRead::Decoded(Box::new(frame)))
            .unwrap_or_else(|error| FrameRead::Refused(error.into()))
    };
    if context.config.guard_rejected_frame_deletes {
        context.state.finish_frame_deletes(
            suspect_delete(&frame),
            frame_closed(&frame, payload),
            context.registry,
        );
    }
    Ok(ReplicationStreamPacketBody::FramePacketBody(Box::new(
        FramePacket {
            configuration,
            events,
            frame,
        },
    )))
}

pub(crate) fn frame_closed(frame: &FrameRead, payload: &[u8]) -> bool {
    let FrameRead::Decoded(frame) = frame else {
        return false;
    };
    production_frame_closed(frame, payload)
}

pub(super) fn production_frame_closed(
    frame: &crate::theater::film::ProductionFrame,
    payload: &[u8],
) -> bool {
    if !matches!(frame.entity_end, Some(ProductionEntityEnd::Marker(_)))
        || !frame
            .controls
            .as_ref()
            .is_some_and(|view| view.stop == FrameViewStop::Complete)
    {
        return false;
    }
    let Some(end) = usize::try_from(frame.end_bit).ok() else {
        return false;
    };
    let Some(remaining) = payload
        .len()
        .checked_mul(8)
        .and_then(|bits| bits.checked_sub(end))
    else {
        return false;
    };
    remaining <= 7 && (end..end + remaining).all(|bit| payload[bit / 8] & (1 << (7 - bit % 8)) == 0)
}

/// A rejected successor cannot corroborate a preceding cross-generation DELETE.
pub(crate) fn suspect_delete(frame: &FrameRead) -> Option<u32> {
    let FrameRead::Decoded(frame) = frame else {
        return None;
    };
    if !matches!(frame.entity_end, Some(ProductionEntityEnd::Rejected { .. })) {
        return None;
    }
    frame.records.last().and_then(|record| {
        (record.header.kind == crate::theater::film::RecordKind::Delete)
            .then_some(record.header.id)
            .flatten()
    })
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
    /// Retained for deserializing historical test snapshots, before the gate fix.
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
    let state_policy = ContinuationStatePolicy::Applied;
    let frame = context.config.decode_production_views(
        payload,
        events.end_bit,
        context.registry,
        context.state,
    );
    Some(EventContinuation {
        start_bit: events.end_bit,
        frame: frame
            .map(|frame| FrameRead::Decoded(Box::new(frame)))
            .unwrap_or_else(|error| FrameRead::Refused(error.into())),
        state_policy,
    })
}

#[cfg(test)]
mod closure_tests {
    use super::*;
    use crate::theater::film::{ByteRange, FilmArchetype, FilmRegistry, RegistryComponent};
    use crate::theater::parser::v41::{
        chunks::replication::state::ReplicationDecodeState, context::V41DecodeConfig,
    };

    #[test]
    fn rejected_delta_after_mismatched_delete_preserves_a_binding() {
        let mut prefix = vec![true, false, false, true, false]; // configuration, events end, DELETE
        prefix.extend((0..13).rev().map(|shift| 123 & (1 << shift) != 0));
        prefix.extend([false, true]); // generation 1
        prefix.extend([false; 32]); // DELETE metadata
        let mut closed_bits = prefix.clone();
        closed_bits.extend([false; 4]); // entity and control terminators
        let mut rejected_bits = prefix;
        rejected_bits.push(true); // delta for an unbound slot
        rejected_bits.extend((0..13).rev().map(|shift| 77 & (1 << shift) != 0));
        rejected_bits.extend([false, true]);
        let encode = |bits: Vec<bool>| {
            let mut payload = vec![0_u8; bits.len().div_ceil(8)];
            for (index, bit) in bits.into_iter().enumerate() {
                payload[index / 8] |= u8::from(bit) << (7 - index % 8);
            }
            payload
        };
        let closed = encode(closed_bits);
        let rejected = encode(rejected_bits);
        let range = ByteRange { start: 0, end: 0 };
        let mut registry = FilmRegistry {
            archetypes: (0..5)
                .map(|index| FilmArchetype {
                    index,
                    components: vec![],
                    source: range,
                    padding: range,
                })
                .collect(),
        };
        let mut name_bytes = vec![0; 256];
        name_bytes[.."high-frequency".len()].copy_from_slice(b"high-frequency");
        registry.archetypes[4].components.push(RegistryComponent {
            name_bytes,
            precision_level: 0,
            source: range,
        });
        let config = V41DecodeConfig {
            guard_rejected_frame_deletes: true,
            ..Default::default()
        };
        for (payload, id, survives, suspect_expected) in [
            (&closed, 123 | (1 << 30), false, false),
            (&rejected, 123 | (1 << 30), false, true),
            (&closed, 123 | (2 << 30), false, false),
            (&rejected, 123 | (2 << 30), true, true),
        ] {
            let mut state = ReplicationDecodeState::default();
            state.bind_full(id, 4);
            let mut context = DecodeContext {
                config: &config,
                registry: &registry,
                state: &mut state,
                event_gate15: Some(true),
            };
            let ReplicationStreamPacketBody::FramePacketBody(frame) =
                decode(payload, &mut context).unwrap()
            else {
                unreachable!()
            };
            assert_eq!(frame.events.stop, EventListStop::Terminator);
            assert_eq!(suspect_delete(&frame.frame).is_some(), suspect_expected);
            assert!(!state.slots.contains_key(&123));
            if survives {
                assert!(state.deferred_mismatched_delete(123 | (1 << 30)));
                assert_eq!(state.admit_delta(id, true),
                    crate::theater::parser::v41::chunks::replication::state::DeltaAdmission::Allowed);
                assert_eq!(state.archetype(123), Some(4));
            }
        }
    }
}
