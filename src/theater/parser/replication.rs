//! Packet-level integration for the sequential reference port.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyframeRecoveryPolicy {
    #[default]
    SequentialOnly,
    LevelUp,
}
