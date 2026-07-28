use std::sync::Arc;

use async_trait::async_trait;
use xbox::cache::ExpiryTokenCache;
use xbox::{XboxClient, auth::XblAuthProvider};

use super::{
    ClearanceTokenSource, SpartanTokenSource, WaypointClearanceProvider, XboxSpartanTokenProvider,
};
use crate::auth::AuthError;

use super::endpoints::AuthEndpoints;

/// Credentials attached to every authenticated Halo Infinite request.
#[derive(Clone, Debug)]
pub struct HaloCredentials {
    pub spartan_token: String,
    pub clearance: Option<String>,
}

/// Authentication contract consumed by [`crate::HaloInfiniteClient`].
#[async_trait]
pub trait HaloAuth: Send + Sync {
    async fn credentials(&self, require_clearance: bool) -> Result<HaloCredentials, AuthError>;
    async fn invalidate(&self);
}

/// Owns Spartan-token and clearance acquisition, caching, and invalidation.
pub struct AuthClient {
    spartan_source: Arc<dyn SpartanTokenSource>,
    clearance_source: Arc<dyn ClearanceTokenSource>,
    spartan_cache: ExpiryTokenCache<String, AuthError>,
    clearance_cache: ExpiryTokenCache<String, AuthError>,
}

impl AuthClient {
    pub fn new(
        spartan_source: Arc<dyn SpartanTokenSource>,
        clearance_source: Arc<dyn ClearanceTokenSource>,
    ) -> Self {
        Self {
            spartan_source,
            clearance_source,
            spartan_cache: ExpiryTokenCache::new(),
            clearance_cache: ExpiryTokenCache::new(),
        }
    }

    /// Builds the standard Waypoint auth stack from an owned or shared Xbox Live client.
    ///
    /// Both `XboxClient<P>` and `Arc<XboxClient<P>>` are accepted. Pass an `Arc` only when the
    /// caller also needs to retain the Xbox client, such as for gamertag-to-XUID resolution.
    pub fn from_xbox_client<P: XblAuthProvider + 'static>(
        xbox: impl Into<Arc<XboxClient<P>>>,
    ) -> Self {
        Self::from_xbox_client_with_endpoints(xbox, &AuthEndpoints::default())
    }

    /// Builds the auth stack with overridable URLs, primarily for proxies and tests.
    pub(crate) fn from_xbox_client_with_endpoints<P: XblAuthProvider + 'static>(
        xbox: impl Into<Arc<XboxClient<P>>>,
        endpoints: &AuthEndpoints,
    ) -> Self {
        Self::new(
            Arc::new(XboxSpartanTokenProvider::with_endpoints(
                xbox.into(),
                endpoints,
            )),
            Arc::new(WaypointClearanceProvider::new(
                &endpoints.current_user_url,
                &endpoints.clearance_url,
            )),
        )
    }
}

#[async_trait]
impl HaloAuth for AuthClient {
    async fn credentials(&self, require_clearance: bool) -> Result<HaloCredentials, AuthError> {
        let spartan_token = self
            .spartan_cache
            .get_or_refresh(|| self.spartan_source.spartan_token())
            .await?;
        let clearance = if require_clearance {
            Some(
                self.clearance_cache
                    .get_or_refresh(|| self.clearance_source.clearance_token(&spartan_token))
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

    async fn invalidate(&self) {
        self.clearance_cache.invalidate().await;
        self.spartan_cache.invalidate().await;
    }
}
