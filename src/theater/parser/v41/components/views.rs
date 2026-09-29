//! Message and control views. Unsupported paths mirror explicit LevelUp gaps.
use super::Cursor;
use super::{Reader, control};
pub(crate) use crate::theater::film::chunks::replication::components::views::{
    ControlEntry, DecodedFrameView, FrameViewStop,
};

/// Reference view A reader with signed positions and wrapping source guards.
/// Invalid negative reads retain the reference panic behavior.
pub(crate) fn decode_message_view_signed(data: &[u8], bit: i64) -> DecodedFrameView {
    decode_view(data, bit, false, None, None)
}

/// Reference control view with the caller's film precision and lazy reference widths.
pub(crate) fn decode_control_view_contextual(
    data: &[u8],
    bit: i64,
    encoding: Option<&super::PositionEncoding>,
    context: Option<&crate::theater::parser::v41::ReaderContext>,
) -> DecodedFrameView {
    let default_encoding = crate::theater::parser::v41::ScanProfile::default()
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
    encoding: Option<&super::PositionEncoding>,
    context: Option<&crate::theater::parser::v41::ReaderContext>,
) -> DecodedFrameView {
    let mut out = DecodedFrameView {
        control_entries: Vec::new(),
        diagnostics: Default::default(),
        start_bit: bit,
        end_bit: bit,
        padded_bits: 0,
        kinds: Vec::new(),
        fields: Vec::new(),
        stop: FrameViewStop::Truncated,
    };
    let cursor = Cursor::guarded(data, bit);
    let mut r = Reader {
        reference_widths: context.map(|c| super::ComponentWidths {
            movement: &c.profile.movement,
            mpp: c.profile.mpp,
            maximum: u64::MAX,
        }),
        width_error: None,

        live_grammar: None,
        position_capture: None,
        position_start: 0,
        position_slot: 0,
        position_fallback: false,
        movement_slot: None,
        references: Vec::new(),
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
        data,
    );
    out.end_bit = r.cursor.position;
    out.padded_bits = usize::try_from(out.end_bit)
        .unwrap_or(0)
        .saturating_sub((data.len() * 8).max(usize::try_from(bit).unwrap_or(0)));
    out.fields = r.fields;
    out.diagnostics = (!r.diagnostics.is_empty()).then(|| Box::new(r.diagnostics));
    out.stop = result.unwrap_or(FrameViewStop::Truncated);
    out
}

fn unsupported(reason: &str) -> FrameViewStop {
    FrameViewStop::Unsupported {
        reason: reason.into(),
    }
}

fn walk<'a>(
    r: &mut Reader<'a>,
    is_control: bool,
    kinds: &mut Vec<u8>,
    entries: &mut Vec<ControlEntry>,
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
        let start_bit = r.cursor.position;
        let baseline = r.gated_value("control.baseline", 7, true)?.map(|v| v as u8);
        r.guard_source("control.index_and_input", 6)?;
        let index = r.r("control.index", 5)? as u8;
        let mut entry = ControlEntry {
            start_bit,
            end_bit: start_bit,
            index,
            baseline,
            short: None,
            analog: None,
            third_analog: None,
            extra: None,
            flags: None,
            action: None,
        };
        if r.bit("control.input.present")? {
            entry.short = r
                .gated_value("control.input.second_field", 2, true)?
                .map(|v| v as u8);
            // Reference consumeCoupleAnalogique guards both scalars and the
            // following presence bit as one 13-bit group.
            r.guard_source("control.input.analog_group", 13)?;
            entry.analog = Some([
                r.r("control.input.analog[0]", 6)? as u8,
                r.r("control.input.analog[1]", 6)? as u8,
            ]);
            if r.bit("control.input.third_analog.present")? {
                entry.third_analog = Some(r.r("control.input.third_analog", 5)? as u8);
            }
            if r.bit("control.input.extra.present")? {
                entry.extra = Some(r.r("control.input.extra", 6)? as u8);
            }
            if r.bit("control.input.alternate")? {
                entry.flags = Some(r.r("control.input.flags", 5)? as u8);
            }
            // The reference action decoder guards its first bit, then uses the
            // zero-tail reader and checks the consumed endpoint afterward.
            r.guard_source("control.actions.first_bit", 1)?;
            r.cursor = Cursor::signed(data, r.cursor.position, None);
            let action = control::actions(r)?;
            if r.cursor.position > (data.len() as i64 * 8) {
                return None;
            }
            entry.action = Some(match action {
                control::ActionBlockRead::Decoded(block) => block,
                control::ActionBlockRead::MissingPositionContext => {
                    return Some(unsupported("action aim position context unavailable"));
                }
            });
            r.cursor = Cursor::guarded(data, r.cursor.position);
        }
        if r.bit("control.secondary.present")? {
            return Some(unsupported("secondary control block"));
        }
        entry.end_bit = r.cursor.position;
        entries.push(entry);
    }
    Some(FrameViewStop::RecordLimit)
}
