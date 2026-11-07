use std::sync::Arc;

use thiserror::Error;

use crate::auth::AuthError;

/// Theater-specific failures, distinct from authentication and HTTP transport errors.
#[derive(Debug, Clone, Error)]
pub enum TheaterError {
    /// The match-film endpoint returned 404. LAN/local matches have no downloadable film;
    /// missing or expired films can produce the same response for other match types.
    #[error("no Theater film available for match {match_id}")]
    FilmUnavailable { match_id: String },

    /// Downloaded film data could not be decompressed as zlib.
    #[error("failed to decompress Halo Infinite Theater film data: {0}")]
    Decompression(#[source] Arc<std::io::Error>),
}

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

    #[error(transparent)]
    Theater(#[from] TheaterError),

    #[error("no Halo Infinite record found for gamertag \"{0}\"")]
    GamertagNotFound(String),

    #[error("Halo Infinite returned no career rank record for player \"{0}\"")]
    CareerRankNotFound(String),

    #[error("an internal concurrent task was cancelled or panicked")]
    TaskJoin,
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
