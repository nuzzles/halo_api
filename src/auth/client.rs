use std::sync::Arc;

use super::cache::{CachedToken, ExpiryTokenCache};
use chrono::{Duration as ChronoDuration, Utc};

use super::{
    ClearanceTokenSource, SpartanTokenSource, WaypointClearanceProvider,
    WaypointSpartanTokenProvider, XstsTicketSource,
};
use crate::auth::AuthError;

use super::endpoints::AuthEndpoints;

/// Owns Halo Waypoint authentication, including Spartan-token and clearance
/// acquisition, caching, and invalidation.
///
/// Clones share the same credentials and caches, so it is inexpensive to retain
/// one alongside a [`crate::HaloInfiniteClient`].
#[derive(Clone)]
pub struct HaloAuthClient {
    state: Arc<HaloAuthState>,
}

struct HaloAuthState {
    spartan_source: Arc<dyn SpartanTokenSource>,
    clearance_source: Arc<dyn ClearanceTokenSource>,
    spartan_cache: ExpiryTokenCache<String>,
    clearance_cache: ExpiryTokenCache<String>,
}

/// Non-refreshable credentials supplied directly by an application.
///
/// The enclosing caches still make token invalidation safe: a 401 clears the
/// cached value and retries with the supplied credentials once. Applications
/// using this source must create a new [`HaloAuthClient`] when their tokens
/// expire or are rotated.
struct StaticSpartanTokenSource {
    token: String,
}

struct StaticClearanceTokenSource {
    token: String,
}

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl SpartanTokenSource for StaticSpartanTokenSource {
    async fn spartan_token(&self) -> Result<CachedToken<String>, AuthError> {
        Ok(static_token(&self.token))
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl ClearanceTokenSource for StaticClearanceTokenSource {
    async fn clearance_token(
        &self,
        _spartan_token: &str,
    ) -> Result<CachedToken<String>, AuthError> {
        Ok(static_token(&self.token))
    }
}

fn static_token(token: &str) -> CachedToken<String> {
    // Raw credentials do not expose their expiry in this API. Keep them cached
    // until an authorization failure or the caller replaces the auth client.
    CachedToken::new(token.to_owned(), Utc::now() + ChronoDuration::days(3650))
}

/// Credentials attached internally to authenticated Halo Infinite requests.
#[derive(Clone, Debug)]
pub(crate) struct HaloCredentials {
    pub(crate) spartan_token: String,
    pub(crate) clearance: Option<String>,
}

impl HaloAuthClient {
    /// Exchanges caller-supplied Halo Waypoint XSTS tickets for Spartan tokens
    /// and clearance. The source must renew its Xbox credentials as needed.
    pub fn from_xsts_source(source: impl XstsTicketSource + 'static) -> Self {
        Self::from_shared_xsts_source(Arc::new(source))
    }

    /// Uses an already shared XSTS ticket source for Halo Waypoint authentication.
    pub fn from_shared_xsts_source(source: Arc<dyn XstsTicketSource>) -> Self {
        Self::from_xsts_source_with_endpoints(source, &AuthEndpoints::default())
    }

    /// Uses existing Halo Waypoint credentials without performing an Xbox
    /// sign-in or token exchange.
    ///
    /// This is intended for applications that already manage token acquisition.
    /// The supplied values are kept private and are never exposed by this
    /// client. They cannot be refreshed automatically; construct a new client
    /// with replacement values after they expire.
    pub fn from_tokens(
        spartan_token: impl Into<String>,
        clearance_token: impl Into<String>,
    ) -> Self {
        Self::with_sources(
            Arc::new(StaticSpartanTokenSource {
                token: spartan_token.into(),
            }),
            Arc::new(StaticClearanceTokenSource {
                token: clearance_token.into(),
            }),
        )
    }

    /// Builds the authentication flow with overridable URLs for crate tests.
    pub(crate) fn from_xsts_source_with_endpoints(
        source: Arc<dyn XstsTicketSource>,
        endpoints: &AuthEndpoints,
    ) -> Self {
        Self::with_sources(
            Arc::new(WaypointSpartanTokenProvider::with_endpoints(
                source, endpoints,
            )),
            Arc::new(WaypointClearanceProvider::new(
                &endpoints.current_user_url,
                &endpoints.clearance_url,
            )),
        )
    }

    pub(crate) fn with_sources(
        spartan_source: Arc<dyn SpartanTokenSource>,
        clearance_source: Arc<dyn ClearanceTokenSource>,
    ) -> Self {
        Self {
            state: Arc::new(HaloAuthState {
                spartan_source,
                clearance_source,
                spartan_cache: ExpiryTokenCache::new(),
                clearance_cache: ExpiryTokenCache::new(),
            }),
        }
    }

    pub(crate) async fn credentials(
        &self,
        require_clearance: bool,
    ) -> Result<HaloCredentials, AuthError> {
        let spartan_token = self
            .state
            .spartan_cache
            .get_or_refresh(|| self.state.spartan_source.spartan_token())
            .await?;
        let clearance = if require_clearance {
            Some(
                self.state
                    .clearance_cache
                    .get_or_refresh(|| self.state.clearance_source.clearance_token(&spartan_token))
                    .await?,
            )
        } else {
            None
        };
        Ok(HaloCredentials {
            spartan_token,
            clearance,
        })
    }

    pub(crate) async fn invalidate(&self) {
        self.state.clearance_cache.invalidate().await;
        self.state.spartan_cache.invalidate().await;
    }
}
