use std::sync::Arc;

use thiserror::Error;

/// Errors produced by this crate.
#[derive(Debug, Clone, Error)]
pub enum HaloApiError {
    /// An HTTP request completed but returned a non-2xx status.
    #[error("http request to {url} failed with status {status}")]
    HttpStatus {
        url: String,
        status: reqwest::StatusCode,
    },

    /// The underlying HTTP request itself failed (connection, timeout, decode, ...).
    #[error("network error: {0}")]
    Network(Arc<reqwest::Error>),

    /// No matchmade record exists for the given gamertag, or the gamertag itself is invalid.
    #[error("no Halo Infinite record found for gamertag \"{0}\"")]
    GamertagNotFound(String),

    /// The caller-supplied spartan token source (e.g. `XboxSpartanTokenProvider`) failed for a
    /// reason specific to that implementation, such as the underlying Xbox Live client
    /// returning an error while acquiring the XSTS ticket a spartan token is minted from.
    #[error("spartan token provider error: {0}")]
    SpartanTokenProvider(String),
}

impl From<reqwest::Error> for HaloApiError {
    fn from(err: reqwest::Error) -> Self {
        HaloApiError::Network(Arc::new(err))
    }
}

impl HaloApiError {
    /// Whether this error represents an HTTP 401 (Unauthorized) response, indicating the
    /// spartan token has expired or been revoked server-side.
    pub fn is_unauthorized(&self) -> bool {
        matches!(
            self,
            HaloApiError::HttpStatus { status, .. } if *status == reqwest::StatusCode::UNAUTHORIZED
        )
    }
}
