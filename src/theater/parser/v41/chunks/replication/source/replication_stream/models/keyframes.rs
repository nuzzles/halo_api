//! Reference data models.
use crate::theater::parser::v41::chunks::replication::source::*;
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum KeyframeStop {
    Complete,
    Truncated,
    InvalidArchetype,
    UnsupportedDefault,
    /// A default/component body ended without an established guard policy.
    MissingCorruptionCheckContext,
    /// A variable-width handle needs an unestablished configuration-zero range.
    UnsupportedReferenceConfiguration,
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
    /// External vehicle construction gate used while reading a snapshot
    /// component; this is parser context, not a recorded wire field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_vehicle_type_gate_used: Option<bool>,
    pub stop: KeyframeStop,
}

impl KeyframeRecord {
    /// Participant explicitly named by a complete type-9 snapshot default.
    pub fn managed_player_participant(&self) -> Option<u8> {
        if self.archetype == 9 && self.stop == KeyframeStop::Complete {
            self.default_state.as_ref()?.managed_player_participant()
        } else {
            None
        }
    }
}
