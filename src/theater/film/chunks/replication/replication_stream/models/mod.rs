use super::super::*;

pub mod records;
pub use records::{RecordHeader, RecordKind};

pub mod datums;
pub use datums::{DatumComponentMask, DatumDecodeError, DatumEntry, DatumTable};

pub mod production_frame;
pub use production_frame::{ProductionEntityEnd, ProductionFrame};

pub mod kill_event_chain;
pub use kill_event_chain::{
    EventField, EventFieldStage, EventFieldValue, EventListRead, EventListStop, EventRecord,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// Decoded contents of a type-0 [`ReplicationStreamPacket`].
///
/// Views occur in wire order: messages, entities, then controls.
pub struct FramePacket {
    /// First payload bit, retained without assigning runtime semantics.
    /// First recorded payload bit. None means the payload contains no such bit.
    pub configuration: Option<bool>,
    /// Message view beginning at payload bit one. Each message owns its fields.
    pub events: EventListRead,
    /// Entity/control views starting at the message-list terminator.
    /// Refused when that boundary cannot be established safely.
    pub frame: FrameRead,
}

/// Outcome of a structurally selected frame read.
///
/// A decoded `ProductionFrame` retains its own partial-record stop information.
/// `Refused` means the reader could not select a valid structural profile and
/// therefore did not begin consuming frame records.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FrameRead {
    Decoded(Box<ProductionFrame>),
    Refused(FrameDecodeError),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, thiserror::Error)]
pub enum FrameDecodeError {
    #[error("profile value cannot be represented for {field}")]
    Width { field: String },
    #[error("frame traversal requires view classes and view tables")]
    MissingViewTables,
    #[error("message list did not establish an entity-view boundary")]
    IncompleteMessageList,
    #[error("message body layouts disagree on the entity-view boundary")]
    ConflictingMessageLayout,
}

impl FrameRead {
    pub fn decoded(&self) -> Option<&ProductionFrame> {
        match self {
            Self::Decoded(frame) => Some(frame.as_ref()),
            Self::Refused(_) => None,
        }
    }

    pub fn decoded_mut(&mut self) -> Option<&mut ProductionFrame> {
        match self {
            Self::Decoded(frame) => Some(frame.as_mut()),
            Self::Refused(_) => None,
        }
    }
}

pub mod entity_records;
pub use entity_records::{ComponentReadStatus, EntityComponentRead, EntityRecord, EntityViewStop};

pub mod frame_views;
pub use frame_views::{ControlEntry, DecodedFrameView, FrameViewStop};

pub mod keyframes;
pub use keyframes::{KeyframeComponentRead, KeyframeRecord, KeyframeStop};

pub mod keyframe_chain;
pub use keyframe_chain::{KeyframeChainAttempt, KeyframeChainStop, KeyframeTable};

pub mod default_state;
pub use default_state::{DefaultState, DefaultStateStatus};
