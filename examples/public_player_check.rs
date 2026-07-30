//! Resolves public players and checks the player-facing APIs available to other callers.

mod common;

use halo_api::clients::hi::Player;
use xbox::models::Xuid;

#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let (_, halo) = common::halo_infinite_client()?;
    let gamertags = common::comma_separated("HALO_GAMERTAGS", "Gamertags (comma-separated)")?;

    let mut xuids = Vec::with_capacity(gamertags.len());
    for gamertag in &gamertags {
        let user = halo.user(&Player::gamertag(gamertag.clone())).await?;
        println!("{} -> xuid({})", user.gamertag, user.xuid);
        xuids.push(Xuid::from(user.xuid));
    }
    let players = xuids.iter().map(Player::from).collect::<Vec<_>>();

    match halo.users(&players).await {
        Ok(users) => println!("users batch: {} result(s)", users.len()),
        Err(error) => println!("users batch: {error}"),
    }
    match halo.player_customizations(&players).await {
        Ok(response) => println!(
            "customizations batch: {} result(s)",
            response.player_customizations.len()
        ),
        Err(error) => println!("customizations batch: {error}"),
    }
    match halo.ban_summary(&players).await {
        Ok(response) => println!("ban summaries batch: {} result(s)", response.results.len()),
        Err(error) => println!("ban summaries batch: {error}"),
    }

    for (gamertag, player) in gamertags.iter().zip(&players) {
        println!("\n{gamertag} ({player})");
        match halo.service_record(player).await {
            Ok(_) => println!("  service record: public"),
            Err(error) => println!("  service record: {error}"),
        }
        match halo.player_matches(player, 0, 1).await {
            Ok(_) => println!("  match history: public"),
            Err(error) => println!("  match history: {error}"),
        }
        match halo.player_match_count(player).await {
            Ok(_) => println!("  match count: public"),
            Err(error) => println!("  match count: {error}"),
        }
    }

    Ok(())
}
