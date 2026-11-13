//! Adapt the shared bounded action reader into the canonical event field stream.
use super::{EventField, EventFieldValue, Reader};
use crate::theater::parser::v41::chunks::replication::components::{
    actions, cursor::ComponentCursor, reader::ComponentReader,
};

pub(super) fn read(r: &mut Reader<'_>, data: &[u8], variant: u64) -> bool {
    if r.over {
        return false;
    }
    let Some(cursor) = ComponentCursor::new(data, r.cursor.position) else {
        r.over = true;
        return false;
    };
    let mut component = ComponentReader {
        // The enclosing event list already required configuration one.
        category_references: true,
        snapshot: false,
        cursor,
        fields: Vec::new(),
        position_encoding: None,
        reference_widths: None,
        width_error: None,
        context_error: None,
        live_grammar: None,
        movement_slot: None,
        references: Vec::new(),
        diagnostics: Default::default(),
    };
    let status = actions::read(&mut component, variant);
    if let Some(trace) = &mut r.trace {
        trace.extend(component.fields.into_iter().map(|field| EventField {
            bit: field.bit,
            width: field.width,
            stage: r.stage,
            value: if field.width <= 64 {
                EventFieldValue::Scalar(field.raw.low_u64())
            } else {
                EventFieldValue::Opaque
            },
        }));
        if status.is_none()
            && let Some(refusal) = component.diagnostics.read_refusals.first()
        {
            trace.push(EventField {
                bit: refusal.bit as usize,
                width: refusal.width as usize,
                stage: r.stage,
                value: EventFieldValue::Unavailable,
            });
        }
    }
    r.cursor.position = component.cursor.position as usize;
    r.over = status.is_none();
    status == Some(true)
}
