//! Resolves gamertags and checks their currently effective bans in one request.

mod common;

use xbox::models::Xuid;

#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let (_, halo) = common::halo_infinite_client()?;
    let gamertags = common::comma_separated(
        "HALO_GAMERTAGS",
        "Gamertags (comma-separated; try Glorified MVP)",
    )?;

    let mut xuids = Vec::with_capacity(gamertags.len());
    for gamertag in &gamertags {
        let user = halo.user(gamertag).await?;
        println!("{} -> xuid({})", user.gamertag, user.xuid);
        xuids.push(Xuid::from(user.xuid));
    }
    let summary = halo.ban_summary(&xuids).await?;

    for result in summary.results {
        println!("{} (result code {})", result.id, result.result_code);
        if result.result.bans_in_effect.is_empty() {
            println!(
                "  Halo's ban-summary endpoint returned no bans currently in effect; \
                 this does not report historical or third-party ban classifications"
            );
        }
        for ban in result.result.bans_in_effect {
            let message = halo.ban_message(&ban.message_path).await?;
            let rendered = message
                .display_message
                .value
                .replace("{0}", &ban.enforce_until.iso8601_date.to_string());
            println!(
                "  type {} scope {} until {} ({})\n  {}\n  {}",
                ban.ban_type,
                ban.scope,
                ban.enforce_until.iso8601_date,
                ban.message_path,
                message.title,
                rendered,
            );
        }
    }
    Ok(())
}
