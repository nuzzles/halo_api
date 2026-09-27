//! Recording-only entry. Parsing stops are data, not discarded packets.
pub use super::parser::anticipated_bindings::AnticipatedDeclaration;
pub use super::parser::bootstrap::FilmIdentity;
pub use super::parser::bot_metadata::{FilmBotEntry, NativeBotCandidate, NativeBotMetadataRead};
pub use super::parser::chain_inference::ChainInferenceOutcome;
pub use super::parser::components::{
    BindingOrigin, ComponentField, DecodedFrameView, EntityComponentAttempt, EntityComponentSpan,
    EntityRecord, EntityViewStop, FrameViewStop, KeyframeChainAttempt, KeyframeChainStop,
    KeyframeComponentSpan, KeyframeRecord, KeyframeStop, NativeActionBlock, NativeControlEntry,
    NativeKeyframeTable,
};
pub use super::parser::datums::{DatumEntry, DatumTable};
pub use super::parser::event_heads::{
    DecodedHeadEvent, EventReference, EventReferenceValue, HeadEventPayload, HeadEventStop,
};
pub use super::parser::fire_events::{
    FilmFireEvent, FireUnitReference, NativeFireAimAttempt, NativeFireAimMethod, NativeFireAimStop,
    NativeFireField, NativeFireHeaderStop, NativeFireRead,
};
pub use super::parser::highlight_events::{
    NativeHighlightEvent, NativeHighlightIdentityRead, NativeHighlightScan, NativeHighlightTailRead,
};
pub use super::parser::kill_event_chain::{
    KillEventFields, NativeEventField, NativeEventFieldStage, NativeEventFieldValue,
    NativeEventListRead, NativeEventListStop, NativeEventRecord,
};
pub use super::parser::medals::MedalAward;
pub use super::parser::native_context::NativeContextRegistryError;
pub use super::parser::native_event_gate::{NativeEventGate15Policy, NativeEventGate15Selection};
pub use super::parser::native_identity::{
    NativeIdentityField, NativeIdentityRead, NativeIdentityValue,
};
pub use super::parser::native_march::NativeFrameMetadata;
pub use super::parser::native_packet_heads::NativePacketHeadRead;
pub use super::parser::native_pickups::{NativePickupOutcome, NativePickupRead};
pub use super::parser::native_profile::{
    NativeMovementProfile, NativePrecisionDescriptor, NativeProfileResolveError,
};
pub use super::parser::native_scan_profile::{
    NativeKeyframeLayout, NativeScanGrammar, NativeScanProfile, NativeSharedWidths,
};
pub use super::parser::native_weapon_damage::{NativeWeaponDamageField, NativeWeaponDamageRead};
pub use super::parser::native_zoom::NativeZoomRead;
pub use super::parser::objective_extract::ObjectiveFooterEvent;
pub use super::parser::player_table::{
    NativePlayerSlotRead, NativeSlotField, NativeSlotValue, PlayerTable, PlayerTableError,
    PlayerTableReport, PlayerTableShorts, PlayerTableSlot,
};
pub use super::parser::position_capture::NativePositionKind;
pub use super::parser::production_frame::{
    ProductionAdmissionDiagnostics, ProductionEntityEnd, ProductionFrame,
};
pub use super::parser::profile::FilmMapBounds;
pub use super::parser::profile_values::{FilmMppWidths, FilmQuantizationRange};
pub use super::parser::read_diagnostics::{
    FilmComponentObservation, FilmReadDiagnostics, NativeAbilityNonPredictedState, NativeCamoState,
    NativeEquipmentCreationField, NativeEquipmentField, NativeGameEngineField,
    NativeManagedObjectField, NativeManagedPropertyField, NativeMovementComponent, NativeMppField,
    NativeNavpointField, NativeObjectParentState, NativeObjectiveField, NativePlayerStateField,
    NativeProbeComponent, NativeReadOperation, NativeReadRefusal, NativeWidthAdjustment,
    NativeWidthPurpose, NativeWidthRefusal,
};
pub use super::parser::records::{RecordHeader, RecordKind};
pub use super::parser::recovery::{AnchorRecovery, RecoveredKeyframeAnchor};
pub use super::parser::registry::{
    FilmArchetype, FilmRegistry, FilmRegistryRead, FilmRegistryReadError, NativeRegistryBlockRead,
    NativeRegistrySlotRead,
};
pub use super::parser::replication::KeyframeRecoveryPolicy;
pub use super::parser::roster_updates::{
    NativeRosterRead, RosterEntry, RosterReport, RosterUpdate,
};
pub use super::parser::source::{
    FilmChunkProvider, FilmInflateError, FilmSource, FilmSourceError, FilmSourceMetadata,
};
pub use super::parser::translocator::{
    NativeTranslocatorEvent, TeleportPosition, TranslocatorEvent, TranslocatorStop,
};
pub use super::parser::types::{FilmPacket, SourceSpan, SummaryEvent, SummaryKind};
pub use super::parser::unit_equipment::{UnitEquipmentEntry, UnitEquipmentRead};
pub use super::parser::unit_references::{NativeUnitReference, NativeUnitReferenceKind};
pub use super::parser::weapon_hit_scan::WeaponDamageRead;
pub use super::parser::weapon_hits::WeaponDamage;
pub use super::parser::world::{FilmViewAdmission, NativeNewBindingRefusal};
use super::*;
use serde::{Deserialize, Serialize};

