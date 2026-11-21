//! Ordered replication decoding with guarded parallel chunk work.
use super::registry::ParsedRegistry;
#[cfg(not(target_arch = "wasm32"))]
use crate::theater::film::ChunkKind;
use crate::theater::film::replication::ReplicationObservation;
use crate::theater::film::{EntitySchemaId, EntitySchemaKind, InflatedFilmChunk, Threading};
pub(crate) mod source;

use crate::theater::film::ParseError;
use crate::theater::parser::v41::context::V41DecodeConfig;
use crate::theater::parser::v41::packet::{PacketRead, PacketStream};
use crate::theater::parser::v41::replication::source::{
    KeyframeTable, ReplicationStreamPacket, ReplicationStreamPacketBody,
};
use crate::theater::parser::{
    DeltaRecordSite, FilmParseContext, ReplayTickSource, ReplayTickStatus, VehicleSnapshotSite,
};
use stream::DecodeContext;

pub(crate) mod bindings;
pub(crate) mod components;
mod event;
mod parsed;
pub(crate) mod state;
pub(crate) mod stream;
mod tag;

/// Read replication packet bodies with state retained across ordered chunks.
pub struct V41ReplicationStreamChunkReader;

/// Decoded observations and tick admission retained until the film is assembled.
#[derive(Debug)]
pub(crate) struct ParsedReplicationStream {
    pub(crate) observations: Vec<ReplicationObservation>,
    tick_status: ReplayTickStatus,
}

#[derive(Debug)]
pub(crate) struct ParsedReplicationStreamChunk {
    pub(crate) index: usize,
    pub(crate) start_ms: i64,
    pub(crate) data: Vec<u8>,
    pub(crate) body: PacketStream<ReplicationStreamPacketBody>,
}

impl ParsedReplicationStreamChunk {
    pub(crate) fn payload(&self, packet: &ReplicationStreamPacket) -> Option<&[u8]> {
        packet.payload(&self.data)
    }
}

