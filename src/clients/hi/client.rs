use std::{sync::Arc, time::Duration};

use reqwest::{Client, StatusCode};
use serde::de::DeserializeOwned;
use serde_json::Value;
use xbox::models::Xuid;
use xbox::util::wrap_xuid;

use super::InfiniteClientError;
use super::endpoints::HaloEndpoints;
use super::models::{
    AppearanceCustomization, BanMessage, BanSummary, CsrRecords, CsrSeason, CsrSeasonCalendar,
    GameModeId, GameVariantAsset, MapAsset, MapId, MapModePairAsset, MatchStats, MatchesPrivacy,
    PlayerCustomizationCollection, PlayerMatchHistory, PlaylistAsset, PlaylistId, PlaylistMetadata,
    RankedArenaMapMode, RankedArenaSeason, SeasonCalendar, ServiceRecord, UserInfo,
};
use crate::auth::{HaloAuth, HaloCredentials};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);
const HALO_PC_USER_AGENT: &str = "SHIVA-2043073184/6.10021.18539.0 (release; PC)";
const HALO_WAYPOINT_USER_AGENT: &str =
    "HaloWaypoint/2021112313511900 CFNetwork/1327.0.4 Darwin/21.2.0";

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

    async fn get_with_clearance_query<T: DeserializeOwned>(
        &self,
        base: &str,
        path: &str,
        query: &[(&str, String)],
    ) -> Result<T, InfiniteClientError> {
        self.get_with_clearance_named_query(base, path, query, "clearanceId")
            .await
    }

    async fn get_with_clearance_named_query<T: DeserializeOwned>(
        &self,
        base: &str,
        path: &str,
        query: &[(&str, String)],
        clearance_query_name: &'static str,
    ) -> Result<T, InfiniteClientError> {
        let credentials = self.auth.credentials(true).await?;
        let mut query = query.to_vec();
        if let Some(clearance) = &credentials.clearance {
            query.push((clearance_query_name, clearance.clone()));
        }
        let url = format!("{base}{path}");
        match self
            .get_once(&url, &query, &credentials, HALO_PC_USER_AGENT)
            .await
        {
            Err(error) if error.is_unauthorized() => {
                self.auth.invalidate().await;
                let credentials = self.auth.credentials(true).await?;
                let mut query = query
                    .into_iter()
                    .filter(|(name, _)| *name != clearance_query_name)
                    .collect::<Vec<_>>();
                if let Some(clearance) = &credentials.clearance {
                    query.push((clearance_query_name, clearance.clone()));
                }
                self.get_once(&url, &query, &credentials, HALO_PC_USER_AGENT)
                    .await
            }
            result => result,
        }
    }

    async fn get_authenticated<T: DeserializeOwned>(
        &self,
        base: &str,
        path: &str,
        query: &[(&str, String)],
        require_clearance: bool,
    ) -> Result<T, InfiniteClientError> {
        self.get_authenticated_with_user_agent(
            base,
            path,
            query,
            require_clearance,
            HALO_PC_USER_AGENT,
        )
        .await
    }

    async fn get_authenticated_with_user_agent<T: DeserializeOwned>(
        &self,
        base: &str,
        path: &str,
        query: &[(&str, String)],
        require_clearance: bool,
        user_agent: &'static str,
    ) -> Result<T, InfiniteClientError> {
        let url = format!("{base}{path}");
        let first = self.auth.credentials(require_clearance).await?;
        match self.get_once(&url, query, &first, user_agent).await {
            Err(error) if error.is_unauthorized() => {
                self.auth.invalidate().await;
                let second = self.auth.credentials(require_clearance).await?;
                self.get_once(&url, query, &second, user_agent).await
            }
            result => result,
        }
    }

    async fn get_once<T: DeserializeOwned>(
        &self,
        url: &str,
        query: &[(&str, String)],
        credentials: &HaloCredentials,
        user_agent: &'static str,
    ) -> Result<T, InfiniteClientError> {
        let mut request = self
            .http
            .get(url)
            .query(query)
            .header("X-343-Authorization-Spartan", &credentials.spartan_token)
            .header("Accept", "application/json")
            .header("User-Agent", user_agent)
            .timeout(DEFAULT_TIMEOUT);
        if let Some(clearance) = &credentials.clearance {
            request = request.header("343-Clearance", clearance);
        }
        let response = request.send().await?;
        let url = response.url().to_string();
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(InfiniteClientError::HttpStatus { url, status, body });
        }
        let body = response.text().await?;
        serde_json::from_str(&body).map_err(|source| InfiniteClientError::Decode {
            url,
            source: Arc::new(source),
            body,
        })
    }

    pub async fn playlist_csr(
        &self,
        playlist: PlaylistId,
        xuid: &Xuid,
    ) -> Result<CsrRecords, InfiniteClientError> {
        self.playlist_csr_batch(playlist, std::slice::from_ref(xuid))
            .await
    }

    pub async fn playlist_csr_batch(
        &self,
        playlist: PlaylistId,
        xuids: &[Xuid],
    ) -> Result<CsrRecords, InfiniteClientError> {
        let players = xuids
            .iter()
            .map(|x| wrap_xuid(x.as_str()))
            .collect::<Vec<_>>()
            .join(",");
        self.get_with_clearance(
            &self.endpoints.skill_base_url,
            &format!("/hi/playlist/{playlist}/csrs"),
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

    pub async fn match_stats(&self, match_id: &str) -> Result<MatchStats, InfiniteClientError> {
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
    pub async fn user(&self, gamertag: &str) -> Result<UserInfo, InfiniteClientError> {
        self.get(
            &self.endpoints.profile_base_url,
            &format!("/users/gt({gamertag})"),
            &[],
        )
        .await
    }

    /// Looks up profile information for multiple XUIDs.
    pub async fn users(&self, xuids: &[Xuid]) -> Result<Vec<UserInfo>, InfiniteClientError> {
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

    /// Gets a player's equipped service tag, emblem, backdrop, pose, and intro emote.
    pub async fn appearance(
        &self,
        xuid: &Xuid,
    ) -> Result<AppearanceCustomization, InfiniteClientError> {
        self.get_with_clearance(
            &self.endpoints.economy_base_url,
            &format!(
                "/hi/players/{}/customization/appearance",
                wrap_xuid(xuid.as_str())
            ),
            &[],
        )
        .await
    }

    /// Gets public customization data for multiple players.
    pub async fn player_customizations(
        &self,
        xuids: &[Xuid],
    ) -> Result<PlayerCustomizationCollection, InfiniteClientError> {
        let players = xuids
            .iter()
            .map(|xuid| wrap_xuid(xuid.as_str()))
            .collect::<Vec<_>>()
            .join(",");
        self.get_authenticated_with_user_agent(
            &self.endpoints.economy_base_url,
            "/hi/customization",
            &[("players", players)],
            true,
            HALO_WAYPOINT_USER_AGENT,
        )
        .await
    }

    /// Resolves a gamertag and gets that player's equipped appearance.
    pub async fn appearance_by_gamertag(
        &self,
        gamertag: &str,
    ) -> Result<AppearanceCustomization, InfiniteClientError> {
        let user = self.user(gamertag).await?;
        let customization = self
            .player_customizations(&[Xuid::from(user.xuid)])
            .await?
            .player_customizations
            .into_iter()
            .next()
            .ok_or_else(|| InfiniteClientError::CustomizationNotFound(gamertag.to_string()))?;
        Ok(AppearanceCustomization {
            status: customization.result_code,
            appearance: customization.result.appearance,
        })
    }

    pub async fn map(&self, map: MapId) -> Result<MapAsset, InfiniteClientError> {
        self.ugc_version("maps", map.asset_id(), map.version_id(), false)
            .await
    }
    pub async fn mode(&self, mode: GameModeId) -> Result<GameVariantAsset, InfiniteClientError> {
        self.ugc_version("ugcGameVariants", mode.asset_id(), mode.version_id(), false)
            .await
    }
    pub async fn playlist(
        &self,
        asset_id: &str,
        version_id: &str,
    ) -> Result<PlaylistAsset, InfiniteClientError> {
        self.ugc_version("playlists", asset_id, version_id, true)
            .await
    }
    pub async fn map_mode_pair(
        &self,
        asset_id: &str,
        version_id: &str,
    ) -> Result<MapModePairAsset, InfiniteClientError> {
        self.ugc_version("mapModePairs", asset_id, version_id, true)
            .await
    }

    pub async fn asset(&self, kind: &str, asset_id: &str) -> Result<Value, InfiniteClientError> {
        self.get_with_clearance(
            &self.endpoints.ugc_base_url,
            &format!("/hi/{kind}/{asset_id}"),
            &[],
        )
        .await
    }

    async fn ugc_version<T: DeserializeOwned>(
        &self,
        kind: &str,
        asset_id: &str,
        version_id: &str,
        clearance_query: bool,
    ) -> Result<T, InfiniteClientError> {
        let path = format!("/hi/{kind}/{asset_id}/versions/{version_id}");
        if clearance_query {
            self.get_with_clearance_query(&self.endpoints.ugc_base_url, &path, &[])
                .await
        } else {
            self.get_with_clearance(&self.endpoints.ugc_base_url, &path, &[])
                .await
        }
    }

    pub async fn playlist_metadata(
        &self,
        playlist_id: &str,
    ) -> Result<PlaylistMetadata, InfiniteClientError> {
        self.get_with_clearance(
            &self.endpoints.game_cms_base_url,
            &format!("/hi/multiplayer/file/playlists/assets/{playlist_id}.json"),
            &[],
        )
        .await
    }

    pub async fn season_calendar(&self) -> Result<SeasonCalendar, InfiniteClientError> {
        self.get_with_clearance(
            &self.endpoints.game_cms_base_url,
            "/hi/progression/file/calendars/seasons/seasoncalendar.json",
            &[],
        )
        .await
    }
    pub async fn medals(&self) -> Result<Value, InfiniteClientError> {
        self.get_with_clearance(
            &self.endpoints.game_cms_base_url,
            "/hi/Waypoint/file/medals/metadata.json",
            &[],
        )
        .await
    }

    /// Gets the Waypoint mapping from emblem configurations to image assets.
    ///
    /// This remains JSON until the live response contract has been captured and tested.
    pub async fn emblem_mapping(&self) -> Result<Value, InfiniteClientError> {
        self.get_with_clearance(
            &self.endpoints.game_cms_base_url,
            "/hi/Waypoint/file/images/emblems/mapping.json",
            &[],
        )
        .await
    }
    /// Returns bans that Halo currently reports as being in effect for the targets.
    ///
    /// An empty result does not indicate whether the player was historically banned or whether a
    /// third-party service independently classifies the account as banned.
    pub async fn ban_summary(&self, xuids: &[Xuid]) -> Result<BanSummary, InfiniteClientError> {
        let targets = xuids
            .iter()
            .map(|xuid| wrap_xuid(xuid.as_str()))
            .collect::<Vec<_>>()
            .join(",");
        self.get(
            &self.endpoints.ban_base_url,
            "/hi/bansummary",
            &[("auth", "st".to_string()), ("targets", targets)],
        )
        .await
    }

    pub async fn ban_summary_by_gamertag(
        &self,
        gamertag: &str,
    ) -> Result<BanSummary, InfiniteClientError> {
        let user = self.user(gamertag).await?;
        self.ban_summary(&[Xuid::from(user.xuid)]).await
    }

    /// Resolves the localized title and body referenced by a ban summary entry.
    pub async fn ban_message(&self, message_path: &str) -> Result<BanMessage, InfiniteClientError> {
        self.get_with_clearance_named_query(
            &self.endpoints.game_cms_base_url,
            &format!("/hi/Banning/file/{}", message_path.trim_start_matches('/')),
            &[],
            "flight",
        )
        .await
    }
    /// Gets the authenticated player's match-history privacy settings.
    ///
    /// Halo rejects attempts to read this setting for a different player.
    pub async fn matches_privacy(
        &self,
        xuid: &Xuid,
    ) -> Result<MatchesPrivacy, InfiniteClientError> {
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

    pub async fn csr_season_calendar(&self) -> Result<CsrSeasonCalendar, InfiniteClientError> {
        self.get_with_clearance(
            &self.endpoints.game_cms_base_url,
            "/hi/Progression/file/Csr/Calendars/CsrSeasonCalendar.json",
            &[],
        )
        .await
    }

    /// Fetches the progression document referenced by a CSR calendar entry.
    ///
    /// The response remains JSON while the season-file schema is being explored.
    pub async fn csr_season_file(&self, file_path: &str) -> Result<Value, InfiniteClientError> {
        let path = format!("/hi/Progression/file/{}", file_path.trim_start_matches('/'));
        self.get(&self.endpoints.game_cms_base_url, &path, &[])
            .await
    }

    /// Returns the CSR season whose date range contains the current time.
    pub async fn current_csr_season(&self) -> Result<Option<CsrSeason>, InfiniteClientError> {
        let calendar = self.csr_season_calendar().await?;
        Ok(calendar.current(chrono::Utc::now()).cloned())
    }

    /// Resolves the current Ranked Arena playlist into its concrete maps and game variants.
    pub async fn current_ranked_arena(
        &self,
    ) -> Result<Option<RankedArenaSeason>, InfiniteClientError> {
        let Some(season) = self.current_csr_season().await? else {
            return Ok(None);
        };
        let playlist_id = PlaylistId::RANKED_ARENA.as_str();
        let metadata = self.playlist_metadata(playlist_id).await?;
        let playlist = self
            .playlist(playlist_id, &metadata.ugc_playlist_version)
            .await?;
        let mut map_modes = Vec::with_capacity(playlist.rotation_entries.len());
        for rotation in playlist.rotation_entries {
            let pair = self
                .map_mode_pair(&rotation.asset.asset_id, &rotation.asset.version_id)
                .await?;
            let map = self
                .map(MapId::new(
                    pair.map.asset_id.clone(),
                    pair.map.version_id.clone(),
                ))
                .await?;
            let mode = self
                .mode(GameModeId::new(
                    pair.mode.asset_id.clone(),
                    pair.mode.version_id.clone(),
                ))
                .await?;
            map_modes.push(RankedArenaMapMode {
                weight: rotation.metadata.weight,
                pair,
                map,
                mode,
            });
        }
        Ok(Some(RankedArenaSeason { season, map_modes }))
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