/// Native reads together with the complete decompressed source they address.
/// This is an incremental decoding report, not a claim that all retained bytes
/// have been decoded. No player tracks, inferred actions or replay are built.
/// Original transport is retained separately; modified byte-for-byte re-encoding
/// is not implemented. Unknown schemas remain explicit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Film {
    /// Exact bytes supplied to the source loader, before tolerant inflation.
    /// Empty only in older serialized native reports. Packet/bit coordinates
    /// address decompressed `chunks`, never this compressed transport.
    #[serde(default)]
    pub original_chunks: Vec<Vec<u8>>,
    pub registry: FilmRegistryRead,
    pub registry_chunk_position: usize,
    pub identity: Option<FilmIdentity>,
    pub identity_error: Option<String>,
    /// Raw identity-section fields and partial reads at the selected build anchor.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identity_read: Option<NativeIdentityRead>,
    pub player_table: Option<PlayerTable>,
    /// Reads at heuristically selected occupied bootstrap slots, in table order.
    /// These do not establish a canonical partition or decode vacant slots.
    #[serde(default)]
    pub player_slot_reads: Vec<NativePlayerSlotRead>,
    /// Parser settings, explicitly separate from recorded values.
    pub frame_config: NativeFrameMetadata,
    pub recovery_policy: KeyframeRecoveryPolicy,
    /// Runtime grammar selection and evidence, separate from recorded values.
    #[serde(default)]
    pub event_gate15: NativeEventGate15Selection,
    /// Caller-supplied translocator quantization context, never recorded map
    /// data. None keeps map-dependent reads explicitly unresolved.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub translocator_map: Option<FilmMapBounds>,
    /// All metadata, including entries with no corresponding loaded chunk.
    pub metadata: Vec<FilmSourceMetadata>,
    pub chunks: Vec<NativeFilmChunk>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NativeFilmChunk {
    /// Reference whole-chunk highlight scan for footer chunks. Candidate ranges
    /// remain separate from the guarded fixed-offset packet summary decoder.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub highlight_scan: Option<NativeHighlightScan>,
    /// Position in the supplied source, never a substituted manifest number.
    pub source_position: usize,
    pub data: Vec<u8>,
    pub packets: Vec<NativeFilmPacket>,
    /// Start of bytes not partitioned into packet headers/payloads. Bootstrap
    /// chunks have zero here; their separately decoded sections are not packets.
    pub packet_walk_end_byte: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NativeFilmPacket {
    pub header: FilmPacket,
    /// All nested bit offsets are relative to this header's payload_offset.
    /// Stops and synthetic padded reads remain in the native reader result.
    pub body: NativeFilmPacketBody,
    /// An independent direct event-head read, never an inferred gameplay event.
    pub event_head: Option<DecodedHeadEvent>,
    /// Sequential event-layout reads at payload bit one. An absent runtime
    /// code-15 gate stops this read; fixed-size opaque bodies remain marked.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_list: Option<NativeEventListRead>,
    /// Entity/control reads beginning at a completed nonempty event list's
    /// terminator. The original generic message-view stop remains in `body`.
    /// Runtime layout selection is recorded by the enclosing event list/gate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_continuation: Option<NativeEventContinuation>,
    /// Dedicated fixed-layout fire reader under the reference's packet-prefix
    /// admission. This does not establish a complete message-body boundary.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fire_read: Option<NativeFireRead>,
    /// Dedicated native padded reads, including selected-reader refusals.
    /// The bounded `event_head` and original payload remain independently kept.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native_head: Option<NativePacketHeadRead>,
    /// Independent native damage projection and raw fields. This endpoint does
    /// not replace event-list termination or imply inferred player identities.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub damage_read: Option<NativeWeaponDamageRead>,
    /// Bounded roster trace, including the first rejected entry.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub roster_read: Option<NativeRosterRead>,
    /// Native type-12 count and heuristic bot-name/reference candidates. The
    /// unpartitioned payload stays opaque; candidates are not canonical records.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bot_metadata_read: Option<NativeBotMetadataRead>,
    /// Reference anchor recovery is heuristic, separate from the sequential
    /// body. None means not requested/not applicable; Some(empty) means scanned.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keyframe_candidates: Option<Vec<NativeKeyframeCandidate>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeEventContinuation {
    /// Payload-relative bit immediately after the event-list terminator.
    pub start_bit: usize,
    /// Only the following entity/control views are counted in this result.
    /// Refusals and partial reads do not erase the established list boundary.
    pub frame: Result<ProductionFrame, String>,
    /// Whether this attempt may update the grammar state used by later packets.
    #[serde(default)]
    pub state_policy: NativeContinuationStatePolicy,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativeContinuationStatePolicy {
    /// Older exports did not report this policy.
    #[default]
    Unknown,
    Applied,
    /// Pinned code-0 body readers disagree on a field's gate polarity. Keep the
    /// generic continuation attempt but do not propagate its binding effects.
    IsolatedConflictingDamageGrammar,
}

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeKeyframeCandidate {
    pub anchor: RecoveredKeyframeAnchor,
    /// Includes stopped native reads and raw component fields. A successful
    /// reader invocation does not imply a complete record or a proven anchor.
    pub read: Result<KeyframeRecord, String>,
    /// Overlap contradicts the selected candidate partition; retain both reads.
    pub crosses_next_anchor: bool,
}

fn keyframe_candidates(
    payload: &[u8],
    config: &NativeFrameConfig,
    registry: &FilmRegistry,
) -> Vec<NativeKeyframeCandidate> {
    let anchors = recover_keyframe_anchors(payload);
    anchors
        .iter()
        .enumerate()
        .map(|(i, anchor)| {
            let read = config
                .read_keyframe_record(payload, anchor.bit, registry)
                .map_err(|error| error.to_string());
            let crosses_next_anchor = read.as_ref().is_ok_and(|record| {
                anchors
                    .get(i + 1)
                    .is_some_and(|next| record.end_bit > next.bit as i64)
            });
            NativeKeyframeCandidate {
                anchor: anchor.clone(),
                read,
                crosses_next_anchor,
            }
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// Preserve the native by-value API while retained diagnostics grow.
#[allow(clippy::large_enum_variant)]
pub enum NativeFilmPacketBody {
    Frame(Box<ProductionFrame>),
    Datums(DatumTable),
    Roster(RosterUpdate),
    Keyframes(NativeKeyframeTable),
    /// Guarded captured-layout reads in wire order; intervening state is opaque.
    Summary {
        declared_events: u32,
        events: Vec<SummaryEvent>,
    },
    /// A reader could not start. The complete payload remains in the chunk.
    Refused {
        message: String,
    },
    /// No body reader selected; this does not imply an empty payload.
    Opaque,
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error(transparent)]
    Registry(#[from] NativeContextRegistryError),
    #[error("native film entry requires a recorded v41 header, got {0:?}")]
    Version(Option<[u32; 2]>),
    #[error(transparent)]
    Profile(#[from] NativeProfileResolveError),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParseOptions {
    /// Explicit caller precision/grammar context for native frame and keyframe
    /// reads. No automatic map inference is performed. Effective settings are
    /// retained in `Film::frame_config`, separately from recorded data.
    /// Recorded/inherited corruption control keeps NativeFilmContext precedence.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frame_profile: Option<NativeScanProfile>,
    pub recovery_policy: KeyframeRecoveryPolicy,
    pub event_gate15: NativeEventGate15Policy,
    /// External context for map-relative translocator vectors. No map is
    /// required to parse the rest of the native recording.
    #[serde(default)]
    pub translocator_map: Option<FilmMapBounds>,
}
impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            frame_profile: None,
            recovery_policy: KeyframeRecoveryPolicy::LevelUp,
            event_gate15: NativeEventGate15Policy::ReferenceInference,
            translocator_map: None,
        }
    }
}

impl Film {
    /// Parse a complete v41 recording source, preserving native order and gaps.
    pub fn parse(source: &FilmSource, options: ParseOptions) -> Result<Self, ParseError> {
        Self::parse_v41_with_options(source, options)
    }

    /// Walk this recording to build chronological indexes and playback state.
    /// The resolved model borrows this film; source bytes are neither reparsed nor copied.
    pub fn resolve(&self) -> super::resolved::ResolvedFilm<'_> {
        super::resolved::ResolvedFilm::from_film(self)
    }

    /// Decode available native packet bodies in input order. A zero clock,
    /// missing roster or stopped body does not reject an otherwise v41 source.
    /// Only sequential keyframe declarations seed the private grammar state;
    /// this entry does not promote scanned candidates to canonical records.
    #[cfg(test)]
    pub(crate) fn parse_v41(source: &FilmSource) -> Result<Self, ParseError> {
        Self::parse_v41_with_recovery(source, KeyframeRecoveryPolicy::LevelUp)
    }

    /// Select whether reference candidate reads accompany the sequential body.
    /// The selection never changes the sequential grammar's binding state.
    #[cfg(test)]
    pub(crate) fn parse_v41_with_recovery(
        source: &FilmSource,
        recovery_policy: KeyframeRecoveryPolicy,
    ) -> Result<Self, ParseError> {
        Self::parse_v41_with_options(
            source,
            ParseOptions {
                recovery_policy,
                ..Default::default()
            },
        )
    }

    pub(crate) fn parse_v41_with_options(
        source: &FilmSource,
        options: ParseOptions,
    ) -> Result<Self, ParseError> {
        let recovery_policy = options.recovery_policy;
        let mut context = NativeFilmContext::new(Some(source));
        let registry = context.registry().map_err(|e| *e)?.clone();
        if registry.header.is_none() || registry.registry.major_version != 41 {
            return Err(ParseError::Version(registry.header));
        }
        let registry_chunk_position = source
            .chunk_position(0)
            .ok_or(NativeContextRegistryError::NoRegistryChunk)?;
        let bootstrap = source.chunk(registry_chunk_position).unwrap();
        let event_gate15 = select_native_event_gate15(
            (0..source.num_chunks())
                .filter(|&position| {
                    position != registry_chunk_position
                        && source
                            .metadata()
                            .get(position)
                            .is_none_or(|m| m.chunk_type == 2)
                })
                .flat_map(|position| source.packets(position))
                .filter(|packet| packet.packet_type == 0)
                .map(|packet| source.payload(packet).expect("indexed source payload")),
            options.event_gate15,
        );
        let identity_read = read_native_identity(bootstrap, &registry.registry);
        let identity = identity_read.identity.clone();
        let identity_error = identity_read.error.clone();
        let player_table = identity
            .as_ref()
            .map(|id| decode_player_table(bootstrap, id));
        let player_slot_reads = player_table
            .as_ref()
            .map(|table| {
                table
                    .slots
                    .iter()
                    .map(|slot| {
                        read_native_player_slot(bootstrap, slot.bit, table.report.perso_bytes * 8)
                    })
                    .collect()
            })
            .unwrap_or_default();
        if let Some(profile) = &options.frame_profile {
            // Freeze mutable calibration maps for this read. Installing caller
            // context does not replace the film's recorded corruption setting.
            context.set_scan_profile(profile.snapshot())?;
        }
        let config = context.scan_frame()?;
        let personalization_bits = context.profile()?.personalization_bytes.map(|n| n * 8);
        // This accumulator exists solely to select subsequent native grammars.
        // It is deliberately neither exported nor interpreted as a replay.
        let mut world = FilmWorld::default();
        let mut chunks = Vec::with_capacity(source.num_chunks());
        for source_position in 0..source.num_chunks() {
            let data = source.chunk(source_position).unwrap();
            world.current_chunk = source
                .metadata()
                .get(source_position)
                .map_or(source_position as i64, |m| m.index);
            let mut chunk = NativeFilmChunk {
                highlight_scan: source
                    .metadata()
                    .get(source_position)
                    .filter(|m| m.chunk_type == 3)
                    .map(|_| read_native_v41_highlights(data)),
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
                        read_native_event_list(payload, 1, event_gate15.selected, payload.len())
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
                        bot_metadata_read: (!footer && header.packet_type == 12)
                            .then(|| read_native_bot_metadata(payload)),
                        body,
                        event_head: (!footer && header.packet_type == 0)
                            .then(|| decode_packet_head_event(payload))
                            .flatten(),
                        event_list,
                        event_continuation,
                        fire_read: (!footer
                            && header.packet_type == 0
                            && payload.first().is_some_and(|b| b & 0x7f == 0x52))
                        .then(|| {
                            let mut read = read_native_fire_event(payload);
                            if let Some(event) = &mut read.event {
                                event.chunk = i64::from(header.chunk_index);
                                event.packet_index = packet_index;
                                event.timestamp_us = header.timestamp_us;
                            }
                            read
                        }),
                        native_head: (!footer)
                            .then(|| {
                                read_native_packet_head(
                                    header.packet_type,
                                    payload,
                                    options.translocator_map.as_ref(),
                                )
                            })
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
                        keyframe_candidates: (!footer
                            && header.packet_type == 2
                            && recovery_policy == KeyframeRecoveryPolicy::LevelUp)
                            .then(|| keyframe_candidates(payload, &config, &registry.registry)),
                    });
                }
            }
            chunks.push(chunk);
        }
        Ok(Self {
            original_chunks: source.original_chunks().to_vec(),
            registry,
            registry_chunk_position,
            identity,
            identity_error,
            identity_read: Some(identity_read),
            player_table,
            player_slot_reads,
            frame_config: config.snapshot(),
            recovery_policy,
            event_gate15,
            translocator_map: options.translocator_map,
            metadata: source.metadata().to_vec(),
            chunks,
        })
    }
}

