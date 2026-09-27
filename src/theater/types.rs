use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::FilmRegistry;

/// Input contract for a versioned, partial decode. Chunk data must be decompressed.
#[derive(Debug, Clone)]
pub struct DecodeOptions {
    /// LegacyFilm major version from the manifest. Only version 41 is supported.
    pub major_version: i32,
    /// Optional match identity retained in portable exports.
    pub match_id: Option<String>,
    /// Manifest duration, in microseconds; otherwise uses the last packet time.
    pub duration_us: Option<u64>,
    /// Retain complete input chunks, packet indexes and accepted bit ranges.
    /// Disabling this creates a compact observation export without opaque source data.
    pub retain_coverage: bool,
}
impl DecodeOptions {
    /// Version-41 defaults. Supply the manifest duration/identity when available.
    pub fn v41() -> Self {
        Self {
            major_version: 41,
            match_id: None,
            duration_us: None,
            retain_coverage: true,
        }
    }
}

/// Structural errors are separate from unsupported record forms in diagnostics.
#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    /// The film version has no checked decoder.
    #[error("unsupported Theater film major version {0}; supported: 41")]
    UnsupportedVersion(i32),
    /// Multiple chunks use the same index.
    #[error("duplicate chunk index {0}")]
    DuplicateChunk(i32),
    /// A packet's header or declared payload is incomplete.
    #[error("truncated chunk {chunk} at byte {offset}")]
    Truncated {
        /// Chunk index.
        chunk: i32,
        /// Byte offset.
        offset: usize,
    },
    /// Required metadata or replication data is absent.
    #[error("missing {0}")]
    Missing(&'static str),
    /// Native kill-source input refusal, preserving its distinct error category.
    #[error(transparent)]
    KillSource(#[from] super::KillSourceFilmError),
    /// A native signed option cannot be represented by this target's indexes.
    #[error(transparent)]
    KillOption(#[from] super::KillDecodeOptionError),
    /// A framed type-1 datum body failed native size validation.
    #[error(transparent)]
    Datums(#[from] super::DatumTableError),
    /// A supported capture's independent guards disagree.
    #[error("inconsistent Theater data: {0}")]
    Inconsistent(String),
}

/// Original location in a decompressed chunk; bits are MSB-first and half-open.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceSpan {
    /// Chunk index.
    pub chunk: i32,
    /// Packet payload byte offset in that chunk (zero for non-packet data).
    pub payload_byte: usize,
    /// First bit relative to the payload.
    pub bit: usize,
    /// First bit after the checked window; does not imply a complete record.
    pub end_bit: usize,
}

/// A value observed at an exact film timestamp, associated with a pawn life.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sample<T> {
    /// Microseconds since the earliest nonzero replication timestamp.
    pub time_us: u64,
    /// Wire identity plus 256 times (generation minus one).
    pub life: u16,
    /// Decoded value; absence of a sample is not a default value.
    pub value: T,
    /// Original checked bytes/bits.
    pub source: SourceSpan,
}

/// Supported raw coordinate bit widths, checked against spawn record boundaries.
///
/// These describe wire encodings, not playlists or map categories. Map bounds
/// predict the widths at precision level 16; spawn boundaries independently
/// check the total width. Other layouts remain unsupported.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoordinateLayout {
    /// X: 13 bits, Y: 12 bits, Z: 11 bits; Aquarius map bounds and spawn.
    X13Y12Z11,
    /// X: 15 bits, Y: 15 bits, Z: 17 bits.
    #[serde(alias = "Controlled")]
    X15Y15Z17,
    /// X: 17 bits, Y: 17 bits, Z: 16 bits; observed in the Bazaar capture.
    X17Y17Z16,
    /// X: 18 bits, Y: 18 bits, Z: 15 bits.
    #[serde(alias = "Ranked")]
    X18Y18Z15,
}
impl CoordinateLayout {
    /// Width of each raw axis, in X/Y/Z order.
    pub const fn axis_bits(self) -> [usize; 3] {
        match self {
            Self::X13Y12Z11 => [13, 12, 11],
            Self::X15Y15Z17 => [15, 15, 17],
            Self::X17Y17Z16 => [17, 17, 16],
            Self::X18Y18Z15 => [18, 18, 15],
        }
    }

