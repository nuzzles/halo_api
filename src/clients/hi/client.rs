use std::{io::Read, sync::Arc, time::Duration};

use reqwest::{Client, StatusCode};
use serde::de::DeserializeOwned;
use xbox::models::Xuid;
use xbox::util::wrap_xuid;

use super::InfiniteClientError;
use super::endpoints::HaloEndpoints;
use super::film::{FilmEvent, FilmEventReport, decode_events, decode_players, validate_events};
use super::models::{
    AppearanceCustomization, BanMessage, BanSummary, CareerRanks, CareerRewardTrack, CsrRecords,
    CsrSeason, CsrSeasonCalendar, CurrentUser, CustomizationItemMetadata, EmblemMapping,
    EmblemMetadata, FilmChunk, FilmChunkData, FilmManifest, GameModeId, GameVariantAsset,
    HipcSettings, MapAsset, MapId, MapModePairAsset, MatchCount, MatchHistoryType, MatchSkill,
    MatchStats, MatchType, MatchesPrivacy, MedalMetadata, OperationRewardTrack, PlayerCareerRank,
    PlayerChallengeDecks, PlayerCustomizationCollection, PlayerMatchHistory, PlayerOperationPasses,
    PlaylistAsset, PlaylistId, PlaylistMetadata, RankedArenaMapMode, RankedArenaSeason,
    SeasonCalendar, ServiceRecord, ServiceRecordFilter, UgcAsset, UgcAssetKind, UgcSearchResults,
    UserInfo,
};
use super::pager::MatchHistoryPager;
use super::player::Player;
use super::rate_limit::RateLimiter;
use crate::auth::{HaloAuthClient, HaloCredentials};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);
/// Default per-origin request rate.
const DEFAULT_REQUESTS_PER_SECOND: u32 = 9;
const HALO_PC_USER_AGENT: &str = "SHIVA-2043073184/6.10021.18539.0 (release; PC)";
const HALO_WAYPOINT_USER_AGENT: &str =
    "HaloWaypoint/2021112313511900 CFNetwork/1327.0.4 Darwin/21.2.0";

/// Returns the `scheme://host[:port]` prefix of a URL for per-origin rate limiting.
///
/// Falls back to the whole string if the URL has no path separator, which still keeps distinct
/// origins in distinct rate-limit buckets.
fn origin_of(url: &str) -> &str {
    match url.split_once("://") {
        Some((scheme, rest)) => {
            let host_len = rest.find('/').unwrap_or(rest.len());
            &url[..scheme.len() + 3 + host_len]
        }
        None => url,
    }
}

/// Halo Infinite API client. Authentication is supplied by a [`HaloAuthClient`].
///
/// Construct one with [`HaloInfiniteClient::new`] for the defaults, or
/// [`HaloInfiniteClient::builder`] to configure the request timeout and per-origin rate limit.
///
/// Cloning is cheap: every field is `Arc`-backed or trivially copyable, and clones share the
/// same credential cache and rate limiter.
#[derive(Clone)]
pub struct HaloInfiniteClient {
    auth: HaloAuthClient,
    http: Client,
    endpoints: HaloEndpoints,
    limiter: RateLimiter,
    timeout: Duration,
}

impl HaloInfiniteClient {
    pub fn new(auth: HaloAuthClient) -> Self {
        Self::builder().build(auth)
    }

    /// Starts configuring a client. See [`HaloInfiniteClientBuilder`].
    pub fn builder() -> HaloInfiniteClientBuilder {
        HaloInfiniteClientBuilder::default()
    }