#[cfg(test)]
#[path = "parser/native_event_continuation_tests.rs"]
mod continuation_tests;

#[cfg(test)]
mod tests {
    use super::*;

    fn packet(kind: u16, payload: &[u8]) -> Vec<u8> {
        let mut bytes = kind.to_le_bytes().to_vec();
        bytes.extend([0xab, 0xcd]);
        bytes.extend((payload.len() as u32).to_le_bytes());
        bytes.extend(0u64.to_le_bytes());
        bytes.extend(payload);
        bytes
    }

    #[test]
    fn native_data_bot_metadata_admission() {
        let mut packets = packet(12, &[0, 0]);
        packets.extend(packet(12, &0u32.to_be_bytes()));
        packets.extend(packet(12, &65u32.to_be_bytes()));
        packets.extend(packet(11, &0u32.to_be_bytes()));
        let chunks = [
            [41u32.to_le_bytes(), 27u32.to_le_bytes()].concat(),
            packets.clone(),
            packets,
        ];
        let metadata = [
            FilmSourceMetadata {
                index: 0,
                chunk_type: 1,
                start_ms: 0,
            },
            FilmSourceMetadata {
                index: 1,
                chunk_type: 2,
                start_ms: 0,
            },
            FilmSourceMetadata {
                index: 2,
                chunk_type: 3,
                start_ms: 0,
            },
        ];
        let source = FilmSource::load(&chunks, &metadata).unwrap();
        let parsed = Film::parse_v41(&source).unwrap();
        let packets = &parsed.chunks[1].packets;
        assert_eq!(
            packets[0].bot_metadata_read.as_ref().unwrap().declared_bots,
            None
        );
        assert_eq!(
            packets[1].bot_metadata_read.as_ref().unwrap().candidates,
            Some(vec![])
        );
        assert_eq!(
            packets[2].bot_metadata_read.as_ref().unwrap().declared_bots,
            Some(65)
        );
        assert_eq!(
            packets[2].bot_metadata_read.as_ref().unwrap().candidates,
            None
        );
        assert!(packets[3].bot_metadata_read.is_none());
        assert!(
            parsed.chunks[2]
                .packets
                .iter()
                .all(|p| p.bot_metadata_read.is_none())
        );
        let mut old = serde_json::to_value(&packets[0]).unwrap();
        old.as_object_mut().unwrap().remove("bot_metadata_read");
        assert!(
            serde_json::from_value::<NativeFilmPacket>(old)
                .unwrap()
                .bot_metadata_read
                .is_none()
        );
        assert_eq!(
            parsed,
            serde_json::from_slice(&serde_json::to_vec(&parsed).unwrap()).unwrap()
        );
    }

