//! The v41 summary record grammar is not yet established.
//! [`SummaryPacketBody`](super::SummaryPacketBody) retains the recorded count and
//! the entire source range of its record stream. Guarded candidate field reads
//! belong to the resolved interpretation layer, not canonical record models.

use crate::theater::film::BitRange;

/// The recorded summary stream whose sequential record grammar is unresolved.
/// Every bit remains available in the enclosing packet payload; no inferred
/// record boundaries are substituted for this opaque source range.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SummaryRecordStream {
    pub source: BitRange,
}
