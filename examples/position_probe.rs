//! Correlates known death/respawn times with Theater replication frames.

mod common;

use halo_api::clients::hi::film::{
    FilmEventKind, decode_events, decode_first_delta_header, decode_first_position_update,
    decode_frame_record_header, decode_keyframe_positions, decode_player_indices, decode_players,
    decode_registry, index_packets,
};
use halo_api::clients::hi::models::{FilmChunk, FilmChunkData};
use std::io::Write;

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
    let positions = decode_keyframe_positions(&chunks);
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
    let mut position_deltas = std::collections::BTreeMap::<u32, usize>::new();
    let mut position_encodings = std::collections::BTreeMap::new();
    let mut position_samples = std::collections::BTreeMap::new();
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
            if let Some(delta) = decode_first_delta_header(payload)
                && delta.component_mask & 1 != 0
            {
                *position_deltas
                    .entry(delta.record.slot.expect("delta has an entity slot"))
                    .or_default() += 1;
            }
            if let Some(update) = decode_first_position_update(payload, 14) {
                *position_encodings
                    .entry((update.slot, update.encoding))
                    .or_insert(0usize) += 1;
                position_samples
                    .entry((update.slot, update.encoding))
                    .or_insert(update.vector);
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
    println!("first-record position updates: {position_deltas:?}");
    println!("decoded position encodings: {position_encodings:?}");
    println!("first decoded vectors: {position_samples:?}");
    if let Ok(path) = std::env::var("HALO_POSITION_CSV") {
        write_position_csv(&path, &chunks, &packets, origin)?;
    }
    println!("average frame payload: {average_frame_size:.1} bytes");
    println!("{} ECS archetypes", registry.archetypes.len());
    for archetype in registry.biped_archetypes() {
        println!(
            "  biped archetype {}: {} ordered components",
            archetype.index,
            archetype.components.len()
        );
    }
    println!(
        "{} unattributed absolute keyframe positions",
        positions.len()
    );
    for position in &positions {
        println!(
            "  {:>8.3}s chunk {:>2}: ({:>8.3}, {:>8.3}, {:>8.3})",
            position.timestamp_ms as f64 / 1_000.0,
            position.chunk_index,
            position.x,
            position.y,
            position.z,
        );
    }
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

fn write_position_csv(
    path: &str,
    chunks: &[FilmChunkData],
    packets: &[halo_api::clients::hi::film::FilmPacket],
    origin: u64,
) -> Result<(), common::ExampleError> {
    let mut file = std::io::BufWriter::new(std::fs::File::create(path)?);
    writeln!(
        file,
        "time_ms,entity_slot,candidate_replication_index,gamertag,x,y,z,encoding"
    )?;
    let mut positions = std::collections::BTreeMap::<u32, [f32; 3]>::new();
    let mut rows = 0usize;
    for packet in packets.iter().filter(|packet| packet.packet_type == 0) {
        let chunk = chunks
            .iter()
            .find(|chunk| chunk.metadata.index == packet.chunk_index)
            .expect("indexed packet has its source chunk");
        let payload =
            &chunk.data[packet.payload_offset..packet.payload_offset + packet.payload_size];
        let Some(update) = decode_first_position_update(payload, 14) else {
            continue;
        };
        if !(512..=519).contains(&update.slot) {
            continue;
        }
        let Some(vector) = update.vector else {
            continue;
        };
        let position = match update.encoding {
            halo_api::clients::hi::film::FilmPositionEncoding::Delta8
            | halo_api::clients::hi::film::FilmPositionEncoding::DeltaAxis => {
                let Some(previous) = positions.get(&update.slot) else {
                    continue;
                };
                [
                    previous[0] + vector[0],
                    previous[1] + vector[1],
                    previous[2] + vector[2],
                ]
            }
            halo_api::clients::hi::film::FilmPositionEncoding::Raw
            | halo_api::clients::hi::film::FilmPositionEncoding::Absolute
            | halo_api::clients::hi::film::FilmPositionEncoding::AbsoluteFallback => vector,
            halo_api::clients::hi::film::FilmPositionEncoding::Unchanged => continue,
        };
        if !position
            .iter()
            .all(|value| value.is_finite() && value.abs() < 200.0)
        {
            continue;
        }
        positions.insert(update.slot, position);
        let time_ms = packet.timestamp_us.saturating_sub(origin) / 1_000;
        writeln!(
            file,
            "{time_ms},{},{},,{:.6},{:.6},{:.6},{:?}",
            update.slot,
            update.slot - 512,
            position[0],
            position[1],
            position[2],
            update.encoding
        )?;
        rows += 1;
    }
    file.flush()?;
    println!("wrote {rows} experimental position rows to {path}");
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
