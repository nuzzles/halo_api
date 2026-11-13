//! Reference data models.
use crate::theater::film::catalog::weapons::Weapon;
use crate::theater::parser::v41::chunks::replication::source::*;
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum EntityViewStop {
    Complete,
    Truncated,
    InvalidEncoding,
    /// A present component ended without an established guard policy.
    MissingCorruptionCheckContext,
    /// A variable-width handle needs an unestablished configuration-zero range.
    UnsupportedReferenceConfiguration,
    InvalidWidthOverride {
        field: String,
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
    /// A component layout depends on game state absent from the recording.
    /// No assumed value is used to select its body layout.
    RuntimeContextUnavailable {
        index: usize,
        name: String,
        field: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EntityComponentRead {
    pub index: usize,
    pub name: String,
    pub start_bit: i64,
    /// Component payload end, including any recorded corruption-check fields.
    pub end_bit: i64,
    pub status: ComponentReadStatus,
    pub fields: Vec<ComponentField>,
    /// Typed weapon identity decoded from a complete weapon-state-type-info
    /// component. None also covers an explicitly empty identity gate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weapon_id: Option<Weapon>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EntityRecord {
    pub header: RecordHeader,
    /// Archetype explicitly recorded by NEW. DELTA/DELETE records omit it;
    /// the parser's inherited schema binding is not published as wire data.
    pub archetype: Option<u32>,
    /// Recorded component-presence mask, when this record carries one.
    pub mask: Option<u64>,
    /// Fields outside the owned default-state and component sections.
    pub fields: Vec<ComponentField>,
    pub default_state: Option<DefaultState>,
    pub components: Vec<EntityComponentRead>,
    pub end_bit: i64,
    pub stop: EntityViewStop,
}

impl EntityRecord {
    /// Participant explicitly named by a complete type-9 NEW creation body.
    /// The raw six-bit value remains in `default_state` even when out of range.
    pub fn managed_player_participant(&self) -> Option<u8> {
        if self.header.kind == RecordKind::New
            && self.archetype == Some(9)
            && self.stop == EntityViewStop::Complete
        {
            self.default_state.as_ref()?.managed_player_participant()
        } else {
            None
        }
    }
}

/// Structural outcome for a selected component body reader.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ComponentReadStatus {
    Complete,
    Unsupported,
    Truncated,
    /// A required layout setting is unavailable; the retained prefix is exact.
    ContextUnavailable,
}
impl From<Option<bool>> for ComponentReadStatus {
    fn from(value: Option<bool>) -> Self {
        match value {
            Some(true) => Self::Complete,
            Some(false) => Self::Unsupported,
            None => Self::Truncated,
        }
    }
}