    #[cfg(test)]
    pub(crate) fn with_endpoints(auth: HaloAuthClient, endpoints: HaloEndpoints) -> Self {
        Self::builder().build_with_endpoints(auth, endpoints)
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
        self.limiter.acquire(origin_of(url)).await;
        let mut request = self
            .http
            .get(url)
            .query(query)
            .header("X-343-Authorization-Spartan", &credentials.spartan_token)
            .header("Accept", "application/json")
            .header("User-Agent", user_agent)
            .timeout(self.timeout);
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

    async fn get_bytes_with_clearance(
        &self,
        base: &str,
        path: &str,
    ) -> Result<Vec<u8>, InfiniteClientError> {
        let url = format!("{base}{path}");
        let first = self.auth.credentials(true).await?;
        match self.get_bytes_once(&url, &first).await {
            Err(error) if error.is_unauthorized() => {
                self.auth.invalidate().await;
                let second = self.auth.credentials(true).await?;
                self.get_bytes_once(&url, &second).await
            }
            result => result,
        }
    }

    async fn get_bytes_once(
        &self,
        url: &str,
        credentials: &HaloCredentials,
    ) -> Result<Vec<u8>, InfiniteClientError> {
        self.limiter.acquire(origin_of(url)).await;
        let mut request = self
            .http
            .get(url)
            .header("X-343-Authorization-Spartan", &credentials.spartan_token)
            .header("User-Agent", HALO_PC_USER_AGENT)
            .timeout(self.timeout);
        if let Some(clearance) = &credentials.clearance {
            request = request.header("343-Clearance", clearance);
        }
        let response = request.send().await?;
        if !response.status().is_success() {
            let url = response.url().to_string();
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(InfiniteClientError::HttpStatus { url, status, body });
        }
        Ok(response.bytes().await?.to_vec())
    }

    /// Resolves a [`Player`] to a numeric [`Xuid`], looking it up via [`Self::user`] if given a
    /// gamertag. A no-op for [`Player::Xuid`].
    async fn resolve_xuid(&self, player: &Player) -> Result<Xuid, InfiniteClientError> {
        match player {
            Player::Xuid(xuid) => Ok(xuid.clone()),
            Player::Gamertag(_) => Ok(Xuid::from(self.user(player).await?.xuid)),
        }
    }

    /// Resolves a slice of [`Player`]s to numeric [`Xuid`]s.
    ///
    /// Gamertags are resolved sequentially, one [`Self::user`] request per gamertag, before the
    /// batch call proceeds. Batch endpoints are normally called with XUIDs already in hand from a
    /// prior response, so this cold path is left simple rather than run concurrently.
    async fn resolve_xuids(&self, players: &[Player]) -> Result<Vec<Xuid>, InfiniteClientError> {
        let mut xuids = Vec::with_capacity(players.len());
        for player in players {
            xuids.push(self.resolve_xuid(player).await?);
        }
        Ok(xuids)
    }

    /// Renders a `xuid(...)`-wrapped, comma-joined player list for a batch query parameter.
    fn join_wrapped_xuids(xuids: &[Xuid]) -> String {
        xuids
            .iter()
            .map(|xuid| wrap_xuid(xuid.as_str()))
            .collect::<Vec<_>>()
            .join(",")
    }

    /// Renders a [`Player`] into the `gt(Name)`/`xuid(123)` form Halo's `/users/{...}`-style
    /// endpoints accept for either variant directly, with no resolution required.
    fn gt_or_xuid_segment(player: &Player) -> String {
        match player {
            Player::Gamertag(gamertag) => format!("gt({gamertag})"),
            Player::Xuid(xuid) => wrap_xuid(xuid.as_str()),
        }
    }

    pub async fn playlist_csr(
        &self,
        playlist: PlaylistId,
        player: &Player,
    ) -> Result<CsrRecords, InfiniteClientError> {
        self.playlist_csr_batch(playlist, std::slice::from_ref(player))
            .await
    }

    pub async fn playlist_csr_batch(
        &self,
        playlist: PlaylistId,
        players: &[Player],
    ) -> Result<CsrRecords, InfiniteClientError> {
        let xuids = self.resolve_xuids(players).await?;
        self.get_with_clearance(
            &self.endpoints.skill_base_url,
            &format!("/hi/playlist/{playlist}/csrs"),
            &[("players", Self::join_wrapped_xuids(&xuids))],
        )
        .await
    }

    /// Gets a player's lifetime matchmade service record.
    ///
    /// Use [`Self::service_record_with`] to scope the record to a season, playlist, or mode, or to
    /// query custom or local games instead of matchmaking.
    pub async fn service_record(
        &self,
        player: &Player,
    ) -> Result<ServiceRecord, InfiniteClientError> {
        self.service_record_with(
            player,
            MatchType::Matchmade,
            &ServiceRecordFilter::default(),
        )
        .await
    }

    /// Gets a player's service record for a given match type, optionally filtered.
    ///
    /// Halo applies [`ServiceRecordFilter`] only to [`MatchType::Matchmade`] and rejects certain
    /// filter combinations with a 400; see [`ServiceRecordFilter`] for the supported sets.
    pub async fn service_record_with(
        &self,
        player: &Player,
        match_type: MatchType,
        filter: &ServiceRecordFilter,
    ) -> Result<ServiceRecord, InfiniteClientError> {
        let not_found = || InfiniteClientError::GamertagNotFound(player.to_string());
        let xuid = match self.resolve_xuid(player).await {
            Err(InfiniteClientError::HttpStatus {
                status: StatusCode::BAD_REQUEST | StatusCode::NOT_FOUND,
                ..
            }) => return Err(not_found()),
            other => other?,
        };
        let result = self
            .get(
                &self.endpoints.halostats_base_url,
                &format!(
                    "/hi/players/{}/{}/servicerecord",
                    wrap_xuid(xuid.as_str()),
                    match_type.as_str()
                ),
                &filter.to_query(),
            )
            .await;
        match result {
            Err(InfiniteClientError::HttpStatus {
                status: StatusCode::BAD_REQUEST | StatusCode::NOT_FOUND,
                ..
            }) => Err(not_found()),
            other => other,
        }
    }

    /// Gets a page of a player's matchmaking history (all match types).
    ///
    /// Use [`Self::player_matches_of_type`] to restrict to matchmaking, custom, or local games.
    /// Halo caps `count` at 25 per page; use [`Self::player_matches_pager`] to walk full history.
    pub async fn player_matches(
        &self,
        player: &Player,
        start: u32,
        count: u32,
    ) -> Result<PlayerMatchHistory, InfiniteClientError> {
        self.player_matches_of_type(player, start, count, MatchHistoryType::All)
            .await
    }

    /// Gets a page of a player's match history restricted to `match_type`.
    pub async fn player_matches_of_type(
        &self,
        player: &Player,
        start: u32,
        count: u32,
        match_type: MatchHistoryType,
    ) -> Result<PlayerMatchHistory, InfiniteClientError> {
        let xuid = self.resolve_xuid(player).await?;
        self.get(
            &self.endpoints.halostats_base_url,
            &format!("/hi/players/{}/matches", wrap_xuid(xuid.as_str())),
            &[
                ("start", start.to_string()),
                ("count", count.to_string()),
                ("type", match_type.as_str().to_string()),
            ],
        )
        .await
    }

    /// Returns a pager that walks `player`'s match history of `match_type`, 25 entries per page.
    pub fn player_matches_pager(
        &self,
        player: Player,
        match_type: MatchHistoryType,
    ) -> MatchHistoryPager {
        MatchHistoryPager::new(self.clone(), player, match_type)
    }

    pub async fn match_stats(&self, match_id: &str) -> Result<MatchStats, InfiniteClientError> {
        self.get(
            &self.endpoints.halostats_base_url,
            &format!("/hi/matches/{match_id}/stats"),
            &[],
        )
        .await
    }

    /// Returns the Theater film manifest and chunk inventory for a match.
    pub async fn match_film(&self, match_id: &str) -> Result<FilmManifest, InfiniteClientError> {
        self.get_with_clearance(
            &self.endpoints.ugc_base_url,
            &format!("/hi/films/matches/{match_id}/spectate"),
            &[],
        )
        .await
    }

    /// Downloads and zlib-decompresses one Theater film chunk.
    pub async fn film_chunk(
        &self,
        film: &FilmManifest,
        chunk: &FilmChunk,
    ) -> Result<FilmChunkData, InfiniteClientError> {
        let base = film.blob_storage_path_prefix.trim_end_matches('/');
        let path = format!("/{}", chunk.file_relative_path.trim_start_matches('/'));
        let compressed = self.get_bytes_with_clearance(base, &path).await?;
        let mut decoder = flate2::read::ZlibDecoder::new(compressed.as_slice());
        let mut data = Vec::new();
        decoder
            .read_to_end(&mut data)
            .map_err(|error| InfiniteClientError::FilmDecompression(Arc::new(error)))?;
        Ok(FilmChunkData {
            metadata: chunk.clone(),
            data,
        })
    }

    /// Downloads and decompresses every retained chunk in a Theater film.
    pub async fn film_chunks(
        &self,
        film: &FilmManifest,
    ) -> Result<Vec<FilmChunkData>, InfiniteClientError> {
        let mut chunks = Vec::with_capacity(film.custom_data.chunks.len());
        for chunk in &film.custom_data.chunks {
            chunks.push(self.film_chunk(film, chunk).await?);
        }
        Ok(chunks)
    }

    /// Downloads a match's Theater film and returns its human-player highlight events.
    ///
    /// Events include kills, deaths, mode-related events, and medals. Theater films do not
    /// contain highlight events for bots or AI opponents, and some events may be absent from a
    /// film. Use [`Self::match_highlight_events_with_validation`] to compare the decoded counts
    /// with Halo's match-statistics record.
    pub async fn match_highlight_events(
        &self,
        match_id: &str,
    ) -> Result<Vec<FilmEvent>, InfiniteClientError> {
        Ok(self.decoded_highlight_events(match_id).await?.0)
    }

    async fn decoded_highlight_events(
        &self,
        match_id: &str,
    ) -> Result<(Vec<FilmEvent>, Vec<super::film::FilmPlayer>), InfiniteClientError> {
        let film = self.match_film(match_id).await?;
        let chunks = self.film_chunks(&film).await?;
        let players = decode_players(&chunks);
        let events = decode_events(&chunks, &players, film.custom_data.film_major_version);
        Ok((events, players))
    }

    /// Downloads a match's Theater highlights and compares each human player's
    /// kill, death, and medal totals with the match-statistics API.
    ///
    /// A mismatch is returned in [`FilmEventReport::validation`] rather than
    /// as an error because Theater films can omit summary events.
    pub async fn match_highlight_events_with_validation(
        &self,
        match_id: &str,
    ) -> Result<FilmEventReport, InfiniteClientError> {
        let ((events, players), match_stats) = tokio::try_join!(
            self.decoded_highlight_events(match_id),
            self.match_stats(match_id)
        )?;
        Ok(FilmEventReport {
            validation: validate_events(&events, &players, &match_stats),
            events,
        })
    }

    /// Gets per-player CSR and MMR skill results for a completed match.
    pub async fn match_skill(
        &self,
        match_id: &str,
        players: &[Player],
    ) -> Result<MatchSkill, InfiniteClientError> {
        let xuids = self.resolve_xuids(players).await?;
        self.get(
            &self.endpoints.skill_base_url,
            &format!("/hi/matches/{match_id}/skill"),
            &[("players", Self::join_wrapped_xuids(&xuids))],
        )
        .await
    }

    /// Looks up profile information by gamertag or XUID.
    pub async fn user(&self, player: &Player) -> Result<UserInfo, InfiniteClientError> {
        self.get(
            &self.endpoints.profile_base_url,
            &format!("/users/{}", Self::gt_or_xuid_segment(player)),
            &[],
        )
        .await
    }

    /// Looks up profile information for multiple players, resolving any gamertags first.
    pub async fn users(&self, players: &[Player]) -> Result<Vec<UserInfo>, InfiniteClientError> {
        let xuids = self.resolve_xuids(players).await?;
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
        player: &Player,
    ) -> Result<AppearanceCustomization, InfiniteClientError> {
        let xuid = self.resolve_xuid(player).await?;
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

    /// Gets public customization data for multiple players, resolving any gamertags first.
    pub async fn player_customizations(
        &self,
        players: &[Player],
    ) -> Result<PlayerCustomizationCollection, InfiniteClientError> {
        let xuids = self.resolve_xuids(players).await?;
        self.get_authenticated_with_user_agent(
            &self.endpoints.economy_base_url,
            "/hi/customization",
            &[("players", Self::join_wrapped_xuids(&xuids))],
            true,
            HALO_WAYPOINT_USER_AGENT,
        )
        .await
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

    /// Fetches the current (unversioned/"latest") revision of a UGC asset by kind and ID.
    ///
    /// For kinds with a dedicated typed method ([`Self::map`], [`Self::mode`],
    /// [`Self::playlist`], [`Self::map_mode_pair`]), prefer those — they parse kind-specific
    /// `CustomData`. This method covers every kind via the common asset envelope only.
    pub async fn asset(
        &self,
        kind: UgcAssetKind,
        asset_id: &str,
    ) -> Result<UgcAsset, InfiniteClientError> {
        self.get_with_clearance(
            &self.endpoints.ugc_base_url,
            &format!("/hi/{}/{asset_id}", kind.path_segment()),
            &[],
        )
        .await
    }

    /// Fetches the current (unversioned) revision of a Theater film asset.
    pub async fn film_asset(&self, asset_id: &str) -> Result<UgcAsset, InfiniteClientError> {
        self.asset(UgcAssetKind::Film, asset_id).await
    }

    /// Fetches the current (unversioned) revision of a prefab asset.
    pub async fn prefab_asset(&self, asset_id: &str) -> Result<UgcAsset, InfiniteClientError> {
        self.asset(UgcAssetKind::Prefab, asset_id).await
    }

    /// Fetches the current (unversioned) revision of an engine game variant asset.
    pub async fn engine_game_variant(
        &self,
        asset_id: &str,
    ) -> Result<UgcAsset, InfiniteClientError> {
        self.asset(UgcAssetKind::EngineGameVariant, asset_id).await
    }

    /// Searches Halo Infinite's live UGC catalog.
    pub async fn search_assets(
        &self,
        kind: UgcAssetKind,
        start: u32,
        count: u32,
    ) -> Result<UgcSearchResults, InfiniteClientError> {
        self.get_authenticated_with_user_agent(
            &self.endpoints.ugc_base_url,
            "/hi/search",
            &[
                ("start", start.to_string()),
                ("count", count.to_string()),
                ("include-times", "false".to_string()),
                ("sort", "PlaysRecent".to_string()),
                ("order", "Desc".to_string()),
                ("assetKind", kind.as_str().to_string()),
            ],
            true,
            HALO_WAYPOINT_USER_AGENT,
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
    /// Gets the localized metadata catalog for all obtainable medals.
    pub async fn medals(&self) -> Result<MedalMetadata, InfiniteClientError> {
        self.get_with_clearance(
            &self.endpoints.game_cms_base_url,
            "/hi/Waypoint/file/medals/metadata.json",
            &[],
        )
        .await
    }

    /// Gets the Waypoint mapping from emblem configurations to image assets.
    pub async fn emblem_mapping(&self) -> Result<EmblemMapping, InfiniteClientError> {
        self.get_with_clearance(
            &self.endpoints.game_cms_base_url,
            "/hi/Waypoint/file/images/emblems/mapping.json",
            &[],
        )
        .await
    }

    /// Gets the localized display metadata for an emblem inventory item.
    pub async fn emblem_metadata(
        &self,
        emblem_path: &str,
    ) -> Result<EmblemMetadata, InfiniteClientError> {
        self.customization_metadata(emblem_path).await
    }

    /// Gets localized display metadata for a customization inventory item or core.
    pub async fn customization_metadata(
        &self,
        item_path: &str,
    ) -> Result<CustomizationItemMetadata, InfiniteClientError> {
        self.get_with_clearance(
            &self.endpoints.game_cms_base_url,
            &format!("/hi/progression/file/{}", item_path.trim_start_matches('/')),
            &[],
        )
        .await
    }

    /// Downloads the display image referenced by customization metadata.
    pub async fn customization_image(
        &self,
        metadata: &CustomizationItemMetadata,
    ) -> Result<Option<Vec<u8>>, InfiniteClientError> {
        let Some(path) = metadata.image_cms_path() else {
            return Ok(None);
        };
        self.game_cms_image(path).await.map(Some)
    }

    /// Downloads a career-rank icon (small, large, or adornment) by its CMS path, e.g.
    /// [`RewardTrackRank::rank_icon`](super::models::RewardTrackRank::rank_icon),
    /// [`RewardTrackRank::rank_large_icon`](super::models::RewardTrackRank::rank_large_icon), or
    /// [`RewardTrackRank::rank_adornment_icon`](super::models::RewardTrackRank::rank_adornment_icon).
    pub async fn rank_icon_image(&self, cms_path: &str) -> Result<Vec<u8>, InfiniteClientError> {
        self.game_cms_image(cms_path).await
    }

    async fn game_cms_image(&self, cms_path: &str) -> Result<Vec<u8>, InfiniteClientError> {
        self.get_bytes_with_clearance(
            &self.endpoints.game_cms_base_url,
            &format!("/hi/images/file/{}", cms_path.trim_start_matches('/')),
        )
        .await
    }

    /// Downloads an emblem PNG with the required Halo authentication headers.
    pub async fn emblem_image(
        &self,
        assets: &super::models::EmblemImageAssets,
    ) -> Result<Vec<u8>, InfiniteClientError> {
        self.waypoint_image(&assets.emblem_cms_path).await
    }

    /// Downloads a nameplate PNG with the required Halo authentication headers.
    pub async fn emblem_nameplate(
        &self,
        assets: &super::models::EmblemImageAssets,
    ) -> Result<Vec<u8>, InfiniteClientError> {
        self.waypoint_image(&assets.nameplate_cms_path).await
    }

    async fn waypoint_image(&self, cms_path: &str) -> Result<Vec<u8>, InfiniteClientError> {
        self.get_bytes_with_clearance(
            &self.endpoints.game_cms_base_url,
            &format!("/hi/Waypoint/file/{}", cms_path.trim_start_matches('/')),
        )
        .await
    }
    /// Returns bans that Halo currently reports as being in effect for the targets, resolving any
    /// gamertags first.
    ///
    /// An empty result does not indicate whether the player was historically banned or whether a
    /// third-party service independently classifies the account as banned.
    pub async fn ban_summary(&self, players: &[Player]) -> Result<BanSummary, InfiniteClientError> {
        let xuids = self.resolve_xuids(players).await?;
        self.get(
            &self.endpoints.ban_base_url,
            "/hi/bansummary",
            &[
                ("auth", "st".to_string()),
                ("targets", Self::join_wrapped_xuids(&xuids)),
            ],
        )
        .await
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
        player: &Player,
    ) -> Result<MatchesPrivacy, InfiniteClientError> {
        let xuid = self.resolve_xuid(player).await?;
        self.get(
            &self.endpoints.halostats_base_url,
            &format!("/hi/players/{}/matches-privacy", wrap_xuid(xuid.as_str())),
            &[],
        )
        .await
    }

    /// Looks up the authenticated player's own profile via `comms.svc.halowaypoint.com`.
    pub async fn current_user(&self) -> Result<CurrentUser, InfiniteClientError> {
        self.get(&self.endpoints.current_user_url, "", &[]).await
    }

    /// Gets a player's match counts across matchmade, custom, and local games.
    pub async fn player_match_count(
        &self,
        player: &Player,
    ) -> Result<MatchCount, InfiniteClientError> {
        let xuid = self.resolve_xuid(player).await?;
        self.get(
            &self.endpoints.halostats_base_url,
            &format!("/hi/players/{}/matches/count", wrap_xuid(xuid.as_str())),
            &[],
        )
        .await
    }

    /// Gets a player's active challenge decks.
    pub async fn challenge_decks(
        &self,
        player: &Player,
    ) -> Result<PlayerChallengeDecks, InfiniteClientError> {
        let xuid = self.resolve_xuid(player).await?;
        self.get(
            &self.endpoints.halostats_base_url,
            &format!("/hi/players/{}/decks", wrap_xuid(xuid.as_str())),
            &[],
        )
        .await
    }

    /// Gets a player's progress on a career-rank reward track.
    ///
    /// `reward_track_id` defaults to `careerRank1` via [`Self::career_rank`].
    pub async fn career_rank_with_track(
        &self,
        player: &Player,
        reward_track_id: &str,
    ) -> Result<PlayerCareerRank, InfiniteClientError> {
        let xuid = self.resolve_xuid(player).await?;
        self.get_with_clearance(
            &self.endpoints.economy_base_url,
            &format!(
                "/hi/players/{}/rewardtracks/careerranks/{reward_track_id}",
                wrap_xuid(xuid.as_str())
            ),
            &[],
        )
        .await
    }

    /// Gets a player's current career rank (the `careerRank1` track).
    ///
    /// This only works for the authenticated player; use [`Self::career_ranks`] for other
    /// players.
    pub async fn career_rank(
        &self,
        player: &Player,
    ) -> Result<PlayerCareerRank, InfiniteClientError> {
        self.career_rank_with_track(player, "careerRank1").await
    }

    /// Gets the current career rank (the `careerRank1` track) for one player.
    ///
    /// Unlike [`Self::career_rank`], this works for any player.
    pub async fn career_rank_of(
        &self,
        player: &Player,
    ) -> Result<PlayerCareerRank, InfiniteClientError> {
        let ranks = self.career_ranks(std::slice::from_ref(player)).await?;
        ranks
            .records
            .into_iter()
            .next()
            .map(|record| record.result)
            .ok_or_else(|| InfiniteClientError::CareerRankNotFound(player.to_string()))
    }

    /// Gets the current career rank (the `careerRank1` track) for multiple players.
    ///
    /// Unlike [`Self::career_rank`], this works for any player.
    pub async fn career_ranks(
        &self,
        players: &[Player],
    ) -> Result<CareerRanks, InfiniteClientError> {
        let xuids = self.resolve_xuids(players).await?;
        self.get_with_clearance(
            &self.endpoints.economy_base_url,
            "/hi/careerranks/careerRank1",
            &[("players", Self::join_wrapped_xuids(&xuids))],
        )
        .await
    }

    /// Gets a player's owned and available operation passes.
    pub async fn reward_track_operations(
        &self,
        player: &Player,
    ) -> Result<PlayerOperationPasses, InfiniteClientError> {
        let xuid = self.resolve_xuid(player).await?;
        self.get_with_clearance(
            &self.endpoints.economy_base_url,
            &format!(
                "/hi/players/{}/rewardtracks/operations",
                wrap_xuid(xuid.as_str())
            ),
            &[],
        )
        .await
    }

    /// Gets the career-rank reward-track definition (rank titles, XP, and rewards).
    pub async fn career_reward_track(&self) -> Result<CareerRewardTrack, InfiniteClientError> {
        self.get_with_clearance(
            &self.endpoints.game_cms_base_url,
            "/hi/Progression/file/RewardTracks/CareerRanks/careerRank1.json",
            &[],
        )
        .await
    }

    /// Gets an operation (battle pass) reward-track definition by its CMS file path.
    ///
    /// The path is typically taken from
    /// [`Season::operation_track_path`](super::models::Season::operation_track_path) or
    /// [`PlayerOperationPass::reward_track_path`](super::models::PlayerOperationPass::reward_track_path),
    /// e.g. `RewardTracks/Operations/S05OpPassM01.json`.
    pub async fn operation_reward_track(
        &self,
        reward_track_path: &str,
    ) -> Result<OperationRewardTrack, InfiniteClientError> {
        self.get_with_clearance(
            &self.endpoints.game_cms_base_url,
            &format!(
                "/hi/Progression/file/{}",
                reward_track_path.trim_start_matches('/')
            ),
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
    /// This intentionally stays untyped raw JSON. No community Halo Infinite client has ever
    /// confirmed a schema for it — even the most complete one (OpenSpartan/grunt) notes this
    /// endpoint returns a 403 Forbidden for them. It may be genuinely inaccessible at the auth
    /// tier this crate uses, not merely unexplored, so a guessed struct would risk being simply
    /// wrong.
    pub async fn csr_season_file(
        &self,
        file_path: &str,
    ) -> Result<serde_json::Value, InfiniteClientError> {
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

        let tasks = playlist
            .rotation_entries
            .into_iter()
            .map(|rotation| {
                let client = self.clone();
                tokio::spawn(async move {
                    let pair = client
                        .map_mode_pair(&rotation.asset.asset_id, &rotation.asset.version_id)
                        .await?;
                    let (map, mode) = tokio::try_join!(
                        client.map(MapId::new(
                            pair.map.asset_id.clone(),
                            pair.map.version_id.clone(),
                        )),
                        client.mode(GameModeId::new(
                            pair.mode.asset_id.clone(),
                            pair.mode.version_id.clone(),
                        ))
                    )?;
                    Ok::<_, InfiniteClientError>(RankedArenaMapMode {
                        weight: rotation.metadata.weight,
                        pair,
                        map,
                        mode,
                    })
                })
            })
            .collect::<Vec<_>>();

        let mut map_modes = Vec::with_capacity(tasks.len());
        for task in tasks {
            map_modes.push(task.await.map_err(|_| InfiniteClientError::TaskJoin)??);
        }
        Ok(Some(RankedArenaSeason { season, map_modes }))
    }

    /// Gets Halo's HIPC endpoint-discovery manifest as JSON.
    ///
    /// This is the same manifest [`examples/discover_endpoints.rs`](https://github.com/nuzzles/halo_api/blob/main/examples/discover_endpoints.rs)
    /// parses as XML directly (no authentication required there); this method fetches the JSON
    /// content-negotiated representation instead, which requires an authenticated client.
    pub async fn settings(&self) -> Result<HipcSettings, InfiniteClientError> {
        self.get(
            &self.endpoints.settings_base_url,
            "/settings/hipc/e2a0a7c6-6efe-42af-9283-c2ab73250c48",
            &[],
        )
        .await
    }
}

/// Configures a [`HaloInfiniteClient`].
///
/// ```no_run
/// # use std::time::Duration;
/// # fn example(auth: halo_api::auth::HaloAuthClient) {
/// use halo_api::clients::hi::HaloInfiniteClient;
///
/// let halo = HaloInfiniteClient::builder()
///     .timeout(Duration::from_secs(30))
///     .requests_per_second(3)
///     .build(auth);
/// # let _ = halo;
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct HaloInfiniteClientBuilder {
    timeout: Duration,
    requests_per_second: u32,
    http: Option<Client>,
}

impl Default for HaloInfiniteClientBuilder {
    fn default() -> Self {
        Self {
            timeout: DEFAULT_TIMEOUT,
            requests_per_second: DEFAULT_REQUESTS_PER_SECOND,
            http: None,
        }
    }
}

impl HaloInfiniteClientBuilder {
    /// Sets the per-request timeout. Defaults to 10 seconds.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Sets the maximum requests per second, enforced independently per Halo Waypoint origin.
    ///
    /// Defaults to 9. Pass `0` to disable client-side pacing entirely. This throttles an
    /// undocumented API, so lowering it is safer than raising it.
    pub fn requests_per_second(mut self, requests_per_second: u32) -> Self {
        self.requests_per_second = requests_per_second;
        self
    }

    /// Supplies a preconfigured [`reqwest::Client`], e.g. with a proxy or connection pool tuning.
    ///
    /// The per-request timeout from [`Self::timeout`] is still applied on top of any the client
    /// carries.
    pub fn http_client(mut self, http: Client) -> Self {
        self.http = Some(http);
        self
    }

    /// Builds the client against the real Halo Waypoint endpoints.
    pub fn build(self, auth: HaloAuthClient) -> HaloInfiniteClient {
        self.build_with_endpoints(auth, HaloEndpoints::default())
    }

    pub(crate) fn build_with_endpoints(
        self,
        auth: HaloAuthClient,
        endpoints: HaloEndpoints,
    ) -> HaloInfiniteClient {
        HaloInfiniteClient {
            auth,
            http: self.http.unwrap_or_default(),
            endpoints,
            limiter: RateLimiter::per_second(self.requests_per_second),
            timeout: self.timeout,
        }
    }
}
