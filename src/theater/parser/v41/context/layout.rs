//! Wire-layout configuration shared by frame and keyframe readers.
use crate::theater::parser::v41::chunks::replication::components::widths::ComponentWidthOverrides;
use crate::theater::parser::v41::chunks::replication::{
    components::position::PositionEncoding, replication_stream::frame::header::RecordIdLayout,
};
use serde::{Deserialize, Serialize};
/// Reference New-record default routing and calibration widths. Neither skip is
/// a component length; terminal bits apply only after a successful New body.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct NewRecordLayout {
    pub deserialize_defaults: bool,
    pub fallback_default_bits: usize,
    /// Raw reference configuration, converted only if this New uses fallback state.
    /// None uses the bounded fallback width.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_fallback_default_bits: Option<i64>,
    pub terminal_bits: usize,
}
impl Default for NewRecordLayout {
    fn default() -> Self {
        Self {
            deserialize_defaults: true,
            fallback_default_bits: 0,
            reference_fallback_default_bits: None,
            terminal_bits: 0,
        }
    }
}
impl NewRecordLayout {
    fn is_default(&self) -> bool {
        self == &Self::default()
    }
}

/// Full-state keyframe framing, independent of New-record framing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct KeyframeLayout {
    pub header_bits: usize,
    pub size_word_bits: usize,
}
impl Default for KeyframeLayout {
    fn default() -> Self {
        Self {
            header_bits: 108,
            size_word_bits: 32,
        }
    }
}
impl KeyframeLayout {
    fn is_default(&self) -> bool {
        *self == Self::default()
    }
    pub(crate) fn valid(&self) -> bool {
        self.header_bits >= 64 && self.size_word_bits <= 32
    }
}

/// Values sourced from bootstrap/map context, not guessed from record contents.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RecordLayout {
    #[serde(default, skip_serializing_if = "KeyframeLayout::is_default")]
    pub keyframe_layout: KeyframeLayout,
    /// Explicit inherited simulation-state gate for configured reference readers. None
    /// uses the completed-body policy.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keyframe_simulation_complete: Option<bool>,
    #[serde(default, skip_serializing_if = "ComponentWidthOverrides::is_empty")]
    pub component_widths: ComponentWidthOverrides,
    #[serde(default, skip_serializing_if = "NewRecordLayout::is_default")]
    pub new_record: NewRecordLayout,
    /// Original signed reference ID width; admitted only after a non-End prefix.
    /// None uses ids.low_bits. Reference readers use wrapping slot arithmetic.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference_id_low_bits: Option<i64>,
    pub ids: RecordIdLayout,
    pub mpp_widths: [usize; 2],
    pub position: Option<PositionEncoding>,
    pub extra_fields: bool,
    pub corruption_check: bool,
}

impl RecordLayout {
    pub(in crate::theater) fn valid(&self) -> bool {
        (self.reference_id_low_bits.is_some()
            || (self.ids.low_bits <= 30 && self.ids.base <= 0x3fff_ffff))
            && self.mpp_widths.iter().all(|w| (1..=32).contains(w))
            && self.position.as_ref().is_none_or(PositionEncoding::valid)
    }
}