    #[test]
    fn native_data_retains_failed_view_request() {
        let chunks = [
            [41u32.to_le_bytes(), 27u32.to_le_bytes()].concat(),
            packet(0, &[0x40]),
        ];
        let metadata = [
            FilmSourceMetadata {
                index: 0,
                chunk_type: 1,
                start_ms: 0,
            },
            FilmSourceMetadata {
                index: 1,
                chunk_type: 2,
                start_ms: 0,
            },
        ];
        let source = FilmSource::load(&chunks, &metadata).unwrap();
        let parsed = Film::parse_v41(&source).unwrap();
        let NativeFilmPacketBody::Frame(frame) = &parsed.chunks[1].packets[0].body else {
            panic!()
        };
        let messages = frame.messages.as_ref().unwrap();
        assert_eq!(
            messages.diagnostics.as_ref().unwrap().read_refusals,
            [NativeReadRefusal {
                field: "record[0].kind".into(),
                bit: 2,
                width: 7,
                source_bits: 8,
                operation: NativeReadOperation::Scalar,
            }]
        );
        assert_eq!(messages.end_bit, 2);
        assert_eq!(messages.fields.len(), 1);
        assert_eq!(
            parsed,
            serde_json::from_slice(&serde_json::to_vec(&parsed).unwrap()).unwrap()
        );
    }

    #[test]
    fn native_data_preserves_zero_clock_partial_packets_and_source_order() {
        let mut data = packet(2, &[0; 9]);
        data.extend(packet(1, &[7])); // Refused datum body must survive.
        data.extend(packet(99, &[3, 2, 1]));
        data.extend(packet(7, &[]));
        data.extend([0xfe, 0xed]); // Bytes after native terminator.
        let bootstrap = [41u32.to_le_bytes(), 27u32.to_le_bytes()].concat();
        let chunks = [data.clone(), bootstrap, packet(1, &[1; 20])];
        let metadata = [
            FilmSourceMetadata {
                index: 8,
                chunk_type: 2,
                start_ms: -100,
            },
            FilmSourceMetadata {
                index: 0,
                chunk_type: 1,
                start_ms: 0,
            },
            FilmSourceMetadata {
                index: 9,
                chunk_type: 99,
                start_ms: 0,
            },
            FilmSourceMetadata {
                index: 123,
                chunk_type: 5,
                start_ms: 10,
            },
        ];
        let source = FilmSource::load(&chunks, &metadata).unwrap();
        let parsed = Film::parse_v41(&source).unwrap();
        let sequential =
            Film::parse_v41_with_recovery(&source, KeyframeRecoveryPolicy::SequentialOnly).unwrap();
        assert_eq!(
            parsed.chunks[0].packets[0].body,
            sequential.chunks[0].packets[0].body
        );
        assert_eq!(
            parsed.chunks[0].packets[0].keyframe_candidates,
            Some(vec![])
        );
        assert!(
            sequential.chunks[0].packets[0]
                .keyframe_candidates
                .is_none()
        );
        assert_eq!(parsed.registry_chunk_position, 1);
        assert!(parsed.identity.is_none());
        assert_eq!(parsed.metadata, metadata);
        assert_eq!(parsed.chunks.len(), 3);
        for (actual, expected) in parsed.chunks.iter().zip(chunks) {
            assert_eq!(actual.data, expected);
        }
        let chunk = &parsed.chunks[0];
        assert_eq!(chunk.packets.len(), 4);
        assert_eq!(chunk.packet_walk_end_byte, data.len() - 2);
        assert_eq!(chunk.packets[0].header.timestamp_us, 0);
        assert_eq!(chunk.packets[0].header.byte_2, 0xab);
        assert!(matches!(
            chunk.packets[0].body,
            NativeFilmPacketBody::Keyframes(_)
        ));
        assert!(matches!(
            chunk.packets[1].body,
            NativeFilmPacketBody::Refused { .. }
        ));
        assert!(matches!(
            chunk.packets[2].body,
            NativeFilmPacketBody::Opaque
        ));
        assert!(parsed.chunks[1].packets.is_empty());
        assert!(parsed.chunks[2].packets.is_empty());
        let json = serde_json::to_vec(&parsed).unwrap();
        assert_eq!(serde_json::from_slice::<Film>(&json).unwrap(), parsed);
    }

    #[test]
    fn native_data_requires_recorded_v41_version() {
        for bytes in [vec![], [40u32.to_le_bytes(), 27u32.to_le_bytes()].concat()] {
            let source = FilmSource::load(&[bytes], &[]).unwrap();
            assert!(matches!(
                Film::parse_v41(&source),
                Err(ParseError::Version(_))
            ));
        }
    }

    #[test]
    fn native_data_retains_event_lists_longer_than_64_records() {
        let mut bits = vec![true]; // recorded packet config
        for _ in 0..65 {
            bits.push(true);
            bits.extend((0..7).rev().map(|bit| 3 & (1 << bit) != 0));
            bits.extend([false; 3]); // absent references, empty code-3 body
        }
        bits.push(false);
        let mut payload = vec![0; bits.len().div_ceil(8)];
        for (bit, value) in bits.iter().enumerate() {
            payload[bit / 8] |= u8::from(*value) << (7 - bit % 8);
        }
        let chunks = [
            [41u32.to_le_bytes(), 27u32.to_le_bytes()].concat(),
            packet(0, &payload),
        ];
        let source = FilmSource::load(&chunks, &[]).unwrap();
        let parsed =
            Film::parse_v41_with_recovery(&source, KeyframeRecoveryPolicy::SequentialOnly).unwrap();
        let events = parsed.chunks[1].packets[0].event_list.as_ref().unwrap();
        assert_eq!(events.stop, NativeEventListStop::Terminator);
        assert_eq!(events.records.len(), 65);
        assert!(
            events
                .records
                .iter()
                .all(|r| r.layout_complete && r.code == Some(3))
        );
        assert_eq!(events.end_bit, bits.len());
    }

