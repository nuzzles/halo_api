//! Shows the Xbox account used by Halo API authentication.

mod common;

use xbox::RelyingParty;

#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let xbox = common::xbox_client()?;
    let ticket = xbox.xsts_ticket(RelyingParty::XBOX).await?;
    println!(
        "logged in as {} (xuid {})",
        ticket.gamertag().unwrap_or("<unknown>"),
        ticket
            .xuid()
            .map(|xuid| xuid.to_string())
            .unwrap_or_default()
    );
    Ok(())
}
