use std::future::Future;
use std::sync::Arc;

use reqwest::Client;
use xbox::XboxClient;
use xbox::auth::XblAuthProvider;
use xbox::cache::ExpiryTokenCache;
use xbox::models::Xuid;
use xbox::util::wrap_xuid;

use crate::auth::{SpartanTokenSource, XboxSpartanTokenProvider};
use crate::constants::PlaylistId;
use crate::endpoints::{HaloEndpoints, csr, match_history, service_record};
use crate::error::HaloApiError;
use crate::models::{CsrRecords, PlayerMatchHistory, ServiceRecord};

/// The top-level entry point for talking to the Halo Infinite API.
///
/// Wraps a [`SpartanTokenSource`] with expiry-aware, single-flight caching for the spartan
/// token, plus automatic invalidate-and-retry-once behavior on HTTP 401 responses.
pub struct HaloClient {
    spartan_source: Arc<dyn SpartanTokenSource>,
    spartan_cache: ExpiryTokenCache<String, HaloApiError>,
    http: Client,
    endpoints: HaloEndpoints,
}

impl HaloClient {
    /// Constructs a client from any [`SpartanTokenSource`], e.g. one obtained elsewhere than
    /// an `xbox` client.
    pub fn new(spartan_source: Arc<dyn SpartanTokenSource>) -> Self {
        Self::with_endpoints(spartan_source, HaloEndpoints::default())
    }

    /// Constructs a client with overridden endpoint URLs, e.g. to point at a mock server in
    /// tests. Most callers should use [`HaloClient::new`] or
    /// [`HaloClient::from_xbox_client`] instead.
    pub fn with_endpoints(
        spartan_source: Arc<dyn SpartanTokenSource>,
        endpoints: HaloEndpoints,
    ) -> Self {
        Self {
            spartan_source,
            spartan_cache: ExpiryTokenCache::new(),
            http: Client::new(),
            endpoints,
        }
    }

    /// Constructs a client backed by the default [`XboxSpartanTokenProvider`], wrapping the
    /// given `xbox::XboxClient`. Callers that also need gamertag/XUID resolution should keep
    /// their own clone of the `Arc<XboxClient<P>>` passed in here.
    pub fn from_xbox_client<P: XblAuthProvider + 'static>(xbox: Arc<XboxClient<P>>) -> Self {
        Self::new(Arc::new(XboxSpartanTokenProvider::new(xbox)))
    }

    /// Like [`HaloClient::from_xbox_client`], but with overridden endpoint URLs, e.g. to point
    /// both the spartan-token exchange and the Halo Waypoint endpoints at a mock server in
    /// tests.
    pub fn from_xbox_client_with_endpoints<P: XblAuthProvider + 'static>(
        xbox: Arc<XboxClient<P>>,
        endpoints: HaloEndpoints,
    ) -> Self {
        let provider = Arc::new(XboxSpartanTokenProvider::with_endpoints(xbox, &endpoints));
        Self::with_endpoints(provider, endpoints)
    }

    async fn spartan_token(&self) -> Result<String, HaloApiError> {
        self.spartan_cache
            .get_or_refresh(|| self.spartan_source.spartan_token())
            .await
    }

    /// Runs `call` with a fresh spartan token, and on an HTTP 401 response invalidates the
    /// cached spartan token before retrying exactly once.
    async fn with_single_retry<T, F, Fut>(&self, call: F) -> Result<T, HaloApiError>
    where
        F: Fn(String) -> Fut,
        Fut: Future<Output = Result<T, HaloApiError>>,
    {
        let token = self.spartan_token().await?;
        match call(token).await {
            Err(err) if err.is_unauthorized() => {
                self.spartan_cache.invalidate().await;
                let token = self.spartan_token().await?;
                call(token).await
            }
            other => other,
        }
    }

    /// Gets Competitive Skill Rank (CSR) for a single player in a given playlist.
    pub async fn playlist_csr(
        &self,
        playlist: PlaylistId,
        xuid: &Xuid,
    ) -> Result<CsrRecords, HaloApiError> {
        let player_id = wrap_xuid(xuid.as_str());
        self.with_single_retry(move |token| {
            let http = self.http.clone();
            let skill_base_url = self.endpoints.skill_base_url.clone();
            let player_id = player_id.clone();
            async move {
                csr::get_playlist_csr(
                    &http,
                    &skill_base_url,
                    &token,
                    playlist.as_str(),
                    &[player_id],
                )
                .await
            }
        })
        .await
    }

    /// Gets CSR for several players at once in a given playlist.
    pub async fn playlist_csr_batch(
        &self,
        playlist: PlaylistId,
        xuids: &[Xuid],
    ) -> Result<CsrRecords, HaloApiError> {
        let player_ids: Vec<String> = xuids.iter().map(|x| wrap_xuid(x.as_str())).collect();
        self.with_single_retry(move |token| {
            let http = self.http.clone();
            let skill_base_url = self.endpoints.skill_base_url.clone();
            let player_ids = player_ids.clone();
            async move {
                csr::get_playlist_csr(
                    &http,
                    &skill_base_url,
                    &token,
                    playlist.as_str(),
                    &player_ids,
                )
                .await
            }
        })
        .await
    }

    /// Gets a player's matchmade service record. `gamertag_or_xuid` may be a plain gamertag or
    /// a raw/wrapped XUID.
    pub async fn service_record(
        &self,
        gamertag_or_xuid: &str,
    ) -> Result<ServiceRecord, HaloApiError> {
        let gamertag_or_xuid = gamertag_or_xuid.to_string();
        self.with_single_retry(move |token| {
            let http = self.http.clone();
            let halostats_base_url = self.endpoints.halostats_base_url.clone();
            let gamertag_or_xuid = gamertag_or_xuid.clone();
            async move {
                service_record::get_service_record(
                    &http,
                    &halostats_base_url,
                    &token,
                    &gamertag_or_xuid,
                )
                .await
            }
        })
        .await
    }

    /// Gets a page of a player's match history.
    pub async fn player_matches(
        &self,
        xuid: &Xuid,
        start: u32,
        count: u32,
    ) -> Result<PlayerMatchHistory, HaloApiError> {
        let player_id = wrap_xuid(xuid.as_str());
        self.with_single_retry(move |token| {
            let http = self.http.clone();
            let halostats_base_url = self.endpoints.halostats_base_url.clone();
            let player_id = player_id.clone();
            async move {
                match_history::get_player_matches(
                    &http,
                    &halostats_base_url,
                    &token,
                    &player_id,
                    start,
                    count,
                )
                .await
            }
        })
        .await
    }
}
