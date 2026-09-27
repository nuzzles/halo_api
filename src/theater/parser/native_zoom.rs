//! Native padded zoom-head decoding, separate from bounded gameplay observations.
use super::{DecodedHeadEvent, bits::Cursor, event_heads};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeZoomRead {
    pub head: DecodedHeadEvent,
    pub source_bits: usize,
    pub padded_bits: usize,
    /// Native domain-four slot; check unit_reference_recorded before treating
    /// every reference bit as recorded rather than padded.
    pub slot: u32,
    /// Native level; check level_recorded before interpreting it as recorded.
    pub level: u8,
}

/// Native decodeZoomHead semantics. The head's references and logical extent
/// retain provenance for the reader's synthetic zero tail. A missing first
/// reference still refuses publication; the config bit is a source-scanner gate.
pub(crate) fn decode_native_zoom_head(data: &[u8]) -> Option<NativeZoomRead> {
    let head = event_heads::decode_head_with_cursor(Cursor::new_padded(data, 0))?;
    if head.kind != 21 {
        return None;
    }
    let (slot, level) = head.zoom()?;
    Some(NativeZoomRead {
        source_bits: data.len() * 8,
        padded_bits: head.end_bit.saturating_sub(data.len() * 8),
        slot,
        level,
        head,
    })
}
