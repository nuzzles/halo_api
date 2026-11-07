//! Contextual player replay over sequential v41 records. This is an interpretation,
//! separate from the canonical Film and its default-context world snapshot.
use super::{SummaryKind, TheaterRuntime, coordinates::PositionBounds};
use crate::theater::{
    film::{
        maps::{MapError, MapGlossary},
        *,
    },
    parser::v41::{
        chunks::replication::{
            replication_stream::{self, DecodeContext},
            state::ReplicationDecodeState,
        },
        context::V41DecodeConfig,
    },
};
use serde::Serialize;
use std::collections::BTreeMap;
mod reference_frames;

/// Map calibration or film evidence prevented contextual replay resolution.
#[derive(Debug, thiserror::Error)]
pub enum ReplayError {
    /// The requested map has no supported calibration.
    #[error(transparent)]
    Map(#[from] MapError),
    /// Required film evidence is missing, ambiguous, or unsupported.
    #[error("{0}")]
    InvalidFilm(&'static str),
}

/// Byte/bit location in the original decompressed replication chunk. Contextual
/// records deliberately do not masquerade as canonical runtime RecordRefs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct PoseSource {
    pub chunk_index: Option<i64>,
    pub payload_byte: usize,
    pub start_bit: i64,
    pub end_bit: i64,
}
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Pose {
    pub timestamp_us: u64,
    pub position: Option<[f64; 3]>,
    /// Unit desired-aim direction, with an approximate angular convention.
    pub aim: Option<[f64; 3]>,
}
/// Accepted contextual record; source points into retained packet bytes.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct PoseObservation {
    pub pose: Pose,
    pub entity_id: u32,
    pub source: PoseSource,
}
/// A source-bounded shot header attributed to a currently bound player biped.
/// This is an event, not a continuous trigger-held or weapon-animation state.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct ShotObservation {
    pub timestamp_us: u64,
    pub entity_id: u32,
    pub source: PoseSource,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct TeamSample {
    pub timestamp_us: u64,
    /// Four-bit team designator; None means the team is not established.
    pub team: Option<u8>,
}
#[derive(Debug, Clone, Copy, Serialize)]
pub enum LifeEnd {
    NextSpawn,
    Delete,
    OwnershipChange,
    SummaryDeath,
    FilmEnd,
}
#[derive(Debug, Clone, Serialize)]
pub struct PlayerLife {
    pub start_us: u64,
    pub end_us: u64,
    pub entity_id: u32,
    pub source: PoseSource,
    pub end_reason: LifeEnd,
}
#[derive(Debug, Clone, Serialize)]
pub struct PlayerReplay {
    pub film_index: usize,
    pub xuid: u64,
    pub gamertag: String,
    pub identity_source: Option<PoseSource>,
    pub lives: Vec<PlayerLife>,
    /// State changes and explicit expiry/gap markers; sorted, no interpolation.
    pub samples: Vec<Pose>,
    pub observations: Vec<PoseObservation>,
    pub shots: Vec<ShotObservation>,
    pub team_samples: Vec<TeamSample>,
    pub camouflage_samples: Vec<CamouflageSample>,
    pub camouflage_observations: Vec<CamouflageObservation>,
    pub vitality_samples: Vec<VitalitySample>,
    pub vitality_observations: Vec<VitalityObservation>,
    pub grenade_samples: Vec<GrenadeSample>,
    pub grenade_observations: Vec<GrenadeObservation>,
    pub weapon_samples: Vec<WeaponSample>,
    pub weapon_observations: Vec<WeaponObservation>,
    pub equipment_samples: Vec<EquipmentSample>,
    pub equipment_observations: Vec<EquipmentObservation>,
    pub position_records: usize,
    pub aim_records: usize,
}
impl PlayerReplay {
    pub fn equipment_at(&self, timestamp_us: u64) -> Option<EquipmentSample> {
        let i = self
            .equipment_samples
            .partition_point(|s| s.timestamp_us <= timestamp_us);
        i.checked_sub(1).map(|i| self.equipment_samples[i].clone())
    }
    pub fn vitality_at(&self, timestamp_us: u64) -> Option<VitalitySample> {
        let i = self
            .vitality_samples
            .partition_point(|s| s.timestamp_us <= timestamp_us);
        i.checked_sub(1).map(|i| self.vitality_samples[i])
    }
    pub fn grenades_at(&self, timestamp_us: u64) -> Option<GrenadeSample> {
        let i = self
            .grenade_samples
            .partition_point(|s| s.timestamp_us <= timestamp_us);
        i.checked_sub(1).map(|i| self.grenade_samples[i])
    }
    pub fn weapons_at(&self, timestamp_us: u64) -> Option<WeaponSample> {
        let i = self
            .weapon_samples
            .partition_point(|p| p.timestamp_us <= timestamp_us);
        i.checked_sub(1).map(|i| self.weapon_samples[i])
    }
    /// Random access and sequential CSV sampling use the same timeline.
    pub fn at(&self, timestamp_us: u64) -> Option<Pose> {
        let i = self
            .samples
            .partition_point(|p| p.timestamp_us <= timestamp_us);
        i.checked_sub(1).map(|i| self.samples[i])
    }
}
#[derive(Debug, Default, Serialize)]
pub struct ReplayDiagnostics {
    pub frame_packets: usize,
    pub refused_frames: usize,
    pub unclosed_frames: usize,
    pub incomplete_records: usize,
    pub refused_grenade_counts: usize,
    pub incomplete_entity_frames: usize,
    pub unbound_deltas: usize,
    pub rejected_spawns: usize,
    pub refused_rosters: usize,
    pub out_of_range_packets: usize,
    /// Counts and first source locations for keyframe stops and frame gaps.
    /// This is decoding coverage, not inferred damage or lost-event counts.
    pub blockers: BTreeMap<String, ReplayBlocker>,
}
#[derive(Debug, Serialize)]
pub struct ReplayBlocker {
    pub count: usize,
    /// None for a baseline before the chosen replay clock origin.
    pub first_timestamp_us: Option<u64>,
    /// Zero-width cursor at the parser stop, not an accepted record span.
    pub first_source: PoseSource,
}
impl ReplayDiagnostics {
    fn blocker(&mut self, reason: String, time: Option<u64>, source: PoseSource) {
        self.blockers
            .entry(reason)
            .or_insert(ReplayBlocker {
                count: 0,
                first_timestamp_us: time,
                first_source: source,
            })
            .count += 1;
    }
}
#[derive(Debug, Serialize)]
pub struct PlayerReplaySet {
    pub origin_timestamp_us: u64,
    pub duration_us: u64,
    pub freshness_us: u64,
    pub players: Vec<PlayerReplay>,
    pub projectiles: Vec<TrackedProjectile>,
    pub diagnostics: ReplayDiagnostics,
}
mod camouflage;
mod equipment;
mod vitality;
pub use camouflage::{CamouflageObservation, CamouflageSample};
pub use equipment::{
    EquipmentChannel, EquipmentObservation, EquipmentSample, EquipmentSelection, HeldEquipment,
};
pub use vitality::{VitalityKind, VitalityObservation, VitalitySample};

