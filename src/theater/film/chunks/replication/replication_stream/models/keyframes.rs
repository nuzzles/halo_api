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
pub struct KeyframeComponentRead {
    /// Reference returned variant; None marks an incomplete read or older export.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variant: Option<u32>,
    /// Reference ported result; None marks an incomplete read or older export.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ported: Option<bool>,
    pub index: usize,
    pub name: String,
    pub start_bit: i64,
    /// Includes the optional corruption check after the component.
    pub end_bit: i64,
    pub fields: Vec<ComponentField>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct KeyframeRecord {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub references: Vec<crate::theater::film::UnitReference>,
    pub start_bit: i64,
    pub end_bit: i64,
    pub id: u32,
    /// Decoded type. The reference isolated reader uses six low bits; the complete
    /// raw header word remains in fields. Table readers admit the full word first.
    pub archetype: u32,
    pub fields: Vec<ComponentField>,
    pub components: Vec<KeyframeComponentRead>,
    pub stop: KeyframeStop,
}
