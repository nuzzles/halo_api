use super::super::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NativeFilmPacket {
    pub header: FilmPacket,
    /// All nested bit offsets are relative to this header's payload_offset.
    /// Stops and synthetic padded reads remain in the native reader result.
    pub body: NativeFilmPacketBody,
    /// An independent direct event-head read, never an inferred gameplay event.
    pub event_head: Option<DecodedHeadEvent>,
    /// Sequential event-layout reads at payload bit one. An absent runtime
    /// code-15 gate stops this read; fixed-size opaque bodies remain marked.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_list: Option<NativeEventListRead>,
    /// Entity/control reads beginning at a completed nonempty event list's
    /// terminator. The original generic message-view stop remains in `body`.
    /// Runtime layout selection is recorded by the enclosing event list/gate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_continuation: Option<NativeEventContinuation>,
    /// Dedicated native padded reads, including selected-reader refusals.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native_head: Option<NativePacketHeadRead>,
    /// Independent native damage projection and raw fields. This endpoint does
    /// not replace event-list termination or imply inferred player identities.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub damage_read: Option<NativeWeaponDamageRead>,
    /// Bounded roster trace, including the first rejected entry.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub roster_read: Option<NativeRosterRead>,
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
pub enum NativeFilmPacketBody {
    Frame(Box<ProductionFrame>),
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
