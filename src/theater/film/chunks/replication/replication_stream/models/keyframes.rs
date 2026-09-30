//! Reference data models.
use crate::theater::film::*;
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum KeyframeStop {
    Complete,
    Truncated,
    InvalidArchetype,
    UnsupportedDefault,
    UnsupportedComponent {
        index: usize,
        name: String,
    },
    /// A component layout depends on game state absent from the recording.
    RuntimeContextUnavailable {
        index: usize,
        name: String,
        field: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct KeyframeComponentRead {
    pub status: ComponentReadStatus,
    pub index: usize,
    pub name: String,
    pub start_bit: i64,
    /// Includes the optional corruption check after the component.
    pub end_bit: i64,
    pub fields: Vec<ComponentField>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct KeyframeRecord {
    pub start_bit: i64,
    pub end_bit: i64,
    pub id: u32,
    /// Decoded type. The reference isolated reader uses six low bits; the complete
    /// raw header word remains in fields. Table readers admit the full word first.
    pub archetype: u32,
    /// Fields outside the owned default-state and component sections.
    pub fields: Vec<ComponentField>,
    pub default_state: Option<DefaultState>,
    pub components: Vec<KeyframeComponentRead>,
    pub stop: KeyframeStop,
}
