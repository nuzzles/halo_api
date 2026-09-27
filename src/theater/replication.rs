//! Packet-level integration for the sequential reference port.
use super::*;
use crate::clients::hi::models::FilmChunkData;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
// Preserve the existing by-value API while native frame records gain retained fields.
#[allow(clippy::large_enum_variant)]
pub enum ReplicationPayload {
    Frame(DecodedReplicationFrame),
    Keyframe(DecodedKeyframeTable),
    Datums(DatumTable),
    /// A rejected datum body; source identity is on the enclosing packet.
    InvalidDatums {
        message: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        native_error: Option<DatumTableError>,
        payload: Vec<u8>,
    },
    /// Other packet families retain their data until their grammar is ported.
    Opaque(Vec<u8>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplicationPacket {
    pub source: FilmPacket,
    pub payload: ReplicationPayload,
    /// Remaining payload bits, including padding or unsupported grammar.
    pub unparsed_tail: Option<ReplicationTail>,
    pub keyframe_recovery: Option<RecoveredKeyframe>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyframeRecoveryPolicy {
    #[default]
    SequentialOnly,
    LevelUp,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveredKeyframeRecord {
    pub anchor: RecoveredKeyframeAnchor,
    pub record: Option<KeyframeRecord>,
    /// A grammar walk crossing another recovered anchor is inconsistent with
    /// that anchor chain; neither boundary is silently accepted as proven.
    pub crosses_next_anchor: bool,
    /// Native zero-padded outcome, retained separately from the bounded read.
    /// None means unavailable in an older export, not an absent native record.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native_read: Option<NativeRecoveredKeyframeRead>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativeRecoveredKeyframeRead {
    /// The complete result equals `record`; share it instead of duplicating fields.
    SameAsBounded,
    Decoded {
        record: Box<KeyframeRecord>,
        crosses_next_anchor: bool,
    },
    Unavailable,
}

impl RecoveredKeyframeRecord {
    /// Borrow the native compatibility read, which may extend past the source.
    /// Compare its endpoint with the enclosing packet's payload size before
    /// treating fields as recorded; padded values do not establish observations.
    pub fn native_record(&self) -> Option<&KeyframeRecord> {
        match self.native_read.as_ref()? {
            NativeRecoveredKeyframeRead::SameAsBounded => self.record.as_ref(),
            NativeRecoveredKeyframeRead::Decoded { record, .. } => Some(record),
            NativeRecoveredKeyframeRead::Unavailable => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveredKeyframe {
    pub records: Vec<RecoveredKeyframeRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplicationTail {
    /// Absolute bit offset within this packet's payload.
    pub start_bit: usize,
    /// Bytes beginning at floor(start_bit / 8); leading consumed bits in the
    /// first byte are retained so the remaining bit sequence is lossless.
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplicationStream {
    pub encoding: FrameEncoding,
    pub packets: Vec<ReplicationPacket>,
    pub bindings: EntityBindings,
    pub recovery_policy: KeyframeRecoveryPolicy,
}

/// Walk indexed replication packets in chunk order using checked packet lengths.
/// Each keyframe starts a fresh binding table. Record failures remain in the
/// result; a failed frame is never scanned forward to invent a new boundary.
pub fn decode_replication_stream(
    chunks: &[FilmChunkData],
    registry: &FilmRegistry,
    encoding: FrameEncoding,
) -> Result<ReplicationStream, DecodeError> {
    decode_replication_stream_with_recovery(
        chunks,
        registry,
        encoding,
        KeyframeRecoveryPolicy::SequentialOnly,
    )
}

/// Add the reference's guarded recovery when requested. Recovered anchors and
/// their independently attempted record walks remain distinct from the sequential
/// table; updates inherit the provenance of the binding they use.
pub fn decode_replication_stream_with_recovery(
    chunks: &[FilmChunkData],
    registry: &FilmRegistry,
    encoding: FrameEncoding,
    recovery_policy: KeyframeRecoveryPolicy,
) -> Result<ReplicationStream, DecodeError> {
    if registry.major_version != 41 {
        return Err(DecodeError::UnsupportedVersion(
            registry.major_version as i32,
        ));
    }
    if !encoding.valid() {
        return Err(DecodeError::Inconsistent(
            "invalid replication encoding".into(),
        ));
    }
    let packets = packets::index(chunks)?;
    let chunks: BTreeMap<_, _> = chunks
        .iter()
        .map(|chunk| (chunk.metadata.index, &chunk.data))
        .collect();
    let mut result = ReplicationStream {
        encoding,
        packets: Vec::new(),
        bindings: EntityBindings::default(),
        recovery_policy,
    };
    for source in packets {
        let data = &chunks[&source.chunk_index]
            [source.payload_offset..source.payload_offset + source.payload_size];
        let mut keyframe_recovery = None;
        let payload = match source.packet_type {
            0 => match decode_replication_frame(
                data,
                registry,
                &result.encoding,
                &mut result.bindings,
            ) {
                Some(frame) => ReplicationPayload::Frame(frame),
                None => ReplicationPayload::Opaque(data.to_vec()),
            },
            1 => match decode_datum_table(data) {
                Ok(table) => ReplicationPayload::Datums(table),
                Err(error) => ReplicationPayload::InvalidDatums {
                    message: error.to_string(),
                    native_error: match &error {
                        DecodeError::Datums(reason) => Some(reason.clone()),
                        _ => None,
                    },
                    payload: data.to_vec(),
                },
            },
            2 => {
                result.bindings.slots.clear();
                if result.recovery_policy == KeyframeRecoveryPolicy::LevelUp {
                    let anchors = recover_keyframe_anchors(data);
                    let records = anchors
                        .iter()
                        .enumerate()
                        .map(|(i, anchor)| {
                            result.bindings.bind_with_origin(
                                anchor.id,
                                anchor.archetype,
                                BindingOrigin::RecoveredKeyframe { bit: anchor.bit },
                            );
                            let record = decode_keyframe_record_with_encoding(
                                data,
                                anchor.bit,
                                registry,
                                &result.encoding,
                            );
                            let crosses_next_anchor = record.as_ref().is_some_and(|r| {
                                anchors
                                    .get(i + 1)
                                    .is_some_and(|next| r.end_bit > (next.bit) as i64)
                            });
                            let native = decode_native_keyframe_record(
                                data,
                                anchor.bit,
                                registry,
                                &result.encoding,
                            );
                            let native_read = Some(match native {
                                Some(native) if record.as_ref() == Some(&native) => {
                                    NativeRecoveredKeyframeRead::SameAsBounded
                                }
                                Some(native) => NativeRecoveredKeyframeRead::Decoded {
                                    crosses_next_anchor: anchors
                                        .get(i + 1)
                                        .is_some_and(|next| native.end_bit > (next.bit) as i64),
                                    record: Box::new(native),
                                },
                                None => NativeRecoveredKeyframeRead::Unavailable,
                            });
                            RecoveredKeyframeRecord {
                                anchor: anchor.clone(),
                                record,
                                crosses_next_anchor,
                                native_read,
                            }
                        })
                        .collect();
                    keyframe_recovery = Some(RecoveredKeyframe { records });
                }
                ReplicationPayload::Keyframe(decode_keyframe_table(
                    data,
                    registry,
                    &result.encoding,
                    &mut result.bindings,
                ))
            }
            _ => ReplicationPayload::Opaque(data.to_vec()),
        };
        let consumed = match &payload {
            ReplicationPayload::Frame(frame) => frame.end_bit,
            ReplicationPayload::Keyframe(table) => table.end_bit,
            ReplicationPayload::Datums(_)
            | ReplicationPayload::InvalidDatums { .. }
            | ReplicationPayload::Opaque(_) => (data.len() * 8) as i64,
        };
        let unparsed_tail =
            (0..(data.len() * 8) as i64)
                .contains(&consumed)
                .then(|| ReplicationTail {
                    start_bit: super::bits::native_address(consumed),
                    bytes: data[super::bits::native_address(consumed) / 8..].to_vec(),
                });
        result.packets.push(ReplicationPacket {
            source,
            payload,
            unparsed_tail,
            keyframe_recovery,
        });
    }
    Ok(result)
}

#[allow(dead_code)]
impl LegacyFilm {
    fn retain_movement_scan(&mut self, chunks: &[FilmChunkData], encoding: &FrameEncoding) {
        let (stream, error) =
            super::scan_movement_states_with_diagnostics(chunks, &self.registry, encoding);
        self.movement_states_error_stats = error.as_ref().map(|_| stream.stats.clone());
        self.movement_states_error = error.map(|error| error.to_string());
        self.movement_states = Some(stream);
    }
    fn retain_objective_scan(&mut self, chunks: &[FilmChunkData], encoding: &FrameEncoding) {
        match super::scan_objectives(chunks, &self.registry, encoding) {
            Ok(scan) => self.objective_scan = Some(scan),
            Err(failure) => {
                self.objective_scan = Some(*failure.scan);
                self.objective_scan_error = Some(failure.error.to_string());
            }
        }
    }
    /// Resolve the recorded build/format and the supplied map's catalog entry,
    /// then decode replication with guarded keyframe recovery. Map identity must
    /// come from match metadata; ID widths must be established independently.
    /// Missing required context is an error, never another map's fallback.
    pub fn try_from_chunks_with_map(
        chunks: &[FilmChunkData],
        options: DecodeOptions,
        map_name: &str,
        ids: RecordIdLayout,
    ) -> Result<Self, DecodeError> {
        Self::try_from_chunks_with_map_context(chunks, options, map_name, None, ids, None)
    }

    /// Decode a custom map with independently established quantization bounds.
    /// Retains its actual name instead of borrowing another catalog map's identity.
    pub fn try_from_chunks_with_map_bounds(
        chunks: &[FilmChunkData],
        options: DecodeOptions,
        map_name: &str,
        map: super::FilmMapBounds,
        ids: RecordIdLayout,
    ) -> Result<Self, DecodeError> {
        Self::try_from_chunks_with_map_context(chunks, options, map_name, Some(map), ids, None)
    }

    /// Decode kill sources once, retain all attribution/diagnostics, and pass their
    /// calibrated grammar into map-aware replication and replay fact extraction.
    /// Kill-source refusal is retained and does not discard the rest of the film.
    pub fn try_from_chunks_with_map_and_kill_sources(
        chunks: &[FilmChunkData],
        options: DecodeOptions,
        map_name: &str,
        map: Option<super::FilmMapBounds>,
        ids: RecordIdLayout,
        kill_options: &super::KillDecodeOptions,
    ) -> Result<Self, DecodeError> {
        Self::try_from_chunks_with_map_context(
            chunks,
            options,
            map_name,
            map,
            ids,
            Some(kill_options),
        )
    }

    fn try_from_chunks_with_map_context(
        chunks: &[FilmChunkData],
        options: DecodeOptions,
        map_name: &str,
        map: Option<super::FilmMapBounds>,
        ids: RecordIdLayout,
        kill_options: Option<&super::KillDecodeOptions>,
    ) -> Result<Self, DecodeError> {
        let mut film = Self::try_from_chunks(chunks, options)?;
        let mut profile =
            resolve_v41_profile(&film.registry, film.identity.as_ref(), Some(map_name))?;
        profile.warn_incomplete(true);
        if let Some(map) = map {
            profile.map = Some(map);
            profile.issues.retain(|issue| {
                !matches!(
                    issue,
                    super::FilmProfileIssue::UnknownMap(_) | super::FilmProfileIssue::MissingMap
                )
            });
        }
        let inherited = kill_options.map(|kill_options| {
            match super::decode_film_kill_sources(
                film.match_id.as_deref().unwrap_or(""),
                chunks,
                film.major_version,
                profile.map.as_ref(),
                kill_options,
            ) {
                Ok(result) => {
                    let inherited = result.calibration.profile.clone();
                    film.kill_sources = Some(result);
                    inherited
                }
                Err(error) => {
                    film.kill_sources_error = Some(error.to_string());
                    super::kill_replay_starting_profile(&film.registry, profile.map.as_ref())
                }
            }
        });
        let precision = super::resolve_replay_precision_context(
            chunks,
            profile
                .map
                .as_ref()
                .ok_or(DecodeError::Missing("map precision"))?,
            inherited.clone().unwrap_or_else(|| {
                super::kill_replay_starting_profile(&film.registry, profile.map.as_ref())
            }),
        )?;
        let sampling_map = precision.sampling_map.as_ref().ok_or_else(|| {
            DecodeError::Inconsistent(format!("inconclusive i0 layout: {:?}", precision.detection))
        })?;
        let world_map = &precision.world_map;
        let mut encoding_profile = profile.clone();
        encoding_profile.map = Some(world_map.clone());
        let mut encoding = encoding_profile.frame_encoding(ids).ok_or_else(|| {
            DecodeError::Inconsistent(format!(
                "cannot resolve replication encoding: {:?}; ID layout: {ids:?}",
                profile.issues
            ))
        })?;
        encoding.position = precision.profile.encoding.position.clone();
        encoding.keyframe_simulation_complete = Some(precision.profile.simulation_complete);
        if let Some(inherited) = &inherited {
            encoding.mpp_widths = inherited.encoding.mpp_widths;
            encoding.corruption_check = inherited.encoding.corruption_check;
        }
        film.corruption_control().apply(&mut encoding);
        film.player_teams = Some(super::scan_film_player_teams_with_context(
            chunks,
            Some(&film.registry),
            &encoding,
        ));
        // Preserve the full calibrated profile through native profile-then-map
        // composition. Only the frame ID width is recalibrated by this scan.
        let native_inherited = inherited.as_ref().map(|_| {
            film.kill_sources
                .as_ref()
                .and_then(|result| result.calibration.native_profile.clone())
                .unwrap_or_else(|| kill_replay_starting_native_profile(profile.map.as_ref()))
        });
        let march_config = native_replay_march_config(
            profile.map.as_ref().unwrap(),
            native_inherited.as_ref(),
            film.corruption_control(),
        );
        film.native_march_facts = Some(
            scan_film_march_facts_with_native_config(chunks, &film.registry, &march_config)
                .map_err(|error| DecodeError::Inconsistent(format!("native march: {error}")))?,
        );
        film.retain_movement_scan(chunks, &encoding);
        let (navpoint, navpoint_error) = super::scan_navpoint_radial_with_diagnostics(
            chunks,
            &film.registry,
            &encoding,
            &chunks
                .iter()
                .map(|c| (c.metadata.index, c.metadata.start_time_offset_ms))
                .collect(),
        );
        film.navpoint_radial = Some(navpoint);
        film.navpoint_radial_error = navpoint_error.map(|error| error.to_string());
        let (managed, managed_error) =
            super::scan_managed_properties_with_diagnostics(chunks, &film.registry, &encoding);
        film.managed_properties = Some(managed);
        film.managed_properties_error = managed_error.map(|error| error.to_string());
        let (equipment_state, equipment_error) =
            super::scan_equipment_state_with_diagnostics(chunks, &film.registry, &encoding);
        film.equipment_state = Some(equipment_state);
        film.equipment_state_error = equipment_error.map(|error| error.to_string());
        film.retain_objective_scan(chunks, &encoding);
        film.replication = Some(decode_replication_stream_with_recovery(
            chunks,
            &film.registry,
            encoding,
            KeyframeRecoveryPolicy::LevelUp,
        )?);
        film.retain_keyframe_measurements();
        let component_position = precision
            .profile
            .encoding
            .position
            .clone()
            .ok_or(DecodeError::Missing("component position precision"))?;
        film.translocations = scan_translocator_events(chunks, Some(world_map))?;
        film.native_translocations = scan_native_translocator_events(chunks, Some(world_map));
        {
            let map = sampling_map;
            for archetype in [37, 42] {
                if let Some(census) = film
                    .world_object_keyframes
                    .get(&archetype)
                    .filter(|c| !c.band.is_empty())
                {
                    film.ground_object_tracks.insert(
                        archetype,
                        super::scan_world_object_tracks_for_band(chunks, world_map, &census.band)?,
                    );
                }
            }
            if let Some(census) = film
                .world_object_keyframes
                .get(&37)
                .filter(|c| !c.band.is_empty())
            {
                film.equipment_placements =
                    Some(super::scan_equipment_placements_for_band_with_position(
                        chunks,
                        world_map,
                        &census.band,
                        &film.registry,
                        &component_position,
                    )?);
            }
            let calibrated = film.equipment_placements.as_ref().map(|scan| {
                let w = scan.stats.calibration.widths;
                [w.lead, w.index]
            });
            let creation_widths = super::resolve_film_mpp(profile.format_version)
                .creation_widths(calibrated, super::NATIVE_MPP_DEFAULT_WIDTHS);
            film.native_vehicles = Some(super::film_vehicles::scan_film_vehicle_facts_observed(
                chunks,
                &film.registry,
                map,
                creation_widths,
                &component_position,
                Some(film.match_id.as_deref().unwrap_or_default()),
            ));
            if film.native_vehicles.as_ref().is_some_and(|v| v.scanned)
                && let Some(march) = &film.native_march_facts
            {
                super::film_vehicles::log_vehicle_death_reads(
                    film.match_id.as_deref().unwrap_or_default(),
                    march,
                );
            }
            if let Some(facts) = film.native_vehicles.as_ref().filter(|v| v.scanned) {
                super::film_vehicles::log_vehicle_scan_summary(
                    facts,
                    film.native_march_facts.as_ref(),
                );
            }
            if let Some(census) = film
                .world_object_keyframes
                .get(&37)
                .filter(|c| !c.band.is_empty())
            {
                // Placement confirmation already read the same creation stream when
                // it had declared or calibrated widths. Reuse those raw readings;
                // an inconclusive placement scan needs the independent pad scan.
                let placement_read = profile.mpp_widths.is_some()
                    || calibrated.is_some_and(|w| w.iter().all(|&v| v > 0));
                film.equipment_pad_creations = Some(
                    if let Some(scan) = film
                        .equipment_placements
                        .as_ref()
                        .filter(|_| placement_read)
                    {
                        scan.creations.clone()
                    } else {
                        super::scan_equipment_creations_for_band_with_position(
                            chunks,
                            world_map,
                            &census.band,
                            &film.registry,
                            creation_widths,
                            &component_position,
                        )?
                    },
                );
            }
            if let Some(census) = film
                .world_object_keyframes
                .get(&42)
                .filter(|c| !c.band.is_empty())
            {
                let mut creation_encoding = precision.profile.encoding.clone();
                creation_encoding.mpp_widths = creation_widths;
                creation_encoding.keyframe_simulation_complete =
                    Some(precision.profile.simulation_complete);
                film.corruption_control().apply(&mut creation_encoding);
                film.ground_weapon_creations =
                    Some(super::scan_ground_weapon_creations_for_band_with_encoding(
                        chunks,
                        world_map,
                        &census.band,
                        &film.registry,
                        &creation_encoding,
                    )?);
            }
            if let Some(census) = film
                .world_object_keyframes
                .get(&41)
                .filter(|c| !c.band.is_empty())
            {
                film.native_projectiles = Some(super::scan_world_object_tracks_for_band(
                    chunks,
                    world_map,
                    &census.band,
                )?);
            }
            let positions = scan_biped_positions(
                chunks,
                map,
                BipedScanOptions::default(),
                &film.translocations,
            )?;
            let biped_context = FrameEncoding {
                position: Some(component_position.clone()),
                keyframe_simulation_complete: Some(precision.profile.simulation_complete),
                native_id_low_bits: None,
                ..precision.profile.encoding.clone()
            };
            let channels = super::scan_biped_channels_with_context(
                chunks,
                &film.registry,
                &positions,
                &biped_context,
            );
            if positions.slot_band.is_some() {
                match super::scan_held_weapon_changes_with_context(
                    chunks,
                    &film.registry,
                    &positions,
                    &biped_context,
                    &film.keyframe_loadouts,
                ) {
                    Ok(scan) => film.weapon_changes = Some(scan),
                    Err(error) => film.weapon_changes_error = Some(error.to_string()),
                }
            }
            match channels {
                Ok(channels) => {
                    match super::scan_equipment_changes_with_context(
                        chunks,
                        &film.registry,
                        &positions,
                        &channels,
                        map,
                        &biped_context,
                    ) {
                        Ok(scan) => film.equipment_changes = Some(scan),
                        Err(error) => film.equipment_changes_error = Some(error.to_string()),
                    }
                    film.biped_channels = Some(channels);
                }
                Err(error) => {
                    film.equipment_changes_error =
                        Some(format!("equipment input channels unavailable: {error}"));
                    film.biped_channels_error = Some(error.to_string());
                }
            }
            match super::scan_inventory_deltas_with_context(
                chunks,
                &film.registry,
                &positions,
                &biped_context,
            ) {
                Ok(scan) => film.inventory_deltas = Some(scan),
                Err(error) => film.inventory_deltas_error = Some(error.to_string()),
            }
            match super::scan_biped_ability_states_with_context(
                chunks,
                &film.registry,
                &positions,
                &biped_context,
            ) {
                Ok(scan) => film.ability_states = Some(scan),
                Err(error) => film.ability_states_error = Some(error.to_string()),
            }
            match super::scan_ability_charges_with_context(
                chunks,
                &film.registry,
                &positions,
                &biped_context,
            ) {
                Ok(scan) => film.ability_charges = Some(scan),
                Err(error) => film.ability_charges_error = Some(error.to_string()),
            }
            match super::scan_unit_equipment_with_context(
                chunks,
                &film.registry,
                &positions,
                &biped_context,
            ) {
                Ok(scan) => film.unit_equipment = Some(scan),
                Err(error) => film.unit_equipment_error = Some(error.to_string()),
            }
            film.biped_positions = Some(positions);
        }
        film.scan_precision = Some(precision);
        film.profile = Some(profile);
        // Independent hit-distance failure must not discard other film channels.
        // The refresh retains its error and any previously decoded direct hits.
        let _ = film.retain_weapon_hits(chunks);
        Ok(film)
    }

    /// Decode existing observations and attach replication records, including
    /// LevelUp's guarded keyframe recovery with explicit binding provenance.
    /// The caller supplies established map/ID widths; bootstrap identity supplies
    /// the actual corruption-check flag, or the native false invariant when absent.
    /// This remains partial:
    /// unsupported component and view payloads are reported at their boundaries.
    pub fn try_from_chunks_with_encoding(
        chunks: &[FilmChunkData],
        options: DecodeOptions,
        mut encoding: FrameEncoding,
    ) -> Result<Self, DecodeError> {
        let mut film = Self::try_from_chunks(chunks, options)?;
        film.corruption_control().apply(&mut encoding);
        film.player_teams = Some(super::scan_film_player_teams_with_context(
            chunks,
            Some(&film.registry),
            &encoding,
        ));
        film.retain_movement_scan(chunks, &encoding);
        let (navpoint, navpoint_error) = super::scan_navpoint_radial_with_diagnostics(
            chunks,
            &film.registry,
            &encoding,
            &chunks
                .iter()
                .map(|c| (c.metadata.index, c.metadata.start_time_offset_ms))
                .collect(),
        );
        film.navpoint_radial = Some(navpoint);
        film.navpoint_radial_error = navpoint_error.map(|error| error.to_string());
        let (managed, managed_error) =
            super::scan_managed_properties_with_diagnostics(chunks, &film.registry, &encoding);
        film.managed_properties = Some(managed);
        film.managed_properties_error = managed_error.map(|error| error.to_string());
        let (equipment_state, equipment_error) =
            super::scan_equipment_state_with_diagnostics(chunks, &film.registry, &encoding);
        film.equipment_state = Some(equipment_state);
        film.equipment_state_error = equipment_error.map(|error| error.to_string());
        film.retain_objective_scan(chunks, &encoding);
        film.replication = Some(decode_replication_stream_with_recovery(
            chunks,
            &film.registry,
            encoding,
            KeyframeRecoveryPolicy::LevelUp,
        )?);
        film.retain_keyframe_measurements();
        Ok(film)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clients::hi::models::FilmChunk;
    use std::io::Read;

    #[test]
    fn map_constructor_retains_independent_missing_archetype_errors() {
        let mut bootstrap = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/bootstrap-v41.zlib")[..])
            .read_to_end(&mut bootstrap)
            .unwrap();
        // Keep a complete terminating block, but no navpoint or biped archetype.
        bootstrap.truncate(8 + 13 * 64 * 260);
        bootstrap[8 + 12 * 64 * 260] = 0xff;
        let mut inputs = vec![bootstrap];
        for (slot, archetype) in [(512, 12), (513, 35), (514, 13)] {
            let body = bits(&[
                (0, 1),
                ((1 << 30) | slot, 32),
                (archetype, 32),
                (0, 44),
                (0, 32),
                (0, 32),
                (0, 64),
            ]);
            let mut packets = Vec::new();
            packets.extend_from_slice(&2u16.to_le_bytes());
            packets.extend_from_slice(&[0, 0]);
            packets.extend_from_slice(&(body.len() as u32).to_le_bytes());
            packets.extend_from_slice(&1000u64.to_le_bytes());
            packets.extend(body);
            inputs.push(packets);
        }
        let chunks: Vec<_> = inputs
            .into_iter()
            .enumerate()
            .map(|(index, data)| FilmChunkData {
                metadata: FilmChunk {
                    index: index as i32,
                    chunk_type: if index == 0 { 1 } else { 2 },
                    start_time_offset_ms: 0,
                    duration_ms: 1,
                    size: data.len() as i64,
                    file_relative_path: String::new(),
                },
                data,
            })
            .collect();
        assert_eq!(
            super::super::biped_scan::biped_slot_band(&chunks).unwrap(),
            Some([513, 513])
        );
        let film = LegacyFilm::try_from_chunks_with_map(
            &chunks,
            DecodeOptions::v41(),
            "Live Fire - Ranked",
            RecordIdLayout {
                low_bits: 13,
                base: 0,
            },
        )
        .unwrap();
        assert_eq!(film.managed_properties.as_ref().unwrap().slots, 1);
        assert!(film.managed_properties.as_ref().unwrap().reads.is_empty());
        assert!(
            film.managed_properties_error
                .as_ref()
                .unwrap()
                .contains("managed-property archetype 13")
        );
        let explicit =
            LegacyFilm::try_from_chunks_with_encoding(&chunks, DecodeOptions::v41(), encoding())
                .unwrap();
        assert_eq!(explicit.managed_properties, film.managed_properties);
        assert_eq!(
            explicit.managed_properties_error,
            film.managed_properties_error
        );
        let restored_explicit: LegacyFilm =
            serde_json::from_slice(&serde_json::to_vec(&explicit).unwrap()).unwrap();
        assert_eq!(restored_explicit, explicit);
        assert_eq!(
            film.movement_states.as_ref().unwrap().stats,
            *film.movement_states_error_stats.as_ref().unwrap()
        );
        assert!(explicit.movement_states.is_some());
        assert!(explicit.movement_states_error.is_some());
        let raw_movement_stats = film.movement_states_error_stats.as_ref().unwrap();
        assert_eq!(
            raw_movement_stats.map_widths,
            film.scan_precision
                .as_ref()
                .unwrap()
                .world_map
                .axis_widths
                .map(|v| v as u64)
        );
        assert_ne!(raw_movement_stats.map_widths, [0; 3]);
        assert!(!raw_movement_stats.scanned);
        assert_eq!(raw_movement_stats.read, 0);
        let radial = film.navpoint_radial.as_ref().unwrap();
        assert_eq!(radial.slots_observed, 1);
        assert_eq!(radial.slots_band, 1);
        assert_eq!(radial.key_census, 1);
        assert!(radial.reads.is_empty());
        assert!(film.biped_channels.is_none());
        assert!(film.ability_states.is_none());
        assert!(film.ability_charges.is_none());
        assert!(film.equipment_changes.is_none());
        for error in [
            &film.movement_states_error,
            &film.biped_channels_error,
            &film.ability_states_error,
            &film.ability_charges_error,
            &film.inventory_deltas_error,
            &film.weapon_changes_error,
            &film.unit_equipment_error,
        ] {
            assert!(error.as_ref().unwrap().contains("biped archetype"));
        }
        assert!(
            film.navpoint_radial_error
                .as_ref()
                .unwrap()
                .contains("navpoint archetype 12")
        );
        assert!(
            film.equipment_changes_error
                .as_ref()
                .unwrap()
                .contains("input channels unavailable")
        );
        assert!(film.replication.is_some());
        assert!(film.native_march_facts.is_some());
        assert!(film.biped_positions.is_some());
        assert!(film.scan_precision.is_some());
        let restored: LegacyFilm =
            serde_json::from_slice(&serde_json::to_vec(&film).unwrap()).unwrap();
        assert_eq!(restored, film);
        assert!(film.biped_creations_error.is_none());
        assert!(explicit.biped_creations_error.is_none());
        let mut missing = chunks.clone();
        missing[2].data = missing[1].data.clone();
        missing[2].metadata.size = missing[2].data.len() as i64;
        for missing_film in [
            LegacyFilm::try_from_chunks_with_encoding(&missing, DecodeOptions::v41(), encoding())
                .unwrap(),
            LegacyFilm::try_from_chunks_with_map(
                &missing,
                DecodeOptions::v41(),
                "Live Fire - Ranked",
                RecordIdLayout {
                    low_bits: 13,
                    base: 0,
                },
            )
            .unwrap(),
        ] {
            assert!(
                missing_film
                    .biped_creations_error
                    .as_ref()
                    .unwrap()
                    .contains("biped creation slot band")
            );
            assert!(missing_film.biped_creations.records.is_empty());
            assert_eq!(missing_film.packets, film.packets);
            assert!(missing_film.replication.is_some());
            let restored: LegacyFilm =
                serde_json::from_slice(&serde_json::to_vec(&missing_film).unwrap()).unwrap();
            assert_eq!(restored, missing_film);
        }
    }

    #[test]
    fn film_constructors_retain_positive_native_objective_scan() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/objective-scan-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        // Positive complete and broken walks with no calibrated widths or corruption
        // guards. Encode each native registry into the LegacyFilm bootstrap too.
        for case in [1, 2, 4, 19] {
            let row = &rows[case];
            assert_eq!(row["check"], false);
            let missing_arch = row["missing_arch"].as_bool().unwrap();
            let mut bootstrap = vec![0; 8 + 13 * 64 * 260];
            bootstrap[..4].copy_from_slice(&41u32.to_le_bytes());
            bootstrap[4..8].copy_from_slice(&27u32.to_le_bytes());
            for (index, name) in row["names"].as_array().unwrap().iter().enumerate() {
                let name = name.as_str().unwrap().as_bytes();
                let offset = 8 + (11 * 64 + index) * 260;
                bootstrap[offset..offset + name.len()].copy_from_slice(name);
            }
            bootstrap[8 + (if missing_arch { 11 } else { 12 }) * 64 * 260] = 0xff;
            let chunk = |index, kind, data: Vec<u8>| FilmChunkData {
                metadata: FilmChunk {
                    index,
                    chunk_type: kind,
                    start_time_offset_ms: 0,
                    duration_ms: 1,
                    size: data.len() as i64,
                    file_relative_path: String::new(),
                },
                data,
            };
            let mut chunks = vec![chunk(0, 1, bootstrap)];
            for c in row["chunks"].as_array().unwrap() {
                let h = c["hex"].as_str().unwrap();
                let data = (0..h.len())
                    .step_by(2)
                    .map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap())
                    .collect();
                chunks.push(chunk(c["index"].as_i64().unwrap() as i32, 2, data));
            }
            let mut custom = encoding();
            custom.ids.low_bits = 13;
            let expected: super::super::ObjectiveScan =
                serde_json::from_value(row["output"].clone()).unwrap();
            assert_eq!(expected.reads.is_empty(), missing_arch);
            for film in [
                LegacyFilm::try_from_chunks_with_encoding(&chunks, DecodeOptions::v41(), custom)
                    .unwrap(),
                LegacyFilm::try_from_chunks_with_map(
                    &chunks,
                    DecodeOptions::v41(),
                    "Live Fire - Ranked",
                    RecordIdLayout {
                        low_bits: 13,
                        base: 0,
                    },
                )
                .unwrap(),
            ] {
                assert_eq!(
                    film.objective_scan_error.is_some(),
                    missing_arch,
                    "case {case}"
                );
                if missing_arch {
                    assert!(
                        film.objective_scan_error
                            .as_ref()
                            .unwrap()
                            .contains("managed-objective archetype 11")
                    );
                }
                let scan = film.objective_scan.as_ref().unwrap();
                assert_eq!(scan.attempts.is_empty(), missing_arch);
                if !missing_arch {
                    assert!(scan.attempts.iter().any(|a| matches!(
                        a.data,
                        super::super::ObjectiveAttemptData::Delta { .. }
                    )));
                    assert!(scan.attempts.iter().any(|a| matches!(
                        a.data,
                        super::super::ObjectiveAttemptData::Keyframe { .. }
                    )));
                }
                let mut accepted = scan.clone();
                accepted.attempts.clear();
                assert_eq!(accepted, expected);
                for attempt in &scan.attempts {
                    let c = chunks
                        .iter()
                        .find(|c| c.metadata.index == attempt.source.chunk_index)
                        .unwrap();
                    assert_eq!(
                        super::super::fire_events::native_chunk_packets(c)[attempt.packet_index],
                        attempt.source
                    );
                }
                let mut emissions = Vec::new();
                for attempt in &scan.attempts {
                    let (keyframe, diagnostics) = match &attempt.data {
                        super::super::ObjectiveAttemptData::Delta { component, .. } => {
                            (false, &component.diagnostics)
                        }
                        super::super::ObjectiveAttemptData::Keyframe {
                            record: Some(record),
                        } => (true, &record.diagnostics),
                        super::super::ObjectiveAttemptData::Keyframe { record: None } => continue,
                    };
                    for observation in &diagnostics.component_observations {
                        if let super::super::FilmComponentObservation::Objective { field, values } =
                            observation
                        {
                            emissions.push(serde_json::json!({"chunk":attempt.source.chunk_index,"packet":attempt.packet_index,"keyframe":keyframe,"field":field.native_code(),"values":values}));
                        }
                    }
                }
                assert_eq!(
                    serde_json::json!(emissions),
                    row["emissions"],
                    "Film callbacks case {case}"
                );
                let restored: LegacyFilm =
                    serde_json::from_slice(&serde_json::to_vec(&film).unwrap()).unwrap();
                assert_eq!(restored, film);
            }
        }
    }

    #[test]
    fn film_constructors_retain_positive_native_navpoint_scan() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/navpoint-source-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(rows.len(), 64);
        let mut positive_keyframes = 0;
        let mut positive_deltas = 0;
        let mut broken = 0;
        for row in rows {
            let case = row["case"].as_u64().unwrap();
            let mut bootstrap = vec![0; 8 + 14 * 64 * 260];
            bootstrap[..4].copy_from_slice(&41u32.to_le_bytes());
            bootstrap[4..8].copy_from_slice(&27u32.to_le_bytes());
            for (index, name) in row["names"].as_array().unwrap().iter().enumerate() {
                let name = name.as_str().unwrap().as_bytes();
                let offset = 8 + (12 * 64 + index) * 260;
                bootstrap[offset..offset + name.len()].copy_from_slice(name);
            }
            bootstrap[8 + 13 * 64 * 260] = 0xff;
            let h = row["hex"].as_str().unwrap();
            let data = (0..h.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap())
                .collect();
            let chunks: Vec<_> = [bootstrap, data]
                .into_iter()
                .enumerate()
                .map(|(i, data)| FilmChunkData {
                    metadata: FilmChunk {
                        index: i as i32,
                        chunk_type: if i == 0 { 1 } else { 2 },
                        start_time_offset_ms: if i == 0 {
                            0
                        } else {
                            row["start"].as_i64().unwrap()
                        },
                        duration_ms: 10000,
                        size: data.len() as i64,
                        file_relative_path: String::new(),
                    },
                    data,
                })
                .collect();
            let mut custom = encoding();
            custom.ids.low_bits = 13;
            let expected: super::super::NavpointRadialScan =
                serde_json::from_value(row["output"].clone()).unwrap();
            broken += usize::from(expected.broken > 0 || expected.key_broken > 0);
            for film in [
                LegacyFilm::try_from_chunks_with_encoding(&chunks, DecodeOptions::v41(), custom)
                    .unwrap(),
                LegacyFilm::try_from_chunks_with_map(
                    &chunks,
                    DecodeOptions::v41(),
                    "Live Fire - Ranked",
                    RecordIdLayout {
                        low_bits: 13,
                        base: 0,
                    },
                )
                .unwrap(),
            ] {
                assert!(film.navpoint_radial_error.is_none(), "case {case}");
                let scan = film.navpoint_radial.as_ref().unwrap();
                let mut accepted = scan.clone();
                accepted.attempts.clear();
                assert_eq!(accepted, expected, "case {case}");
                let mut emissions = Vec::new();
                for attempt in &scan.attempts {
                    let source = attempt.source.as_ref().unwrap();
                    let ordinal = attempt.packet_index.unwrap();
                    assert_eq!(source.chunk_index, 1);
                    assert_eq!(
                        &super::super::fire_events::native_chunk_packets(&chunks[1])[ordinal],
                        source
                    );
                    assert!(attempt.record_start_bit < source.payload_size * 8);
                    assert_eq!(
                        i64::from(attempt.time_ms),
                        row["start"].as_i64().unwrap()
                            + (source.timestamp_us as i64 - ((case + 5000) * 1000) as i64) / 1000
                    );
                    let (keyframe, diagnostics) = match &attempt.data {
                        super::super::NavpointAttemptData::Delta { component, .. } => {
                            (false, &component.diagnostics)
                        }
                        super::super::NavpointAttemptData::Keyframe {
                            record: Some(record),
                        } => (true, &record.diagnostics),
                        super::super::NavpointAttemptData::Keyframe { record: None } => continue,
                    };
                    for observation in &diagnostics.component_observations {
                        if let super::super::FilmComponentObservation::Navpoint { field, values } =
                            observation
                        {
                            let field = match field {
                                super::super::NativeNavpointField::RadialProgress => 0,
                                super::super::NativeNavpointField::ManualTimerInitial => 1,
                                super::super::NativeNavpointField::ManualTimerCurrent => 2,
                            };
                            positive_keyframes += usize::from(keyframe);
                            positive_deltas += usize::from(!keyframe);
                            emissions.push(serde_json::json!({"keyframe":keyframe,"field":field,"values":values}));
                        }
                    }
                }
                assert_eq!(
                    serde_json::json!(emissions),
                    row["emissions"],
                    "callbacks case {case}"
                );
                let restored: LegacyFilm =
                    serde_json::from_slice(&serde_json::to_vec(&film).unwrap()).unwrap();
                assert_eq!(restored, film);
            }
        }
        assert!(positive_keyframes > 0 && positive_deltas > 0 && broken > 0);
    }

    #[test]
    fn film_constructors_retain_positive_native_managed_properties() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/managed-source-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(rows.len(), 54);
        let mut rejected_callbacks = 0;
        for row in rows {
            let case = row["case"].as_u64().unwrap();
            let chunks: Vec<_> = ["bootstrap", "hex"]
                .into_iter()
                .enumerate()
                .map(|(i, key)| {
                    let h = row[key].as_str().unwrap();
                    let data: Vec<u8> = (0..h.len())
                        .step_by(2)
                        .map(|j| u8::from_str_radix(&h[j..j + 2], 16).unwrap())
                        .collect();
                    FilmChunkData {
                        metadata: FilmChunk {
                            index: i as i32,
                            chunk_type: if i == 0 { 1 } else { 2 },
                            start_time_offset_ms: 12345,
                            duration_ms: 10000,
                            size: data.len() as i64,
                            file_relative_path: String::new(),
                        },
                        data,
                    }
                })
                .collect();
            let mut custom = encoding();
            custom.ids.low_bits = 13;
            let expected: super::super::ManagedPropertyScan =
                serde_json::from_value(row["output"].clone()).unwrap();
            assert!(!expected.reads.is_empty());
            for film in [
                LegacyFilm::try_from_chunks_with_encoding(&chunks, DecodeOptions::v41(), custom)
                    .unwrap(),
                LegacyFilm::try_from_chunks_with_map(
                    &chunks,
                    DecodeOptions::v41(),
                    "Live Fire - Ranked",
                    RecordIdLayout {
                        low_bits: 13,
                        base: 0,
                    },
                )
                .unwrap(),
            ] {
                assert!(film.managed_properties_error.is_none(), "case {case}");
                assert_eq!(super::replay_managed_property_reads(&film), expected.reads);
                // Inject a retained partial-result/error pair to exercise the
                // consumer contract without claiming this source failed late.
                let mut failed = film.clone();
                failed.managed_properties_error = Some("managed scan failed".into());
                assert!(super::replay_managed_property_reads(&failed).is_empty());
                let restored: LegacyFilm =
                    serde_json::from_value(serde_json::to_value(&failed).unwrap()).unwrap();
                assert_eq!(restored.managed_properties, failed.managed_properties);
                assert!(super::replay_managed_property_reads(&restored).is_empty());

                let scan = film.managed_properties.as_ref().unwrap();
                let mut accepted = scan.clone();
                accepted.attempts.clear();
                assert_eq!(accepted, expected, "case {case}");
                let packets = super::super::fire_events::native_chunk_packets(&chunks[1]);
                let mut emissions = Vec::new();
                for attempt in &scan.attempts {
                    let source = attempt.source.as_ref().unwrap();
                    let ordinal = attempt.packet_index.unwrap();
                    assert!(ordinal >= 2); // Keyframe and unrelated packet precede deltas.
                    assert_eq!(source, &packets[ordinal]);
                    assert_eq!(source.chunk_index, 1);
                    assert_eq!(attempt.timestamp_us, source.timestamp_us);
                    assert!(attempt.record_start_bit < source.payload_size * 8);
                    for observation in &attempt.component.diagnostics.component_observations {
                        if let super::super::FilmComponentObservation::ManagedProperty {
                            field,
                            values,
                        } = observation
                        {
                            let field = match field {
                                super::super::NativeManagedPropertyField::Scalar => 0,
                                super::super::NativeManagedPropertyField::PerPlayer => 1,
                            };
                            rejected_callbacks +=
                                usize::from(attempt.status != Some(true) || !attempt.in_bounds);
                            emissions.push(serde_json::json!({"packet":ordinal,"timestamp_us":attempt.timestamp_us,"field":field,"values":values}));
                        }
                    }
                }
                assert_eq!(
                    serde_json::json!(emissions),
                    row["emissions"],
                    "callbacks {case}"
                );
                let restored: LegacyFilm =
                    serde_json::from_slice(&serde_json::to_vec(&film).unwrap()).unwrap();
                assert_eq!(restored, film);
            }
        }
        assert!(rejected_callbacks > 0);
    }

    #[test]
    fn film_retains_native_padded_translocator_without_promoting_actor() {
        let mut bootstrap = vec![0; 8 + 64 * 260];
        bootstrap[..4].copy_from_slice(&41u32.to_le_bytes());
        bootstrap[4..8].copy_from_slice(&27u32.to_le_bytes());
        bootstrap[8] = 0xff;
        let mut data = vec![0u8; 16];
        data[4..8].copy_from_slice(&2u32.to_le_bytes());
        data[8..16].copy_from_slice(&123u64.to_le_bytes());
        data.extend_from_slice(&[0xfa, 0xc0]);
        let chunks: Vec<_> = [bootstrap, data]
            .into_iter()
            .enumerate()
            .map(|(i, data)| FilmChunkData {
                metadata: FilmChunk {
                    index: i as i32,
                    chunk_type: if i == 0 { 1 } else { 2 },
                    start_time_offset_ms: 0,
                    duration_ms: 1,
                    size: data.len() as i64,
                    file_relative_path: String::new(),
                },
                data,
            })
            .collect();
        for film in [
            LegacyFilm::try_from_chunks(&chunks, DecodeOptions::v41()).unwrap(),
            LegacyFilm::try_from_chunks_with_encoding(&chunks, DecodeOptions::v41(), encoding())
                .unwrap(),
            LegacyFilm::try_from_chunks_with_map(
                &chunks,
                DecodeOptions::v41(),
                "Live Fire - Ranked",
                RecordIdLayout {
                    low_bits: 13,
                    base: 0,
                },
            )
            .unwrap(),
        ] {
            assert!(film.translocations.is_empty());
            assert_eq!(film.native_translocations.len(), 1);
            let event = &film.native_translocations[0];
            assert_eq!(event.packet_index, 0);
            assert_eq!(event.source.timestamp_us, 123);
            assert_eq!(event.read.source_bits, 16);
            assert!(!event.read.unit_reference_recorded());
            assert!(event.read.padded_bits > 0);
            assert!(event.read.event.positions().is_none());
            assert!(film.native_translocations_scanned);
            let slots = [event.read.event.slot].into_iter().collect();
            let published =
                super::replay_translocations::build_retained_translocations(&film, &slots, 0, 1);
            assert_eq!(published.coverage.events, 1);
            assert_eq!(published.coverage.published, 1);
            assert_eq!(published.coverage.positioned, 0);
            assert_eq!(published.jumps[0].t, 123);
            assert_eq!(published.jumps[0].slot, event.read.event.slot);
            let mut legacy = film.clone();
            legacy.translocations.push(super::FilmTranslocatorEvent {
                source: event.source,
                event: event.read.event.clone(),
            });
            legacy.native_translocations.clear();
            // An explicitly empty native scan must not resurrect legacy reads.
            assert_eq!(
                super::replay_translocations::build_retained_translocations(&legacy, &slots, 0, 1)
                    .coverage
                    .events,
                0
            );
            legacy.native_translocations_scanned = false;
            assert_eq!(
                super::replay_translocations::build_retained_translocations(&legacy, &slots, 0, 1),
                published
            );

            let restored: LegacyFilm =
                serde_json::from_slice(&serde_json::to_vec(&film).unwrap()).unwrap();
            assert_eq!(restored, film);
        }
    }

    #[test]
    fn film_retains_native_padded_zoom_without_promoting_state() {
        let mut bootstrap = vec![0; 8 + 64 * 260];
        bootstrap[..4].copy_from_slice(&41u32.to_le_bytes());
        bootstrap[4..8].copy_from_slice(&27u32.to_le_bytes());
        bootstrap[8] = 0xff;
        let mut data = vec![0u8; 16];
        data[4..8].copy_from_slice(&2u32.to_le_bytes());
        data[8..16].copy_from_slice(&123u64.to_le_bytes());
        data.extend_from_slice(&[0xca, 0xc0]);
        let chunks: Vec<_> = [bootstrap, data]
            .into_iter()
            .enumerate()
            .map(|(i, data)| FilmChunkData {
                metadata: FilmChunk {
                    index: i as i32,
                    chunk_type: if i == 0 { 1 } else { 2 },
                    start_time_offset_ms: 0,
                    duration_ms: 1,
                    size: data.len() as i64,
                    file_relative_path: String::new(),
                },
                data,
            })
            .collect();
        for film in [
            LegacyFilm::try_from_chunks(&chunks, DecodeOptions::v41()).unwrap(),
            LegacyFilm::try_from_chunks_with_encoding(&chunks, DecodeOptions::v41(), encoding())
                .unwrap(),
            LegacyFilm::try_from_chunks_with_map(
                &chunks,
                DecodeOptions::v41(),
                "Live Fire - Ranked",
                RecordIdLayout {
                    low_bits: 13,
                    base: 0,
                },
            )
            .unwrap(),
        ] {
            assert!(film.zoom_events.is_empty());
            assert_eq!(film.native_zoom_events.len(), 1);
            let event = &film.native_zoom_events[0];
            assert_eq!(event.packet_index, 0);
            assert_eq!(event.source.timestamp_us, 123);
            assert_eq!(event.read.source_bits, 16);
            assert!(!event.read.unit_reference_recorded());
            assert!(event.read.padded_bits > 0);
            assert!(!event.read.level_recorded());
            let restored: LegacyFilm =
                serde_json::from_slice(&serde_json::to_vec(&film).unwrap()).unwrap();
            assert_eq!(restored, film);
        }
    }

    #[test]
    fn film_retains_native_padded_pickup_without_promoting_state() {
        let mut bootstrap = vec![0; 8 + 64 * 260];
        bootstrap[..4].copy_from_slice(&41u32.to_le_bytes());
        bootstrap[4..8].copy_from_slice(&27u32.to_le_bytes());
        bootstrap[8] = 0xff;
        let mut data = vec![0u8; 16];
        data[4..8].copy_from_slice(&4u32.to_le_bytes());
        data[8..16].copy_from_slice(&123u64.to_le_bytes());
        data.extend_from_slice(&[196, 192, 64, 192]);
        let chunks: Vec<_> = [bootstrap, data]
            .into_iter()
            .enumerate()
            .map(|(i, data)| FilmChunkData {
                metadata: FilmChunk {
                    index: i as i32,
                    chunk_type: if i == 0 { 1 } else { 2 },
                    start_time_offset_ms: 0,
                    duration_ms: 1,
                    size: data.len() as i64,
                    file_relative_path: String::new(),
                },
                data,
            })
            .collect();
        for film in [
            LegacyFilm::try_from_chunks(&chunks, DecodeOptions::v41()).unwrap(),
            LegacyFilm::try_from_chunks_with_encoding(&chunks, DecodeOptions::v41(), encoding())
                .unwrap(),
            LegacyFilm::try_from_chunks_with_map(
                &chunks,
                DecodeOptions::v41(),
                "Live Fire - Ranked",
                RecordIdLayout {
                    low_bits: 13,
                    base: 0,
                },
            )
            .unwrap(),
        ] {
            assert!(film.pickups.records.is_empty());
            assert_eq!(film.native_pickups.stats.published, 1);
            let event = &film.native_pickups.attempts[0];
            assert_eq!(event.packet_index, 0);
            assert_eq!(event.source.timestamp_us, 123);
            assert!(event.published);
            assert!(event.read.padded_bits > 0);
            assert!(!event.read.catalog_recorded());
            let restored: LegacyFilm =
                serde_json::from_slice(&serde_json::to_vec(&film).unwrap()).unwrap();
            assert_eq!(restored, film);
        }
    }

    #[test]
    fn film_retains_native_equipment_spawn_source() {
        let mut bootstrap = vec![0; 8 + 64 * 260];
        bootstrap[..4].copy_from_slice(&41u32.to_le_bytes());
        bootstrap[4..8].copy_from_slice(&27u32.to_le_bytes());
        bootstrap[8] = 0xff;
        let mut data = vec![0u8; 16];
        data[4..8].copy_from_slice(&8u32.to_le_bytes());
        data[8..16].copy_from_slice(&123u64.to_le_bytes());
        data.extend_from_slice(&[243, 192, 64, 192, 64, 192, 192, 64]);
        let chunks: Vec<_> = [bootstrap, data]
            .into_iter()
            .enumerate()
            .map(|(i, data)| FilmChunkData {
                metadata: FilmChunk {
                    index: i as i32,
                    chunk_type: if i == 0 { 1 } else { 2 },
                    start_time_offset_ms: 0,
                    duration_ms: 1,
                    size: data.len() as i64,
                    file_relative_path: String::new(),
                },
                data,
            })
            .collect();
        for film in [
            LegacyFilm::try_from_chunks(&chunks, DecodeOptions::v41()).unwrap(),
            LegacyFilm::try_from_chunks_with_encoding(&chunks, DecodeOptions::v41(), encoding())
                .unwrap(),
            LegacyFilm::try_from_chunks_with_map(
                &chunks,
                DecodeOptions::v41(),
                "Live Fire - Ranked",
                RecordIdLayout {
                    low_bits: 13,
                    base: 0,
                },
            )
            .unwrap(),
        ] {
            assert!(film.equipment_spawns_error.is_none());
            assert_eq!(film.equipment_spawns.stats.events, 1);
            let event = &film.equipment_spawns.records[0];
            assert_eq!(event.packet_index, Some(0));
            assert_eq!(event.packet.timestamp_us, 123);
            assert_eq!(
                event.source.unwrap(),
                ObjectLife {
                    slot: 544,
                    generation: 1
                }
            );
            assert_eq!(
                event.spawned.unwrap(),
                ObjectLife {
                    slot: 544,
                    generation: 1
                }
            );
            let restored: LegacyFilm =
                serde_json::from_slice(&serde_json::to_vec(&film).unwrap()).unwrap();
            assert_eq!(restored, film);
        }
    }

    #[test]
    fn malformed_datums_preserve_surrounding_film_packets() {
        let cases: Vec<serde_json::Value> =
            serde_json::from_slice(include_bytes!("fixtures/datum-lengths-v41.json")).unwrap();
        let cases = &cases[..129]; // Small lengths exercise interleaved LegacyFilm assembly.
        let mut bootstrap = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/bootstrap-v41.zlib")[..])
            .read_to_end(&mut bootstrap)
            .unwrap();
        let mut packet_bytes = Vec::new();
        let mut ordinal = 0u64;
        let mut push = |kind: u16, data: &[u8]| {
            ordinal += 1;
            packet_bytes.extend_from_slice(&kind.to_le_bytes());
            packet_bytes.extend_from_slice(&[0, 0]);
            packet_bytes.extend_from_slice(&(data.len() as u32).to_le_bytes());
            packet_bytes.extend_from_slice(&(ordinal * 1000).to_le_bytes());
            packet_bytes.extend_from_slice(data);
        };
        push(0, &[0]);
        for case in cases {
            push(1, &vec![0; case["length"].as_u64().unwrap() as usize]);
            push(0, &[0]);
        }
        let chunks: Vec<_> = [bootstrap, packet_bytes]
            .into_iter()
            .enumerate()
            .map(|(index, data)| FilmChunkData {
                metadata: FilmChunk {
                    index: index as i32,
                    chunk_type: if index == 0 { 1 } else { 2 },
                    start_time_offset_ms: 0,
                    duration_ms: 1,
                    size: data.len() as i64,
                    file_relative_path: String::new(),
                },
                data,
            })
            .collect();
        let film =
            LegacyFilm::try_from_chunks_with_encoding(&chunks, DecodeOptions::v41(), encoding())
                .unwrap();
        let stream = film.replication.as_ref().unwrap();
        assert_eq!(stream.packets.len(), 1 + cases.len() * 2);
        let mut failures = film.datum_failures.iter();
        let mut tables = film.datum_tables.iter();
        for (i, case) in cases.iter().enumerate() {
            let packet = &stream.packets[1 + i * 2];
            let payload = vec![0; case["length"].as_u64().unwrap() as usize];
            let direct = decode_datum_table(&payload);
            assert_eq!(direct.is_err(), case["error"].as_bool().unwrap());
            assert_eq!(packet.source.timestamp_us, (2 + i as u64 * 2) * 1000);
            if let Err(error) = direct {
                let failure = failures.next().unwrap();
                assert_eq!(failure.source, packet.source);
                assert_eq!(failure.payload, payload);
                assert_eq!(failure.message, error.to_string());
                assert_eq!(failure.message, case["message"].as_str().unwrap());
                assert!(failure.native_error.is_some());
                assert_eq!(
                    packet.payload,
                    ReplicationPayload::InvalidDatums {
                        message: failure.message.clone(),
                        native_error: failure.native_error.clone(),
                        payload
                    }
                );
                assert!(packet.unparsed_tail.is_none()); // whole refused body is retained in payload
            } else {
                let snapshot = tables.next().unwrap();
                assert_eq!(
                    snapshot.table.slot_count,
                    case["slots"].as_u64().unwrap() as usize
                );
                assert_eq!(
                    packet.payload,
                    ReplicationPayload::Datums(snapshot.table.clone())
                );
            }
        }
        assert!(failures.next().is_none() && tables.next().is_none());
        assert_eq!(film.datum_tables.len(), 2);
        assert_eq!(film.datum_failures.len(), 127);
        for packet in stream.packets.iter().step_by(2) {
            let ReplicationPayload::Frame(frame) = &packet.payload else {
                panic!("lost surrounding frame")
            };
            assert_eq!(
                frame.controls.as_ref().unwrap().stop,
                FrameViewStop::Complete
            );
        }
        let restored: LegacyFilm =
            serde_json::from_slice(&serde_json::to_vec(&film).unwrap()).unwrap();
        assert_eq!(restored, film);
        let mut options = DecodeOptions::v41();
        options.retain_coverage = false;
        let compact = LegacyFilm::try_from_chunks(&chunks, options).unwrap();
        assert_eq!(compact.datum_failures, film.datum_failures);
    }

    #[test]
    fn film_retains_native_control_prefixes_and_padding() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/views-levelup-v41.json.zlib")[..])
            .read_to_end(&mut raw)
            .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        let expected: Vec<_> = rows
            .iter()
            .flat_map(|row| row["prefixes"].as_array().unwrap())
            .filter_map(|prefix| prefix.get("film"))
            .collect();
        let mut bootstrap = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/bootstrap-v41.zlib")[..])
            .read_to_end(&mut bootstrap)
            .unwrap();
        let mut packets = Vec::new();
        let mut inputs = Vec::new();
        for (i, row) in expected.iter().enumerate() {
            let hex = row["hex"].as_str().unwrap();
            let data: Vec<_> = (0..hex.len())
                .step_by(2)
                .map(|j| u8::from_str_radix(&hex[j..j + 2], 16).unwrap())
                .collect();
            packets.extend_from_slice(&[0, 0, 0, 0]);
            packets.extend_from_slice(&(data.len() as u32).to_le_bytes());
            packets.extend_from_slice(&((i as u64 + 1) * 1000).to_le_bytes());
            let offset = packets.len();
            packets.extend_from_slice(&data);
            inputs.push((offset, data));
        }
        let chunks: Vec<_> = [bootstrap, packets]
            .into_iter()
            .enumerate()
            .map(|(index, data)| FilmChunkData {
                metadata: FilmChunk {
                    index: index as i32,
                    chunk_type: if index == 0 { 1 } else { 2 },
                    start_time_offset_ms: 0,
                    duration_ms: 1,
                    size: data.len() as i64,
                    file_relative_path: String::new(),
                },
                data,
            })
            .collect();
        let film =
            LegacyFilm::try_from_chunks_with_encoding(&chunks, DecodeOptions::v41(), encoding())
                .unwrap();
        let stream = film.replication.as_ref().unwrap();
        assert_eq!(stream.packets.len(), expected.len());
        let mut padded = 0;
        for (i, ((packet, row), (offset, input))) in stream
            .packets
            .iter()
            .zip(&expected)
            .zip(&inputs)
            .enumerate()
        {
            assert_eq!(packet.source.chunk_index, 1);
            assert_eq!(packet.source.payload_offset, *offset);
            assert_eq!(packet.source.payload_size, input.len());
            assert_eq!(packet.source.timestamp_us, (i as u64 + 1) * 1000);
            let ReplicationPayload::Frame(frame) = &packet.payload else {
                panic!("missing frame {i}")
            };
            assert_eq!(frame.end_bit, row["end"], "frame {i}");
            assert_eq!(frame.messages.stop, FrameViewStop::Complete);
            let entities = frame.entities.as_ref().unwrap();
            assert_eq!(entities.stop, EntityViewStop::Complete);
            assert!(
                entities
                    .records
                    .iter()
                    .all(|record| record.header.kind == RecordKind::End)
            );
            let controls = frame.controls.as_ref().unwrap();
            assert_eq!(serde_json::to_value(&controls.kinds).unwrap(), row["kinds"]);
            assert_eq!(
                controls.stop == FrameViewStop::Complete,
                row["complete"].as_bool().unwrap()
            );
            assert_eq!(
                serde_json::json!(controls.padded_bits),
                serde_json::json!(crate::theater::bits::padded_from_native(
                    frame.end_bit,
                    input.len() * 8
                ))
            );
            assert_eq!(
                2 + usize::from(controls.stop == FrameViewStop::Complete),
                row["views"]
            );
            padded += usize::from(controls.padded_bits > 0);
            if frame.end_bit < (input.len() * 8) as i64 {
                let tail = packet.unparsed_tail.as_ref().unwrap();
                assert_eq!(
                    serde_json::json!(tail.start_bit),
                    serde_json::json!(frame.end_bit)
                );
                assert_eq!(
                    tail.bytes,
                    input[crate::theater::bits::native_address(frame.end_bit / 8)..]
                );
            } else {
                assert!(packet.unparsed_tail.is_none());
            }
        }
        assert_eq!(expected.len(), 1559);
        assert_eq!(padded, 11);
        let restored: LegacyFilm =
            serde_json::from_slice(&serde_json::to_vec(&film).unwrap()).unwrap();
        assert_eq!(restored, film);
    }

    #[test]
    fn explicit_encoding_retains_equipment_error_counters() {
        let mut bootstrap = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/bootstrap-v41.zlib")[..])
            .read_to_end(&mut bootstrap)
            .unwrap();
        // End the registry before ti=37; a keyframe still records an equipment
        // slot. Native ScanEquipmentState sets Slots before resolving ti=37.
        bootstrap.truncate(8 + 38 * 64 * 260);
        bootstrap[8 + 37 * 64 * 260] = 0xff;
        assert!(parse_registry(&bootstrap).unwrap().archetype(37).is_none());
        let body = bits(&[
            (0, 1),
            ((1 << 30) | 512, 32),
            (37, 32),
            (0, 44),
            (0, 32),
            (0, 32),
            (0, 64),
        ]);
        let mut packet = Vec::new();
        packet.extend_from_slice(&2u16.to_le_bytes());
        packet.extend_from_slice(&[0, 0]);
        packet.extend_from_slice(&(body.len() as u32).to_le_bytes());
        packet.extend_from_slice(&1000u64.to_le_bytes());
        packet.extend(body);
        let chunks: Vec<_> = [bootstrap, packet]
            .into_iter()
            .enumerate()
            .map(|(index, data)| FilmChunkData {
                metadata: FilmChunk {
                    index: index as i32,
                    chunk_type: if index == 0 { 1 } else { 2 },
                    start_time_offset_ms: 0,
                    duration_ms: 1,
                    size: data.len() as i64,
                    file_relative_path: String::new(),
                },
                data,
            })
            .collect();
        let film =
            LegacyFilm::try_from_chunks_with_encoding(&chunks, DecodeOptions::v41(), encoding())
                .unwrap();
        assert!(
            film.equipment_state_error
                .as_ref()
                .unwrap()
                .contains("equipment archetype 37")
        );
        assert!(film.navpoint_radial.is_some());
        assert!(film.navpoint_radial_error.is_none());
        let navpoint = film.navpoint_radial.as_ref().unwrap();
        assert_eq!(navpoint.slots_observed, 0);
        assert!(navpoint.reads.is_empty());
        assert!(navpoint.attempts.is_empty());
        let state = film.equipment_state.as_ref().unwrap();
        assert_eq!(state.stats.slots, 1);
        assert_eq!(state.stats.records, 0);
        assert_eq!(state.stats.mask_census, vec![0; 64]);
        assert!(state.samples.is_empty());
        assert!(film.replication.is_some());
        let restored: LegacyFilm =
            serde_json::from_slice(&serde_json::to_vec(&film).unwrap()).unwrap();
        assert_eq!(restored, film);
        assert_eq!(film.objective_scan.as_ref().unwrap().slots, 0);
        assert!(
            film.objective_scan_error
                .as_ref()
                .unwrap()
                .contains("observed managed-objective slots")
        );
        let mut old = serde_json::to_value(&film).unwrap();
        old.as_object_mut().unwrap().remove("objective_scan");
        old.as_object_mut().unwrap().remove("objective_scan_error");
        let old: LegacyFilm = serde_json::from_value(old).unwrap();
        assert!(old.objective_scan.is_none());
        assert!(old.objective_scan_error.is_none());
    }

    #[test]
    fn map_api_retains_other_channels_after_optional_scan_failure() {
        let inflate = |compressed: &[u8]| {
            let mut bytes = Vec::new();
            flate2::read::ZlibDecoder::new(compressed)
                .read_to_end(&mut bytes)
                .unwrap();
            bytes
        };
        let bootstrap = inflate(include_bytes!("fixtures/bootstrap-v41.zlib"));
        let keyframe = inflate(include_bytes!("fixtures/captured-keyframe-v41.zlib"));
        let registry = parse_registry(&bootstrap).unwrap();
        let mut packet = Vec::new();
        packet.extend_from_slice(&2u16.to_le_bytes());
        packet.extend_from_slice(&[0, 0]);
        packet.extend_from_slice(&(keyframe.len() as u32).to_le_bytes());
        packet.extend_from_slice(&1000u64.to_le_bytes());
        packet.extend(keyframe);
        let biped = bits(&[
            (0, 1),
            ((1 << 30) | 512, 32),
            (35, 32),
            (0, 44),
            (0, 32),
            (0, 32),
            (0, 64),
        ]);
        let mut biped_packet = Vec::new();
        biped_packet.extend_from_slice(&2u16.to_le_bytes());
        biped_packet.extend_from_slice(&[0, 0]);
        biped_packet.extend_from_slice(&(biped.len() as u32).to_le_bytes());
        biped_packet.extend_from_slice(&2000u64.to_le_bytes());
        biped_packet.extend(biped);
        for (remove_weapon, remove_inventory) in
            [(false, false), (true, false), (false, true), (true, true)]
        {
            let mut changed = bootstrap.clone();
            let mut changed_names = 0;
            for (index, name) in registry
                .archetype(35)
                .unwrap()
                .components
                .iter()
                .enumerate()
            {
                if (remove_weapon && name == "weapon-state-type-info")
                    || (remove_inventory
                        && matches!(
                            name.as_str(),
                            "unit-grenade-counts-component"
                                | "biped-desired-grenade-set-component"
                                | "biped-desired-grenade-set"
                                | "weapon-state-ammo"
                                | "weapon-state-rounds-inventory"
                        ))
                {
                    let offset = 8 + (35 * 64 + index) * 260;
                    assert_eq!(&changed[offset..offset + name.len()], name.as_bytes());
                    changed[offset] = b'X';
                    changed_names += 1;
                }
            }
            assert_eq!(changed_names > 0, remove_weapon || remove_inventory);
            let chunks: Vec<_> = [
                (0, 1, changed),
                (1, 2, packet.clone()),
                (2, 2, biped_packet.clone()),
            ]
            .into_iter()
            .map(|(index, chunk_type, data)| FilmChunkData {
                metadata: FilmChunk {
                    index,
                    chunk_type,
                    start_time_offset_ms: 0,
                    duration_ms: 1,
                    size: data.len() as i64,
                    file_relative_path: String::new(),
                },
                data,
            })
            .collect();
            assert_eq!(
                super::super::biped_scan::biped_slot_band(&chunks).unwrap(),
                Some([512, 512])
            );
            let film = LegacyFilm::try_from_chunks_with_map(
                &chunks,
                DecodeOptions::v41(),
                "Live Fire - Ranked",
                RecordIdLayout {
                    low_bits: 13,
                    base: 0,
                },
            )
            .unwrap();
            assert_eq!(film.inventory_deltas_error.is_some(), remove_inventory);
            assert_eq!(film.weapon_changes_error.is_some(), remove_weapon);
            assert_eq!(film.inventory_deltas.is_some(), !remove_inventory);
            assert_eq!(film.weapon_changes.is_some(), !remove_weapon);
            for error in [
                &film.movement_states_error,
                &film.navpoint_radial_error,
                &film.biped_channels_error,
                &film.ability_states_error,
                &film.ability_charges_error,
                &film.equipment_changes_error,
            ] {
                assert!(error.is_none());
            }
            assert!(film.ability_states.is_some());
            assert!(film.ability_charges.is_some());
            assert!(film.unit_equipment.is_some());
            assert!(film.biped_positions.is_some());
            assert!(film.replication.is_some());
            assert_eq!(
                film.diagnostics.limitations.iter().any(
                    |message| message.starts_with("Legacy signature observations unavailable:")
                ),
                remove_inventory
            );
            if remove_inventory {
                assert!(film.players.iter().all(|player| player.lives.is_empty()
                    && player.positions.is_empty()
                    && player.appearance.is_empty()));
                assert!(film.clocks.is_empty());
                assert!(film.projectiles.is_empty());
            }
            let restored: LegacyFilm =
                serde_json::from_slice(&serde_json::to_vec(&film).unwrap()).unwrap();
            assert_eq!(restored, film);
        }
    }

    #[test]
    fn custom_encoding_uses_native_corruption_fallback_without_identity() {
        let mut bootstrap = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/bootstrap-v41.zlib")[..])
            .read_to_end(&mut bootstrap)
            .unwrap();
        let registry = parse_registry(&bootstrap).unwrap();
        let identity = decode_film_identity(&bootstrap, &registry)
            .unwrap()
            .unwrap();
        bootstrap[identity.build_offset..identity.build_offset + 3].fill(0);
        assert!(
            decode_film_identity(&bootstrap, &registry)
                .unwrap()
                .is_none()
        );
        let mut packet = Vec::new();
        packet.extend_from_slice(&2u16.to_le_bytes());
        packet.extend_from_slice(&[0, 0]);
        packet.extend_from_slice(&1u32.to_le_bytes());
        packet.extend_from_slice(&1000u64.to_le_bytes());
        packet.push(0);
        let chunks: Vec<_> = [bootstrap, packet]
            .into_iter()
            .enumerate()
            .map(|(i, data)| FilmChunkData {
                metadata: FilmChunk {
                    index: i as i32,
                    chunk_type: if i == 0 { 1 } else { 2 },
                    start_time_offset_ms: 0,
                    duration_ms: 1,
                    size: data.len() as i64,
                    file_relative_path: String::new(),
                },
                data,
            })
            .collect();
        let mut custom = encoding();
        custom.corruption_check = true;
        let film = LegacyFilm::try_from_chunks_with_encoding(&chunks, DecodeOptions::v41(), custom)
            .unwrap();
        assert!(film.corruption_control().uses_fallback());
        assert!(film.player_teams.is_some());
        assert!(!film.replication.unwrap().encoding.corruption_check);
    }

    #[test]
    fn map_api_retains_resolved_context_and_recovered_records() {
        let mut chunks = Vec::new();
        for (index, kind, compressed) in [
            (0, 1, &include_bytes!("fixtures/bootstrap-v41.zlib")[..]),
            (
                1,
                2,
                &include_bytes!("fixtures/captured-keyframe-v41.zlib")[..],
            ),
        ] {
            let mut data = Vec::new();
            flate2::read::ZlibDecoder::new(compressed)
                .read_to_end(&mut data)
                .unwrap();
            if kind == 2 {
                let mut packet = Vec::new();
                packet.extend_from_slice(&2u16.to_le_bytes());
                packet.extend_from_slice(&[0, 0]);
                packet.extend_from_slice(&(data.len() as u32).to_le_bytes());
                packet.extend_from_slice(&1000u64.to_le_bytes());
                packet.extend(data);
                data = packet;
            }
            chunks.push(FilmChunkData {
                metadata: FilmChunk {
                    index,
                    chunk_type: kind,
                    start_time_offset_ms: 0,
                    duration_ms: 1,
                    size: data.len() as i64,
                    file_relative_path: String::new(),
                },
                data,
            });
        }
        let mut oracle = String::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/translocator-levelup-v41.json.zlib")[..],
        )
        .read_to_string(&mut oracle)
        .unwrap();
        let cases: Vec<serde_json::Value> = serde_json::from_str(&oracle).unwrap();
        let case = cases
            .iter()
            .find(|c| c["source"] == "synthetic:live fire" && c["event"]["HasPositions"] == true)
            .unwrap();
        let hex = case["hex"].as_str().unwrap();
        let teleport: Vec<u8> = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect();
        chunks[1].data.extend_from_slice(&[0; 4]);
        chunks[1]
            .data
            .extend_from_slice(&(teleport.len() as u32).to_le_bytes());
        chunks[1].data.extend_from_slice(&2000u64.to_le_bytes());
        chunks[1].data.extend(teleport);
        chunks[1].metadata.size = chunks[1].data.len() as i64;
        let plain = LegacyFilm::try_from_chunks(&chunks, DecodeOptions::v41()).unwrap();
        assert_eq!(plain.translocations.len(), 1);
        assert_eq!(
            plain.translocations[0].event.stop,
            TranslocatorStop::MissingMap
        );
        let ids = RecordIdLayout {
            low_bits: 13,
            base: 0,
        };
        let film = LegacyFilm::try_from_chunks_with_map(
            &chunks,
            DecodeOptions::v41(),
            "Live Fire - Ranked",
            ids,
        )
        .unwrap();
        let profile = film.profile.as_ref().unwrap();
        assert_eq!(profile.map_name.as_deref(), Some("live fire"));
        assert!(profile.issues.is_empty());
        assert_eq!(film.translocations.len(), 1);
        let jump = &film.translocations[0];
        assert_eq!(jump.source.timestamp_us, 2000);
        let positions = jump.event.positions().unwrap();
        for (position, field) in positions.iter().zip(["From", "To"]) {
            for (i, value) in position.iter().enumerate() {
                assert_eq!(
                    value.to_bits(),
                    (case["event"][field][i].as_f64().unwrap() as f32).to_bits()
                );
            }
        }
        let stream = film.replication.as_ref().unwrap();
        assert_eq!(stream.encoding.position.as_ref().unwrap().index_bits, 2);
        assert_eq!(
            stream.encoding.corruption_check,
            film.identity.as_ref().unwrap().corruption_checks
        );
        assert_eq!(
            stream.packets[0]
                .keyframe_recovery
                .as_ref()
                .unwrap()
                .records
                .len(),
            123
        );
        assert!(
            stream.packets[0]
                .keyframe_recovery
                .as_ref()
                .unwrap()
                .records
                .iter()
                .all(|record| record.native_read.is_some())
        );
        let custom = LegacyFilm::try_from_chunks_with_map_bounds(
            &chunks,
            DecodeOptions::v41(),
            "Custom Forge Map",
            profile.map.clone().unwrap(),
            ids,
        )
        .unwrap();
        let mut expected_custom = film.clone();
        expected_custom.profile.as_mut().unwrap().map_name = Some("custom forge map".into());
        assert_eq!(custom, expected_custom);
        assert!(film.weapon_hits_error.is_none());
        let expected_hits = scan_film_weapon_hits(
            &chunks,
            &film.registry,
            film.scan_precision.as_ref().unwrap().sampling_map.as_ref(),
        )
        .unwrap();
        assert_eq!(film.weapon_hits.as_ref(), Some(&expected_hits));
        let exported = serde_json::to_vec(&film).unwrap();
        let restored: LegacyFilm = serde_json::from_slice(&exported).unwrap();
        assert_eq!(restored.profile, film.profile);
        assert!(film.native_vehicles.is_some());
        assert_eq!(restored.native_vehicles, film.native_vehicles);
        assert!(film.native_march_facts.is_some());
        assert_eq!(restored.native_march_facts, film.native_march_facts);
        assert_eq!(restored.replication, film.replication);
        assert_eq!(restored.weapon_hits, film.weapon_hits);
        let spans = film.keyframe_record_spans.as_ref().unwrap();
        assert_eq!(spans.len(), 123);
        assert!(
            spans
                .iter()
                .all(|s| s.chunk == 1 && s.packet_index == 0 && s.timestamp_us == 1000)
        );
        assert_eq!(restored.keyframe_record_spans, film.keyframe_record_spans);
        assert_eq!(
            restored.vehicle_keyframe_states,
            film.vehicle_keyframe_states
        );
        assert_eq!(restored.translocations, film.translocations);
        assert_eq!(restored.biped_creations, film.biped_creations);
        assert_eq!(restored.pickups, film.pickups);
        assert_eq!(restored.zoom_events, film.zoom_events);
        assert_eq!(restored.equipment_spawns, film.equipment_spawns);
        assert!(film.movement_states.is_some());
        assert_eq!(restored.movement_states, film.movement_states);
        assert_eq!(restored.keyframe_loadouts, film.keyframe_loadouts);
        assert!(film.keyframe_inventory.is_some());
        assert_eq!(restored.keyframe_inventory, film.keyframe_inventory);
        assert_eq!(
            film.weapon_changes.is_some(),
            film.biped_positions.as_ref().unwrap().slot_band.is_some()
        );
        assert_eq!(restored.weapon_changes, film.weapon_changes);
        assert!(film.biped_positions.as_ref().unwrap().slot_band.is_none());
        assert!(film.unit_equipment.is_none());
        assert_eq!(
            film.unit_equipment_error.as_deref(),
            Some(DecodeError::Missing("biped slot band").to_string().as_str())
        );
        assert_eq!(restored.unit_equipment, film.unit_equipment);
        assert_eq!(restored.unit_equipment_error, film.unit_equipment_error);
        assert!(film.ability_charges.is_some());
        assert_eq!(restored.ability_charges, film.ability_charges);
        assert!(film.ability_states.is_some());
        assert_eq!(restored.ability_states, film.ability_states);
        assert!(film.equipment_changes.is_some());
        assert_eq!(restored.equipment_changes, film.equipment_changes);
        assert!(film.inventory_deltas.is_some());
        assert_eq!(restored.inventory_deltas, film.inventory_deltas);
        assert!(film.biped_channels.is_some());
        assert_eq!(restored.biped_channels, film.biped_channels);
        assert!(film.biped_positions.is_some());
        assert_eq!(restored.biped_positions, film.biped_positions);
        assert!(
            LegacyFilm::try_from_chunks_with_map(&chunks, DecodeOptions::v41(), "unknown-map", ids)
                .is_err()
        );
    }

    #[test]
    fn recovered_bindings_reach_later_updates_without_losing_provenance() {
        fn inflate(data: &[u8]) -> Vec<u8> {
            let mut out = Vec::new();
            flate2::read::ZlibDecoder::new(data)
                .read_to_end(&mut out)
                .unwrap();
            out
        }
        let keyframe = inflate(include_bytes!("fixtures/captured-keyframe-v41.zlib"));
        let reg = parse_registry(&inflate(include_bytes!("fixtures/bootstrap-v41.zlib"))).unwrap();
        let frame = bits(&[
            (1, 1),
            (0, 1),
            (1, 1),
            (2, 5),
            (1, 2),
            (0, 1),
            (0, 4),
            (0, 3),
            (0, 1),
        ]);
        let mut data = Vec::new();
        for (kind, payload) in [(2u16, keyframe), (0, frame)] {
            data.extend_from_slice(&kind.to_le_bytes());
            data.extend_from_slice(&[0, 0]);
            data.extend_from_slice(&(payload.len() as u32).to_le_bytes());
            data.extend_from_slice(&1000u64.to_le_bytes());
            data.extend_from_slice(&payload);
        }
        let chunks = [FilmChunkData {
            metadata: FilmChunk {
                index: 1,
                chunk_type: 2,
                start_time_offset_ms: 0,
                duration_ms: 0,
                size: data.len() as i64,
                file_relative_path: String::new(),
            },
            data,
        }];
        let stream = decode_replication_stream_with_recovery(
            &chunks,
            &reg,
            encoding(),
            KeyframeRecoveryPolicy::LevelUp,
        )
        .unwrap();
        assert_eq!(
            stream.packets[0]
                .keyframe_recovery
                .as_ref()
                .unwrap()
                .records
                .len(),
            123
        );
        assert_eq!(
            stream.bindings.slots[&0].origin,
            BindingOrigin::SequentialKeyframe { bit: 1 }
        );
        assert_eq!(
            stream.bindings.slots[&2].origin,
            BindingOrigin::RecoveredKeyframe { bit: 4773 }
        );
        let ReplicationPayload::Frame(frame) = &stream.packets[1].payload else {
            panic!()
        };
        let entities = frame.entities.as_ref().unwrap();
        assert_eq!(entities.stop, EntityViewStop::Complete);
        assert_eq!(
            entities.records[0].binding_origin,
            Some(BindingOrigin::RecoveredKeyframe { bit: 4773 })
        );
        // A refused table must not reset recovered entities or prevent the next
        // independently framed update from being decoded.
        let native_cases: Vec<serde_json::Value> =
            serde_json::from_slice(include_bytes!("fixtures/datum-lengths-v41.json")).unwrap();
        for length in [0, 63, (8192usize * 335 + 160).div_ceil(8)] {
            let expected = native_cases
                .iter()
                .find(|row| row["length"].as_u64() == Some(length as u64))
                .unwrap();
            assert_eq!(expected["error"], true);
            let original_frame = stream.packets[1].source;
            let header_at = original_frame.payload_offset - 16;
            let mut altered = chunks.clone();
            let original_data = &chunks[0].data;
            let mut data = original_data[..header_at].to_vec();
            data.extend_from_slice(&1u16.to_le_bytes());
            data.extend_from_slice(&[0, 0]);
            data.extend_from_slice(&(length as u32).to_le_bytes());
            data.extend_from_slice(&1000u64.to_le_bytes());
            data.resize(data.len() + length, 0);
            data.extend_from_slice(&original_data[header_at..]);
            altered[0].metadata.size = data.len() as i64;
            altered[0].data = data;
            let with_failure = decode_replication_stream_with_recovery(
                &altered,
                &reg,
                encoding(),
                KeyframeRecoveryPolicy::LevelUp,
            )
            .unwrap();
            assert_eq!(with_failure.packets.len(), 3);
            assert_eq!(with_failure.bindings, stream.bindings);
            assert_eq!(with_failure.packets[0], stream.packets[0]);
            let failed = &with_failure.packets[1];
            let ReplicationPayload::InvalidDatums {
                message,
                native_error,
                payload,
            } = &failed.payload
            else {
                panic!("missing datum refusal")
            };
            assert_eq!(message, expected["message"].as_str().unwrap());
            assert!(native_error.is_some());
            assert_eq!(payload, &vec![0; length]);
            assert_eq!(failed.source.payload_offset, header_at + 16);
            assert_eq!(failed.source.payload_size, length);
            assert_eq!(with_failure.packets[2].payload, stream.packets[1].payload);
            assert_eq!(
                with_failure.packets[2].unparsed_tail,
                stream.packets[1].unparsed_tail
            );
            let mut expected_source = original_frame;
            expected_source.payload_offset += 16 + length;
            assert_eq!(with_failure.packets[2].source, expected_source);
            let restored: ReplicationStream =
                serde_json::from_slice(&serde_json::to_vec(&with_failure).unwrap()).unwrap();
            assert_eq!(restored, with_failure);
        }
        let json = serde_json::to_vec(&stream).unwrap();
        assert_eq!(
            serde_json::from_slice::<ReplicationStream>(&json).unwrap(),
            stream
        );
    }

    fn bits(fields: &[(u64, usize)]) -> Vec<u8> {
        let mut bytes = vec![0; fields.iter().map(|(_, w)| w).sum::<usize>().div_ceil(8)];
        let mut at = 0;
        for &(value, width) in fields {
            for i in (0..width).rev() {
                bytes[at / 8] |= (((value >> i) & 1) as u8) << (7 - at % 8);
                at += 1;
            }
        }
        bytes
    }

    fn encoding() -> FrameEncoding {
        FrameEncoding {
            keyframe_layout: Default::default(),
            keyframe_simulation_complete: None,
            native_id_low_bits: None,
            component_widths: Default::default(),
            new_record: Default::default(),
            position_capture: None,
            ids: RecordIdLayout {
                low_bits: 5,
                base: 0,
            },
            mpp_widths: [9, 5],
            position: None,
            extra_fields: false,
            corruption_check: false,
        }
    }

    fn registry() -> FilmRegistry {
        FilmRegistry {
            archetypes: vec![FilmArchetype {
                index: 0,
                components: Vec::new(),
                levels: Vec::new(),
            }],
            major_version: 41,
            format_version: 27,
            end_byte: 0,
            truncated: false,
        }
    }

    #[test]
    fn keyframes_seed_frames_and_new_snapshots_clear_stale_bindings() {
        let keyframe = bits(&[
            (1, 1),
            (0x40000001, 32),
            (0, 32),
            (0, 32),
            (0, 4),
            (0, 8),
            (0, 32),
            (0, 32),
            (0x40000002, 32),
            (u32::MAX as u64, 32),
            (0, 32),
            (0, 4),
            (0, 8),
        ]);
        let frame = bits(&[
            (1, 1),
            (0, 1),
            (1, 1),
            (1, 5),
            (1, 2),
            (0, 1),
            (0, 4),
            (0, 3),
            (0, 1),
        ]);
        let mut data = Vec::new();
        for (kind, payload) in [
            (2u16, keyframe),
            (0, frame.clone()),
            (2, vec![0]),
            (0, frame),
            (8, vec![0x12, 0x34]),
        ] {
            data.extend_from_slice(&kind.to_le_bytes());
            data.extend_from_slice(&[0, 0]);
            data.extend_from_slice(&(payload.len() as u32).to_le_bytes());
            data.extend_from_slice(&1000u64.to_le_bytes());
            data.extend_from_slice(&payload);
        }
        let chunks = [FilmChunkData {
            metadata: FilmChunk {
                index: 1,
                chunk_type: 2,
                start_time_offset_ms: 0,
                duration_ms: 0,
                size: data.len() as i64,
                file_relative_path: String::new(),
            },
            data,
        }];
        let result = decode_replication_stream(&chunks, &registry(), encoding()).unwrap();
        let ReplicationPayload::Keyframe(table) = &result.packets[0].payload else {
            panic!()
        };
        assert_eq!(table.records.len(), 2);
        assert_eq!(table.records[1].end_bit - table.records[1].start_bit, 108);
        assert!(matches!(table.stop, KeyframeTableStop::PayloadEnd { .. }));
        let ReplicationPayload::Frame(first) = &result.packets[1].payload else {
            panic!()
        };
        assert_eq!(
            first.entities.as_ref().unwrap().stop,
            EntityViewStop::Complete
        );
        assert_eq!(
            first.controls.as_ref().unwrap().stop,
            FrameViewStop::Complete
        );
        let ReplicationPayload::Frame(second) = &result.packets[3].payload else {
            panic!()
        };
        assert_eq!(
            second.entities.as_ref().unwrap().stop,
            EntityViewStop::MissingBinding { id: 0x40000001 }
        );
        assert!(second.controls.is_none());
        assert!(result.bindings.slots.is_empty());
        assert_eq!(
            result.packets[4].payload,
            ReplicationPayload::Opaque(vec![0x12, 0x34])
        );
        let encoded = serde_json::to_vec(&result).unwrap();
        assert_eq!(
            serde_json::from_slice::<ReplicationStream>(&encoded).unwrap(),
            result
        );
    }

    #[test]
    fn keyframe_table_rejects_invalid_headers_and_slot_order() {
        let mut data = vec![(0, 1)];
        for slot in [2, 1] {
            data.extend([
                (0x40000000 | slot, 32),
                (u32::MAX as u64, 32),
                (0, 32),
                (0, 4),
                (0, 8),
            ]);
        }
        let mut bindings = EntityBindings::default();
        let table = decode_keyframe_table(&bits(&data), &registry(), &encoding(), &mut bindings);
        assert_eq!(table.stop, KeyframeTableStop::NonIncreasingSlot);
        assert_eq!(table.records.len(), 1);
        assert!(bindings.slots.is_empty());
        let truncated = bits(&[(0, 1), (0x40000001, 32), (u32::MAX as u64, 32)]);
        assert_eq!(
            decode_keyframe_table(&truncated, &registry(), &encoding(), &mut bindings).stop,
            KeyframeTableStop::Truncated
        );
        let invalid = bits(&[(0, 1), (1, 32), (0, 32)]);
        assert_eq!(
            decode_keyframe_table(&invalid, &registry(), &encoding(), &mut bindings).stop,
            KeyframeTableStop::InvalidHeader
        );
    }
}
