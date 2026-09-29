//! Native packet-head events. These readers do not imply complete event-list coverage.
use super::bits::Cursor;
pub(crate) use crate::theater::resolved::interpretation::packet::event_heads::{
    DecodedHeadEvent, EventReference, EventReferenceValue, HeadEventPayload, HeadEventStop,
};

pub(super) fn reference(r: &mut Cursor<'_>, domain: u8) -> Option<EventReference> {
    let result = (|| {
        let start_bit = r.position;
        let value = if r.read(1)? == 0 {
            None
        } else {
            let narrow = if domain == 1 {
                Some(r.read(1)? != 0)
            } else {
                None
            };
            let width = match domain {
                1 if narrow == Some(true) => 9,
                0 | 1 | 7 | 8 => 13,
                2 | 5 => 8,
                3 => 7,
                4 | 6 => 9,
                _ => return None,
            };
            // The native bounded helper guards index and generation together.
            let packed = r.read(width + 2)?;
            Some(EventReferenceValue {
                index: (packed >> 2) as u32,
                generation: (packed & 3) as u8,
                narrow,
            })
        };
        Some(EventReference {
            domain,
            start_bit,
            end_bit: r.position,
            value,
        })
    })();
    if result.is_none() {
        // Native refTronquee sentinel, not a consumed or zero-filled source bit.
        r.position = r.source_bits().saturating_add(1);
    }
    result
}

pub(super) fn decode_head_with_cursor(mut r: Cursor<'_>) -> Option<DecodedHeadEvent> {
    let config = r.read(1)? != 0;
    if r.read(1)? == 0 {
        return None;
    }
    let kind = r.read(7)? as u8;
    let mut out = DecodedHeadEvent {
        config,
        kind,
        references: Vec::new(),
        payload: None,
        end_bit: r.position,
        stop: HeadEventStop::UnsupportedType,
    };
    let domains = match kind {
        8 => [2, 3, 7],
        9 => [2, 7, 8],
        21 => [4, 8, 7],
        22 => [1, 1, 7],
        103 => [0, 0, 7],
        _ => return Some(out),
    };
    out.stop = HeadEventStop::Truncated;
    for domain in domains {
        let Some(value) = reference(&mut r, domain) else {
            out.end_bit = r.position;
            return Some(out);
        };
        out.references.push(value);
    }
    out.payload = match kind {
        9 => pickup_payload(&mut r),
        8 | 22 => r
            .read(6)
            .map(|seat| HeadEventPayload::VehicleSeat { seat: seat as u8 }),
        21 => r
            .read(2)
            .map(|level| HeadEventPayload::Zoom { level: level as u8 }),
        103 => Some(HeadEventPayload::SpawnedObjectReferences),
        _ => unreachable!(),
    };
    out.end_bit = r.position;
    if out.payload.is_some() {
        out.stop = HeadEventStop::ReferenceFieldsRead;
    }
    Some(out)
}

fn pickup_payload(r: &mut Cursor<'_>) -> Option<HeadEventPayload> {
    let class = r.read(3)? as u8;
    let (catalog_id, more_events) = if r.read(1)? != 0 {
        (Some(r.read(32)? as u32), Some(r.read(1)? != 0))
    } else {
        (None, None)
    };
    Some(HeadEventPayload::Pickup {
        class,
        catalog_id,
        more_events,
    })
}

impl DecodedHeadEvent {
    /// Reference unit_zoom's measured domain-four base. Generation remains in
    /// references[0]; level zero means leaving scope.
    pub(crate) fn zoom(&self) -> Option<(u32, u8)> {
        let HeadEventPayload::Zoom { level } = self.payload.as_ref()? else {
            return None;
        };
        let unit = self.references.first()?.value.as_ref()?;
        Some((unit.index.checked_add(512)?, *level))
    }
}
