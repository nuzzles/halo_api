use super::super::*;

pub mod packet_header;
pub use packet_header::{FilmPacketHeader, SourceSpan};

pub mod records;
pub use records::{RecordHeader, RecordKind};

pub mod roster_updates;
pub use roster_updates::{RosterEntry, RosterReport, RosterUpdate};

pub mod datums;
pub use datums::{DatumEntry, DatumTable};

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

pub mod player;
pub use player::{PlayerTableShorts, PlayerTableSlot};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilmPacket {
    pub header: FilmPacketHeader,
    pub body: FilmPacketBody,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FramePacket {
    /// Generic frame read beginning at the packet's recorded frame boundary.
    pub frame: Result<ProductionFrame, String>,
    /// Sequential event-layout read at payload bit one.
    pub events: EventListRead,
    /// Entity/control records following a completed nonempty event list.
    pub continuation: Option<EventContinuation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventContinuation {
    /// Payload-relative bit immediately after the event-list terminator.
    pub start_bit: usize,
    /// Only the following entity/control views are counted in this result.
    /// Refusals and partial reads do not erase the established list boundary.
    pub frame: Result<ProductionFrame, String>,
    /// Whether this attempt may update the grammar state used by later packets.
    #[serde(default)]
    pub state_policy: ContinuationStatePolicy,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)]
pub enum FilmPacketBody {
    Frame(Box<FramePacket>),
    Datums(DatumTable),
    Roster(RosterUpdate),
    Keyframes(KeyframeTable),
    /// Guarded captured-layout reads in wire order; intervening state is opaque.
    Summary {
        declared_events: u32,
        events: Vec<SummaryEvent>,
    },
    /// A reader could not start. The complete payload remains in the chunk.
    Refused {
        message: String,
    },
    /// No body reader selected; this does not imply an empty payload.
    Opaque,
}