    #[test]
    fn native_data_runtime_gate_policy_preserves_inference_and_uncertainty() {
        let mut payload = vec![0; 20];
        payload[..2].copy_from_slice(&[0xc7, 0x80]); // config, continuation, code 15
        let chunks = [
            [41u32.to_le_bytes(), 27u32.to_le_bytes()].concat(),
            packet(0, &payload),
        ];
        let source = FilmSource::load(&chunks, &[]).unwrap();
        for (policy, selected, end, stop) in [
            (
                NativeEventGate15Policy::ReferenceInference,
                Some(false),
                36,
                NativeEventListStop::Terminator,
            ),
            (
                NativeEventGate15Policy::Explicit(true),
                Some(true),
                51,
                NativeEventListStop::Terminator,
            ),
            (
                NativeEventGate15Policy::Unknown,
                None,
                12,
                NativeEventListStop::MissingRuntimeGate15,
            ),
        ] {
            let parsed = Film::parse_v41_with_options(
                &source,
                ParseOptions {
                    recovery_policy: KeyframeRecoveryPolicy::SequentialOnly,
                    event_gate15: policy,
                    ..Default::default()
                },
            )
            .unwrap();
            assert_eq!(parsed.event_gate15.policy, policy);
            assert_eq!(parsed.event_gate15.selected, selected);
            assert_eq!(
                parsed.event_gate15.localized_candidate_counts,
                (policy == NativeEventGate15Policy::ReferenceInference).then_some([0, 0])
            );
            assert_eq!(parsed.event_gate15.eligible_packets, 0);
            let events = parsed.chunks[1].packets[0].event_list.as_ref().unwrap();
            assert_eq!(events.gate15, selected);
            assert_eq!(events.end_bit, end);
            assert_eq!(events.stop, stop);
            assert_eq!(
                serde_json::from_value::<Film>(serde_json::to_value(&parsed).unwrap()).unwrap(),
                parsed
            );
        }
    }

    #[test]
    fn native_data_keeps_fire_attempts_and_packet_admission() {
        let mut payload = vec![0; 18];
        payload[0] = 0xd2;
        let mut data = packet(0, &payload);
        data.extend(packet(99, &payload));
        payload[0] = 0xd3;
        data.extend(packet(0, &payload));
        data.extend(packet(0, &[0xd2]));
        let chunks = [[41u32.to_le_bytes(), 27u32.to_le_bytes()].concat(), data];
        let source = FilmSource::load(&chunks, &[]).unwrap();
        let parsed =
            Film::parse_v41_with_recovery(&source, KeyframeRecoveryPolicy::SequentialOnly).unwrap();
        let packets = &parsed.chunks[1].packets;
        let fire = packets[0]
            .fire_read
            .as_ref()
            .unwrap()
            .event
            .as_ref()
            .unwrap();
        assert_eq!(
            (fire.chunk, fire.packet_index, fire.timestamp_us),
            (1, 0, 0)
        );
        assert!(packets[1].fire_read.is_none());
        assert!(packets[2].fire_read.is_none());
        let truncated = packets[3].fire_read.as_ref().unwrap();
        assert!(truncated.event.is_none());
        assert_eq!(truncated.source_bits, 8);
        assert_eq!(
            serde_json::from_value::<Film>(serde_json::to_value(&parsed).unwrap()).unwrap(),
            parsed
        );
    }

