use std::sync::Arc;

use xbox::XboxClient;
use xbox::auth::{RelyingParty, XblAuthProvider};

use super::{AuthError, HaloAuthClient, XstsTicketSource};

/// Xbox relying party used by Halo Waypoint's Spartan-token exchange.
pub const HALO_RELYING_PARTY: RelyingParty =
    RelyingParty::new("https://prod.xsts.halowaypoint.com/");

/// Adapts an Xbox 0.3 client into a Halo Waypoint XSTS ticket source.
///
/// The Xbox client and its authentication provider remain caller-owned. The
/// source requests a fresh or cached ticket whenever Halo credentials refresh.
pub struct XboxXstsTicketSource<P: XblAuthProvider> {
    xbox: Arc<XboxClient<P>>,
}

impl<P: XblAuthProvider> XboxXstsTicketSource<P> {
    pub fn new(xbox: impl Into<Arc<XboxClient<P>>>) -> Self {
        Self { xbox: xbox.into() }
    }
}

impl HaloAuthClient {
    /// Exchanges XSTS tickets from an Xbox client for Halo Waypoint credentials.
    /// The Xbox client and its authentication provider remain caller-owned.
    pub fn from_xbox_xsts_source<P: XblAuthProvider + 'static>(
        xbox: impl Into<Arc<XboxClient<P>>>,
    ) -> Self {
        Self::from_xsts_source(Arc::new(XboxXstsTicketSource::new(xbox)))
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl<P: XblAuthProvider + 'static> XstsTicketSource for XboxXstsTicketSource<P> {
    async fn xsts_ticket(&self) -> Result<String, AuthError> {
        self.xbox
            .xsts_ticket(HALO_RELYING_PARTY)
            .await
            .map(|ticket| ticket.token)
            .map_err(|error| AuthError::SpartanTokenProvider(error.to_string()))
    }
}
