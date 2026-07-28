use std::sync::Arc;

use thiserror::Error;

use crate::auth::AuthError;

/// Errors produced by the Halo Infinite API client.
#[derive(Debug, Clone, Error)]
pub enum InfiniteClientError {
    #[error("authentication failed: {0}")]
    Auth(#[from] AuthError),

    #[error("Halo Infinite request to {url} failed with status {status}")]
    HttpStatus {
        url: String,
        status: reqwest::StatusCode,
    },

    #[error("Halo Infinite network error: {0}")]
    Network(Arc<reqwest::Error>),

    #[error("no Halo Infinite record found for gamertag \"{0}\"")]
    GamertagNotFound(String),
}

impl From<reqwest::Error> for InfiniteClientError {
    fn from(error: reqwest::Error) -> Self {
        Self::Network(Arc::new(error))
    }
}

impl InfiniteClientError {
    pub(crate) fn is_unauthorized(&self) -> bool {
        matches!(
            self,
            Self::HttpStatus { status, .. } if *status == reqwest::StatusCode::UNAUTHORIZED
        )
    }
}
