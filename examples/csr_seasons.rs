//! Lists every CSR season and marks seasons active at the current UTC time.

mod common;

#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let (_, halo) = common::halo_infinite_client()?;
    let calendar = halo.csr_season_calendar().await?;
    let now = chrono::Utc::now();

    println!("Current UTC time: {now}");
    for season in calendar.seasons {
        let active = season.start_date.iso8601_date <= now && now < season.end_date.iso8601_date;
        println!(
            "{}: {} to {}{}",
            season.csr_season_file_path,
            season.start_date.iso8601_date,
            season.end_date.iso8601_date,
            if active { " [ACTIVE]" } else { "" }
        );
    }
    Ok(())
}
