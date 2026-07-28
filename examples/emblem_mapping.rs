//! Prints the mapping from emblem configurations to Waypoint image assets.

mod common;

#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let (_, halo) = common::halo_infinite_client()?;
    println!("{:#}", halo.emblem_mapping().await?);
    Ok(())
}