mod grenades;
pub use grenades::{GrenadeCounts, GrenadeObservation, GrenadeSample};
mod projectiles;
pub use projectiles::{ProjectileKind, ProjectilePosition, TrackedProjectile};

mod weapons;
pub use weapons::{WeaponChange, WeaponObservation, WeaponSample};

const FRESHNESS_US: u64 = 100_000;
const GRENADE_COUNT_MAX: u8 = 2;

fn shot_owner(
    read: &super::interpretation::FireRead,
    bindings: &BTreeMap<u32, usize>,
) -> Option<(usize, u32, usize)> {
    use super::interpretation::FireHeaderStop;
    if read.header_stop != FireHeaderStop::Read {
        return None;
    }
    let end = read.header_end_bit.filter(|end| *end <= read.source_bits)?;
    let event = read.event.as_ref()?;
    if !event.has_shooter || !event.unit.present || !(0..32).contains(&event.film_index) {
        return None;
    }
    // The historical reader adds 512 for both reference forms. Only the narrow
    // domain-1 probe uses that base; a wide reference carries the absolute slot.
    let slot = if event.unit.probe {
        event.unit.slot
    } else {
        event.unit.slot.checked_sub(512)?
    };
    let id = slot | (event.unit.generation << 30);
    let player = *bindings.get(&id)?;
    (player == event.film_index as usize).then_some((player, id, end))
}