    #[test]
    #[ignore = "requires retained local v41 films; integration coverage audit"]
    fn native_data_captured_corpus() {
        #[derive(Deserialize)]
        struct BaselineRow {
            file: String,
            offset: usize,
            views: usize,
            end: i64,
        }
        let baseline_file = std::fs::File::open(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("src/theater/fixtures/native-baseline-d61443e-v41.jsonl.zlib"),
        )
        .unwrap();
        let mut baseline_reference = std::collections::BTreeMap::new();
        let mut expected_baseline_complete = 0;
        for line in std::io::BufReader::new(flate2::read::ZlibDecoder::new(baseline_file)).lines() {
            let row: BaselineRow = serde_json::from_str(&line.unwrap()).unwrap();
            expected_baseline_complete += usize::from(row.views == 3);
            assert!(
                baseline_reference
                    .insert((row.file, row.offset), (row.views, row.end))
                    .is_none()
            );
        }

        use std::io::BufRead;
        use std::io::Read;
        let mut highlight_json = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/highlights-corpus-v41.json.zlib")[..],
        )
        .read_to_end(&mut highlight_json)
        .unwrap();
        let mut highlight_reference: std::collections::BTreeMap<String, serde_json::Value> =
            serde_json::from_slice(&highlight_json).unwrap();
        let mut head_json = Vec::new();
        let mut damage_json = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/native-damage-v41.json.zlib")[..])
            .read_to_end(&mut damage_json)
            .unwrap();
        let damage_rows: Vec<serde_json::Value> = serde_json::from_slice(&damage_json).unwrap();
        let mut damage_reference = std::collections::BTreeMap::new();
        for row in damage_rows {
            damage_reference.insert(
                (
                    row["file"].as_str().unwrap().to_owned(),
                    row["packet"].as_u64().unwrap() as usize,
                ),
                row,
            );
        }
        drop(damage_json);
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/native-packet-heads-v41.json.zlib")[..],
        )
        .read_to_end(&mut head_json)
        .unwrap();
        let head_rows: Vec<serde_json::Value> = serde_json::from_slice(&head_json).unwrap();
        let mut head_reference = std::collections::BTreeMap::new();
        for row in head_rows {
            head_reference.insert(
                (
                    row["file"].as_str().unwrap().to_owned(),
                    row["packet"].as_u64().unwrap() as usize,
                ),
                row,
            );
        }
        drop(head_json);
        #[derive(Deserialize)]
        struct FireRow {
            source: String,
            event: FilmFireEvent,
            valid: bool,
        }
        let mut fire_json = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/fire-d61443e-v41.json.zlib")[..])
            .read_to_end(&mut fire_json)
            .unwrap();
        let fire_rows: Vec<FireRow> = serde_json::from_slice(&fire_json).unwrap();
        let mut fire_reference = std::collections::BTreeMap::new();
        for row in fire_rows {
            if !row.source.contains("/experiments/films/") {
                continue;
            }
            let file = row
                .source
                .split("/experiments/films/")
                .nth(1)
                .unwrap()
                .to_owned();
            fire_reference.insert((file, row.event.packet_index), row);
        }
        drop(fire_json);
        #[derive(Deserialize)]
        struct ReferenceRow {
            file: String,
            offset: usize,
            anchor: ReferenceAnchor,
            trace: ReferenceTrace,
        }
        #[derive(Deserialize)]
        #[serde(rename_all = "PascalCase")]
        struct ReferenceAnchor {
            slot: u32,
            #[serde(rename = "TI")]
            archetype: u32,
            #[serde(rename = "Gen")]
            generation: u32,
            bit: usize,
        }
        #[derive(Deserialize)]
        #[serde(rename_all = "PascalCase")]
        struct ReferenceTrace {
            type_index: u32,
            desync_at: i64,
            end_bit: i64,
            comps: Option<Vec<ReferenceComponent>>,
        }
        #[derive(Deserialize)]
        #[serde(rename_all = "PascalCase")]
        struct ReferenceComponent {
            index: usize,
            name: String,
            start_bit: i64,
            variant: u32,
            ported: bool,
        }
        let fixture = std::fs::File::open(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("src/theater/fixtures/keyframe-anchor-bodies-d61443e-v41.jsonl.zlib"),
        )
        .unwrap();
        let mut reference = std::collections::BTreeMap::<(String, usize), Vec<ReferenceRow>>::new();
        for line in std::io::BufReader::new(flate2::read::ZlibDecoder::new(fixture)).lines() {
            let row: ReferenceRow = serde_json::from_str(&line.unwrap()).unwrap();
            reference
                .entry((row.file.clone(), row.offset))
                .or_default()
                .push(row);
        }
        fn manifests(directory: &std::path::Path, paths: &mut Vec<std::path::PathBuf>) {
            for entry in std::fs::read_dir(directory).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    manifests(&path, paths);
                } else if path.file_name().unwrap() == "film.json" {
                    paths.push(path);
                }
            }
        }
        let root = std::env::var_os("HALO_FILM_CORPUS")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("experiments/films")
            });
        let mut paths = Vec::new();
        manifests(&root, &mut paths);
        paths.sort();
        assert_eq!(paths.len(), 32);
        let mut totals = std::collections::BTreeMap::<String, usize>::new();
        for path in paths {
            let manifest: serde_json::Value =
                serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
            let mut bytes = Vec::new();
            let mut metadata = Vec::new();
            let mut files = Vec::new();
            for row in manifest["chunks"].as_array().unwrap() {
                files.push(
                    path.parent()
                        .unwrap()
                        .strip_prefix(&root)
                        .unwrap()
                        .join(row["file"].as_str().unwrap())
                        .to_str()
                        .unwrap()
                        .to_owned(),
                );
                bytes.push(
                    std::fs::read(path.parent().unwrap().join(row["file"].as_str().unwrap()))
                        .unwrap(),
                );
                metadata.push(FilmSourceMetadata {
                    index: row["index"].as_i64().unwrap(),
                    chunk_type: row["chunk_type"].as_i64().unwrap(),
                    start_ms: row["start_time_offset_ms"].as_i64().unwrap(),
                });
            }
            let source = FilmSource::load(&bytes, &metadata).unwrap();
            let parsed = Film::parse_v41(&source).unwrap();
            let registry_reads = parsed.registry.block_reads.as_ref().unwrap();
            assert_eq!(
                registry_reads.iter().filter(|block| block.accepted).count(),
                parsed.registry.registry.archetypes.len()
            );
            assert!(!registry_reads.last().unwrap().accepted);
            assert_eq!(
                registry_reads.last().unwrap().start_byte,
                parsed.registry.registry.end_byte
            );
            for block in registry_reads {
                for slot in &block.slots {
                    assert_eq!(
                        &slot.name_bytes,
                        &parsed.chunks[parsed.registry_chunk_position].data
                            [slot.start_byte..slot.start_byte + 256]
                    );
                }
            }
            let identity_read = parsed.identity_read.as_ref().unwrap();
            assert_eq!(identity_read.identity, parsed.identity);
            assert_eq!(identity_read.error, parsed.identity_error);
            let final_identity_field = identity_read.fields.last().unwrap();
            assert_eq!(
                final_identity_field.bit + final_identity_field.bits,
                parsed.identity.as_ref().unwrap().body_bit
            );
            // Compare the previous generic-only traversal with the integrated
            // traversal. A rejected unbound header could previously count as a
            // completed view; a newly known binding may instead expose a real
            // unsupported body. Completion counts alone cannot judge this.
            let baseline_config = NativeFilmContext::new(Some(&source)).scan_frame().unwrap();
            let mut baseline_world = FilmWorld::default();
            assert_eq!(parsed.metadata, metadata);
            assert!(parsed.identity.is_some(), "{}", path.display());
            assert_eq!(
                parsed.event_gate15.policy,
                NativeEventGate15Policy::ReferenceInference
            );
            *totals
                .entry(format!(
                    "gate15_selected_{}",
                    parsed.event_gate15.selected.unwrap()
                ))
                .or_default() += 1;
            *totals.entry("gate15_eligible_packets".into()).or_default() +=
                parsed.event_gate15.eligible_packets;
            let scores = parsed.event_gate15.localized_candidate_counts.unwrap();
            *totals.entry("gate15_ties".into()).or_default() += usize::from(scores[0] == scores[1]);
            for (position, chunk) in parsed.chunks.iter().enumerate() {
                baseline_world.current_chunk = source
                    .metadata()
                    .get(position)
                    .map_or(position as i64, |m| m.index);
                assert_eq!(chunk.data, source.chunk(position).unwrap());
                if let Some(scan) = &chunk.highlight_scan {
                    let folder = path
                        .parent()
                        .unwrap()
                        .strip_prefix(&root)
                        .unwrap()
                        .to_str()
                        .unwrap();
                    let expected = highlight_reference.remove(folder).unwrap();
                    assert_eq!(
                        metadata[position].index,
                        expected["index"].as_i64().unwrap()
                    );
                    let expected_events: Vec<NativeHighlightEvent> = serde_json::from_value(
                        expected["events"]
                            .as_array()
                            .cloned()
                            .unwrap_or_default()
                            .into(),
                    )
                    .unwrap();
                    assert_eq!(scan.events, expected_events, "highlights {folder}");
                    *totals.entry("native_highlight_events".into()).or_default() +=
                        scan.events.len();
                    *totals
                        .entry("native_highlight_identities".into())
                        .or_default() += scan.identities.len();
                    *totals.entry("native_highlight_tails".into()).or_default() +=
                        scan.identities.iter().map(|i| i.tails.len()).sum::<usize>();
                    for identity in &scan.identities {
                        if let Some(fields) = &identity.objective_fields {
                            *totals
                                .entry("native_objective_footer_fields".into())
                                .or_default() += 1;
                            let bytes = identity.tails[0].data.as_ref().unwrap();
                            assert_eq!((fields.slot, fields.team), (bytes[36], bytes[37]));
                        }
                        for tail in &identity.tails {
                            if let (Some(start), Some(bytes)) = (tail.data_start_bit, &tail.data) {
                                for (i, byte) in bytes.iter().enumerate() {
                                    assert_eq!(
                                        super::super::bits::Bits(&chunk.data)
                                            .read(start + i * 8, 8),
                                        Some(u64::from(*byte))
                                    );
                                }
                            }
                        }
                    }
                }

                assert!(chunk.packet_walk_end_byte <= chunk.data.len());
                for (ordinal, packet) in chunk.packets.iter().enumerate() {
                    assert_eq!(&packet.header, &source.packets(position)[ordinal]);
                    let end = packet.header.payload_offset + packet.header.payload_size;
                    assert!(end <= chunk.data.len());
                    if let Some(read) = &packet.bot_metadata_read {
                        assert_eq!(packet.header.packet_type, 12);
                        let payload = &chunk.data[packet.header.payload_offset..end];
                        assert_eq!(
                            read.declared_bots,
                            payload
                                .get(..4)
                                .map(|b| u32::from_be_bytes(b.try_into().unwrap()))
                        );
                        *totals.entry("bot_metadata_reads".into()).or_default() += 1;
                        *totals.entry("bot_metadata_candidates".into()).or_default() +=
                            read.candidates.as_ref().map_or(0, Vec::len);
                    }
                    if let Some(read) = &packet.fire_read {
                        let mut expected = fire_reference
                            .remove(&(files[position].clone(), ordinal))
                            .unwrap();
                        // Reference fixtures tag manifest numbers; this entry
                        // consistently tags source positions, retained above.
                        expected.event.chunk = position as i64;
                        assert_eq!(read.event, expected.valid.then_some(expected.event));
                        *totals.entry("fire_reads".into()).or_default() += 1;
                        *totals.entry("fire_fixed_fields".into()).or_default() += read.fields.len();
                        *totals.entry("fire_aim_attempts".into()).or_default() +=
                            read.aim_attempts.len();
                        *totals.entry("fire_accepted_aims".into()).or_default() +=
                            read.aim_attempts.iter().filter(|a| a.accepted).count();
                    }
                    if let Some(read) = &packet.native_head {
                        let expected = head_reference
                            .remove(&(files[position].clone(), ordinal))
                            .unwrap();
                        assert_eq!(expected["offset"], packet.header.payload_offset);
                        assert_eq!(expected["size"], packet.header.payload_size);
                        assert_eq!(expected["time"], packet.header.timestamp_us);
                        native_packet_heads::tests::check_read(read, &expected);
                        let (family, padded) = match read {
                            NativePacketHeadRead::Zoom(read) => {
                                ("zoom", read.as_ref().map_or(0, |r| r.padded_bits))
                            }
                            NativePacketHeadRead::Pickup(read) => ("pickup", read.padded_bits),
                            NativePacketHeadRead::Translocator(read) => {
                                ("translocator", read.as_ref().map_or(0, |r| r.padded_bits))
                            }
                        };
                        assert_eq!(expected["family"], family);
                        *totals.entry(format!("native_head_{family}")).or_default() += 1;
                        *totals.entry("native_head_padded_bits".into()).or_default() += padded;
                    }
                    if let Some(read) = &packet.damage_read {
                        let expected = damage_reference
                            .remove(&(files[position].clone(), ordinal))
                            .unwrap();
                        assert_eq!(expected["offset"], packet.header.payload_offset);
                        assert_eq!(expected["size"], packet.header.payload_size);
                        assert_eq!(expected["time"], packet.header.timestamp_us);
                        assert_eq!(expected["ok"], read.read.is_some());
                        assert_eq!(expected["end"], read.end_bit);
                        if let Some(projection) = &read.read {
                            let damage: WeaponDamage =
                                serde_json::from_value(expected["damage"].clone()).unwrap();
                            assert_eq!(projection.damage, damage);
                            assert_eq!(expected["secondary"], projection.secondary_mag_raw);
                            assert_eq!(
                                expected["body_victim"],
                                serde_json::json!(projection.body_victim_idx)
                            );
                            assert_eq!(projection.source, Some(packet.header));
                            assert_eq!(projection.packet_index, Some(ordinal));
                            *totals.entry("damage_reads_accepted".into()).or_default() += 1;
                            if let Some(first) = packet
                                .event_list
                                .as_ref()
                                .and_then(|e| e.records.first())
                                .filter(|r| r.code == Some(0) && r.layout_complete)
                                && first.end_bit != read.end_bit
                            {
                                *totals
                                    .entry("damage_layout_body_end_differences".into())
                                    .or_default() += 1;
                                if packet.event_list.as_ref().unwrap().stop
                                    == NativeEventListStop::Terminator
                                {
                                    *totals
                                        .entry("damage_layout_conflict_with_terminator".into())
                                        .or_default() += 1;
                                }
                            }
                        }
                        native_weapon_damage::tests::check_fields(
                            read,
                            source.payload(&packet.header).unwrap(),
                        );
                        *totals.entry("damage_reads".into()).or_default() += 1;
                        *totals.entry("damage_fields".into()).or_default() += read.fields.len();
                        *totals.entry("damage_opaque_fields".into()).or_default() +=
                            read.fields.iter().filter(|f| f.opaque).count();
                        *totals.entry("damage_padded_bits".into()).or_default() +=
                            read.padding_bits;
                    }
                    if let Some(events) = &packet.event_list {
                        *totals
                            .entry(format!("event_list_stop_{:?}", events.stop))
                            .or_default() += 1;
                        *totals.entry("event_layout_records".into()).or_default() +=
                            events.records.iter().filter(|r| r.layout_complete).count();
                        *totals.entry("event_opaque_fields".into()).or_default() += events
                            .fields
                            .iter()
                            .filter(|f| f.value == NativeEventFieldValue::Opaque && f.width > 0)
                            .count();
                        assert_eq!(
                            packet.event_continuation.is_some(),
                            events.stop == NativeEventListStop::Terminator
                                && !events.records.is_empty()
                        );
                    }
                    if let Some(continuation) = &packet.event_continuation {
                        let events = packet.event_list.as_ref().unwrap();
                        assert_eq!(continuation.start_bit, events.end_bit);
                        assert!(continuation.start_bit <= packet.header.payload_size * 8);
                        let frame = continuation.frame.as_ref().unwrap();
                        assert!(frame.messages.is_none());
                        *totals.entry("event_continuations".into()).or_default() += 1;
                        *totals
                            .entry(format!(
                                "continuation_state_{:?}",
                                continuation.state_policy
                            ))
                            .or_default() += 1;
                        *totals
                            .entry(format!("continuation_views_{}", frame.views_completed))
                            .or_default() += 1;
                        *totals.entry("continuation_records".into()).or_default() +=
                            frame.records.len();
                        *totals.entry("continuation_fields".into()).or_default() +=
                            frame.records.iter().map(|r| r.fields.len()).sum::<usize>();
                    }
                    if let Some(candidates) = &packet.keyframe_candidates {
                        let expected = reference
                            .remove(&(files[position].clone(), packet.header.payload_offset))
                            .unwrap();
                        assert_eq!(candidates.len(), expected.len());
                        for (i, (candidate, expected)) in
                            candidates.iter().zip(expected).enumerate()
                        {
                            *totals.entry("candidate_reads".into()).or_default() += 1;
                            assert_eq!(
                                (
                                    candidate.anchor.bit,
                                    candidate.anchor.id & 0x3fff_ffff,
                                    candidate.anchor.id >> 30,
                                    candidate.anchor.archetype
                                ),
                                (
                                    expected.anchor.bit,
                                    expected.anchor.slot,
                                    expected.anchor.generation,
                                    expected.anchor.archetype
                                )
                            );
                            let record = candidate.read.as_ref().unwrap();
                            assert_eq!(record.end_bit, expected.trace.end_bit);
                            assert_eq!(record.archetype, expected.trace.type_index);
                            let desync = match &record.stop {
                                KeyframeStop::Complete => {
                                    *totals.entry("candidate_complete".into()).or_default() += 1;
                                    -1
                                }
                                KeyframeStop::UnsupportedComponent { index, .. } => *index as i64,
                                other => panic!("unexpected candidate stop {other:?}"),
                            };
                            assert_eq!(desync, expected.trace.desync_at);
                            let components = expected.trace.comps.unwrap_or_default();
                            assert_eq!(record.attempts.len(), components.len());
                            for (actual, expected) in record.attempts.iter().zip(components) {
                                assert_eq!(
                                    (
                                        actual.index,
                                        &actual.name,
                                        actual.start_bit,
                                        actual.variant,
                                        actual.ported
                                    ),
                                    (
                                        expected.index,
                                        &expected.name,
                                        expected.start_bit,
                                        Some(expected.variant),
                                        Some(expected.ported)
                                    )
                                );
                            }
                            assert_eq!(
                                candidate.crosses_next_anchor,
                                candidates
                                    .get(i + 1)
                                    .is_some_and(|next| record.end_bit > next.anchor.bit as i64)
                            );
                            *totals
                                .entry("candidate_crosses_next_anchor".into())
                                .or_default() += usize::from(candidate.crosses_next_anchor);
                        }
                    }
                    let key = match &packet.body {
                        NativeFilmPacketBody::Frame(frame) => {
                            let baseline = baseline_config
                                .decode_production_views(
                                    source.payload(&packet.header).unwrap(),
                                    2,
                                    &parsed.registry.registry,
                                    &mut baseline_world,
                                )
                                .unwrap();
                            let expected_baseline = baseline_reference
                                .remove(&(files[position].clone(), packet.header.payload_offset))
                                .unwrap();
                            assert_eq!(
                                (baseline.views_completed, baseline.end_bit),
                                expected_baseline,
                                "baseline {}/{}",
                                files[position],
                                packet.header.payload_offset
                            );
                            *totals
                                .entry(format!("baseline_frame_views_{}", baseline.views_completed))
                                .or_default() += 1;
                            if baseline != **frame {
                                *totals
                                    .entry("frames_changed_by_continuation_bindings".into())
                                    .or_default() += 1;
                            }
                            if baseline.views_completed == 3 && frame.views_completed < 3 {
                                let cause = match &baseline.entity_end {
                                    Some(ProductionEntityEnd::Rejected {
                                        reason: FilmViewAdmission::Unbound,
                                        ..
                                    }) => "unbound_header",
                                    Some(ProductionEntityEnd::Rejected { .. }) => {
                                        "other_rejected_header"
                                    }
                                    Some(ProductionEntityEnd::Marker(_)) => "end_marker",
                                    _ => "other",
                                };
                                *totals
                                    .entry(format!("baseline_complete_now_partial_{cause}"))
                                    .or_default() += 1;
                                let reason = match &frame.entity_end {
                                    Some(ProductionEntityEnd::Failure(
                                        EntityViewStop::UnsupportedComponent { .. },
                                    )) => "unsupported_component",
                                    Some(ProductionEntityEnd::Failure(_)) => "other_record_failure",
                                    Some(ProductionEntityEnd::Rejected { .. }) => "rejected_header",
                                    Some(ProductionEntityEnd::Marker(_)) => "end_marker",
                                    _ => "other",
                                };
                                *totals
                                    .entry(format!("now_partial_reason_{reason}"))
                                    .or_default() += 1;
                            }
                            format!("frame_views_{}", frame.views_completed)
                        }
                        NativeFilmPacketBody::Keyframes(table) => {
                            for attempt in &table.records {
                                if attempt.record.is_some() {
                                    baseline_world.bind_keyframe(
                                        attempt.id >> 30,
                                        attempt.id & 0x3fff_ffff,
                                        attempt.archetype,
                                    );
                                }
                            }
                            *totals
                                .entry("sequential_keyframe_attempts".into())
                                .or_default() += table.records.len();
                            for record in table.records.iter().filter_map(|a| a.record.as_ref()) {
                                if let KeyframeStop::UnsupportedComponent { name, .. } =
                                    &record.stop
                                {
                                    *totals
                                        .entry(format!("keyframe_component_stop_{name}"))
                                        .or_default() += 1;
                                }
                            }
                            format!("keyframe_stop_{:?}", table.stop)
                        }
                        NativeFilmPacketBody::Datums(_) => "datum_packets".into(),
                        NativeFilmPacketBody::Roster(roster) => {
                            *totals.entry("roster_entries".into()).or_default() +=
                                roster.entries.len();
                            *totals.entry("roster_refused_entries".into()).or_default() +=
                                roster.report.refused;
                            "roster_packets".into()
                        }
                        NativeFilmPacketBody::Summary {
                            declared_events,
                            events,
                        } => {
                            *totals.entry("summary_declared".into()).or_default() +=
                                *declared_events as usize;
                            *totals.entry("summary_decoded".into()).or_default() += events.len();
                            // Source positions, not timestamps, determine native order.
                            assert!(
                                events
                                    .windows(2)
                                    .all(|w| w[0].source.unwrap().bit <= w[1].source.unwrap().bit)
                            );
                            "summary_packets".into()
                        }
                        NativeFilmPacketBody::Refused { .. } => "refused_packets".into(),
                        NativeFilmPacketBody::Opaque => {
                            format!("opaque_packet_{}", packet.header.packet_type)
                        }
                    };
                    *totals.entry(key).or_default() += 1;
                }
            }
            eprintln!(
                "native entry retained {}",
                path.strip_prefix(&root).unwrap().display()
            );
        }
        eprintln!(
            "native entry coverage: {}",
            serde_json::to_string(&totals).unwrap()
        );
        assert_eq!(totals["summary_declared"], 3667);
        assert_eq!(totals["summary_decoded"], 3667);
        assert!(highlight_reference.is_empty());
        assert!(reference.is_empty());
        assert!(fire_reference.is_empty());
        assert!(head_reference.is_empty());
        assert!(damage_reference.is_empty());
        assert_eq!(totals["candidate_reads"], 134657);
        assert_eq!(totals["candidate_complete"], 114357);
        assert!(baseline_reference.is_empty());
        assert_eq!(totals["baseline_frame_views_3"], expected_baseline_complete);
        assert_eq!(totals["damage_layout_body_end_differences"], 811);
        assert_eq!(totals["damage_layout_conflict_with_terminator"], 714);
        assert!(!totals.contains_key("event_list_stop_MissingRuntimeGate15"));
        assert!(!totals.contains_key("event_list_stop_RecordLimit"));
    }
}

#[cfg(test)]
#[path = "parser/native_data_profile_tests.rs"]
mod profile_tests;
