//! Snapshot of the effective native parser configuration.
use super::*;
use serde::{Deserialize, Serialize};

/// Frozen parser configuration, separate from recorded wire data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NativeFrameMetadata {
    pub extra_fields: bool,
    pub id_low_bits: i64,
    pub id_base: u32,
    pub new_default_state_bits: i64,
    pub packet_preamble_bits: i64,
    pub profile: NativeScanProfile,
}
impl NativeFrameConfig {
    pub(crate) fn snapshot(&self) -> NativeFrameMetadata {
        NativeFrameMetadata {
            extra_fields: self.extra_fields,
            id_low_bits: self.id_low_bits,
            id_base: self.id_base,
            new_default_state_bits: self.new_default_state_bits,
            packet_preamble_bits: self.packet_preamble_bits,
            profile: self.context.profile.snapshot(),
        }
    }
}