fn value(fields: &[ComponentField], name: &str, width: usize) -> Option<u64> {
    let mut found = fields.iter().filter(|f| f.name == name);
    let f = found.next()?;
    (found.next().is_none()
        && f.width == width
        && f.raw.bit_len == width
        && f.raw.bytes.len() == width.div_ceil(8))
    .then(|| f.raw.low_u64())
}
fn component<'a>(record: &'a EntityRecord, name: &str) -> Option<&'a EntityComponentRead> {
    let mut found = record.components.iter().filter(|c| c.name == name);
    let c = found.next()?;
    (found.next().is_none() && c.status == ComponentReadStatus::Complete).then_some(c)
}
fn admitted_biped(state: &ReplicationDecodeState, id: u32) -> bool {
    state.slots.get(&(id & 0x3fff_ffff)).is_some_and(|binding| {
        binding.archetype == 35 && (binding.generation_any || binding.full_id == id)
    })
}
fn owner(record: &EntityRecord) -> Option<usize> {
    if record.archetype != Some(35) || record.stop != EntityViewStop::Complete {
        return None;
    }
    let defaults = record.default_state.as_ref()?;
    if defaults.status != DefaultStateStatus::Complete {
        return None;
    }
    let player = value(&defaults.fields, "player_index", 5)?;
    let control = component(record, "unit-control-component")?;
    (value(&control.fields, "control.present", 1)? == 1
        && value(&control.fields, "control.index", 5)? == player)
        .then_some(player as usize)
}
fn position(
    record: &EntityRecord,
    bounds: PositionBounds,
    region_index_bits: usize,
) -> Option<[f64; 3]> {
    let c = component(record, "object-position-dynamic-precision-component")?;
    let f = &c.fields;
    for gate in [
        "use_prediction",
        "delta",
        "precision_high",
        "default_region",
    ] {
        if value(f, gate, 1)? != 0 {
            return None;
        }
    }
    if value(f, "region_index", region_index_bits)? != u64::from(bounds.region) {
        return None;
    }
    // This two-bit tail is read after finite coordinates; its value is not a validity flag.
    value(f, "finite", 2)?;
    let widths = bounds.axis_bits()?;
    let mut raw = [0; 3];
    for axis in 0..3 {
        raw[axis] = value(f, &format!("position[{axis}]"), widths[axis])? as u32;
    }
    bounds.dequantize(raw, widths)
}
fn aim(record: &EntityRecord) -> Option<[f64; 3]> {
    let f = &component(record, "unit-desired-aiming-vector-component")?.fields;
    if value(f, "single", 1)? != 1 || value(f, "flag", 1)? != 0 {
        return None;
    }
    Some(desired_direction(
        value(f, "yaw", 12)? as u32,
        value(f, "pitch", 11)? as u32,
    ))
}
/// Controlled yaw/pitch clips establish direction and monotonicity; exact engine
/// quantizer endpoints and camera orientation remain uncalibrated.
fn desired_direction(yaw: u32, pitch: u32) -> [f64; 3] {
    let yaw = f64::from(yaw) * std::f64::consts::TAU / 4096.;
    let pitch = (f64::from(pitch) - 1024.) * std::f64::consts::TAU / 2048.;
    [
        pitch.cos() * yaw.cos(),
        pitch.cos() * yaw.sin(),
        pitch.sin(),
    ]
}
#[derive(Clone, Copy)]
struct Update {
    time: u64,
    channel: usize,
    value: Option<[f64; 3]>,
}
#[derive(Default)]
struct Builder {
    xuid: u64,
    gamertag: String,
    identity_source: Option<PoseSource>,
    observations: Vec<PoseObservation>,
    shots: Vec<ShotObservation>,
    team_samples: Vec<TeamSample>,
    weapon_observations: Vec<WeaponObservation>,
    weapon_gaps: Vec<(u64, usize)>,
    grenade_observations: Vec<GrenadeObservation>,
    grenade_gaps: Vec<(u64, usize)>,
    vitality_observations: Vec<VitalityObservation>,
    vitality_gaps: Vec<(u64, usize)>,
    camouflage_observations: Vec<CamouflageObservation>,
    camouflage_gaps: Vec<(u64, usize)>,
    equipment_observations: Vec<EquipmentObservation>,
    equipment_gaps: Vec<(u64, usize)>,
    lives: Vec<PlayerLife>,
    updates: Vec<Update>,
    positions: usize,
    aims: usize,
}
impl Builder {
    fn close(&mut self, time: u64, reason: LifeEnd) {
        if let Some(life) = self.lives.last_mut()
            && time < life.end_us
        {
            life.end_us = time.max(life.start_us);
            life.end_reason = reason;
        }
        self.clear(time);
    }
    fn clear(&mut self, time: u64) {
        self.clear_uncertain_frame(time);
        self.weapon_gaps
            .push((time, self.weapon_observations.len()));
    }
    fn clear_uncertain_frame(&mut self, time: u64) {
        self.camouflage_gaps
            .push((time, self.camouflage_observations.len()));
        self.equipment_gaps
            .push((time, self.equipment_observations.len()));
        self.vitality_gaps
            .push((time, self.vitality_observations.len()));
        self.grenade_gaps
            .push((time, self.grenade_observations.len()));
        for channel in 0..2 {
            self.updates.push(Update {
                time,
                channel,
                value: None,
            });
        }
    }
    fn finish(mut self, film_index: usize) -> PlayerReplay {
        self.lives.retain(|l| l.start_us < l.end_us);
        self.shots.retain(|shot| {
            self.lives.iter().any(|life| {
                life.entity_id == shot.entity_id
                    && life.start_us <= shot.timestamp_us
                    && shot.timestamp_us < life.end_us
            })
        });
        self.observations.retain(|o| {
            self.lives.iter().any(|l| {
                l.entity_id == o.entity_id
                    && l.start_us <= o.pose.timestamp_us
                    && o.pose.timestamp_us < l.end_us
            })
        });
        self.positions = self
            .observations
            .iter()
            .filter(|o| o.pose.position.is_some())
            .count();
        self.aims = self
            .observations
            .iter()
            .filter(|o| o.pose.aim.is_some())
            .count();
        // Apply lifetime bounds before sampling, including independently recovered
        // death summaries. An observation from one life never crosses into another.
        self.updates.retain(|u| {
            self.lives
                .iter()
                .any(|l| l.start_us <= u.time && u.time < l.end_us)
        });
        let mut timeline = BTreeMap::<u64, Vec<(Update, bool)>>::new();
        // Boundaries precede observations when death/respawn share a timestamp.
        for life in &self.lives {
            for channel in 0..2 {
                timeline.entry(life.end_us).or_default().push((
                    Update {
                        time: life.end_us,
                        channel,
                        value: None,
                    },
                    false,
                ));
            }
        }
        for u in self.updates {
            timeline.entry(u.time).or_default().push((u, false));
            if u.value.is_some() {
                timeline
                    .entry(u.time.saturating_add(FRESHNESS_US + 1))
                    .or_default()
                    .push((u, true));
            }
        }
        let mut state = [None, None];
        let mut last = [None, None];
        let mut samples = Vec::new();
        for (time, updates) in timeline {
            // Expiries first; a coincident fresh observation wins.
            for (u, expiry) in updates
                .iter()
                .filter(|(_, e)| *e)
                .chain(updates.iter().filter(|(_, e)| !*e))
            {
                if *expiry {
                    if last[u.channel] == Some(u.time) {
                        state[u.channel] = None;
                        last[u.channel] = None;
                    }
                } else {
                    state[u.channel] = u.value;
                    last[u.channel] = u.value.map(|_| time);
                }
            }
            samples.push(Pose {
                timestamp_us: time,
                position: state[0],
                aim: state[1],
            });
        }
        let weapon_samples =
            weapons::timeline(&mut self.weapon_observations, self.weapon_gaps, &self.lives);
        let grenade_samples = grenades::timeline(
            &mut self.grenade_observations,
            self.grenade_gaps,
            &self.lives,
        );
        let vitality_samples = vitality::timeline(
            &mut self.vitality_observations,
            self.vitality_gaps,
            &self.lives,
        );
        PlayerReplay {
            camouflage_samples: camouflage::timeline(
                &mut self.camouflage_observations,
                self.camouflage_gaps,
                &self.lives,
            ),
            camouflage_observations: self.camouflage_observations,
            equipment_samples: equipment::timeline(
                &mut self.equipment_observations,
                self.equipment_gaps,
                &self.lives,
            ),
            equipment_observations: self.equipment_observations,
            vitality_samples,
            vitality_observations: self.vitality_observations,
            grenade_samples,
            grenade_observations: self.grenade_observations,
            weapon_samples,
            weapon_observations: self.weapon_observations,
            film_index,
            xuid: self.xuid,
            gamertag: self.gamertag,
            identity_source: self.identity_source,
            lives: self.lives,
            samples,
            observations: self.observations,
            shots: self.shots,
            team_samples: self.team_samples,
            position_records: self.positions,
            aim_records: self.aims,
        }
    }
}

