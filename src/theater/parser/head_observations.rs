//! Reference gameplay observations assembled from bounded event-head reads.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct BipedPickupStats {
    pub packets: usize,
    pub type_9: usize,
    pub type_8: usize,
    /// Native fallback counter; the 0xC4 packet gate admits only types 8/9.
    #[serde(default)]
    pub other_type: usize,
    pub published: usize,
    pub multi_event: usize,
    pub refused_no_ref: usize,
    pub refused_no_catalog: usize,
    pub refused_off_band: usize,
    pub unexpected_wide_ref: usize,
    /// Rust additionally distinguishes truncated data from an absent reference.
    pub truncated: usize,
}
