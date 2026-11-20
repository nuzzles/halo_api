//! Reference data models.
use crate::theater::parser::v41::replication::source::*;
#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityComponentRead {
    pub index: usize,
    pub name: crate::theater::film::FilmName,
    pub start_bit: i64,
    /// Component payload end, including any recorded corruption-check fields.
    pub end_bit: i64,
    pub status: ComponentReadStatus,
    pub field_range: std::ops::Range<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityRecord {
    pub header: RecordHeader,
    /// Archetype explicitly recorded by NEW. DELTA/DELETE records omit it;
    /// the parser's inherited schema binding is not published as wire data.
    pub archetype: Option<u32>,
    /// Recorded component-presence mask, when this record carries one.
    pub mask: Option<u64>,
    /// All record fields in read order. Component ranges refer into this buffer.
    pub fields: Vec<ComponentField>,
    pub default_state: Option<DefaultState>,
    pub components: Vec<EntityComponentRead>,
    pub end_bit: i64,
    pub stop: EntityViewStop,
}

impl EntityRecord {
    /// Fields outside every component's addressable source range.
    pub fn record_scope_fields(&self) -> impl Iterator<Item = &ComponentField> {
        self.fields.iter().filter(|field| {
            !self.components.iter().any(|component| {
                let (Ok(start), Ok(end)) = (
                    usize::try_from(component.start_bit),
                    usize::try_from(component.end_bit),
                ) else {
                    return false;
                };
                field.bit >= start && field.bit < end
            })
        })
    }

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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