    pub(super) fn coordinate_bits(self) -> usize {
        self.axis_bits().iter().sum()
    }

    /// Position component: five leading bits, coordinates, two trailing bits.
    pub(super) fn position_bits(self) -> usize {
        5 + self.coordinate_bits() + 2
    }
}

/// Recorded raw position. Use independently supplied [`super::CoordinateBounds`]
/// for world units and origin; bounds are not extracted from the film.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    /// X, Y, Z extraction windows.
    pub raw: [u32; 3],
    /// Whether this value comes from a spawn rather than a position update.
    pub spawn: bool,
    /// Input axes in the same checked input chain, when present.
    pub input: Option<InputAxes>,
}
/// Recorded velocity. [`Velocity::speed`] and [`Velocity::vector`] decode the
/// magnitude in world units per second without enlarging serialized samples.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "form")]
pub enum Velocity {
    /// Explicit stationary form: pawn `01`, projectile `1`.
    Stationary,
    /// Quantized unit direction and nonlinear magnitude (pawn `00`, projectile `0`).
    Directed {
        /// Original 19-bit direction code.
        direction_code: u32,
        /// Unit direction in film X/Y/Z axes, before any map-coordinate scaling.
        direction: [f32; 3],
        /// Original 10-bit magnitude. Not speed in world units; zero in this
        /// form is distinct from the explicit stationary form.
        magnitude_code: u16,
    },
}
/// Two analog movement axes; 31 is neutral and observed values span 0–62.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputAxes {
    /// Forward/back axis.
    pub forward: u8,
    /// Left/right axis.
    pub left: u8,
}
/// Raw desired aim; angular conversions remain provisional.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Aim {
    /// Cyclic 12-bit yaw.
    pub yaw: u16,
    /// 11-bit pitch, upward increasing near level 1024.
    pub pitch: u16,
}
/// Raw body-health observation, without calibrated hit points.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BodyVitality {
    /// Raw observed amount, full-scale preview 126.
    pub raw: u8,
    /// Three-bit state code.
    pub state: u8,
}
/// Raw shield observation and checked regeneration countdown.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShieldVitality {
    /// Raw observed amount, full-scale preview 64.
    pub raw: u8,
    /// Countdown in nominal 60 Hz ticks, up to 300.
    pub delay_ticks: u16,
    /// Four-bit state code.
    pub state: u8,
}
/// Firing activity; sequence gaps do not establish missing bullets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Firing {
    /// Wrapping eight-bit event sequence.
    pub sequence: u8,
    /// Raw 40-bit weapon window. See [`Self::weapon_slot`] and
    /// [`Self::weapon_fingerprint`] for its checked subfields.
    pub weapon_window: u64,
}
impl Firing {
    /// Recorded inventory slot from the checked `slot3 + 1 + key32 + 0100`
    /// form. Slots 0 and 1 are independently matched to magazine components;
    /// other forms remain unknown. No magazine observation is required.
    pub fn weapon_slot(&self) -> Option<u8> {
        if self.weapon_window & 0xf != 4 {
            return None;
        }
        match self.weapon_window >> 36 {
            1 => Some(0),
            3 => Some(1),
            _ => None,
        }
    }

