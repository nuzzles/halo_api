//! Resolves a match's Theater film and prints its downloadable chunks.

mod common;

#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let (_, halo) = common::halo_infinite_client()?;
    let match_id = common::value("HALO_MATCH_ID", "Match ID")?;
    let film = halo.match_film(&match_id).await?;

    println!("Film asset: {}", film.asset_id);
    println!("Match ID:   {}", film.custom_data.match_id);
    println!("Version:    {}", film.custom_data.film_major_version);
    println!("Length:     {} ms", film.custom_data.film_length);
    println!("Complete:   {}", film.custom_data.has_game_ended);
    println!("Chunks:     {}", film.custom_data.chunks.len());
    for chunk in &film.custom_data.chunks {
        println!(
            "  {:>3}: type {} · offset {} ms · duration {} ms · {} bytes\n       {}",
            chunk.index,
            chunk.chunk_type,
            chunk.start_time_offset_ms,
            chunk.duration_ms,
            chunk.size,
            film.chunk_url(chunk),
        );
    }
    Ok(())
}
