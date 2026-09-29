//! Reference data models.
use super::*;
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ZoomRead {
    pub head: DecodedHeadEvent,
    pub source_bits: usize,
    pub padded_bits: usize,
    /// Reference domain-four slot; check unit_reference_recorded before treating
    /// every reference bit as recorded rather than padded.
    pub slot: u32,
    /// Reference level; check level_recorded before interpreting it as recorded.
    pub level: u8,
}

use super::event_heads;
use crate::theater::parser::bits::Cursor;

/// Reference decodeZoomHead semantics. The head's references and logical extent
/// retain provenance for the reader's synthetic zero tail. A missing first
/// reference still refuses publication; the config bit is a source-scanner gate.
pub(super) fn decode_zoom_head(data: &[u8]) -> Option<ZoomRead> {
    let head = event_heads::decode_head_with_cursor(Cursor::new_padded(data, 0))?;
    if head.kind != 21 {
        return None;
    }
    let (slot, level) = head.zoom()?;
    Some(ZoomRead {
        source_bits: data.len() * 8,
        padded_bits: head.end_bit.saturating_sub(data.len() * 8),
        slot,
        level,
        head,
    })
}
