//! Native data models.
use crate::theater::film::*;
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum BindingOrigin {
    #[default]
    Supplied,
    SequentialKeyframe {
        bit: usize,
    },
    Creation {
        bit: i64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum EntityViewStop {
    Complete,
    Truncated,
    InvalidEncoding,
    InvalidWidthOverride {
        adjustment: Box<crate::theater::film::NativeWidthAdjustment>,
    },
    InvalidArchetype {
        archetype: u32,
    },
    MissingBinding {
        id: u32,
    },
    GenerationMismatch {
        id: u32,
        bound_id: u32,
    },
    UnsupportedDefault {
        archetype: u32,
    },
    InvalidComponent {
        index: usize,
    },
    UnsupportedComponent {
        index: usize,
        name: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EntityComponentSpan {
    pub index: usize,
    pub name: String,
    pub start_bit: i64,
    /// Component payload end, before any corruption check.
    pub end_bit: i64,
}

/// An attempted component, including known fields before an unsupported or truncated body.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EntityComponentAttempt {
    pub span: EntityComponentSpan,
    /// Native returned variant, including zero for a calibrated skip and
    /// u32::MAX for no variant. None marks an incomplete read or older export.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variant: Option<u32>,
    /// true = complete, false = unsupported, None = truncated.
    pub status: Option<bool>,
    /// Ordered diagnostic observations emitted by this attempt. Absent in older serialized data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observation_range: Option<[usize; 2]>,
    pub field_start: usize,
    pub field_end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EntityRecord {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub references: Vec<crate::theater::film::NativeUnitReference>,
    #[serde(
        default,
        skip_serializing_if = "crate::theater::film::FilmReadDiagnostics::is_empty"
    )]
    pub diagnostics: crate::theater::film::FilmReadDiagnostics,
    pub header: RecordHeader,
    pub archetype: Option<u32>,
    /// Configured signed width returned as native New Trace.DefaultBits, even
    /// when the selected archetype decoder ignores it. This is decoder context,
    /// not a recorded bit length. None denotes an older export, a non-New record,
    /// a traversal that did not reach the default-state trace, or a legacy width
    /// outside the native signed domain.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_state_bits: Option<i64>,
    /// A disabled/missing default decoder used the configured fallback. Verified
    /// zero-bit stubs with a zero configured skip leave this false.
    #[serde(default)]
    pub default_state_fallback: bool,
    pub binding_origin: Option<BindingOrigin>,
    pub mask: Option<u64>,
    pub fields: Vec<ComponentField>,
    pub components: Vec<EntityComponentSpan>,
    #[serde(default)]
    pub attempts: Vec<EntityComponentAttempt>,
    pub end_bit: i64,
    /// Tail bits supplied by the native padding convention, absent from the payload.
    #[serde(default)]
    pub padded_bits: usize,
    pub stop: EntityViewStop,
}
