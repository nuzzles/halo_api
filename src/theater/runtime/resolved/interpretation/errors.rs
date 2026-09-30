/// Structural errors are separate from unsupported record forms in diagnostics.
#[derive(Debug, thiserror::Error)]
pub(crate) enum BootstrapReadError {
    /// The film version has no checked decoder.
    #[error("unsupported Theater film major version {0}; supported: 41")]
    UnsupportedVersion(i32),
    /// A supported capture's independent guards disagree.
    #[error("inconsistent Theater data: {0}")]
    Inconsistent(String),
}
