//! Reference signed-index portability error.

/// An explicit portability boundary, not a reference source-data refusal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("kill-source option {field}={value} exceeds this target's execution index domain")]
pub(crate) struct KillDecodeOptionError {
    pub field: &'static str,
    pub value: i64,
}