    /// Stable 32-bit fingerprint in the checked firing form. This is calibrated
    /// against recorded weapon references, not an established universal asset ID.
    pub fn weapon_fingerprint(&self) -> Option<u32> {
        self.weapon_slot()?;
        Some((self.weapon_window >> 4) as u32)
    }
}
/// Observed magazine quantity, not reserve inventory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Magazine {
    /// Zero-based weapon slot, currently 0 or 1.
    pub slot: u8,
    /// Checked quantity 0–36; zero has a different wire width.
    pub rounds: u8,
}
/// Held-weapon evidence from firing, or an observed weapon-set change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeaponObservation {
    /// Zero-based slot from a checked firing field, magazine association or selection.
    pub slot: Option<u8>,
    /// Firing identifies this window independently of ammo. None invalidates the held name.
    pub weapon_window: Option<u64>,
}
/// Scope stage, deliberately distinct from magnification or camera FOV.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZoomStage {
    /// Stage zero.
    Unscoped,
    /// Stage one.
    First,
    /// Stage two.
    Second,
}
impl ZoomStage {
    /// Raw stage number 0/1/2.
    pub fn raw(self) -> u8 {
        match self {
            Self::Unscoped => 0,
            Self::First => 1,
            Self::Second => 2,
        }
    }
}
/// A checked spawn and its associated death/next-spawn boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Life {
    /// Unique identity across the four observed generation tags.
    pub id: u16,
    /// Eight-bit wire identity.
    pub wire: u8,
    /// Logical generation 1..=4, encoded on the wire as 01, 10, 11, 00.
    pub generation: u8,
    /// Roster index, independent from spawn serial.
    pub player: u8,
    /// Spawn time in film microseconds.
    pub start_us: u64,
    /// Next spawn for this player, or the film duration.
    pub end_us: u64,
    /// Summary death, when present; zero is not synthesized at death.
    pub death_us: Option<u64>,
    /// The next life begins after a checked round-clock transition without death.
    pub round_reset: bool,
    /// Raw spawn coordinates.
    pub position: [u32; 3],
    /// Checked coordinate format.
    pub layout: CoordinateLayout,
    /// Checked spawn fields.
    pub source: SourceSpan,
}
/// A recorded model-region choice. IDs match Game CMS RegionData identifiers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArmorRegion {
    /// Model region; signed string identifier, not a cosmetic inventory ID.
    pub region_id: i32,
    /// Selected geometry permutation for this region.
    pub permutation_id: i32,
}
/// Recorded attachment tag identifiers, matched to Game CMS TagId fields.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArmorAttachments {
    /// Chest attachment tag; zero does not identify an item.
    pub chest_tag_id: i32,
    /// Utility/hip attachment tag; zero does not identify an item.
    pub utility_tag_id: i32,
    /// Wrist attachment tag; zero does not identify an item.
    pub wrist_tag_id: i32,
    /// Left shoulder tag; geometry-based variants may use other fields.
    pub left_shoulder_tag_id: i32,
    /// Right shoulder tag; geometry-based variants may use other fields.
    pub right_shoulder_tag_id: i32,
}
/// Partial recorded customization. Unparsed accessory slots are not inferred.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArmorAppearance {
    /// The 22 base model-region pairs in the supported snapshot form.
    pub regions: Vec<ArmorRegion>,
    /// Matches Game CMS VisorId.m_identifier; may be a sentinel for other forms.
    pub visor_id: i32,
    /// Matches Game CMS ColorVariant.m_identifier.
    pub visor_color_id: i32,
    /// Matches armor-theme VariantId.m_identifier; not a universal core ID.
    pub armor_variant_id: i32,
    /// Matches armor-coating StyleId.m_identifier; not the inventory path/hash.
    pub coating_style_id: i32,
    /// Checked attachment fields. None in older exports means unavailable, not empty.
    #[serde(default)]
    pub attachments: Option<ArmorAttachments>,
    /// Four recorded mythic-effect identifiers matching Game CMS FxIds, padded
    /// with zeroes. None in older exports means unavailable.
    #[serde(default)]
    pub mythic_effect_ids: Option<[i32; 4]>,
}
/// Player-level snapshot, which can precede spawning and persists independently of lives.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppearanceSample {
    /// Recorded packet timestamp relative to the film origin.
    pub time_us: u64,
    /// Identifiers read from the film, without live-account defaults or name lookups.
    pub value: ArmorAppearance,
    /// Roster and appearance window; intervening fields remain opaque.
    pub source: SourceSpan,
}
/// Per-player typed streams. Empty streams mean no accepted observations.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PlayerTrack {
    /// Roster index.
    pub id: u8,
    /// XUID encoded as text to avoid JavaScript integer precision loss.
    pub xuid: Option<String>,
    /// Recorded gamertag, or a fallback for an unbound roster entry.
    pub name: String,
    /// Recorded customization snapshots, bound by XUID/name rather than pawn life.
    #[serde(default)]
    pub appearance: Vec<AppearanceSample>,
    /// Independently identified spawns/lifetimes.
    pub lives: Vec<Life>,
    /// Raw positions, including checked spawns.
    pub positions: Vec<Sample<Position>>,
    /// Pawn velocity observations with exact component source ranges.
    #[serde(default)]
    pub velocities: Vec<Sample<Velocity>>,
    /// Desired aim observations.
    pub aim: Vec<Sample<Aim>>,
    /// Movement input from checked wire-0/generation-1, roster-0 input chains.
    pub inputs: Vec<Sample<InputAxes>>,
    /// Recorded crouch input in supported command tails, including roster 0/1
    /// across respawns and combined jump/crouch forms; not physical posture.
    #[serde(default)]
    pub crouch_input: Vec<Sample<bool>>,
    /// Guarded firing activity.
    pub firing: Vec<Sample<Firing>>,
    /// Victim-bound weapon-hit notifications; amount and full hit payload are unknown.
    #[serde(default)]
    pub damage: Vec<Sample<()>>,
    /// Guarded melee activity, with the opaque weapon window.
    pub melee: Vec<Sample<u64>>,
    /// Guarded grenade throw activity; type and hit outcome are unknown.
    pub grenades: Vec<Sample<()>>,
    /// Reload starts; manual/automatic cause and animation duration are unknown.
    pub reloads: Vec<Sample<()>>,
    /// Magazine values.
    pub magazines: Vec<Sample<Magazine>>,
    /// Checked zero-based selected weapon slot (currently 0 or 1).
    pub selections: Vec<Sample<u8>>,
    /// Firing weapon identities, optional slot associations, and weapon-set invalidations.
    pub weapons: Vec<Sample<WeaponObservation>>,
    /// Partial scope-stage observations.
    pub zoom: Vec<Sample<ZoomStage>>,
    /// Independent body health observations.
    pub body: Vec<Sample<BodyVitality>>,
    /// Independent shield observations.
    pub shields: Vec<Sample<ShieldVitality>>,
}
/// A summary event decoded independently from replication records.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SummaryEvent {
    /// Recorded XUID as decimal text.
    pub xuid: String,
    /// Bound roster index, if available.
    pub player: Option<u8>,
    /// Recorded gamertag.
    pub name: String,
    /// LegacyFilm-relative time in microseconds.
    pub time_us: u64,
    /// Summary kind, including unrecognized numeric codes.
    pub kind: SummaryKind,
    /// Raw metadata byte (medal code for medal events).
    pub metadata: u8,
    /// Raw medal flag.
    pub medal_flag: u8,
    /// Original type byte; for medals this is the recorded sorting weight.
    #[serde(default)]
    pub type_code: Option<u8>,
    /// Named medal and its distinct stats API identifier, when known.
    #[serde(default)]
    pub medal: Option<super::MedalAward>,
    /// Exact 60-byte event tail; intervening identity state is not decoded.
    #[serde(default)]
    pub source: Option<SourceSpan>,
    /// Recorded XUID field, independently of the roster and gamertag.
    #[serde(default)]
    pub identity_source: Option<SourceSpan>,
}
/// Summary kinds; no pairing of kills/deaths is implied.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SummaryKind {
    /// Mode marker.
    Mode,
    /// Death.
    Death,
    /// Kill.
    Kill,
    /// Medal; metadata retains its film code.
    Medal,
    /// Unrecognized code.
    Other(u8),
}
impl SummaryKind {
    /// Interpret the event-type byte and medal flag. Byte decoders separately
    /// validate which flag values their supported recording layout accepts.
    pub const fn from_fields(code: u8, medal_flag: u8) -> Self {
        if medal_flag != 0 {
            return Self::Medal;
        }
        match code {
            10 => Self::Mode,
            20 => Self::Death,
            50 => Self::Kill,
            other => Self::Other(other),
        }
    }
}
/// A checked clock observation. Round signature changes are retained explicitly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClockSample {
    /// LegacyFilm-relative time.
    pub time_us: u64,
    /// Raw 26-bit signature after three uninterpreted leading bits.
    pub signature: u32,
    /// Eight-bit wrapping counter.
    pub counter: u8,
    /// Original location.
    pub source: SourceSpan,
}
/// Recorded projectile path, bound to a checked spawn identity and thrower.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectileTrack {
    /// Recorded entity identity, including the replication class bit.
    pub id: u16,
    /// Raw two-bit generation tag; reused slots start separate tracks.
    #[serde(default = "projectile_generation")]
    pub generation: u8,
    /// Owner roster index from the spawn's recorded reference.
    pub player: u8,
    /// Thrower's life, independent of projectile lifetime.
    pub life: u16,
    /// Projectile spawn time.
    pub start_us: u64,
    /// End of observed tracking: terminal time when available, otherwise one
    /// microsecond after the last observation. Not an explosion or object death.
    pub end_us: u64,
    /// Recorded coordinates, including spawn; never a simulated ballistic arc.
    pub positions: Vec<Sample<[u32; 3]>>,
    /// Recorded fixed-precision velocity, never used to invent positions.
    #[serde(default)]
    pub velocities: Vec<Sample<Velocity>>,
    /// Explicit projectile-at-rest flag; resting does not mean exploding.
    #[serde(default)]
    pub at_rest: Vec<Sample<bool>>,
    /// Terminal-event source, only in the established single-projectile control.
    pub terminal: Option<SourceSpan>,
}
fn projectile_generation() -> u8 {
    1
}
/// A structurally checked region; it may include opaque fields, not decoded semantics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckedRegion {
    /// Original range.
    pub source: SourceSpan,
    /// Record/guard family.
    pub kind: String,
}
/// Diagnostics distinguish unsupported data from decoded observations.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecodeDiagnostics {
    /// Number of replication frames inspected.
    pub frames: usize,
    /// Counts by rejection/coverage reason. These are candidates, not missed events.
    pub rejected: BTreeMap<String, usize>,
    /// All checked windows; their complement remains unparsed. Windows may overlap.
    pub checked_regions: Vec<CheckedRegion>,
    /// Explicit scope limits for consumers of the portable export.
    pub limitations: Vec<String>,
}
/// Typed, serializable partial Theater film, independent of filesystem and network.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[allow(dead_code)]
pub(crate) struct LegacyFilm {
    /// Complete decompressed inputs in caller order, including chunks and bytes
    /// no reader recognized. Present by default; absent in compact/older exports.
    /// Offsets remain relative to these bytes. This preserves source information,
    /// not a claim that all bytes were decoded or that compressed re-encoding exists.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_chunks: Option<Vec<FilmSourceChunk>>,

