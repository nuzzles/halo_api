//! Structural v41 decoding. No candidate searches or inferred runtime settings.
use super::*;
use crate::theater::film::*;

fn continue_event_views(
    payload: &[u8],
    events: &NativeEventListRead,
    config: &NativeFrameConfig,
    registry: &FilmRegistry,
    world: &mut FilmWorld,
) -> Option<NativeEventContinuation> {
    // Empty lists are already handled by the generic production reader. Never
    // scan for a replacement boundary after an unsupported or truncated body.
    if events.stop != NativeEventListStop::Terminator || events.records.is_empty() {
        return None;
    }
    let state_policy = if events.records.iter().any(|r| r.code == Some(0)) {
        NativeContinuationStatePolicy::IsolatedConflictingDamageGrammar
    } else {
        NativeContinuationStatePolicy::Applied
    };
    let frame = if state_policy == NativeContinuationStatePolicy::Applied {
        config.decode_production_views(payload, events.end_bit, registry, world)
    } else {
        config.decode_production_views(payload, events.end_bit, registry, &mut world.clone())
    };
    Some(NativeEventContinuation {
        start_bit: events.end_bit,
        frame: frame.map_err(|error| error.to_string()),
        state_policy,
    })
}

/// Version-specific chunk decoder selected by `Film::parse` after registry validation.
/// Kept internal so callers use the single version-dispatching entry point.
pub(crate) struct V41ChunkParser;

