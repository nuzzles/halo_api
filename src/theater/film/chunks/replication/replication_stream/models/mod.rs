use super::super::*;

pub mod packet_header;
pub use packet_header::{FilmPacketHeader, SourceSpan};

pub mod records;
pub use records::{RecordHeader, RecordKind};

pub mod roster_updates;
pub use roster_updates::{NativeRosterRead, RosterEntry, RosterReport, RosterUpdate};

pub mod datums;
pub use datums::{DatumEntry, DatumTable};

pub mod production_frame;
pub use production_frame::{ProductionAdmissionDiagnostics, ProductionEntityEnd, ProductionFrame};

pub mod kill_event_chain;
pub use kill_event_chain::{
    KillEventFields, NativeEventField, NativeEventFieldStage, NativeEventFieldValue,
    NativeEventListRead, NativeEventListStop, NativeEventRecord,
};

pub mod world;
pub use world::{FilmViewAdmission, NativeNewBindingRefusal};

pub mod anticipated_bindings;
pub use anticipated_bindings::AnticipatedDeclaration;

pub mod player;
pub use player::{
    NativePlayerSlotRead, NativeSlotField, NativeSlotValue, PlayerTableShorts, PlayerTableSlot,
};

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
    pub events: NativeEventListRead,
    /// Entity/control records following a completed nonempty event list.
    pub continuation: Option<NativeEventContinuation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeEventContinuation {
    /// Payload-relative bit immediately after the event-list terminator.
    pub start_bit: usize,
    /// Only the following entity/control views are counted in this result.
    /// Refusals and partial reads do not erase the established list boundary.
    pub frame: Result<ProductionFrame, String>,
    /// Whether this attempt may update the grammar state used by later packets.
    #[serde(default)]
    pub state_policy: NativeContinuationStatePolicy,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativeContinuationStatePolicy {
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
    Keyframes(NativeKeyframeTable),
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
