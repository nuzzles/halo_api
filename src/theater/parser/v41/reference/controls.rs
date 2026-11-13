//! Test-only historical reference analysis control traversal for the unchanged d61443e
//! endpoint oracle. Frozen from halo_api f2ff6fd; never used by Film::parse.
//! New production layouts are checked against separate reference reader fixtures.
use crate::theater::parser::v41::chunks::replication::source::replication_stream::models::frame_views::{
    DecodedFrameView, FrameViewStop,
};
use crate::theater::parser::v41::chunks::replication::components::control;
use crate::theater::parser::v41::chunks::replication::components::cursor::ComponentCursor as Cursor;
use crate::theater::parser::v41::chunks::replication::components::position::PositionEncoding;
use crate::theater::parser::v41::chunks::replication::components::reader::{
    ComponentReader, ComponentWidths,
};
use crate::theater::parser::v41::context::profile::DecodeProfile;

/// Reference control view with the caller's film precision and lazy reference widths.
pub(crate) fn decode_control_view_contextual(
    data: &[u8],
    bit: i64,
    encoding: Option<&PositionEncoding>,
    context: Option<&DecodeProfile>,
) -> DecodedFrameView {
    let default_encoding = DecodeProfile::default()
        .component_encoding()
        .expect("default reference widths");
    decode_view(
        data,
        bit,
        true,
        Some(encoding.unwrap_or(&default_encoding)),
        context,
    )
}

fn decode_view(
    data: &[u8],
    bit: i64,
    is_control: bool,
    encoding: Option<&PositionEncoding>,
    context: Option<&DecodeProfile>,
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
        category_references: true,
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
    let result = walk(&mut r, is_control, &mut out.kinds, data);
    out.end_bit = r.cursor.position;
    out.fields = r.fields;
    out.stop = result.unwrap_or(FrameViewStop::Truncated);
    out
}

fn unsupported(reason: &str) -> FrameViewStop {
    FrameViewStop::Unsupported {
        reason: reason.into(),
    }
}

fn walk<'a>(
    r: &mut ComponentReader<'a>,
    is_control: bool,
    kinds: &mut Vec<u8>,
    data: &'a [u8],
) -> Option<FrameViewStop> {
    for record in 0..64 {
        if !r.bit(&format!("record[{record}].present"))? {
            return Some(FrameViewStop::Complete);
        }
        let kind = r.r(
            &format!("record[{record}].kind"),
            if is_control { 2 } else { 7 },
        )? as u8;
        kinds.push(kind);
        if !is_control {
            return Some(unsupported("message payload"));
        }
        match kind {
            3 => continue,
            0 => {}
            _ => return Some(unsupported("control handler kind 1 or 2")),
        }
        r.gate("control.baseline", 7, true)?;
        r.guard_source("control.index_and_input", 6)?;
        r.r("control.index", 5)?;
        if r.bit("control.input.present")? {
            let _ = r
                .gated_value("control.input.second_field", 2, true)?
                .map(|v| v as u8);
            // Reference consumeCoupleAnalogique guards both scalars and the
            // following presence bit as one 13-bit group.
            r.guard_source("control.input.analog_group", 13)?;
            let _ = Some([
                r.r("control.input.analog[0]", 6)? as u8,
                r.r("control.input.analog[1]", 6)? as u8,
            ]);
            if r.bit("control.input.third_analog.present")? {
                let _ = Some(r.r("control.input.third_analog", 5)? as u8);
            }
            if r.bit("control.input.extra.present")? {
                let _ = Some(r.r("control.input.extra", 6)? as u8);
            }
            if r.bit("control.input.alternate")? {
                let _ = Some(r.r("control.input.flags", 5)? as u8);
            }
            // The reference action decoder guards its first bit, then uses the
            // zero-tail reader and checks the consumed endpoint afterward.
            r.guard_source("control.actions.first_bit", 1)?;
            r.cursor = Cursor::signed(data, r.cursor.position, None);
            let action = control::actions(r)?;
            if r.cursor.position > (data.len() as i64 * 8) {
                return None;
            }
            match action {
                control::ActionBlockRead::Decoded => {}
                control::ActionBlockRead::MissingPositionContext => {
                    return Some(unsupported("action aim position context unavailable"));
                }
            }
            r.cursor = Cursor::guarded(data, r.cursor.position);
        }
        if r.bit("control.secondary.present")? {
            return Some(unsupported("secondary control block"));
        }
    }
    Some(FrameViewStop::RecordLimit)
}
