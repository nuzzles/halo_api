//! Temporary parser context for writer-dependent layouts.
use std::collections::BTreeMap;

/// External writer context needed for layouts that the film does not select.
/// Supply only values established for the recording; `None` preserves an
/// explicit parser stop when a branch cannot be decided from recorded fields.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct FilmParseContext {
    /// Script event 15's writer-side optional 15-bit prefix, when the
    /// recorder simulation cannot be read from complete game options.
    pub script_event_prefix: Option<bool>,
    /// Whether event 85 carries its 32/32/4-bit playback suffix. The film does
    /// not record the writer's runtime playback switches. A supplied value is
    /// reported on the affected event list, separate from its wire fields.
    pub kill_event_tail: Option<bool>,
    /// Per-record vehicle construction state. Keys use the supplied chunk's
    /// `RawFilmChunk.index` and zero-based packet index.
    pub vehicle_snapshot_gates: BTreeMap<VehicleSnapshotSite, bool>,
    /// Archetype independently established for an exact DELTA record. The
    /// supplied schema applies to that record only and never becomes a
    /// recorded NEW declaration or inherited parser binding.
    pub delta_record_archetypes: BTreeMap<DeltaRecordSite, u32>,
}
/// Source site of a DELTA header. `start_bit` is relative to its packet payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct DeltaRecordSite {
    pub chunk_index: usize,
    pub packet_index: usize,
    pub start_bit: i64,
    pub id: u32,
}
/// Exact source site of a vehicle snapshot whose unrecorded construction gate
/// is established by an external source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct VehicleSnapshotSite {
    pub chunk_index: usize,
    pub packet_index: usize,
    pub id: u32,
}
