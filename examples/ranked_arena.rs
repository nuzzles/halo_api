mod common;

#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let (_, halo) = common::halo_infinite_client()?;
    match halo.current_ranked_arena().await? {
        Some(content) => {
            println!("CSR season: {}", content.season.csr_season_file_path);
            for entry in content.map_modes {
                println!(
                    "{} — {} (weight {})",
                    entry.map.asset.public_name, entry.mode.asset.public_name, entry.weight
                );
            }
        }
        None => println!("The CSR calendar contains no seasons."),
    }
    Ok(())
}
