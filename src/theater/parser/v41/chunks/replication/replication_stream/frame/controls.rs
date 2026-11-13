//! Message and control views with explicit unsupported paths.
use crate::theater::parser::v41::chunks::replication::source::replication_stream::models::frame_views::{
    ControlEntry, ControlInput, ControlPayload, DecodedFrameView, FrameViewStop,
};
use crate::theater::parser::v41::chunks::replication::components::control;
use crate::theater::parser::v41::chunks::replication::components::cursor::ComponentCursor as Cursor;
use crate::theater::parser::v41::chunks::replication::components::position::PositionEncoding;
use crate::theater::parser::v41::chunks::replication::components::reader::{
    ComponentReader, ComponentWidths,
};
use crate::theater::parser::v41::context::profile::DecodeProfile;

mod commands;
mod secondary;
#[cfg(test)]
mod tests;

// Resource policy, not an established maximum number of records on the wire.
const RECORD_LIMIT: usize = 1 << 16;

/// Reference view A reader with signed positions and wrapping source guards.
/// Invalid negative reads retain the reference panic behavior.
pub(crate) fn decode_message_view_signed(data: &[u8], bit: i64) -> DecodedFrameView {
    decode_view(data, bit, false, None, None, true, RECORD_LIMIT)
}

/// Reference control view with the caller's film precision and lazy reference widths.
#[cfg(test)]
pub(crate) fn decode_control_view_contextual(
    data: &[u8],
    bit: i64,
    encoding: Option<&PositionEncoding>,
    context: Option<&DecodeProfile>,
    category_references: bool,
) -> DecodedFrameView {
    decode_control_view_with_limit(
        data,
        bit,
        encoding,
        context,
        category_references,
        RECORD_LIMIT,
    )
}

pub(crate) fn decode_control_view_with_limit(
    data: &[u8],
    bit: i64,
    encoding: Option<&PositionEncoding>,
    context: Option<&DecodeProfile>,
    category_references: bool,
    record_limit: usize,
) -> DecodedFrameView {
    decode_view(
        data,
        bit,
        true,
        encoding,
        context,
        category_references,
        record_limit,
    )
}

fn decode_view(
    data: &[u8],
    bit: i64,
    is_control: bool,
    encoding: Option<&PositionEncoding>,
    context: Option<&DecodeProfile>,
    category_references: bool,
    record_limit: usize,
) -> DecodedFrameView {
    let mut out = DecodedFrameView {
        control_entries: Vec::new(),
        start_bit: bit,
        end_bit: bit,
        kinds: Vec::new(),
        fields: Vec::new(),
        stop: FrameViewStop::Truncated,
    };
    let cursor = Cursor::guarded(data, bit);
    let mut r = ComponentReader {
        category_references,
        snapshot: false,
        references: Vec::new(),
        reference_widths: context.map(|c| ComponentWidths {
            movement: &c.movement,
            mpp: c.mpp,
            maximum: u64::MAX,
        }),
        width_error: None,
        context_error: None,

        live_grammar: None,
        movement_slot: None,
        diagnostics: Default::default(),
        cursor,
        fields: Vec::new(),
        position_encoding: encoding,
    };
    let result = walk(
        &mut r,
        is_control,
        &mut out.kinds,
        &mut out.control_entries,
        record_limit,
    );
    out.end_bit = r.cursor.position;
    out.fields = r.fields;
    out.stop = r
        .context_error
        .map_or_else(|| result.unwrap_or(FrameViewStop::Truncated), unsupported);
    out
}

fn unsupported(reason: &str) -> FrameViewStop {
    FrameViewStop::Unsupported {
        reason: reason.into(),
    }
}

fn walk(
    r: &mut ComponentReader<'_>,
    is_control: bool,
    kinds: &mut Vec<u8>,
    entries: &mut Vec<ControlEntry>,
    record_limit: usize,
) -> Option<FrameViewStop> {
    for record in 0..=record_limit.min(r.cursor.source_bits()) {
        if !r.bit(&format!("record[{record}].present"))? {
            return Some(FrameViewStop::Complete);
        }
        // The continuation bit precedes the record budget check. In particular,
        // an exactly-full list may still close; a missing terminator is truncated.
        if record == record_limit {
            return Some(FrameViewStop::RecordLimit);
        }
        let kind = r.r(
            &format!("record[{record}].kind"),
            if is_control { 2 } else { 7 },
        )? as u8;
        kinds.push(kind);
        if !is_control {
            return Some(unsupported("message payload"));
        }
        // References disagree about kind 3, including whether a baseline follows.
        // Preserve the established prefix and stop before choosing either layout.
        if kind == 3 {
            return Some(unsupported("control handler kind 3"));
        }
        let start_bit = r.cursor.position;
        let baseline = r.gated_value("control.baseline", 7, true)?.map(|v| v as u8);
        if kind == 2 {
            let value = r
                .gated_value("control.kind2.value", 13, true)?
                .map(|v| v as u16);
            entries.push(ControlEntry {
                start_bit,
                end_bit: r.cursor.position,
                baseline,
                payload: ControlPayload::Kind2 { value },
            });
            continue;
        }
        if kind == 1 {
            let index = r.r("control.index", 5)? as u8;
            let records = commands::read(r, baseline.is_some())?;
            entries.push(ControlEntry {
                start_bit,
                end_bit: r.cursor.position,
                baseline,
                payload: ControlPayload::Commands { index, records },
            });
            continue;
        }
        r.guard_source("control.index_and_input", 6)?;
        let index = r.r("control.index", 5)? as u8;
        let mut input = ControlInput {
            index,
            short: None,
            analog: None,
            third_analog: None,
            extra: None,
            flags: None,
            secondary: None,
        };
        if r.bit("control.input.present")? {
            input.short = r
                .gated_value("control.input.second_field", 2, true)?
                .map(|v| v as u8);
            // Reference consumeCoupleAnalogique guards both scalars and the
            // following presence bit as one 13-bit group.
            r.guard_source("control.input.analog_group", 13)?;
            input.analog = Some([
                r.r("control.input.analog[0]", 6)? as u8,
                r.r("control.input.analog[1]", 6)? as u8,
            ]);
            if r.bit("control.input.third_analog.present")? {
                input.third_analog = Some(r.r("control.input.third_analog", 5)? as u8);
            }
            if r.bit("control.input.extra.present")? {
                input.extra = Some(r.r("control.input.extra", 6)? as u8);
            }
            if r.bit("control.input.alternate")? {
                input.flags = Some(r.r("control.input.flags", 5)? as u8);
            }
            let action = control::actions(r)?;
            match action {
                control::ActionBlockRead::Decoded => {}
                control::ActionBlockRead::MissingPositionContext => {
                    return Some(unsupported("action aim position context unavailable"));
                }
            }
        }
        if r.bit("control.secondary.present")? {
            let start = usize::try_from(r.cursor.position).ok()?;
            if !secondary::read(r)? {
                return Some(unsupported(
                    "secondary control position context unavailable",
                ));
            }
            input.secondary = Some(crate::theater::parser::packet::BitRange {
                start,
                end: usize::try_from(r.cursor.position).ok()?,
            });
        }
        entries.push(ControlEntry {
            start_bit,
            end_bit: r.cursor.position,
            baseline,
            payload: ControlPayload::Input(input),
        });
    }
    unreachable!("the final continuation bit terminates or hits the record limit")
}
