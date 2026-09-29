//! Reference data models.
use crate::theater::film::*;
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum KeyframeStop {
    Complete,
    Truncated,
    InvalidArchetype,
    UnsupportedDefault,
    UnsupportedComponent { index: usize, name: String },
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct KeyframeComponentSpan {
    /// Reference returned variant; None marks an incomplete read or older export.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variant: Option<u32>,
    /// Reference ported result; None marks an incomplete read or older export.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ported: Option<bool>,
    /// Ordered retained fields, independent of overlapping source bit ranges.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub field_range: Option<[usize; 2]>,
    /// Ordered diagnostic observations emitted by this attempt. Absent in older serialized data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observation_range: Option<[usize; 2]>,
    pub index: usize,
    pub name: String,
    pub start_bit: i64,
    /// Includes the optional corruption check after the component.
    pub end_bit: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct KeyframeRecord {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub references: Vec<crate::theater::film::UnitReference>,
    #[serde(
        default,
        skip_serializing_if = "crate::theater::film::FilmReadDiagnostics::is_empty"
    )]
    pub diagnostics: crate::theater::film::FilmReadDiagnostics,
    pub start_bit: i64,
    pub end_bit: i64,
    pub id: u32,
    /// Decoded type. The reference isolated reader uses six low bits; the complete
    /// raw header word remains in fields. Table readers admit the full word first.
    pub archetype: u32,
    pub fields: Vec<ComponentField>,
    pub components: Vec<KeyframeComponentSpan>,
    /// Ordered attempts, including the final unsupported or incomplete reader.
    /// Empty in older exports where this trace was unavailable.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attempts: Vec<KeyframeComponentSpan>,
    pub stop: KeyframeStop,
}