    /// Direct weapon-hit events, pairing, and optional distance evidence.
    /// Constructors retain direct reads; map-aware decoding adds distance evidence.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weapon_hits: Option<super::FilmWeaponHits>,
    /// Failure of the latest weapon-hit scan. Older exports have no result or
    /// failure; a failed explicit refresh preserves any previous successful result.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weapon_hits_error: Option<String>,
    /// Recovered anchor spans, including gaps across skipped records.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keyframe_record_spans: Option<Vec<super::KeyframeRecordSpan>>,
    /// Native extra-block measurements; not evidence of occupant identity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vehicle_keyframe_states: Option<Vec<super::VehicleKeyframeState>>,
    /// Resolved sampling and world-object precision, with native fallback evidence.
    #[serde(default)]
    pub scan_precision: Option<super::ReplayPrecisionContext>,
    /// Portable JSON schema version, separate from the Halo film major version.
    pub schema_version: u32,
    /// Halo film major version.
    pub major_version: i32,
    /// Optional manifest match identity.
    pub match_id: Option<String>,
    /// LegacyFilm duration in microseconds.
    pub duration_us: u64,
    /// Raw origin timestamp; sample times are relative to this value.
    pub origin_timestamp_us: u64,
    /// Native chunk-one origin, independent of the earliest nonzero timestamp
    /// used by legacy samples. None means unavailable in an older export.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native_clock_origin: Option<super::ReplayClockOriginRead>,
    /// Component names; runtime registry bytes are not treated as stable IDs.
    pub registry: FilmRegistry,
    /// Native registry truncation and bootstrap source identity. Missing in older
    /// exports means unavailable, not a measured zero-byte tail.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub registry_diagnostics: Option<super::FilmRegistrySourceDiagnostics>,
    /// Build and type versions read from the bootstrap, when present.
    #[serde(default)]
    pub identity: Option<super::FilmIdentity>,
    /// Bootstrap player slots and calibration diagnostics, independent of legacy roster recovery.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_table: Option<super::PlayerTable>,
    /// Source player-table refusal and per-call native metric increments. Absent
    /// in older exports means unavailable, not a successful read.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_table_diagnostics: Option<super::ReplayFilmPlayerTableDiagnostics>,
    /// Type-8 session populations; None means no supported build identity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub roster_updates: Option<Vec<super::FilmRosterUpdate>>,
    /// Native long fire records; shooter indices are film indices, not identities.
    #[serde(default)]
    pub fire_events: Vec<super::FilmFireEvent>,
    /// Native grenade throws with explicit grammar fallbacks and scan diagnostics.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grenade_throws: Option<super::GrenadeThrowStream>,
    /// Entity allocation/generation tables and component masks from type-1 packets.
    #[serde(default)]
    pub datum_tables: Vec<super::DatumSnapshot>,
    /// Rejected type-1 payloads, retained in packet order alongside successful tables.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub datum_failures: Vec<super::DatumDecodeFailure>,
    /// Sequential records decoded with an explicitly supplied film/map encoding.
    /// None for legacy exports and when no encoding context was supplied.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replication: Option<super::ReplicationStream>,
    /// Build, format and map context used by the map-aware replication decoder.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<super::V41FilmProfile>,
    /// Native recorded-key verdict, independent of precision availability.
    #[serde(default)]
    pub key: Option<super::FilmKey>,
    /// Native credit/fatal-source decode, retained by the integrated map decoder.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kill_sources: Option<super::FilmKillSourceResult>,
    /// A failed kill-source pass is distinct from an unrequested pass or empty kills.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kill_sources_error: Option<String>,
    /// Native first-event reads, including unknown types and explicit truncation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub head_events: Vec<super::FilmHeadEvent>,
    #[serde(default)]
    pub pickups: super::BipedPickupStream,
    /// Native pickup attempts and counters, with explicit zero-tail provenance.
    #[serde(default)]
    pub native_pickups: super::NativePickupStream,
    /// Native BOT_METADATA declarations, retaining IDs and original name locations.
    #[serde(default)]
    pub bot_metadata: super::FilmBotMetadata,
    /// Native statborg records with all four channels and explicit cap status.
    #[serde(default)]
    pub statborg: super::FilmStatborgStream,
    /// Ordered native source-scan diagnostics, including record-cap truncation.
    /// Does not include later named-event, identity or score-pass diagnostics.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub statborg_diagnostics: Vec<super::StatborgDiagnostic>,
    /// Team designators and refusal counts from managed-player keyframes.
    /// Absent when bootstrap identity cannot establish the corruption-check layout.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_teams: Option<super::FilmPlayerTeams>,
    /// Native bit-scanned highlights and the source for the replay death feed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native_highlights: Option<super::FilmHighlightStream>,
    /// Native death feed and unfiltered replication index table for the death roster.
    /// None marks older exports; read failures and skipped reads are retained inside.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native_identity_inputs: Option<super::FilmIdentityInputs>,
    /// Native chronological death/occupancy scan, including calibration and coverage.
    /// Available after map-aware decoding; absent from older exports.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native_march_facts: Option<super::FilmMarchFacts>,
    /// Vehicle census, creation, position, boarding/exit and occupant-aim reads.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native_vehicles: Option<super::FilmVehicleFacts>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub zoom_events: Vec<super::BipedZoomEvent>,
    /// Native zoom outcomes, with zero-padding provenance. Padded slot/level values
    /// are decoder output, not fully recorded gameplay observations.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub native_zoom_events: Vec<super::FilmNativeZoomEvent>,
    #[serde(default)]
    pub equipment_spawns: super::EquipmentSpawnStream,
    /// Failed native source scan, distinct from a successful scan with no spawns.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub equipment_spawns_error: Option<String>,
    /// Type-117 teleports; position failures retain the dated unit reference.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub translocations: Vec<super::FilmTranslocatorEvent>,
    /// Native type-117 outcomes, including explicitly labeled zero-padded references.
    /// These are decoder observations, not proof of a fully recorded actor/action.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub native_translocations: Vec<super::FilmNativeTranslocatorEvent>,
    /// Distinguishes an empty native scan from older exports containing only legacy reads.
    #[serde(default)]
    pub native_translocations_scanned: bool,
    /// Reference position-scan candidates and rejection reasons, when a map is supplied.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub biped_positions: Option<super::BipedPositionStream>,
    /// Native comb-pattern probe; inferred team labels are spatial heuristics.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keyframe_position_probes: Vec<super::KeyframePositionProbe>,
    /// Legacy weapon-pattern observations and heuristic frame-marker timing.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub weapon_patterns: Vec<super::WeaponPatternChunk>,
    /// Registry-driven ability and camouflage reads, with complete component fields.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub biped_channels: Option<super::BipedChannels>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub biped_channels_error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inventory_deltas: Option<super::InventoryDeltaStream>,
    /// Independent inventory-scan failure; other LegacyFilm observations remain available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inventory_deltas_error: Option<String>,
    /// Equipment transitions, including counter-gated recovery and diagnostic windows.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub equipment_changes: Option<super::EquipmentChangeStream>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub equipment_changes_error: Option<String>,
    /// Predicted/non-predicted ability bodies, thruster impulses and grapple anchors.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ability_states: Option<super::BipedAbilityStates>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ability_states_error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ability_charges: Option<super::AbilityChargeStream>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ability_charges_error: Option<String>,
    /// Native i26 reference lists, including explicit empty lists and closed entries.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit_equipment: Option<super::UnitEquipmentStream>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit_equipment_error: Option<String>,
    /// Native crouch, slide, clamber, sprint and derived-jump observations.
    /// Encoding/map constructors retain the report even on setup failure; inspect
    /// movement_states_error and stats.scanned. None means the pass was not run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub movement_states: Option<super::MovementStateStream>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub movement_states_error: Option<String>,
    /// Raw scanner counters returned on failure, before replay resets its inputs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub movement_states_error_stats: Option<super::MovementStateStats>,
    /// Native projectile mobile lifetimes, available with map quantization and a slot band.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native_projectiles: Option<super::WorldObjectTrackStream>,
    /// Equipment placements confirmed against mobile lifetimes, with calibration diagnostics.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub equipment_placements: Option<super::EquipmentPlacementStream>,
    /// Recorded equipment-object state and scan denominators; not inferred uses.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub equipment_state: Option<super::EquipmentStateStream>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub equipment_state_error: Option<String>,
    /// Native managed-objective diagnostic scan, including partial attempt evidence.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub objective_scan: Option<super::ObjectiveScan>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub objective_scan_error: Option<String>,
    /// Independent power-up creation scan for pads, even without confirmed placement widths.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub equipment_pad_creations: Option<super::EquipmentCreationStream>,
    /// Raw ground-weapon creations and ammunition; these are not confirmed drops.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ground_weapon_creations: Option<super::EquipmentCreationStream>,
    /// Complete motion scans for the weapon and power-up pad archetypes.
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub ground_object_tracks: std::collections::BTreeMap<u32, super::WorldObjectTrackStream>,
    /// Presence census for reference world-object archetypes 37, 38, 41 and 42.
    #[serde(default)]
    pub world_object_keyframes: std::collections::BTreeMap<u32, super::WorldObjectKeyframes>,
    /// Catalog-family signatures in recovered biped keyframe records, in bit order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keyframe_loadouts: Vec<super::KeyframeLoadout>,
    /// Carrier-marker evidence with keyframe and record denominators.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub carrier_marks: Option<super::CarrierMarkScan>,
    /// Map-profiled radial progress with raw readings and scan diagnostics.
    /// Partial census remains available with `navpoint_radial_error` on setup failure.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub navpoint_radial: Option<super::NavpointRadialScan>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub navpoint_radial_error: Option<String>,
    /// Raw scalar and per-player managed-property observations from ti=13.
    /// A partial slot census is retained alongside `managed_properties_error`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub managed_properties: Option<super::ManagedPropertyScan>,
    /// A scan that could not run is distinct from a successfully read empty channel.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub managed_properties_error: Option<String>,
    /// Independent six-tier capture bursts, dated with gameplay-chunk metadata.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub capture_bursts_ms: Vec<i64>,
    /// Raw objective footer interactions, retaining film teams and participant XUIDs.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub objective_footer: Vec<super::ObjectiveFooterEvent>,
    /// Ground-weapon family observations, retaining slot generations.
    #[serde(default)]
    pub keyframe_ground_weapons: Vec<super::KeyframeGroundWeapon>,
    /// Inferred ammo, grenades and ability ranks, with ambiguity and fallback diagnostics.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keyframe_inventory: Option<super::KeyframeInventoryStream>,
    /// Independent inventory source failure; any scan counts/fallback remain retained.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keyframe_inventory_error: Option<String>,
    /// Requires a map and a recovered biped slot band; unavailable otherwise.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weapon_changes: Option<super::HeldWeaponChangeStream>,
    /// Independent held-weapon scan failure, distinct from an empty change stream.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weapon_changes_error: Option<String>,
    /// Native creation-to-participant links and signature rejection diagnostics.
    #[serde(default)]
    pub biped_creations: super::BipedCreationStream,
    /// Independent native creation scan failure, distinct from zero creations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub biped_creations_error: Option<String>,
    /// Strictly indexed replication packets, optionally omitted for compact export.
    pub packets: Vec<FilmPacket>,
    /// Roster/lives and all accepted player observations.
    pub players: Vec<PlayerTrack>,
    /// Independent summary events.
    pub summary_events: Vec<SummaryEvent>,
    /// Declared/decoded footer counts and unsupported footer packet types.
    /// None means unavailable in an older export; Some with no packets means
    /// the decoder inspected the supplied chunks and found no summary packets.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary_diagnostics: Option<super::SummaryDecodeDiagnostics>,
    /// Checked clocks, including changes between rounds.
    pub clocks: Vec<ClockSample>,
    /// Supported identity-bound projectile paths; coverage is partial.
    pub projectiles: Vec<ProjectileTrack>,
    /// Unsupported candidates, checked source ranges and scope limitations.
    pub diagnostics: DecodeDiagnostics,
}