impl ParsedReplicationStream {
    pub(crate) fn parse(
        registry: &ParsedRegistry,
        inputs: impl IntoIterator<Item = InflatedFilmChunk>,
        parse_context: &FilmParseContext,
        max_packets: usize,
        max_control_records: usize,
        threading: Threading,
    ) -> Result<Self, ParseError> {
        #[cfg(target_arch = "wasm32")]
        let _ = threading;
        for &archetype in parse_context.delta_record_archetypes.values() {
            if archetype >= 50
                || registry
                    .body
                    .registry
                    .archetype(archetype as usize)
                    .is_none()
            {
                return Err(ParseError::InvalidDeltaContextSchema { schema: archetype });
            }
        }
        let mut config = V41DecodeConfig::from_registry(&registry.body);
        components::reader::clear_interned_names();
        config.control_record_limit = max_control_records;
        config.event_kill_tail = parse_context.kill_event_tail;
        let recorded_script_prefix =
            crate::theater::parser::v41::context::events::recorded_script_prefix(&registry.body);
        if recorded_script_prefix
            .zip(parse_context.script_event_prefix)
            .is_some_and(|(recorded, supplied)| recorded != supplied)
        {
            return Err(ParseError::ContradictoryScriptPrefixContext);
        }
        config.event_script_prefix = parse_context
            .script_event_prefix
            .filter(|_| recorded_script_prefix.is_none());
        let mut state = state::ReplicationDecodeState::default();
        let mut context = DecodeContext {
            config: &config,
            registry: &registry.body.registry,
            state: &mut state,
            event_gate15: recorded_script_prefix.or(parse_context.script_event_prefix),
        };
        let inputs = inputs.into_iter().collect::<Vec<_>>();
        if inputs.is_empty() {
            return Err(ParseError::MissingReplication);
        }
        let schema_kinds = registry
            .registry
            .schemas
            .iter()
            .map(|schema| (schema.index, schema.kind))
            .collect::<std::collections::BTreeMap<EntitySchemaId, EntitySchemaKind>>();
        let mut tick_status = ReplayTickStatus::Absent;

        #[cfg(not(target_arch = "wasm32"))]
        if threading == Threading::Multi
            && inputs.len() >= 8
            && !config.guard_rejected_frame_deletes
            && config.schema_recovery.is_none()
        {
            // Shorter streams benefit from a slightly wider decode window;
            // long, dependent streams waste work on stale snapshots.
            let workers = std::thread::available_parallelism().map_or(1, |count| {
                let cores = count.get();
                if inputs.len() <= 64 && cores >= 8 {
                    cores.saturating_add(2).min(12)
                } else {
                    cores.min(8)
                }
            });
            if workers > 1 {
                let mut inputs = inputs.into_iter();
                let first = inputs.next().expect("nonempty replication stream");
                context.state.current_chunk = first.index;
                let first = V41ReplicationStreamChunkReader::read(
                    first,
                    &mut context,
                    parse_context,
                    max_packets,
                    &mut None,
                )?;
                update_tick_status(&mut tick_status, &first, 0);
                let mut projection_pending = vec![first];
                return std::thread::scope(|scope| {
                    let mut projection_handles = Vec::new();
                    let mut projection_parts = Vec::new();
                    let mut remaining = inputs.enumerate().map(|(index, input)| (index + 1, input));
                    let mut pending = std::collections::VecDeque::new();
                    let registry_source = context.registry;
                    let config_ref = &config;
                    let event_gate15 = context.event_gate15;
                    let mut failures = 0usize;
                    let mut sequential_rest = false;
                    for _ in 0..workers {
                        let Some((ordinal, input)) = remaining.next() else {
                            break;
                        };
                        let seed = context.state.clone();
                        pending.push_back((
                            ordinal,
                            scope.spawn(move || {
                                speculative_chunk(
                                    input,
                                    seed,
                                    registry_source,
                                    config_ref,
                                    parse_context,
                                    max_packets,
                                    event_gate15,
                                )
                            }),
                        ));
                    }
                    while let Some((ordinal, handle)) = pending.pop_front() {
                        let (
                            chunk,
                            mut speculative_state,
                            initial_namespace,
                            initial_view,
                            initial_frame_sequence,
                        ) = handle.join().expect("speculative film worker panicked")?;
                        let dependencies = speculative_state
                            .dependencies
                            .take()
                            .expect("tracked chunk");
                        let independent = can_merge_speculative_chunk(
                            context.state,
                            &speculative_state,
                            &dependencies,
                            initial_namespace,
                            initial_view,
                        );
                        let chunk = if independent {
                            for slot in dependencies.writes {
                                if let Some(binding) = speculative_state.slots.remove(&slot) {
                                    context.state.slots.insert(slot, binding);
                                } else {
                                    context.state.slots.remove(&slot);
                                }
                            }
                            context.state.keyframe_namespace = speculative_state.keyframe_namespace;
                            context.state.current_view = speculative_state.current_view;
                            context.state.current_chunk = chunk.index;
                            context.state.frame_sequence +=
                                speculative_state.frame_sequence - initial_frame_sequence;
                            chunk
                        } else {
                            failures += 1;
                            let ParsedReplicationStreamChunk {
                                index,
                                start_ms,
                                data,
                                body,
                            } = chunk;
                            drop(body);
                            context.state.current_chunk = index;
                            V41ReplicationStreamChunkReader::read(
                                InflatedFilmChunk {
                                    kind: ChunkKind::Replication,
                                    index,
                                    start_ms,
                                    data,
                                },
                                &mut context,
                                parse_context,
                                max_packets,
                                &mut None,
                            )?
                        };
                        update_tick_status(&mut tick_status, &chunk, ordinal);
                        projection_pending.push(chunk);
                        if projection_pending.len() == 2 {
                            let group = std::mem::take(&mut projection_pending);
                            let schema_kinds = &schema_kinds;
                            projection_handles.push(
                                scope.spawn(move || parsed::project_chunks(group, schema_kinds)),
                            );
                            if projection_handles.len() == workers {
                                projection_parts.push(
                                    projection_handles
                                        .remove(0)
                                        .join()
                                        .expect("film projection worker panicked"),
                                );
                            }
                        }
                        if failures >= 4 && failures * 2 >= ordinal {
                            sequential_rest = true;
                        }
                        if !sequential_rest && let Some((ordinal, input)) = remaining.next() {
                            let seed = context.state.clone();
                            pending.push_back((
                                ordinal,
                                scope.spawn(move || {
                                    speculative_chunk(
                                        input,
                                        seed,
                                        registry_source,
                                        config_ref,
                                        parse_context,
                                        max_packets,
                                        event_gate15,
                                    )
                                }),
                            ));
                        }
                    }
                    // When speculation is ineffective, retain the established
                    // one-chunk keyframe lookahead on the ordered decoder.
                    let mut current = remaining.next();
                    let mut prepared = current.as_ref().and_then(|(_, input)| {
                        prepare_keyframes(
                            input,
                            context.registry,
                            &config,
                            parse_context,
                            max_packets,
                        )
                    });
                    while let Some((ordinal, input)) = current {
                        let next = remaining.next();
                        let (chunk, next_prepared) = std::thread::scope(|scope| {
                            let worker = next.as_ref().map(|(_, input)| {
                                scope.spawn(|| {
                                    prepare_keyframes(
                                        input,
                                        context.registry,
                                        &config,
                                        parse_context,
                                        max_packets,
                                    )
                                })
                            });
                            context.state.current_chunk = input.index;
                            let read = V41ReplicationStreamChunkReader::read(
                                input,
                                &mut context,
                                parse_context,
                                max_packets,
                                &mut prepared,
                            );
                            let next_prepared = worker
                                .map(|worker| worker.join().expect("keyframe worker panicked"));
                            read.map(|chunk| (chunk, next_prepared))
                        })?;
                        update_tick_status(&mut tick_status, &chunk, ordinal);
                        projection_pending.push(chunk);
                        if projection_pending.len() == 2 {
                            let group = std::mem::take(&mut projection_pending);
                            let schema_kinds = &schema_kinds;
                            projection_handles.push(
                                scope.spawn(move || parsed::project_chunks(group, schema_kinds)),
                            );
                            if projection_handles.len() == workers {
                                projection_parts.push(
                                    projection_handles
                                        .remove(0)
                                        .join()
                                        .expect("film projection worker panicked"),
                                );
                            }
                        }
                        current = next;
                        prepared = next_prepared.unwrap_or(None);
                    }
                    if !projection_pending.is_empty() {
                        let schema_kinds = &schema_kinds;
                        projection_handles.push(scope.spawn(move || {
                            parsed::project_chunks(projection_pending, schema_kinds)
                        }));
                    }
                    projection_parts.extend(
                        projection_handles
                            .into_iter()
                            .map(|handle| handle.join().expect("film projection worker panicked")),
                    );
                    let mut observations =
                        Vec::with_capacity(projection_parts.iter().map(Vec::len).sum());
                    for part in &mut projection_parts {
                        observations.append(part);
                    }
                    Ok(Self {
                        observations,
                        tick_status,
                    })
                });
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        if threading == Threading::Multi && inputs.len() >= 2 {
            let workers =
                std::thread::available_parallelism().map_or(1, |count| count.get().min(8));
            if workers > 1 {
                let group_size = inputs.len().div_ceil(workers);
                return std::thread::scope(|scope| {
                    let mut pending = Vec::with_capacity(group_size);
                    let mut handles = Vec::with_capacity(workers);
                    let mut inputs = inputs.into_iter();
                    let mut current = inputs.next().expect("nonempty replication stream");
                    let mut prepared = prepare_keyframes(
                        &current,
                        context.registry,
                        &config,
                        parse_context,
                        max_packets,
                    );
                    let mut chunk_index = 0;
                    loop {
                        let next = inputs.next();
                        let (chunk, next_prepared) = std::thread::scope(|scope| {
                            let worker = next.as_ref().map(|input| {
                                scope.spawn(|| {
                                    prepare_keyframes(
                                        input,
                                        context.registry,
                                        &config,
                                        parse_context,
                                        max_packets,
                                    )
                                })
                            });
                            context.state.current_chunk = current.index;
                            let read = V41ReplicationStreamChunkReader::read(
                                current,
                                &mut context,
                                parse_context,
                                max_packets,
                                &mut prepared,
                            );
                            let next_prepared = worker
                                .map(|worker| worker.join().expect("keyframe worker panicked"));
                            read.map(|chunk| (chunk, next_prepared))
                        })?;
                        update_tick_status(&mut tick_status, &chunk, chunk_index);
                        pending.push(chunk);
                        if pending.len() == group_size {
                            let group =
                                std::mem::replace(&mut pending, Vec::with_capacity(group_size));
                            let schema_kinds = &schema_kinds;
                            handles.push(
                                scope.spawn(move || parsed::project_chunks(group, schema_kinds)),
                            );
                        }
                        let Some(input) = next else { break };
                        current = input;
                        prepared = next_prepared.expect("next chunk has prepared keyframes");
                        chunk_index += 1;
                    }
                    if !pending.is_empty() {
                        let schema_kinds = &schema_kinds;
                        handles.push(
                            scope.spawn(move || parsed::project_chunks(pending, schema_kinds)),
                        );
                    }
                    let mut parts = handles
                        .into_iter()
                        .map(|handle| handle.join().expect("film projection worker panicked"))
                        .collect::<Vec<_>>();
                    let mut observations = Vec::with_capacity(parts.iter().map(Vec::len).sum());
                    for part in &mut parts {
                        observations.append(part);
                    }
                    Ok(Self {
                        observations,
                        tick_status,
                    })
                });
            }
        }

        let mut chunks = Vec::with_capacity(inputs.len());
        for (chunk_index, input) in inputs.into_iter().enumerate() {
            context.state.current_chunk = input.index;
            let chunk = V41ReplicationStreamChunkReader::read(
                input,
                &mut context,
                parse_context,
                max_packets,
                &mut None,
            )?;
            update_tick_status(&mut tick_status, &chunk, chunk_index);
            chunks.push(chunk);
        }
        Ok(Self {
            observations: parsed::project_chunks(chunks, &schema_kinds),
            tick_status,
        })
    }

    /// Source-backed tick admission, evaluated before dropping packet bytes.
    pub(crate) fn replay_tick_status(&self) -> ReplayTickStatus {
        self.tick_status
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn can_merge_speculative_chunk(
    ordered: &state::ReplicationDecodeState,
    speculative: &state::ReplicationDecodeState,
    dependencies: &state::ChunkDependencies,
    initial_namespace: Option<u8>,
    initial_view: i8,
) -> bool {
    initial_namespace == ordered.keyframe_namespace
        && initial_view == ordered.current_view
        && ordered.pending_frame_deletes.is_none()
        && ordered.deferred_frame_deletes.is_empty()
        && ordered.schema_recoveries.is_empty()
        && ordered.anticipations.is_empty()
        && dependencies
            .reads
            .iter()
            .all(|(slot, expected)| ordered.slots.get(slot) == expected.as_ref())
        && speculative.schema_recoveries.is_empty()
        && speculative.pending_frame_deletes.is_none()
        && speculative.deferred_frame_deletes.is_empty()
        && speculative.anticipations.is_empty()
}

fn update_tick_status(
    status: &mut ReplayTickStatus,
    chunk: &ParsedReplicationStreamChunk,
    chunk_index: usize,
) {
    if matches!(status, ReplayTickStatus::Found(_)) {
        return;
    }
    let mut incomplete =
        matches!(status, ReplayTickStatus::Undetermined) || !chunk.body.opaque.is_empty();
    for (packet_index, packet) in chunk.body.packets.iter().enumerate() {
        let Some(payload) = chunk.payload(packet) else {
            incomplete = true;
            continue;
        };
        if packet.header.packet_type == 0
            && let Some((payload_bit, _)) = crate::theater::parser::ticks::tick_counter(payload)
        {
            *status = ReplayTickStatus::Found(ReplayTickSource {
                chunk: chunk_index,
                packet: packet_index,
                payload_bit,
            });
            return;
        }
    }
    *status = if incomplete {
        ReplayTickStatus::Undetermined
    } else {
        ReplayTickStatus::Absent
    };
}

#[cfg(not(target_arch = "wasm32"))]
fn speculative_chunk(
    input: InflatedFilmChunk,
    mut state: state::ReplicationDecodeState,
    registry: &super::registry::source::FilmRegistry,
    config: &V41DecodeConfig,
    parse_context: &FilmParseContext,
    max_packets: usize,
    event_gate15: Option<bool>,
) -> Result<
    (
        ParsedReplicationStreamChunk,
        state::ReplicationDecodeState,
        Option<u8>,
        i8,
        u64,
    ),
    ParseError,
> {
    let initial_namespace = state.keyframe_namespace;
    let initial_view = state.current_view;
    let initial_frame_sequence = state.frame_sequence;
    state.current_chunk = input.index;
    state.track_chunk();
    let mut context = DecodeContext {
        config,
        registry,
        state: &mut state,
        event_gate15,
    };
    let chunk = V41ReplicationStreamChunkReader::read(
        input,
        &mut context,
        parse_context,
        max_packets,
        &mut None,
    )?;
    Ok((
        chunk,
        state,
        initial_namespace,
        initial_view,
        initial_frame_sequence,
    ))
}

#[cfg(not(target_arch = "wasm32"))]
fn prepare_keyframes(
    input: &InflatedFilmChunk,
    registry: &crate::theater::parser::v41::registry::source::FilmRegistry,
    config: &V41DecodeConfig,
    parse_context: &FilmParseContext,
    max_packets: usize,
) -> Option<(usize, KeyframeTable)> {
    let (packet_index, payload) = first_keyframe_packet(&input.data, max_packets)?;
    let vehicle_gate = |id| {
        parse_context
            .vehicle_snapshot_gates
            .get(&VehicleSnapshotSite {
                chunk_index: input.index,
                packet_index,
                id,
            })
            .copied()
    };
    let table = stream::keyframes::predecode(&input.data[payload], registry, config, vehicle_gate);
    Some((packet_index, table))
}

#[cfg(not(target_arch = "wasm32"))]
fn first_keyframe_packet(
    data: &[u8],
    max_packets: usize,
) -> Option<(usize, std::ops::Range<usize>)> {
    let mut offset = 0usize;
    let mut packet_index = 0usize;
    while packet_index < max_packets && offset < data.len() {
        let header = data.get(offset..offset.saturating_add(16))?;
        let size = u32::from_le_bytes(header[4..8].try_into().unwrap()) as usize;
        let start = offset + 16;
        let end = start.checked_add(size).filter(|&end| end <= data.len())?;
        if u16::from_le_bytes(header[..2].try_into().unwrap()) == 2 {
            return Some((packet_index, start..end));
        }
        offset = end;
        packet_index += 1;
    }
    None
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod prefetch_tests {
    use super::{
        can_merge_speculative_chunk, first_keyframe_packet, state::ReplicationDecodeState,
    };

    fn packet(kind: u16, payload: &[u8]) -> Vec<u8> {
        let mut bytes = vec![0; 16];
        bytes[..2].copy_from_slice(&kind.to_le_bytes());
        bytes[4..8].copy_from_slice(&(payload.len() as u32).to_le_bytes());
        bytes.extend_from_slice(payload);
        bytes
    }

    #[test]
    fn lookahead_uses_first_complete_keyframe_inside_packet_limit() {
        let mut data = packet(0, &[1, 2]);
        let start = data.len() + 16;
        data.extend(packet(2, &[3, 4, 5]));
        data.extend(packet(2, &[6]));
        assert_eq!(first_keyframe_packet(&data, 3), Some((1, start..start + 3)));
        assert_eq!(first_keyframe_packet(&data, 1), None);
        data[2 + 4..2 + 8].copy_from_slice(&u32::MAX.to_le_bytes());
        assert_eq!(first_keyframe_packet(&data, 3), None);
    }

    #[test]
    fn speculative_chunks_require_unchanged_inherited_bindings() {
        let mut seed = ReplicationDecodeState::default();
        seed.bind_full(7, 9);
        let mut worker = seed.clone();
        worker.track_chunk();
        assert_eq!(worker.archetype(7), Some(9));
        assert_eq!(worker.archetype(100), None);
        worker.bind_full(8, 35);
        assert_eq!(worker.archetype(8), Some(35));
        let dependencies = worker.dependencies.take().unwrap();
        assert_eq!(dependencies.reads.len(), 2);
        assert!(dependencies.writes.contains(&8));
        assert!(can_merge_speculative_chunk(
            &seed,
            &worker,
            &dependencies,
            None,
            0
        ));

        let mut changed = seed.clone();
        changed.bind_full(7, 40);
        assert!(!can_merge_speculative_chunk(
            &changed,
            &worker,
            &dependencies,
            None,
            0
        ));
        changed = seed.clone();
        changed.bind_full(100, 9);
        assert!(!can_merge_speculative_chunk(
            &changed,
            &worker,
            &dependencies,
            None,
            0
        ));
        changed = seed;
        changed.current_view = 1;
        assert!(!can_merge_speculative_chunk(
            &changed,
            &worker,
            &dependencies,
            None,
            0
        ));
    }
}

impl V41ReplicationStreamChunkReader {
    pub(crate) fn read(
        input: InflatedFilmChunk,
        context: &mut DecodeContext<'_>,
        parse_context: &FilmParseContext,
        max_packets: usize,
        prepared_keyframes: &mut Option<(usize, KeyframeTable)>,
    ) -> Result<ParsedReplicationStreamChunk, crate::theater::film::ParseError> {
        let chunk_index = input.index;
        let body = input.read_packets(max_packets, |packet_index, header, payload| {
            if header.packet_type == 2
                && prepared_keyframes
                    .as_ref()
                    .is_some_and(|(index, _)| *index == packet_index)
            {
                let (_, table) = prepared_keyframes.take().unwrap();
                return PacketRead::Decoded(stream::keyframes::publish_table(table, context));
            }
            let vehicle_gate = |id| {
                parse_context
                    .vehicle_snapshot_gates
                    .get(&VehicleSnapshotSite {
                        chunk_index,
                        packet_index,
                        id,
                    })
                    .copied()
            };
            let delta_schema = |start_bit, id| {
                parse_context
                    .delta_record_archetypes
                    .get(&DeltaRecordSite {
                        chunk_index,
                        packet_index,
                        start_bit,
                        id,
                    })
                    .copied()
            };
            match stream::decode_with_context(
                header.packet_type,
                payload,
                context,
                vehicle_gate,
                delta_schema,
            ) {
                Ok(body) => PacketRead::Decoded(body),
                Err(reason) => PacketRead::Opaque { reason },
            }
        });
        Ok(ParsedReplicationStreamChunk {
            index: input.index,
            start_ms: input.start_ms,
            data: input.data,
            body: body?,
        })
    }
}
