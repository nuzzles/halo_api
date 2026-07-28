//! Correlates known death/respawn times with Theater replication frames.

mod common;

use halo_api::clients::hi::film::{
    FilmEventKind, decode_events, decode_frame_record_header, decode_player_indices,
    decode_players, decode_registry, index_packets,
};
use halo_api::clients::hi::models::{FilmChunk, FilmChunkData};

#[tokio::main]
async fn main() -> Result<(), common::ExampleError> {
    let chunks = if let Ok(directory) = std::env::var("HALO_FILM_DIR") {
        local_chunks(&directory)?
    } else {
        let (_, halo) = common::halo_infinite_client()?;
        let match_id = common::value("HALO_MATCH_ID", "Match ID")?;
        let film = halo.match_film(&match_id).await?;
        halo.film_chunks(&film).await?
    };
    let players = decode_players(&chunks);
    let player_indices = decode_player_indices(&chunks, &players);
    let events = decode_events(&chunks, &players);
    let packets = index_packets(&chunks);
    let registry = decode_registry(&chunks).ok_or("Film had no ECS registry")?;
    let origin = packets
        .iter()
        .filter(|packet| packet.timestamp_us > 0)
        .map(|packet| packet.timestamp_us)
        .min()
        .ok_or("Film had no timestamped replication packets")?;
    let frames = packets
        .iter()
        .filter(|packet| packet.packet_type == 0)
        .collect::<Vec<_>>();

    let mut packet_types = std::collections::BTreeMap::<u16, usize>::new();
    let mut first_record_kinds = std::collections::BTreeMap::new();
    let mut first_record_slots = std::collections::BTreeMap::new();
    for packet in &packets {
        *packet_types.entry(packet.packet_type).or_default() += 1;
        if packet.packet_type == 0 {
            let chunk = chunks
                .iter()
                .find(|chunk| chunk.metadata.index == packet.chunk_index)
                .expect("indexed packet has its source chunk");
            let payload =
                &chunk.data[packet.payload_offset..packet.payload_offset + packet.payload_size];
            if let Some(header) = decode_frame_record_header(payload) {
                *first_record_kinds.entry(header.kind).or_insert(0usize) += 1;
                if let Some(slot) = header.slot {
                    *first_record_slots.entry(slot).or_insert(0usize) += 1;
                }
            }
        }
    }
    let average_frame_size =
        frames.iter().map(|frame| frame.payload_size).sum::<usize>() as f64 / frames.len() as f64;

    println!("{} packets, {} frames", packets.len(), frames.len());
    for player in &players {
        println!(
            "player {:<16} xuid {} -> replication index {:?}",
            player.gamertag,
            player.xuid,
            player_indices.get(&player.xuid)
        );
    }
    println!("packet types: {packet_types:?}");
    println!("first FRAME record kinds: {first_record_kinds:?}");
    println!(
        "distinct first-record entity slots: {}",
        first_record_slots.len()
    );
    let mut frequent_slots = first_record_slots.into_iter().collect::<Vec<_>>();
    frequent_slots.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
    println!(
        "most frequent first-record slots: {:?}",
        frequent_slots.into_iter().take(10).collect::<Vec<_>>()
    );
    println!("average frame payload: {average_frame_size:.1} bytes");
    println!("{} ECS archetypes", registry.archetypes.len());
    println!("death -> candidate respawn frame near death + 2 seconds");
    let mut respawn_sizes = Vec::new();
    for death in events
        .iter()
        .filter(|event| event.kind == FilmEventKind::Death)
        .take(25)
    {
        let target_us = origin + u64::from(death.timestamp_ms + 2_000) * 1_000;
        let Some(frame) = frames
            .iter()
            .min_by_key(|frame| frame.timestamp_us.abs_diff(target_us))
        else {
            continue;
        };
        let frame_ms = frame.timestamp_us.saturating_sub(origin) / 1_000;
        respawn_sizes.push(frame.payload_size);
        println!(
            "{:>8.3}s {:<16} -> {:>8.3}s chunk {:>2}, payload {:>6} bytes @ {}",
            death.timestamp_ms as f64 / 1_000.0,
            death.gamertag,
            frame_ms as f64 / 1_000.0,
            frame.chunk_index,
            frame.payload_size,
            frame.payload_offset,
        );
    }
    println!(
        "candidate respawn average payload: {:.1} bytes",
        respawn_sizes.iter().sum::<usize>() as f64 / respawn_sizes.len() as f64
    );
    Ok(())
}

fn local_chunks(directory: &str) -> Result<Vec<FilmChunkData>, common::ExampleError> {
    let mut chunks = Vec::new();
    for entry in std::fs::read_dir(directory)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let Some(stem) = name
            .strip_prefix("chunk-")
            .and_then(|name| name.strip_suffix(".bin"))
        else {
            continue;
        };
        let Some((index, chunk_type)) = stem.split_once("-type-") else {
            continue;
        };
        let index = index.parse::<i32>()?;
        let chunk_type = chunk_type.parse::<i32>()?;
        chunks.push(FilmChunkData {
            metadata: FilmChunk {
                index,
                start_time_offset_ms: i64::from(index.saturating_sub(1)) * 20_000,
                duration_ms: 0,
                size: 0,
                file_relative_path: name,
                chunk_type,
            },
            data: std::fs::read(entry.path())?,
        });
    }
    chunks.sort_by_key(|chunk| chunk.metadata.index);
    Ok(chunks)
}
