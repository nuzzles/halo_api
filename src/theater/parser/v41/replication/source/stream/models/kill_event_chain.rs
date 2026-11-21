//! Reference data models.
use crate::theater::parser::v41::reference::references::{EventReference, ReferenceTriplet};
/// Read order and source ranges from the reference event-layout walker. Scalar
/// values remain unnamed unless their semantics are established elsewhere.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventFieldStage {
    Header,
    References,
    Body,
    /// Optional guard and sentinel following a nonempty message body.
    CorruptionCheck,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventFieldValue {
    Scalar(u64),
    /// A supported layout skipped this region without interpreting its fields.
    Opaque,
    /// The requested read exceeded the source; no zero value is substituted.
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventField {
    /// Payload-relative start of the read or read attempt.
    pub bit: usize,
    /// Requested width. For `Unavailable`, this is not a recorded source range.
    pub width: usize,
    pub stage: EventFieldStage,
    pub value: EventFieldValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventRecord {
    pub start_bit: usize,
    /// End of the furthest source-backed field, excluding unavailable attempts.
    /// Nested partial reads remain inside this extent even if their reader
    /// returned without advancing the parent cursor.
    pub end_bit: usize,
    pub code: Option<u8>,
    pub body_start_bit: Option<usize>,
    /// A complete reference header. Outer absence means the header did not decode.
    /// Inner absent slots correspond exactly to unset presence bits.
    pub references: Option<ReferenceTriplet<EventReference>>,
    /// Header, reference, body, and corruption fields in their recorded read order.
    pub fields: Vec<EventField>,
    /// The layout reached its end; opaque fields can still be present.
    pub layout_complete: bool,
}

impl EventRecord {
    /// Recorded body only, excluding the reference header and guard trailer.
    pub(crate) fn body_range(&self) -> Option<std::ops::Range<usize>> {
        if !self.layout_complete {
            return None;
        }
        let start = self.body_start_bit?;
        let end = self
            .fields
            .iter()
            // Guards are a suffix: inspect at most the trailer and its predecessor,
            // rather than scanning every body field again during projection.
            .rev()
            .take_while(|field| field.stage == EventFieldStage::CorruptionCheck)
            .last()
            .map_or(self.end_bit, |field| field.bit);
        (self.start_bit.checked_add(8)? <= start && start <= end && end <= self.end_bit)
            .then_some(start..end)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventListStop {
    Terminator,
    SourceBoundary,
    Truncated,
    UnsupportedCode,
    UnsupportedReferences,
    /// A field in the recorded event-version table is unavailable.
    MissingEventVersion {
        event_code: u8,
    },
    /// The table entry names a different writer, including when it is not the
    /// event code at the current packet position. No references/body are read.
    UnsupportedEventVersion {
        event_code: u8,
        recorded: u32,
        supported: u32,
    },
    /// Configuration zero requires an unestablished global reference range.
    UnsupportedReferenceConfiguration,
    UnsupportedBody,
    MissingRuntimeGate15,
    MissingVariantContext,
    MissingKillPlaybackContext,
    ContradictoryKillPlaybackContext,
    /// The body ended, but the presence of its guard trailer is unknown.
    MissingCorruptionCheckContext,
    RecordLimit,
    InvalidStart,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventListRead {
    pub start_bit: usize,
    pub end_bit: usize,
    pub gate15: Option<bool>,
    /// External script-prefix selection actually used in this event list.
    pub script_prefix_context_used: Option<bool>,
    /// Contextual replay hypotheses actually used by this walk. Canonical
    /// parsing leaves this empty; these are not fields recorded in the film.
    pub assumptions: Vec<EventAssumption>,
    /// External tail selection actually used for event 85. This is parser
    /// context, not a field recorded in the film.
    pub kill_tail_context_used: Option<bool>,
    pub records: Vec<EventRecord>,
    /// Recorded list terminator, if reached.
    pub terminator: Option<EventField>,
    pub stop: EventListStop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventAssumption {
    /// Both unrecorded playback feature switches retain their disabled defaults.
    KillPlaybackDisabled,
    /// Unavailable variant context is interpreted using the non-engine-1 path.
    TeleportEngineNotOne,
}
