//! Chronological v41 packet decoding and grammar state.
use super::*;
pub(super) fn continue_event_views(
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

impl V41RegistryChunkReader {
    fn read(source: FilmChunk, data: Vec<u8>) -> Result<RegistryChunk, ParseError> {
        let definition = registry::parse_registry_chunk(&data)?;
        Ok(RegistryChunk {
            definition,
            source,
            source_position: 0,
            data,
        })
    }
}

struct V41PacketChunk {
    source: FilmChunk,
    source_position: usize,
    data: Vec<u8>,
    packets: Vec<NativeFilmPacket>,
}

enum V41DataChunkReader {
    Replication(V41ReplicationStreamChunkReader),
    Summary(V41SummaryChunkReader),
}

impl V41DataChunkReader {
    fn for_kind(kind: ChunkKind) -> Result<Self, ParseError> {
        match kind {
            ChunkKind::Registry => Err(ParseError::MultipleRegistries),
            ChunkKind::Replication => Ok(Self::Replication(V41ReplicationStreamChunkReader)),
            ChunkKind::Summary => Ok(Self::Summary(V41SummaryChunkReader)),
        }
    }

    fn is_summary(&self) -> bool {
        matches!(self, Self::Summary(_))
    }
}

impl V41ChunkReader {
    pub(crate) fn read(
        self,
        registry_source: FilmChunk,
        registry_data: Vec<u8>,
        inputs: impl IntoIterator<Item = FilmChunk>,
    ) -> Result<Film, ParseError> {
        let registry = V41RegistryChunkReader::read(registry_source, registry_data)?;
        let config = NativeFrameConfig::default();
        // These runtime/build-dependent values are not established by structural
        // registry decoding. Preserve stopped reads instead of choosing a layout.
        let gate15 = None;
        let personalization_bits: Option<usize> = None;
        // This accumulator exists solely to select subsequent native grammars.
        // It is deliberately neither exported nor interpreted as a replay.
        let mut world = FilmWorld::default();
        let mut chunks = Vec::new();
        for (index, input) in inputs.into_iter().enumerate() {
            let reader = V41DataChunkReader::for_kind(input.kind)?;
            let source_position = index + 1;
            let chunk_index = i32::try_from(source_position)
                .map_err(|_| ParseError::Source("too many chunks".into()))?;
            let data = transport::inflate_film_chunk(&input.data).into_owned();
            world.current_chunk = input.index.unwrap_or(source_position as i64);
            let footer = reader.is_summary();
            let mut chunk = V41PacketChunk {
                source: input,
                source_position,
                data,
                packets: Vec::new(),
            };
            let headers = transport::native_packet_bytes(&chunk.data, chunk_index);
            let walk_end = headers
                .last()
                .map(|header| header.payload_offset + header.payload_size)
                .unwrap_or(0);
            if walk_end < chunk.data.len() {
                tracing::warn!(
                    kind = ?chunk.source.kind,
                    source_position,
                    byte_offset = walk_end,
                    remaining_bytes = chunk.data.len() - walk_end,
                    "stopped parsing film chunk before the end"
                );
            }
            for (packet_index, header) in headers.iter().enumerate() {
                let payload =
                    &chunk.data[header.payload_offset..header.payload_offset + header.payload_size];
                let event_list = (!footer && header.packet_type == 0).then(|| {
                    // Every event consumes at least eight header bits.
                    read_native_event_list(payload, 1, gate15, payload.len())
                });
                let roster_read = if !footer && header.packet_type == 8 {
                    personalization_bits.map(|width| {
                        read_native_roster_update(
                            payload,
                            registry.definition.registry.format_version,
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
                        .decode_production_views(
                            payload,
                            2,
                            &registry.definition.registry,
                            &mut world,
                        )
                        .map(|frame| NativeFilmPacketBody::Frame(Box::new(frame)))
                        .unwrap_or_else(|error| NativeFilmPacketBody::Refused {
                            message: error.to_string(),
                        }),
                    (false, 1) => decode_datum_table(payload)
                        .map(NativeFilmPacketBody::Datums)
                        .unwrap_or_else(|error| NativeFilmPacketBody::Refused {
                            message: error.to_string(),
                        }),
                    (false, 2) => {
                        match config.read_keyframe_table(payload, &registry.definition.registry) {
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
                        }
                    }
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
                        &registry.definition.registry,
                        &mut world,
                    )
                });
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
                            let mut read = read_native_weapon_damage(payload, header.timestamp_us);
                            if let Some(damage) = &mut read.read {
                                damage.source = Some(*header);
                                damage.packet_index = Some(packet_index);
                            }
                            read
                        }),
                });
            }
            chunks.push(chunk);
        }

        let mut replication = Vec::new();
        let mut summaries = Vec::new();
        for chunk in chunks {
            match chunk.source.kind {
                ChunkKind::Registry => unreachable!("registry chunks rejected above"),
                ChunkKind::Replication => replication.push(ReplicationStreamChunk {
                    source: chunk.source,
                    source_position: chunk.source_position,
                    data: chunk.data,
                    packets: chunk.packets,
                }),
                ChunkKind::Summary => summaries.push(SummaryChunk {
                    source: chunk.source,
                    source_position: chunk.source_position,
                    data: chunk.data,
                    packets: chunk.packets,
                }),
            }
        }
        Ok(Film {
            registry,
            replication: ReplicationStream {
                chunks: replication,
            },
            summaries: SummaryEvents { chunks: summaries },
        })
    }
}
