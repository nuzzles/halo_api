//! A pager for walking a player's full match history.

use super::InfiniteClientError;
use super::client::HaloInfiniteClient;
use super::models::{MatchHistoryEntry, MatchHistoryType};
use super::player::Player;

/// Halo caps a single match-history page at this many entries.
pub const MAX_PAGE_SIZE: u32 = 25;

/// Walks a player's match history page by page.
///
/// Construct one with [`HaloInfiniteClient::player_matches_pager`]. Call [`Self::next_page`]
/// repeatedly; it returns an empty `Vec` once exhausted. A page shorter than
/// [`MAX_PAGE_SIZE`] is treated as the final page — `start` advances by the returned page length
/// (not always the full page size), so a short final page cannot cause the next call to skip or
/// duplicate entries.
pub struct MatchHistoryPager {
    client: HaloInfiniteClient,
    player: Player,
    match_type: MatchHistoryType,
    start: u32,
    done: bool,
}

impl MatchHistoryPager {
    pub(crate) fn new(
        client: HaloInfiniteClient,
        player: Player,
        match_type: MatchHistoryType,
    ) -> Self {
        Self {
            client,
            player,
            match_type,
            start: 0,
            done: false,
        }
    }

    /// Fetches the next page (up to [`MAX_PAGE_SIZE`] entries).
    ///
    /// Returns an empty `Vec` once the player's history is exhausted; subsequent calls after that
    /// also return an empty `Vec` without making another request.
    pub async fn next_page(&mut self) -> Result<Vec<MatchHistoryEntry>, InfiniteClientError> {
        if self.done {
            return Ok(Vec::new());
        }
        let page = self
            .client
            .player_matches_of_type(&self.player, self.start, MAX_PAGE_SIZE, self.match_type)
            .await?;
        let len = page.results.len() as u32;
        self.start += len;
        if len < MAX_PAGE_SIZE {
            self.done = true;
        }
        Ok(page.results)
    }

    /// True once a page shorter than [`MAX_PAGE_SIZE`] has been returned (or the first page was
    /// empty). A cheap synchronous alternative to checking for an empty page from
    /// [`Self::next_page`].
    pub fn is_done(&self) -> bool {
        self.done
    }
}