impl V41ChunkParser {
    pub(crate) fn parse(
        inputs: Vec<FilmChunk>,
        source: &FilmSource,
        registry: FilmRegistryRead,
        registry_chunk_position: usize,
    ) -> Result<Film, ParseError> {
        let config = NativeFrameConfig::default();
        // These runtime/build-dependent values are not established by structural
        // registry decoding. Preserve stopped reads instead of choosing a layout.
        let gate15 = None;
        let personalization_bits: Option<usize> = None;
        // This accumulator exists solely to select subsequent native grammars.
        // It is deliberately neither exported nor interpreted as a replay.
        let mut world = FilmWorld::default();
        let mut chunks = Vec::with_capacity(source.num_chunks());
        for (source_position, input) in inputs.into_iter().enumerate() {
            let data = source.chunk(source_position).unwrap();
            world.current_chunk = source
                .metadata()
                .get(source_position)
                .map_or(source_position as i64, |m| m.index);
            let mut chunk = ParsedChunk {
                source: input,
                source_position,
                data: data.to_vec(),
                packets: Vec::new(),
                packet_walk_end_byte: 0,
            };
            if source_position != registry_chunk_position
                && source
                    .metadata()
                    .get(source_position)
                    .is_none_or(|m| matches!(m.chunk_type, 2 | 3))
            {
                for (packet_index, header) in source.packets(source_position).iter().enumerate() {
                    let payload = source.payload(header).expect("indexed source payload");

                    let footer = source
                        .metadata()
                        .get(source_position)
                        .is_some_and(|m| m.chunk_type == 3);
                    let event_list = (!footer && header.packet_type == 0).then(|| {
                        // Every event consumes at least eight header bits.
                        read_native_event_list(payload, 1, gate15, payload.len())
                    });
                    let roster_read = if !footer && header.packet_type == 8 {
                        personalization_bits.map(|width| {
                            read_native_roster_update(
                                payload,
                                registry.registry.format_version,
                                width,
                            )
                        })
                    } else {
                        None
                    };
                    let body = match (footer, header.packet_type) {
                        (true, 9) => summary::read_summary_packet(
                            payload,
                            header.chunk_index,
                            header.payload_offset,
                        )
                        .map(|(declared_events, events)| NativeFilmPacketBody::Summary {
                            declared_events,
                            events,
                        })
                        .unwrap_or_else(|| NativeFilmPacketBody::Refused {
                            message: "truncated summary count".into(),
                        }),
                        (true, _) => NativeFilmPacketBody::Opaque,
                        (false, 0) => config
                            .decode_production_views(payload, 2, &registry.registry, &mut world)
                            .map(|frame| NativeFilmPacketBody::Frame(Box::new(frame)))
                            .unwrap_or_else(|error| NativeFilmPacketBody::Refused {
                                message: error.to_string(),
                            }),
                        (false, 1) => decode_datum_table(payload)
                            .map(NativeFilmPacketBody::Datums)
                            .unwrap_or_else(|error| NativeFilmPacketBody::Refused {
                                message: error.to_string(),
                            }),
                        (false, 2) => match config.read_keyframe_table(payload, &registry.registry)
                        {
                            Ok(table) => {
                                for attempt in &table.records {
                                    if attempt.record.is_some() {
                                        world.bind_keyframe(
                                            attempt.id >> 30,
                                            attempt.id & 0x3fff_ffff,
                                            attempt.archetype,
                                        );
                                    }
                                }
                                NativeFilmPacketBody::Keyframes(table)
                            }
                            Err(error) => NativeFilmPacketBody::Refused {
                                message: error.to_string(),
                            },
                        },
                        (false, 8) => roster_read
                            .as_ref()
                            .map(|read| NativeFilmPacketBody::Roster(read.roster.clone()))
                            .unwrap_or_else(|| NativeFilmPacketBody::Refused {
                                message: "unknown build personalization width".into(),
                            }),
                        _ => NativeFilmPacketBody::Opaque,
                    };
                    let event_continuation = event_list.as_ref().and_then(|events| {
                        continue_event_views(
                            payload,
                            events,
                            &config,
                            &registry.registry,
                            &mut world,
                        )
                    });
                    chunk.packet_walk_end_byte = header.payload_offset + header.payload_size;
                    chunk.packets.push(NativeFilmPacket {
                        header: *header,
                        roster_read,
                        body,
                        event_head: (!footer && header.packet_type == 0)
                            .then(|| decode_packet_head_event(payload))
                            .flatten(),
                        event_list,
                        event_continuation,
                        native_head: (!footer)
                            .then(|| read_native_packet_head(header.packet_type, payload, None))
                            .flatten(),
                        damage_read: (!footer
                            && header.packet_type == 0
                            && payload.len() >= 2
                            && payload[0] == 0xc0)
                            .then(|| {
                                let mut read =
                                    read_native_weapon_damage(payload, header.timestamp_us);
                                if let Some(damage) = &mut read.read {
                                    damage.source = Some(*header);
                                    damage.packet_index = Some(packet_index);
                                }
                                read
                            }),
                    });
                }
            }
            chunks.push(chunk);
        }

        let mut registry_chunk = None;
        let mut replication = Vec::new();
        let mut summaries = Vec::new();
        for chunk in chunks {
            match chunk.source.kind {
                ChunkKind::Registry => registry_chunk = Some(chunk),
                ChunkKind::Replication => replication.push(chunk),
                ChunkKind::Summary => summaries.push(chunk),
            }
        }
        Ok(Film {
            registry: Registry {
                definition: registry,
                chunk: registry_chunk.expect("validated registry"),
            },
            replication: ReplicationStream {
                chunks: replication,
            },
            summaries: SummaryEvents { chunks: summaries },
        })
    }
}

#[cfg(test)]
pub(crate) fn test_chunks(source: &FilmSource) -> Vec<FilmChunk> {
    source
        .original_chunks()
        .iter()
        .enumerate()
        .map(|(i, bytes)| {
            let metadata = source.metadata().get(i);
            FilmChunk {
                kind: match metadata
                    .map(|m| m.chunk_type)
                    .unwrap_or(if i == 0 { 1 } else { 2 })
                {
                    1 => ChunkKind::Registry,
                    3 => ChunkKind::Summary,
                    _ => ChunkKind::Replication,
                },
                index: metadata.map(|m| m.index),
                start_ms: metadata.map(|m| m.start_ms),
                data: bytes.clone(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests;
