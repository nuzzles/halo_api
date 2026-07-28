use std::sync::Arc;

use thiserror::Error;

use crate::auth::AuthError;

/// Errors produced by the Halo Infinite API client.
#[derive(Debug, Clone, Error)]
pub enum InfiniteClientError {
    #[error("authentication failed: {0}")]
    Auth(#[from] AuthError),

    #[error("Halo Infinite request to {url} failed with status {status}: {body}")]
    HttpStatus {
        url: String,
        status: reqwest::StatusCode,
        body: String,
    },

    #[error("Halo Infinite network error: {0}")]
    Network(Arc<reqwest::Error>),

    #[error(
        "Halo Infinite response from {url} did not match the expected schema: {source}; body: {body}"
    )]
    Decode {
        url: String,
        source: Arc<serde_json::Error>,
        body: String,
    },

    #[error("failed to decompress Halo Infinite Theater film data: {0}")]
    FilmDecompression(Arc<std::io::Error>),

    #[error("no Halo Infinite record found for gamertag \"{0}\"")]
    GamertagNotFound(String),

    #[error("Halo Infinite returned no customization for player \"{0}\"")]
    CustomizationNotFound(String),
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
