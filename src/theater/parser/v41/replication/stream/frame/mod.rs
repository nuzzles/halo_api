//! Packet type 0: incremental frame, event, entity, and control records.

pub(crate) mod controls;
pub(crate) mod entities;
pub(crate) mod events;
pub(crate) mod header;
pub(crate) mod records;
use crate::theater::parser::v41::packet::PacketDecodeError;
use crate::theater::parser::v41::replication::source::{
    EventListStop, FrameDecodeError, FramePacket, FrameRead, FrameViewStop, ProductionEntityEnd,
    ReplicationStreamPacketBody,
};
use crate::theater::parser::v41::replication::stream::DecodeContext;
use crate::theater::parser::v41::replication::stream::frame::events::{
    EventLayout, FireV3, read_event_list,
};

pub(crate) fn decode(
    payload: &[u8],
    context: &mut DecodeContext<'_>,
    delta_schema: &dyn Fn(i64, u32) -> Option<u32>,
) -> Result<ReplicationStreamPacketBody, PacketDecodeError> {
    if context.config.guard_rejected_frame_deletes {
        context.state.begin_frame_deletes();
    }
    let events = read_event_list(
        payload,
        1,
        payload.len(),
        EventLayout {
            versions: context.config.event_versions,
            category_references: payload.first().is_some_and(|b| b & 0x80 != 0),
            corruption_check: context.config.corruption_check(),
            gate15: context.event_gate15,
            script_prefix_context: context.config.event_script_prefix,
            variant: context.config.event_variant,
            runtime_defaults: context.config.event_runtime_defaults,
            kill_tail: context.config.event_kill_tail,
            event_vector: context
                .config
                .event_quantized_vectors
                .then_some(&context.config.profile.movement.world_object),
            fire_v3: context.config.event_fire_v3.then_some(FireV3 {
                position: context
                    .config
                    .event_quantized_vectors
                    .then_some(&context.config.profile.movement.world_object),
            }),
            projectile_positions: context.config.projectile_event_positions.as_ref(),
        },
    );
    let configuration = payload.first().map(|byte| byte & 0x80 != 0);
    let frame = if events.stop != EventListStop::Terminator {
        FrameRead::Refused(FrameDecodeError::IncompleteMessageList)
    } else {
        context
            .config
            .decode_entity_control_views_with_schemas(
                payload,
                events.end_bit,
                context.registry,
                context.state,
                Some(delta_schema),
            )
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
    frame: &crate::theater::parser::v41::replication::source::ProductionFrame,
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
        (record.header.kind == crate::theater::parser::v41::replication::source::RecordKind::Delete)
            .then_some(record.header.id)
            .flatten()
    })
}

// Reference-oracle adapter; never part of the canonical recording.