/// One checked byte-aligned replication packet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FilmPacket {
    pub chunk_index: i32,
    pub packet_type: u16,
    pub byte_2: u8,
    pub byte_3: u8,
    pub payload_offset: usize,
    pub payload_size: usize,
    pub timestamp_us: u64,
}

/// Counts of event categories represented in a Theater-film summary.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FilmEventCounts {
    pub kills: usize,
    pub deaths: usize,
    pub medals: usize,
}

/// A verbatim decompressed input chunk and its supplied manifest metadata.
/// Kept independently of packet discovery so unknown regions remain accessible.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilmSourceChunk {
    pub index: i32,
    pub chunk_type: i32,
    pub start_time_offset_ms: i64,
    pub duration_ms: i64,
    pub declared_size: i64,
    pub file_relative_path: String,
    pub data: Vec<u8>,
}
impl From<&crate::clients::hi::models::FilmChunkData> for FilmSourceChunk {
    fn from(chunk: &crate::clients::hi::models::FilmChunkData) -> Self {
        let m = &chunk.metadata;
        Self {
            index: m.index,
            chunk_type: m.chunk_type,
            start_time_offset_ms: m.start_time_offset_ms,
            duration_ms: m.duration_ms,
            declared_size: m.size,
            file_relative_path: m.file_relative_path.clone(),
            data: chunk.data.clone(),
        }
    }
}
