//! Loaded native movement scan with publication-time entity binding evidence.
use super::*;
use std::sync::{Arc, Mutex};

#[derive(Debug, thiserror::Error)]
pub enum ContextMovementError {
    #[error("aucun chunk de donnees dans le film")]
    NoChunks,
    #[error("archétype biped 35 absent du registre")]
    NoArchetype,
    #[error(transparent)]
    Registry(#[from] NativeContextRegistryError),
    #[error(transparent)]
    Profile(#[from] NativeProfileResolveError),
    #[error(transparent)]
    Reader(#[from] NativeReaderProfileError),
}
#[derive(Debug, Clone, PartialEq)]
pub struct ContextMovementFrame {
    pub chunk: i64,
    pub source: FilmPacket,
    pub packet_index: usize,
    pub frame: InferenceViews,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextMovementObservation {
    pub chunk: i64,
    pub source: FilmPacket,
    pub packet_index: usize,
    /// The archetype bound to the hook's capture slot at publication time.
    pub archetype: Option<u32>,
    pub value: FilmComponentObservation,
}
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ContextMovementScan {
    /// Native nil on early absence/error; allocated, possibly empty after a scan.
    pub reads: Option<Vec<MovementStateRead>>,
    pub stats: MovementStateStats,
    /// Includes rejected/unbound and non-state hooks before deduplication.
    pub observations: Vec<ContextMovementObservation>,
    pub frames: Vec<ContextMovementFrame>,
}
#[derive(Default)]
struct Collector {
    scanner: movement_states::Scanner,
    packet: Option<(i64, FilmPacket, usize)>,
    observations: Vec<ContextMovementObservation>,
}
/// Native ScanMovementStates over loaded numbered chunks. Caller-installed map
/// widths are retained. A private observer receives only movement publications;
/// all attempted frames and rejected publications remain separate from reads.
pub fn scan_context_movement_states(
    context: &NativeFilmContext<'_>,
) -> (ContextMovementScan, Option<ContextMovementError>) {
    let mut out = ContextMovementScan::default();
    let collector = Arc::new(Mutex::new(Collector::default()));
    let error = (|| -> Result<(), ContextMovementError> {
        collector.lock().unwrap().scanner.stats.map_widths =
            context.scan_profile()?.movement.world_object.axis_bits;
        let chunks = context.chunk_numbers();
        if chunks.is_empty() {
            return Err(ContextMovementError::NoChunks);
        }
        let registry = &context.registry().map_err(|e| *e)?.registry;
        let arch = registry
            .archetype(35)
            .ok_or(ContextMovementError::NoArchetype)?;
        if [
            "unit-crouch",
            "biped-slide",
            "biped-mobility-action",
            "biped-spartan-ability",
        ]
        .iter()
        .all(|name| ability_states::component_index(arch, name).is_none())
        {
            let mut c = collector.lock().unwrap();
            c.scanner.stats.absent = true;
            c.scanner.stats.scanned = true;
            return Ok(());
        }
        let mut cfg = context.scan_frame()?;
        let observer = NativeFilmObserver::default();
        let binding = observer.movement_binding_context();
        let captured = collector.clone();
        observer.set_hook(
            NativeHookKind::MovementState,
            Some(Arc::new(move |p| {
                let NativeHookPublication::Component(
                    value @ FilmComponentObservation::MovementState { slot, .. },
                ) = p
                else {
                    return;
                };
                let mut c = captured.lock().unwrap();
                let (chunk, source, packet_index) =
                    c.packet.expect("packet set before frame decoding");
                let archetype = binding.archetype(*slot);
                c.scanner
                    .receive(value, archetype, source.timestamp_us, chunk, packet_index);
                c.observations.push(ContextMovementObservation {
                    chunk,
                    source,
                    packet_index,
                    archetype,
                    value: value.clone(),
                });
            })),
        );
        cfg.context.observer = Some(observer);
        let mut world = FilmWorld {
            anticipated: Some(build_source_anticipated_bindings(context.source())),
            ..Default::default()
        };
        for &chunk in chunks {
            let Some((data, packets)) = context.chunk_at(chunk) else {
                continue;
            };
            world.current_chunk = chunk;
            for packet in packets.iter().filter(|p| p.packet_type == 2) {
                let payload =
                    &data[packet.payload_offset..packet.payload_offset + packet.payload_size];
                for a in recover_keyframe_anchors(payload) {
                    world.bind_keyframe(a.id >> 30, a.id & 0x3fffffff, a.archetype);
                }
            }
            for packet in packets.iter().filter(|p| p.packet_type == 2) {
                let payload =
                    &data[packet.payload_offset..packet.payload_offset + packet.payload_size];
                let datums = recover_keyframe_datums(payload);
                let mut c = collector.lock().unwrap();
                c.scanner.stats.datum_bindings += datums.bind_missing(&mut world);
                c.scanner.stats.datum_ambiguous += datums.ambiguous;
            }
            for (packet_index, source) in packets.iter().copied().enumerate() {
                if source.packet_type != 0 || source.payload_size == 0 {
                    continue;
                }
                let payload =
                    &data[source.payload_offset..source.payload_offset + source.payload_size];
                let start = if decode_packet_head_event(payload).is_some() {
                    collector.lock().unwrap().scanner.stats.event_packets += 1;
                    let Some(start) = cfg.locate_strict_entity_view(payload, registry, &world)?
                    else {
                        collector
                            .lock()
                            .unwrap()
                            .scanner
                            .stats
                            .event_packets_unlocated += 1;
                        continue;
                    };
                    collector
                        .lock()
                        .unwrap()
                        .scanner
                        .stats
                        .event_packets_located += 1;
                    start as i64
                } else {
                    2
                };
                {
                    let mut c = collector.lock().unwrap();
                    c.scanner.stats.packets += 1;
                    c.packet = Some((chunk, source, packet_index));
                }
                let frame = cfg.decode_inference_views(payload, start, 3, registry, &mut world)?;
                {
                    let mut c = collector.lock().unwrap();
                    for record in frame
                        .entities
                        .iter()
                        .flat_map(|v| &v.records)
                        .filter(|r| r.archetype == Some(35))
                    {
                        c.scanner.stats.records += 1;
                        c.scanner.stats.desyncs += usize::from(
                            record
                                .decoded
                                .as_ref()
                                .is_some_and(|r| r.stop != EntityViewStop::Complete),
                        );
                    }
                }
                out.frames.push(ContextMovementFrame {
                    chunk,
                    source,
                    packet_index,
                    frame,
                });
            }
        }
        let mut c = collector.lock().unwrap();
        c.scanner.derive_jumps();
        c.scanner.stats.scanned = true;
        out.reads = Some(
            std::mem::take(&mut c.scanner.readings)
                .into_values()
                .collect(),
        );
        Ok(())
    })()
    .err();
    let mut c = collector.lock().unwrap();
    out.stats = std::mem::take(&mut c.scanner.stats);
    out.observations = std::mem::take(&mut c.observations);
    (out, error)
}
