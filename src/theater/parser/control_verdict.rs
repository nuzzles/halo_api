//! Native view-C closure verdict; raw entries remain on the original view.
use super::*;
use serde::{Deserialize, Serialize};

/// The reference's LectureVueC publication. Closure is a parser boundary check,
/// not proof that an inferred entry offset is a canonical recording boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct NativeControlVerdict {
    pub reached: bool,
    pub closed: bool,
    /// None when the preceding view prevented reaching the control view.
    pub stop: Option<FrameViewStop>,
    /// Only closed views publish entries. Raw/partial reads remain in DecodedFrameView.
    pub entries: Vec<NativeControlEntry>,
}
impl NativeControlVerdict {
    pub(crate) fn from_view(payload: &[u8], view: Option<&DecodedFrameView>) -> Self {
        let Some(view) = view else {
            return Self {
                reached: false,
                closed: false,
                stop: None,
                entries: Vec::new(),
            };
        };
        let closed = view.stop == FrameViewStop::Complete
            && control_view_closes_packet(payload, view.end_bit);
        Self {
            reached: true,
            closed,
            stop: Some(view.stop.clone()),
            entries: if closed {
                view.control_entries.clone()
            } else {
                Vec::new()
            },
        }
    }
}
/// The native final-view boundary rule: zero to seven remaining bits, all zero.
/// Signed invalid or padded endpoints cannot close a packet.
pub(crate) fn control_view_closes_packet(payload: &[u8], end_bit: i64) -> bool {
    let Ok(end) = usize::try_from(end_bit) else {
        return false;
    };
    let Some(remaining) = (payload.len() * 8).checked_sub(end) else {
        return false;
    };
    remaining <= 7
        && (end..payload.len() * 8).all(|bit| payload[bit / 8] & (1 << (7 - bit % 8)) == 0)
}
#[cfg(test)]
impl ProductionFrame {
    pub(crate) fn control_verdict(&self, payload: &[u8]) -> NativeControlVerdict {
        NativeControlVerdict::from_view(payload, self.controls.as_ref())
    }
}
