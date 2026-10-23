use halo_api::HcsClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let entries = HcsClient::new().leaderboard().await?;
    let mut csv = csv::WriterBuilder::new()
        .has_headers(false)
        .from_writer(std::io::stdout());
    csv.write_record(["gamertag", "points"])?;
    for entry in entries {
        csv.serialize(entry)?;
    }
    csv.flush()?;
    Ok(())
}
