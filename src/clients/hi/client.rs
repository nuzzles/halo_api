use std::{sync::Arc, time::Duration};

use reqwest::{Client, StatusCode};
use serde::de::DeserializeOwned;
use serde_json::Value;
use xbox::models::Xuid;
use xbox::util::wrap_xuid;

use super::InfiniteClientError;
use super::constants::PlaylistId;
use super::endpoints::HaloEndpoints;
use super::models::{CsrRecords, PlayerMatchHistory, ServiceRecord};
use crate::auth::{HaloAuth, HaloCredentials};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);

/// Halo Infinite API client. Authentication is supplied by a separate [`HaloAuth`] client.
pub struct HaloInfiniteClient {
    auth: Arc<dyn HaloAuth>,
    http: Client,
    endpoints: HaloEndpoints,
}

impl HaloInfiniteClient {
    pub fn new<A: HaloAuth + 'static>(auth: impl Into<Arc<A>>) -> Self {
        Self::with_endpoints(auth, HaloEndpoints::default())
    }

    pub(crate) fn with_endpoints<A: HaloAuth + 'static>(
        auth: impl Into<Arc<A>>,
        endpoints: HaloEndpoints,
    ) -> Self {
        let auth: Arc<A> = auth.into();
        Self {
            auth,
            http: Client::new(),
            endpoints,
        }
    }

    async fn get<T: DeserializeOwned>(
        &self,
        base: &str,
        path: &str,
        query: &[(&str, String)],
    ) -> Result<T, InfiniteClientError> {
        self.get_authenticated(base, path, query, false).await
    }

    async fn get_with_clearance<T: DeserializeOwned>(
        &self,
        base: &str,
        path: &str,
        query: &[(&str, String)],
    ) -> Result<T, InfiniteClientError> {
        self.get_authenticated(base, path, query, true).await
    }

    async fn get_authenticated<T: DeserializeOwned>(
        &self,
        base: &str,
        path: &str,
        query: &[(&str, String)],
        require_clearance: bool,
    ) -> Result<T, InfiniteClientError> {
        let url = format!("{base}{path}");
        let first = self.auth.credentials(require_clearance).await?;
        match self.get_once(&url, query, &first).await {
            Err(error) if error.is_unauthorized() => {
                self.auth.invalidate().await;
                let second = self.auth.credentials(require_clearance).await?;
                self.get_once(&url, query, &second).await
            }
            result => result,
        }
    }

    async fn get_once<T: DeserializeOwned>(
        &self,
        url: &str,
        query: &[(&str, String)],
        credentials: &HaloCredentials,
    ) -> Result<T, InfiniteClientError> {
        let mut request = self
            .http
            .get(url)
            .query(query)
            .header("X-343-Authorization-Spartan", &credentials.spartan_token)
            .header("Accept", "application/json")
            .timeout(DEFAULT_TIMEOUT);
        if let Some(clearance) = &credentials.clearance {
            request = request.header("343-Clearance", clearance);
        }
        let response = request.send().await?;
        if !response.status().is_success() {
            return Err(InfiniteClientError::HttpStatus {
                url: response.url().to_string(),
                status: response.status(),
            });
        }
        Ok(response.json().await?)
    }

    pub async fn playlist_csr(
        &self,
        playlist: PlaylistId,
        xuid: &Xuid,
    ) -> Result<CsrRecords, InfiniteClientError> {
        self.playlist_csr_by_id(playlist.as_str(), std::slice::from_ref(xuid))
            .await
    }

    pub async fn playlist_csr_batch(
        &self,
        playlist: PlaylistId,
        xuids: &[Xuid],
    ) -> Result<CsrRecords, InfiniteClientError> {
        self.playlist_csr_by_id(playlist.as_str(), xuids).await
    }

    /// Gets CSR using any playlist asset ID, including playlists unknown to [`PlaylistId`].
    pub async fn playlist_csr_by_id(
        &self,
        playlist_id: &str,
        xuids: &[Xuid],
    ) -> Result<CsrRecords, InfiniteClientError> {
        let players = xuids
            .iter()
            .map(|x| wrap_xuid(x.as_str()))
            .collect::<Vec<_>>()
            .join(",");
        self.get(
            &self.endpoints.skill_base_url,
            &format!("/hi/playlist/{playlist_id}/csrs"),
            &[("players", players)],
        )
        .await
    }

    pub async fn service_record(&self, player: &str) -> Result<ServiceRecord, InfiniteClientError> {
        let result = self
            .get(
                &self.endpoints.halostats_base_url,
                &format!("/hi/players/{player}/Matchmade/servicerecord"),
                &[],
            )
            .await;
        match result {
            Err(InfiniteClientError::HttpStatus {
                status: StatusCode::BAD_REQUEST | StatusCode::NOT_FOUND,
                ..
            }) => Err(InfiniteClientError::GamertagNotFound(player.to_string())),
            other => other,
        }
    }

    pub async fn player_matches(
        &self,
        xuid: &Xuid,
        start: u32,
        count: u32,
    ) -> Result<PlayerMatchHistory, InfiniteClientError> {
        self.get(
            &self.endpoints.halostats_base_url,
            &format!("/hi/players/{}/matches", wrap_xuid(xuid.as_str())),
            &[("start", start.to_string()), ("count", count.to_string())],
        )
        .await
    }

    pub async fn match_stats(&self, match_id: &str) -> Result<Value, InfiniteClientError> {
        self.get(
            &self.endpoints.halostats_base_url,
            &format!("/hi/matches/{match_id}/stats"),
            &[],
        )
        .await
    }

    pub async fn match_skill(
        &self,
        match_id: &str,
        xuids: &[Xuid],
    ) -> Result<Value, InfiniteClientError> {
        let players = xuids
            .iter()
            .map(|x| wrap_xuid(x.as_str()))
            .collect::<Vec<_>>()
            .join(",");
        self.get(
            &self.endpoints.skill_base_url,
            &format!("/hi/matches/{match_id}/skill"),
            &[("players", players)],
        )
        .await
    }

    /// Looks up profile information by gamertag.
    pub async fn user(&self, gamertag: &str) -> Result<Value, InfiniteClientError> {
        self.get(
            &self.endpoints.profile_base_url,
            &format!("/users/gt({gamertag})"),
            &[],
        )
        .await
    }

    /// Looks up profile information for multiple XUIDs.
    pub async fn users(&self, xuids: &[Xuid]) -> Result<Value, InfiniteClientError> {
        self.get(
            &self.endpoints.profile_base_url,
            "/users",
            &[(
                "xuids",
                xuids
                    .iter()
                    .map(|xuid| xuid.as_str())
                    .collect::<Vec<_>>()
                    .join(","),
            )],
        )
        .await
    }

    pub async fn map(
        &self,
        asset_id: &str,
        version_id: &str,
    ) -> Result<Value, InfiniteClientError> {
        self.ugc_version("Maps", asset_id, version_id).await
    }
    pub async fn mode(
        &self,
        asset_id: &str,
        version_id: &str,
    ) -> Result<Value, InfiniteClientError> {
        self.ugc_version("UgcGameVariants", asset_id, version_id)
            .await
    }
    pub async fn playlist(
        &self,
        asset_id: &str,
        version_id: &str,
    ) -> Result<Value, InfiniteClientError> {
        self.ugc_version("Playlists", asset_id, version_id).await
    }
    pub async fn map_mode_pair(
        &self,
        asset_id: &str,
        version_id: &str,
    ) -> Result<Value, InfiniteClientError> {
        self.ugc_version("MapModePairs", asset_id, version_id).await
    }

    pub async fn asset(&self, kind: &str, asset_id: &str) -> Result<Value, InfiniteClientError> {
        self.get(
            &self.endpoints.ugc_base_url,
            &format!("/hi/{kind}/{asset_id}"),
            &[],
        )
        .await
    }

    async fn ugc_version(
        &self,
        kind: &str,
        asset_id: &str,
        version_id: &str,
    ) -> Result<Value, InfiniteClientError> {
        self.get(
            &self.endpoints.ugc_base_url,
            &format!("/hi/{kind}/{asset_id}/versions/{version_id}"),
            &[],
        )
        .await
    }

    pub async fn playlist_metadata(&self, playlist_id: &str) -> Result<Value, InfiniteClientError> {
        self.get_with_clearance(
            &self.endpoints.game_cms_base_url,
            &format!("/hi/multiplayer/file/playlists/assets/{playlist_id}.json"),
            &[],
        )
        .await
    }

    pub async fn season_calendar(&self) -> Result<Value, InfiniteClientError> {
        self.get(
            &self.endpoints.game_cms_base_url,
            "/hi/progression/file/calendars/seasons/seasoncalendar.json",
            &[],
        )
        .await
    }
    pub async fn medals(&self) -> Result<Value, InfiniteClientError> {
        self.get(
            &self.endpoints.game_cms_base_url,
            "/hi/Waypoint/file/medals/metadata.json",
            &[],
        )
        .await
    }
    pub async fn ban_summary(&self, xuid: &Xuid) -> Result<Value, InfiniteClientError> {
        self.get(
            &self.endpoints.ban_base_url,
            "/hi/bansummary",
            &[
                ("auth", "st".to_string()),
                ("targets", wrap_xuid(xuid.as_str())),
            ],
        )
        .await
    }
    pub async fn matches_privacy(&self, xuid: &Xuid) -> Result<Value, InfiniteClientError> {
        self.get(
            &self.endpoints.halostats_base_url,
            &format!("/hi/players/{}/matches-privacy", wrap_xuid(xuid.as_str())),
            &[],
        )
        .await
    }

    pub async fn current_user(&self) -> Result<Value, InfiniteClientError> {
        self.get(&self.endpoints.current_user_url, "", &[]).await
    }

    pub async fn player_match_count(&self, xuid: &Xuid) -> Result<Value, InfiniteClientError> {
        self.get(
            &self.endpoints.halostats_base_url,
            &format!("/hi/players/{}/matches/count", wrap_xuid(xuid.as_str())),
            &[],
        )
        .await
    }

    pub async fn csr_season_calendar(&self) -> Result<Value, InfiniteClientError> {
        self.get(
            &self.endpoints.game_cms_base_url,
            "/hi/Progression/file/Csr/Calendars/CsrSeasonCalendar.json",
            &[],
        )
        .await
    }

    pub async fn settings(&self) -> Result<Value, InfiniteClientError> {
        self.get(
            &self.endpoints.settings_base_url,
            "/settings/hipc/e2a0a7c6-6efe-42af-9283-c2ab73250c48",
            &[],
        )
        .await
    }
}
