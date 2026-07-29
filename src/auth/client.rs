use std::sync::Arc;

use xbox::cache::ExpiryTokenCache;
use xbox::{XboxClient, auth::XblAuthProvider};

use super::{
    ClearanceTokenSource, SpartanTokenSource, WaypointClearanceProvider, XboxSpartanTokenProvider,
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
    spartan_cache: ExpiryTokenCache<String, AuthError>,
    clearance_cache: ExpiryTokenCache<String, AuthError>,
}

/// Credentials attached internally to authenticated Halo Infinite requests.
#[derive(Clone, Debug)]
pub(crate) struct HaloCredentials {
    pub(crate) spartan_token: String,
    pub(crate) clearance: Option<String>,
}

impl HaloAuthClient {
    /// Builds the standard Waypoint authentication flow from an owned or shared
    /// Xbox Live client.
    ///
    /// Both `XboxClient<P>` and `Arc<XboxClient<P>>` are accepted. Pass an `Arc`
    /// when the caller also needs the Xbox client, such as for gamertag-to-XUID
    /// resolution.
    pub fn from_xbox_client<P: XblAuthProvider + 'static>(
        xbox: impl Into<Arc<XboxClient<P>>>,
    ) -> Self {
        Self::from_xbox_client_with_endpoints(xbox, &AuthEndpoints::default())
    }

    /// Builds the authentication flow with overridable URLs for crate tests.
    pub(crate) fn from_xbox_client_with_endpoints<P: XblAuthProvider + 'static>(
        xbox: impl Into<Arc<XboxClient<P>>>,
        endpoints: &AuthEndpoints,
    ) -> Self {
        Self::with_sources(
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
