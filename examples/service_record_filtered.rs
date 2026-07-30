mod common;

use halo_api::clients::hi::Player;
use halo_api::clients::hi::models::{MatchType, ServiceRecordFilter};

#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let (_, halo) = common::halo_infinite_client()?;
    let gamertag = common::value("HALO_GAMERTAG", "Gamertag")?;
    let player = Player::gamertag(gamertag.clone());

    // A season file path looks like "Csr/Seasons/CsrSeason5-1.json". Combine with a
    // game-variant category to scope the record further.
    let season = common::value("HALO_SEASON_ID", "CSR season file path")?;
    let filter = ServiceRecordFilter::for_season(season);

    println!("Filtered service record for {gamertag}:");
    println!(
        "{:#?}",
        halo.service_record_with(&player, MatchType::Matchmade, &filter)
            .await?
    );
    Ok(())
}
