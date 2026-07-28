//! Prints the full progression document for one CSR season calendar entry.

mod common;

#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let (_, halo) = common::halo_infinite_client()?;
    let file = common::value(
        "HALO_CSR_SEASON_FILE",
        "CSR season file (for example Csr/Seasons/CsrSeason13-7.json)",
    )?;
    println!("{:#?}", halo.csr_season_file(&file).await?);
    Ok(())
}
