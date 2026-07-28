use std::sync::Arc;

use thiserror::Error;

/// Errors produced while acquiring or refreshing Halo credentials.
#[derive(Debug, Clone, Error)]
pub enum AuthError {
    #[error("authentication request to {url} failed with status {status}")]
    HttpStatus {
        url: String,
        status: reqwest::StatusCode,
    },

    #[error("authentication network error: {0}")]
    Network(Arc<reqwest::Error>),

    #[error("spartan token provider error: {0}")]
    SpartanTokenProvider(String),
}

impl From<reqwest::Error> for AuthError {
    fn from(error: reqwest::Error) -> Self {
        Self::Network(Arc::new(error))
    }
}