// Pinned LevelUp roster_type8.go layout, interpreted outside canonical Film.
// A missing index, partial body, repeated index, or unexplained tail is refused.
type RosterEntry = (usize, u64, String, usize, usize);
fn roster(payload: &[u8], personalization_bits: usize) -> Option<Vec<RosterEntry>> {
    use super::resolved::interpretation::player_slot::{SlotReader, slot_body};
    use crate::theater::parser::bits::Cursor;
    let mut r = SlotReader::new(Cursor::new(payload, 0)?, false);
    let count = r.read("announced", 32)?;
    if count > 32 {
        return None;
    }
    let mut entries = Vec::new();
    for _ in 0..count {
        let start = r.cursor.position;
        if r.read("option_gate", 1)? != 0 {
            return None;
        }
        let index = r.read("option", 5)? as usize;
        let xuid = r.read("identity_wire", 64)?.swap_bytes();
        let (name, _) = slot_body(&mut r, personalization_bits)?;
        if xuid == 0
            || name.is_empty()
            || name.chars().any(char::is_control)
            || entries
                .iter()
                .any(|e: &RosterEntry| e.0 == index || e.1 == xuid)
        {
            return None;
        }
        entries.push((index, xuid, name, start, r.cursor.position));
    }
    let remaining = payload.len() * 8 - r.cursor.position;
    if remaining > 7 || (remaining > 0 && r.read("padding", remaining)? != 0) {
        return None;
    }
    Some(entries)
}

impl TheaterRuntime {
    /// Resolve source-linked biped spawns and ordinary absolute position/desired
    /// aim updates with bounds selected from the map asset. Unknown records stay gaps.
    /// Map calibration is validated before film evidence is interpreted.
    /// Lifetimes end at the earliest next spawn, matching DELETE, recovered death
    /// summary, or film end; these are observed/derived bounds, not complete lives.
    pub fn resolve_player_replay(
        &self,
        map_asset_id: uuid::Uuid,
        maps: &MapGlossary,
        duration_us: u64,
    ) -> Result<PlayerReplaySet, ReplayError> {
        let map = maps.resolve(&map_asset_id)?;
        self.resolve_player_replay_with_bounds(map.bounds, map.region_index_bits, duration_us)
            .map_err(ReplayError::InvalidFilm)
    }

    fn resolve_player_replay_with_bounds(
        &self,
        bounds: PositionBounds,
        region_index_bits: usize,
        duration_us: u64,
    ) -> Result<PlayerReplaySet, &'static str> {
        let widths = bounds.axis_bits().ok_or("Invalid map bounds")?;
        let table = self
            .interpretations()
            .player_table
            .as_ref()
            .ok_or("Missing player table")?;
        if table.error.is_some() {
            return Err("Refused player table");
        }
        let mut builders = BTreeMap::<usize, Builder>::new();
        for player in &table.slots {
            if builders
                .insert(
                    player.film_index,
                    Builder {
                        xuid: player.xuid,
                        gamertag: player.gamertag.clone(),
                        ..Default::default()
                    },
                )
                .is_some()
            {
                return Err("Ambiguous roster");
            }
        }
        let film = self.film();
        let weapon_slots = weapons::slot_indices(&film.registry.body.registry);
        let weapon_ammo_slots = weapons::ammo_indices(&film.registry.body.registry);
        let origin = film
            .replication_stream
            .iter()
            .flat_map(|c| &c.body.packets)
            .map(|p| p.header.timestamp_us)
            .filter(|t| *t != 0)
            .min()
            .ok_or("Missing packet clock")?;
        let mut config = V41DecodeConfig::default();
        config.profile.movement.world_object.axis_bits = widths.map(|w| w as u64);
        config.profile.movement.world_object.index_bits = region_index_bits as u64;
        config.profile.movement.world_object.region = bounds.region;
        // These are v41 event grammar choices of the contextual resolver, not
        // map calibration. The canonical reader retains its conservative defaults.
        config.event_quantized_vectors = true;
        config.event_fire_v3 = true;
        let event_position = |level| {
            Some(
                crate::theater::parser::v41::context::movement::PrecisionDescriptor {
                    region: bounds.region,
                    index_bits: region_index_bits as u64,
                    axis_bits: bounds.axis_bits_at_precision(level)?.map(|w| w as u64),
                },
            )
        };
        config.projectile_event_positions = Some(
            replication_stream::frame::events::ProjectilePositionEncoding {
                detonation: event_position(15).ok_or("Invalid detonation bounds")?,
                impact: event_position(12).ok_or("Invalid impact bounds")?,
            },
        );
        config.guard_rejected_frame_deletes = true;
        let evidence = replication_stream::keyframes::declarations::evidence_from_keyframes(film);
        let mut reference_frames = reference_frames::ReferenceFrames::default();
        let mut state = ReplicationDecodeState {
            anticipated: Some(evidence.anticipated),
            ..Default::default()
        };

