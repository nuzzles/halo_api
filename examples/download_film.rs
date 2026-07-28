//! Downloads and decompresses every Theater chunk into a local directory.

mod common;

use std::fs;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let (_, halo) = common::halo_infinite_client()?;
    let match_id = common::value("HALO_MATCH_ID", "Match ID")?;
    let output = PathBuf::from(common::value("HALO_FILM_DIR", "Output directory")?);
    let film = halo.match_film(&match_id).await?;
    let chunks = halo.film_chunks(&film).await?;

    fs::create_dir_all(&output)?;
    for chunk in chunks {
        let path = output.join(format!(
            "chunk-{:03}-type-{}.bin",
            chunk.metadata.index, chunk.metadata.chunk_type
        ));
        fs::write(&path, &chunk.data)?;
        println!("{}: {} bytes", path.display(), chunk.data.len());
    }
    Ok(())
}
