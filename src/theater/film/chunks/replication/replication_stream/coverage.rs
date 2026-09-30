//! Coverage of established structural reads, derived from their owned fields.
use super::super::{BitRange, ComponentField, SourceRegion, SourceRegionKind};
use super::*;

fn region(start: usize, end: usize, kind: SourceRegionKind) -> SourceRegion {
    SourceRegion {
        source: BitRange { start, end },
        kind,
    }
}
fn signed(start: i64, end: i64) -> SourceRegion {
    // Invalid public coordinates remain invalid, rather than being clamped.
    region(
        usize::try_from(start).unwrap_or(usize::MAX),
        usize::try_from(end).unwrap_or(usize::MAX),
        SourceRegionKind::Fields,
    )
}
fn fields(out: &mut Vec<SourceRegion>, fields: &[ComponentField]) {
    out.extend(fields.iter().map(|field| {
        region(
            field.bit,
            field.bit.saturating_add(field.width),
            SourceRegionKind::Fields,
        )
    }));
}
fn defaults(out: &mut Vec<SourceRegion>, state: Option<&DefaultState>) {
    if let Some(state) = state {
        if state.status == DefaultStateStatus::Opaque {
            out.push(SourceRegion {
                source: state.source,
                kind: SourceRegionKind::Opaque,
            });
        } else {
            fields(out, &state.fields);
        }
    }
}
fn header(out: &mut Vec<SourceRegion>, header: &RecordHeader) {
    if let Some(prefix) = &header.prefix {
        fields(out, std::slice::from_ref(prefix));
    }
    out.push(signed(header.start_bit, header.end_bit));
}
fn event(out: &mut Vec<SourceRegion>, field: &EventField) {
    let kind = match field.value {
        EventFieldValue::Scalar(_) => SourceRegionKind::Fields,
        EventFieldValue::Opaque => SourceRegionKind::Opaque,
        EventFieldValue::Unavailable => return,
    };
    out.push(region(
        field.bit,
        field.bit.saturating_add(field.width),
        kind,
    ));
}

pub(super) fn regions(body: &ReplicationStreamPacketBody) -> Vec<SourceRegion> {
    let mut out = Vec::new();
    match body {
        ReplicationStreamPacketBody::FramePacketBody(packet) => {
            if packet.configuration.is_some() {
                out.push(region(0, 1, SourceRegionKind::Fields));
            }
            for record in &packet.events.records {
                for field in &record.fields {
                    event(&mut out, field);
                }
            }
            if let Some(terminator) = &packet.events.terminator {
                event(&mut out, terminator);
            }
            if let Some(frame) = packet.frame.decoded() {
                for view in frame.messages.iter().chain(frame.controls.iter()) {
                    fields(&mut out, &view.fields);
                }
                for record in &frame.records {
                    header(&mut out, &record.header);
                    fields(&mut out, &record.fields);
                    defaults(&mut out, record.default_state.as_ref());
                    for component in &record.components {
                        fields(&mut out, &component.fields);
                    }
                }
                if let Some(
                    ProductionEntityEnd::Marker(value)
                    | ProductionEntityEnd::Rejected { header: value },
                ) = &frame.entity_end
                {
                    header(&mut out, value);
                }
            }
        }
        ReplicationStreamPacketBody::DatumsPacketBody(table) => {
            for source in table
                .entries
                .iter()
                .map(|entry| entry.source)
                .chain(table.component_masks.iter().map(|mask| mask.source))
                .chain(std::iter::once(table.tail_source))
            {
                out.push(SourceRegion {
                    source,
                    kind: SourceRegionKind::Fields,
                });
            }
            out.push(SourceRegion {
                source: table.padding_source,
                kind: SourceRegionKind::Padding,
            });
        }
        ReplicationStreamPacketBody::KeyframesPacketBody(table) => {
            out.push(region(0, 1, SourceRegionKind::Fields));
            for entry in &table.records {
                if let Some(record) = &entry.record {
                    fields(&mut out, &record.fields);
                    defaults(&mut out, record.default_state.as_ref());
                    for component in &record.components {
                        fields(&mut out, &component.fields);
                    }
                } else {
                    out.push(signed(entry.start_bit, entry.end_bit));
                }
            }
        }
        // Packet envelope acknowledges the type, but establishes no payload grammar.
        ReplicationStreamPacketBody::RosterPacketBody
        | ReplicationStreamPacketBody::EndPacketBody => {}
    }
    out
}
