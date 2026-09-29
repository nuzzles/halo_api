//! Reference data models.
use crate::theater::film::*;
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum EntityViewStop {
    Complete,
    Truncated,
    InvalidEncoding,
    InvalidWidthOverride { field: String },
    InvalidArchetype { archetype: u32 },
    MissingBinding { id: u32 },
    GenerationMismatch { id: u32, bound_id: u32 },
    UnsupportedDefault { archetype: u32 },
    InvalidComponent { index: usize },
    UnsupportedComponent { index: usize, name: String },
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EntityComponentRead {
    pub index: usize,
    pub name: String,
    pub start_bit: i64,
    /// Component payload end, before any corruption check.
    pub end_bit: i64,
    /// Reference returned variant, including zero for a calibrated skip and
    /// u32::MAX for no variant. None marks an incomplete read or older export.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variant: Option<u32>,
    /// true = complete, false = unsupported, None = truncated.
    pub status: Option<bool>,
    pub fields: Vec<ComponentField>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EntityRecord {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub references: Vec<crate::theater::film::UnitReference>,
    pub header: RecordHeader,
    pub archetype: Option<u32>,
    /// Configured signed width returned as reference New Trace.DefaultBits, even
    /// when the selected archetype decoder ignores it. This is decoder context,
    /// not a recorded bit length. None denotes an older export, a non-New record,
    /// a traversal that did not reach the default-state trace, or a legacy width
    /// outside the reference signed domain.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mask: Option<u64>,
    pub fields: Vec<ComponentField>,
    pub components: Vec<EntityComponentRead>,
    pub end_bit: i64,
    pub stop: EntityViewStop,
}
