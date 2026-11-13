//! Default-state sections preceding component updates in NEW/baseline records.
use crate::theater::parser::packet::BitRange;
use crate::theater::parser::v41::chunks::replication::source::components::field::ComponentField;

/// Source-backed default-state section. Guards and masks remain on the enclosing
/// record. This section owns its ordered body fields and any default-state check.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DefaultState {
    pub source: BitRange,
    pub fields: Vec<ComponentField>,
    pub status: DefaultStateStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DefaultStateStatus {
    /// A required layout setting is unavailable; the retained prefix is exact.
    ContextUnavailable,
    Complete,
    /// An established layout skips fields without decoding their structure.
    Opaque,
    Unsupported,
    Truncated,
}

impl DefaultState {
    /// Type-9 managed-player creation names an engine participant with one
    /// six-bit word. Values 32..63 remain visible in `fields` but are outside
    /// the reader's admitted participant domain.
    pub(crate) fn managed_player_participant(&self) -> Option<u8> {
        if self.status != DefaultStateStatus::Complete {
            return None;
        }
        let mut candidates = self
            .fields
            .iter()
            .filter(|field| field.name == "player_index");
        let field = candidates.next()?;
        if candidates.next().is_some()
            || field.width != 6
            || field.raw.bit_len != 6
            || field.bit < self.source.start
            || field.bit.checked_add(6)? > self.source.end
        {
            return None;
        }
        let raw = field.raw.low_u64();
        (raw < 32).then_some(raw as u8)
    }
}
