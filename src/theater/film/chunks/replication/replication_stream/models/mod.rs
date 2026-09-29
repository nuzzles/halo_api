use super::super::*;

pub mod records;
pub use records::{RecordHeader, RecordKind};

pub mod datums;
pub use datums::{DatumDecodeError, DatumEntry, DatumTable};

pub mod production_frame;
pub use production_frame::{ProductionAdmissionDiagnostics, ProductionEntityEnd, ProductionFrame};

pub mod kill_event_chain;
pub use kill_event_chain::{
    EventField, EventFieldStage, EventFieldValue, EventListRead, EventListStop, EventRecord,
    KillEventFields,
};

pub mod world;
pub use world::{FilmViewAdmission, NewBindingRefusal};

pub mod anticipated_bindings;
pub use anticipated_bindings::AnticipatedDeclaration;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// Decoded contents of a type-0 [`ReplicationStreamPacket`].
///
/// The initial frame read and the event continuation remain separate because
/// they begin at distinct recorded bit boundaries.
pub struct FramePacket {
    /// Generic frame read beginning at the packet's recorded frame boundary.
    pub frame: FrameRead,
    /// Sequential event-layout read at payload bit one.
    pub events: EventListRead,
    /// Entity/control records following a completed nonempty event list.
    pub continuation: Option<EventContinuation>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventContinuation {
    /// Payload-relative bit immediately after the event-list terminator.
    pub start_bit: usize,
    /// Only the following entity/control views are counted in this result.
    /// Refusals and partial reads do not erase the established list boundary.
    pub frame: FrameRead,
    /// Whether this attempt may update the grammar state used by later packets.
    #[serde(default)]
    pub state_policy: ContinuationStatePolicy,
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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContinuationStatePolicy {
    /// Older exports did not report this policy.
    #[default]
    Unknown,
    Applied,
    /// Pinned code-0 body readers disagree on a field's gate polarity. Keep the
    /// generic continuation attempt but do not propagate its binding effects.
    IsolatedConflictingDamageGrammar,
}

pub mod entity_records;
pub use entity_records::{
    BindingOrigin, EntityComponentAttempt, EntityComponentSpan, EntityRecord, EntityViewStop,
};

pub mod frame_views;
pub use frame_views::{ControlEntry, DecodedFrameView, FrameViewStop};

pub mod keyframes;
pub use keyframes::{KeyframeComponentSpan, KeyframeRecord, KeyframeStop};

pub mod keyframe_chain;
pub use keyframe_chain::{KeyframeChainAttempt, KeyframeChainStop, KeyframeTable};
