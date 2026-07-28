mod common;

#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let (_, halo) = common::halo_infinite_client()?;
    match halo.current_csr_season().await? {
        Some(season) => println!("{season:#?}"),
        None => println!("The CSR calendar contains no currently active season."),
    }
    Ok(())
}
