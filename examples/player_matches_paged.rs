//! Walks a player's full match history using `MatchHistoryPager`.

mod common;

use halo_api::clients::hi::models::MatchHistoryType;

#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let (_, halo) = common::halo_infinite_client()?;
    let (gamertag, player) = common::player_xuid(&halo).await?;

    let mut pager = halo.player_matches_pager(player, MatchHistoryType::All);
    let mut total = 0;
    loop {
        let page = pager.next_page().await?;
        if page.is_empty() {
            break;
        }
        total += page.len();
        println!(
            "{gamertag}: fetched {} more matches (total {total})",
            page.len()
        );
    }
    println!("{gamertag}: {total} total matches");
    Ok(())
}