        let mut decode = DecodeContext {
            config: &config,
            registry: &film.registry.body.registry,
            state: &mut state,
            event_gate15: Some(true),
        };
        // Full IDs include generation. Schema declarations alone never grant ownership.
        let mut bindings = BTreeMap::<u32, usize>::new();
        let mut team_bindings = BTreeMap::<u32, usize>::new();
        let mut equipment_objects = equipment::ObjectIdentities::default();
        let mut projectiles = projectiles::Projectiles::default();
        let mut diagnostics = ReplayDiagnostics::default();
        let mut previous_time = 0;
        let fire_reads: BTreeMap<_, _> = self
            .interpretations()
            .packets
            .iter()
            .filter_map(|p| {
                p.fire
                    .as_ref()
                    .map(|fire| ((p.source.chunk, p.source.packet), fire))
            })
            .collect();
        for chunk in &film.replication_stream {
            decode.state.current_chunk = chunk.source.index.unwrap_or(chunk.source_position as i64);
            for (packet_index, packet) in chunk.body.packets.iter().enumerate() {
                let Some(payload) = chunk.payload(packet) else {
                    continue;
                };
                if packet.header.packet_type == 8 {
                    let Some(time) = packet
                        .header
                        .timestamp_us
                        .checked_sub(origin)
                        .filter(|t| *t <= duration_us)
                    else {
                        continue;
                    };
                    // Reference type-8 interpretation; require the complete bounded
                    // list, explicit indices, and only byte-alignment padding.
                    if let Some(entries) = roster(payload, table.report.perso_bytes * 8) {
                        for (index, xuid, name, start, end) in entries {
                            if let Some(b) = builders.get(&index) {
                                if b.xuid != xuid {
                                    return Err(
                                        "Roster index reuse requires a time-varying identity resolver",
                                    );
                                }
                            } else {
                                builders.insert(
                                    index,
                                    Builder {
                                        xuid,
                                        gamertag: name,
                                        identity_source: Some(PoseSource {
                                            chunk_index: chunk.source.index,
                                            payload_byte: packet.source.payload.start,
                                            start_bit: start as i64,
                                            end_bit: end as i64,
                                        }),
                                        ..Default::default()
                                    },
                                );
                            }
                        }
                    } else {
                        diagnostics.refused_rosters += 1;
                        equipment_objects.clear();
                        projectiles.clear(time);
                        for b in builders.values_mut() {
                            b.clear(time);
                        }
                    }
                }
                if let Some(records) = evidence
                    .baselines
                    .get(&(chunk.source_position, packet_index))
                {
                    for record in records {
                        reference_frames.baseline(record);
                    }
                }
                let body =
                    replication_stream::decode(packet.header.packet_type, payload, &mut decode);

                if let Ok(ReplicationStreamPacketBody::KeyframesPacketBody(table)) = &body
                    && table.stop != KeyframeChainStop::End
                {
                    let last = table.records.last();
                    let reason = match last.and_then(|a| a.record.as_ref()).map(|r| &r.stop) {
                        Some(KeyframeStop::UnsupportedComponent { name, .. }) => {
                            format!("keyframe/unsupported_component/{name}")
                        }
                        Some(KeyframeStop::RuntimeContextUnavailable { name, field, .. }) => {
                            format!("keyframe/runtime_context/{name}/{field}")
                        }
                        Some(stop) if *stop != KeyframeStop::Complete => {
                            format!("keyframe/record/{stop:?}")
                        }
                        _ => format!("keyframe/table/{:?}", table.stop),
                    };
                    let bit = last.map_or(1, |a| a.end_bit);
                    diagnostics.blocker(
                        reason,
                        packet.header.timestamp_us.checked_sub(origin),
                        PoseSource {
                            chunk_index: chunk.source.index,
                            payload_byte: packet.source.payload.start,
                            start_bit: bit,
                            end_bit: bit,
                        },
                    );
                }
                if packet.header.packet_type != 0 {
                    continue;
                }
                diagnostics.frame_packets += 1;
                let Some(time) = packet
                    .header
                    .timestamp_us
                    .checked_sub(origin)
                    .filter(|t| *t <= duration_us)
                else {
                    diagnostics.out_of_range_packets += 1;
                    continue;
                };
                if time < previous_time {
                    return Err(
                        "Nonmonotonic packet clocks; explicit clock reconstruction required",
                    );
                }
                previous_time = time;
                // The supported fire header precedes the entity list. A later
                // NEW in this packet cannot retroactively establish its owner.
                if let Some(read) = fire_reads.get(&(chunk.source_position, packet_index))
                    && let Some((player, id, end_bit)) = shot_owner(read, &bindings)
                {
                    builders
                        .get_mut(&player)
                        .unwrap()
                        .shots
                        .push(ShotObservation {
                            timestamp_us: time,
                            entity_id: id,
                            source: PoseSource {
                                chunk_index: chunk.source.index,
                                payload_byte: packet.source.payload.start,
                                start_bit: 1,
                                end_bit: end_bit as i64,
                            },
                        });
                }
                let Ok(ReplicationStreamPacketBody::FramePacketBody(frame)) = body else {
                    continue;
                };
                let frame_closed = replication_stream::frame::frame_closed(&frame.frame, payload);
                let suspect_delete = replication_stream::frame::suspect_delete(&frame.frame);
                if !frame_closed {
                    diagnostics.unclosed_frames += 1;
                }
                // Every preceding event must establish the next boundary.
                for event in frame.events.records.iter().skip(1) {
                    if event.code != Some(36) {
                        continue;
                    }
                    let read = super::interpretation::fire_events::read_fire_event_at(
                        payload,
                        event.start_bit,
                    );
                    if let Some((player, id, end_bit)) = shot_owner(&read, &bindings)
                        && end_bit <= event.end_bit
                    {
                        builders
                            .get_mut(&player)
                            .unwrap()
                            .shots
                            .push(ShotObservation {
                                timestamp_us: time,
                                entity_id: id,
                                source: PoseSource {
                                    chunk_index: chunk.source.index,
                                    payload_byte: packet.source.payload.start,
                                    start_bit: event.start_bit as i64,
                                    end_bit: end_bit as i64,
                                },
                            });
                    }
                }
                let FrameRead::Decoded(frame) = frame.frame else {
                    let reason = if frame.events.stop != EventListStop::Terminator {
                        format!(
                            "frame/events/{:?}/last_code_{:?}",
                            frame.events.stop,
                            frame.events.records.last().and_then(|e| e.code)
                        )
                    } else {
                        format!("frame/refused/{:?}", frame.frame)
                    };
                    diagnostics.blocker(
                        reason,
                        Some(time),
                        PoseSource {
                            chunk_index: chunk.source.index,
                            payload_byte: packet.source.payload.start,
                            start_bit: frame.events.end_bit as i64,
                            end_bit: frame.events.end_bit as i64,
                        },
                    );
                    diagnostics.refused_frames += 1;
                    equipment_objects.clear();
                    projectiles.clear(time);
                    for b in builders.values_mut() {
                        b.clear_uncertain_frame(time);
                    }
                    continue;
                };
                if !matches!(frame.entity_end, Some(ProductionEntityEnd::Marker(_))) {
                    let (reason, bit) = match &frame.entity_end {
                        Some(ProductionEntityEnd::Rejected { header }) => {
                            let slot = header.id.unwrap_or_default() & 0x3fff_ffff;
                            let reason = if decode.state.slots.contains_key(&slot) {
                                "view_rejection"
                            } else {
                                "missing_schema"
                            };
                            (
                                format!("frame/entities/{reason}/slot_{slot}"),
                                header.start_bit,
                            )
                        }
                        Some(ProductionEntityEnd::Failure(
                            EntityViewStop::UnsupportedComponent { name, .. },
                        )) => (
                            format!("frame/entities/unsupported_component/{name}"),
                            frame.records.last().map_or(frame.end_bit, |r| r.end_bit),
                        ),
                        other => (format!("frame/entities/{other:?}"), frame.end_bit),
                    };
                    diagnostics.blocker(
                        reason,
                        Some(time),
                        PoseSource {
                            chunk_index: chunk.source.index,
                            payload_byte: packet.source.payload.start,
                            start_bit: bit,
                            end_bit: bit,
                        },
                    );
                    diagnostics.incomplete_entity_frames += 1;
                    // A rejected frame can hide later player updates, but it
                    // does not itself change a known weapon slot. Keep the
                    // sparse inventory until a bound player record is refused
                    // or the life ends; still clear less durable channels.
                    equipment_objects.clear();
                    for b in builders.values_mut() {
                        b.clear_uncertain_frame(time);
                    }
                }
                for r in &frame.records {
                    let Some(id) = r.header.id else {
                        continue;
                    };
                    let record_source = PoseSource {
                        chunk_index: chunk.source.index,
                        payload_byte: packet.source.payload.start,
                        start_bit: r.header.start_bit,
                        end_bit: r.end_bit,
                    };
                    if r.header.kind == RecordKind::New
                        && decode
                            .state
                            .slots
                            .get(&(id & 0x3fff_ffff))
                            .is_none_or(|binding| {
                                binding.full_id != id || r.archetype != Some(binding.archetype)
                            })
                    {
                        if r.archetype == Some(35) {
                            diagnostics.rejected_spawns += 1;
                        }
                        diagnostics.blocker(
                            "frame/spawn/conflicting_schema".into(),
                            Some(time),
                            record_source,
                        );
                        continue;
                    }
                    // A failed suffix or a delta for the admitted generation in the
                    // same packet cannot establish a replacement generation.
                    if r.header.kind == RecordKind::Delta
                        && decode.state.slots.get(&(id & 0x3fff_ffff)).map_or(
                            !frame_closed,
                            |binding| {
                                !binding.generation_any
                                    && binding.full_id != id
                                    && (!frame_closed
                                        || frame.records.iter().any(|other| {
                                            other.header.kind == RecordKind::Delta
                                                && other.header.id == Some(binding.full_id)
                                        }))
                            },
                        )
                    {
                        continue;
                    }
                    let ignored_delete = r.header.kind == RecordKind::Delete
                        && ((suspect_delete == Some(id)
                            && decode.state.deferred_mismatched_delete(id))
                            || (!frame_closed
                                && decode.state.slots.get(&(id & 0x3fff_ffff)).is_some_and(
                                    |binding| {
                                        binding.archetype == 35
                                            && !binding.generation_any
                                            && binding.full_id != id
                                    },
                                )));
                    let stale: Vec<_> = team_bindings
                        .keys()
                        .copied()
                        .filter(|old| {
                            !ignored_delete
                                && *old & 0x3fff_ffff == id & 0x3fff_ffff
                                && (*old != id || r.header.kind == RecordKind::New)
                        })
                        .collect();
                    for old in stale {
                        let player = team_bindings.remove(&old).unwrap();
                        builders
                            .get_mut(&player)
                            .unwrap()
                            .team_samples
                            .push(TeamSample {
                                timestamp_us: time,
                                team: None,
                            });
                    }
                    if r.header.kind == RecordKind::New
                        && r.archetype == Some(9)
                        && r.stop == EntityViewStop::Complete
                        && let Some(defaults) = &r.default_state
                        && defaults.status == DefaultStateStatus::Complete
                        && let Some(player) =
                            value(&defaults.fields, "player_index", 6).map(|v| v as usize)
                        && builders.contains_key(&player)
                    {
                        team_bindings.insert(id, player);
                        builders
                            .get_mut(&player)
                            .unwrap()
                            .team_samples
                            .push(TeamSample {
                                timestamp_us: time,
                                team: None,
                            });
                    }
                    if let Some(&player) = team_bindings.get(&id) {
                        if r.header.kind == RecordKind::Delete {
                            team_bindings.remove(&id);
                            builders
                                .get_mut(&player)
                                .unwrap()
                                .team_samples
                                .push(TeamSample {
                                    timestamp_us: time,
                                    team: None,
                                });
                        } else if let Some(c) = r
                            .components
                            .iter()
                            .find(|c| c.name == "managed-player-team-designator-component")
                        {
                            builders
                                .get_mut(&player)
                                .unwrap()
                                .team_samples
                                .push(TeamSample {
                                    timestamp_us: time,
                                    team: (r.stop == EntityViewStop::Complete
                                        && c.status == ComponentReadStatus::Complete)
                                        .then(|| value(&c.fields, "team", 4).map(|v| v as u8))
                                        .flatten(),
                                });
                        }
                    }
                    if !ignored_delete {
                        reference_frames.observe(r);
                    }
                    projectiles.observe(
                        r,
                        time,
                        record_source,
                        bounds,
                        region_index_bits,
                        &bindings,
                    );
                    equipment_objects.observe(r, record_source);
                    for b in builders.values_mut() {
                        let invalidated = b
                            .equipment_observations
                            .iter()
                            .rev()
                            .find(|o| o.channel == EquipmentChannel::Inventory)
                            .and_then(|o| o.objects.as_ref())
                            .is_some_and(|objects| {
                                objects.iter().any(|o| {
                                    o.entity_id & 0x3fff_ffff == id & 0x3fff_ffff
                                        && (o.entity_id != id || r.header.kind != RecordKind::Delta)
                                })
                            });
                        if invalidated {
                            b.equipment_gaps
                                .push((time, b.equipment_observations.len()));
                        }
                    }

                    if r.stop != EntityViewStop::Complete {
                        diagnostics.incomplete_records += 1;
                    }
                    // A delta from a different generation also invalidates the
                    // older entity even when its replacement spawn was missed.
                    let stale: Vec<_> = bindings
                        .keys()
                        .copied()
                        .filter(|old| {
                            !ignored_delete && *old != id && old & 0x3fff_ffff == id & 0x3fff_ffff
                        })
                        .collect();
                    for old in stale {
                        let p = bindings.remove(&old).unwrap();
                        builders
                            .get_mut(&p)
                            .unwrap()
                            .close(time, LifeEnd::OwnershipChange);
                    }
                    if r.header.kind == RecordKind::New {
                        // A replacement declaration invalidates ownership of that slot.
                        let replaced: Vec<_> = bindings
                            .keys()
                            .copied()
                            .filter(|old| old & 0x3fff_ffff == id & 0x3fff_ffff)
                            .collect();
                        for old in replaced {
                            let p = bindings.remove(&old).unwrap();
                            builders
                                .get_mut(&p)
                                .unwrap()
                                .close(time, LifeEnd::NextSpawn);
                        }
                        let Some(player) = owner(r).filter(|p| builders.contains_key(p)) else {
                            if r.archetype == Some(35) {
                                diagnostics.rejected_spawns += 1;
                            }
                            continue;
                        };
                        let b = builders.get_mut(&player).unwrap();
                        b.close(time, LifeEnd::NextSpawn);
                        bindings.retain(|_, p| *p != player);
                        b.lives.push(PlayerLife {
                            start_us: time,
                            end_us: duration_us,
                            entity_id: id,
                            source: PoseSource {
                                chunk_index: chunk.source.index,
                                payload_byte: packet.source.payload.start,
                                start_bit: r.header.start_bit,
                                end_bit: r.end_bit,
                            },
                            end_reason: LifeEnd::FilmEnd,
                        });
                        bindings.insert(id, player);
                    }
                    let Some(&player) = bindings.get(&id) else {
                        if r.header.kind == RecordKind::Delta {
                            diagnostics.unbound_deltas += 1;
                        }
                        continue;
                    };
                    if r.header.kind != RecordKind::Delete && !admitted_biped(decode.state, id) {
                        diagnostics.blocker(
                            "frame/player/schema_changed".into(),
                            Some(time),
                            record_source,
                        );
                        builders
                            .get_mut(&player)
                            .unwrap()
                            .close(time, LifeEnd::OwnershipChange);
                        bindings.remove(&id);
                        continue;
                    }
                    let b = builders.get_mut(&player).unwrap();
                    if r.header.kind == RecordKind::Delete {
                        b.close(time, LifeEnd::Delete);
                        bindings.remove(&id);
                        continue;
                    }
                    if r.stop != EntityViewStop::Complete {
                        b.clear(time);
                        continue;
                    }
                    // An explicit changed control index invalidates this ownership.
                    if let Some(c) = component(r, "unit-control-component")
                        && (value(&c.fields, "control.present", 1) != Some(1)
                            || value(&c.fields, "control.index", 5) != Some(player as u64))
                    {
                        b.close(time, LifeEnd::OwnershipChange);
                        bindings.remove(&id);
                        continue;
                    }
                    if let Some(observation) = camouflage::observe(
                        r,
                        time,
                        PoseSource {
                            chunk_index: chunk.source.index,
                            payload_byte: packet.source.payload.start,
                            start_bit: r.header.start_bit,
                            end_bit: r.end_bit,
                        },
                    ) {
                        b.camouflage_observations.push(observation);
                    }
                    b.vitality_observations.extend(vitality::observe(
                        r,
                        time,
                        PoseSource {
                            chunk_index: chunk.source.index,
                            payload_byte: packet.source.payload.start,
                            start_bit: r.header.start_bit,
                            end_bit: r.end_bit,
                        },
                    ));
                    if let Some(equipment) = equipment::observe_selection(
                        r,
                        time,
                        PoseSource {
                            chunk_index: chunk.source.index,
                            payload_byte: packet.source.payload.start,
                            start_bit: r.header.start_bit,
                            end_bit: r.end_bit,
                        },
                    ) {
                        b.equipment_observations.push(equipment);
                    }
                    if let Some(inventory) =
                        equipment::observe_inventory(r, time, record_source, &equipment_objects)
                    {
                        b.equipment_observations.push(inventory);
                    }
                    b.weapon_observations.extend(weapons::observe(
                        r,
                        &weapon_slots,
                        &weapon_ammo_slots,
                        time,
                        PoseSource {
                            chunk_index: chunk.source.index,
                            payload_byte: packet.source.payload.start,
                            start_bit: r.header.start_bit,
                            end_bit: r.end_bit,
                        },
                    ));
                    if let Some(counts) = grenades::observe(
                        r,
                        time,
                        GRENADE_COUNT_MAX,
                        PoseSource {
                            chunk_index: chunk.source.index,
                            payload_byte: packet.source.payload.start,
                            start_bit: r.header.start_bit,
                            end_bit: r.end_bit,
                        },
                    ) {
                        if counts.counts.is_none() {
                            diagnostics.refused_grenade_counts += 1;
                        }
                        b.grenade_observations.push(counts);
                    }
                    let pos = position(r, bounds, region_index_bits)
                        .or_else(|| reference_frames.position(r));
                    let direction = aim(r);
                    if let Some(c) = component(r, "object-position-dynamic-precision-component")
                        && value(&c.fields, "precision_high", 1) == Some(1)
                        && pos.is_none()
                    {
                        diagnostics.blocker(
                            "frame/position/unresolved_bounded_reference".into(),
                            Some(time),
                            PoseSource {
                                start_bit: c.start_bit,
                                end_bit: c.end_bit,
                                ..record_source
                            },
                        );
                    }

                    if pos.is_some() || direction.is_some() {
                        b.observations.push(PoseObservation {
                            pose: Pose {
                                timestamp_us: time,
                                position: pos,
                                aim: direction,
                            },
                            entity_id: id,
                            source: PoseSource {
                                chunk_index: chunk.source.index,
                                payload_byte: packet.source.payload.start,
                                start_bit: r.header.start_bit,
                                end_bit: r.end_bit,
                            },
                        });
                    }
                    for (channel, name, v) in [
                        (0, "object-position-dynamic-precision-component", pos),
                        (1, "unit-desired-aiming-vector-component", direction),
                    ] {
                        if r.components.iter().any(|c| c.name == name) {
                            if v.is_some() {
                                if channel == 0 {
                                    b.positions += 1;
                                } else {
                                    b.aims += 1;
                                }
                            }
                            b.updates.push(Update {
                                time,
                                channel,
                                value: v,
                            });
                        }
                    }
                }
            }
        }
        for event in self
            .summary_events()
            .iter()
            .filter(|e| e.kind() == SummaryKind::Death)
        {
            let Some(b) = builders.values_mut().find(|b| b.xuid == event.actor.xuid) else {
                continue;
            };
            if let Some(life) = b
                .lives
                .iter_mut()
                .find(|l| l.start_us <= event.timestamp_us && event.timestamp_us < l.end_us)
            {
                life.end_us = event.timestamp_us;
                life.end_reason = LifeEnd::SummaryDeath;
            }
        }
        Ok(PlayerReplaySet {
            origin_timestamp_us: origin,
            duration_us,
            freshness_us: FRESHNESS_US,
            players: builders.into_iter().map(|(p, b)| b.finish(p)).collect(),
            projectiles: projectiles.finish(duration_us),
            diagnostics,
        })
    }
}

#[cfg(test)]
mod tests;
