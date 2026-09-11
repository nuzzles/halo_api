use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::{FilmPacket, FilmRegistry};

/// Input contract for a versioned, partial decode. Chunk data must be decompressed.
#[derive(Debug, Clone)]
pub struct DecodeOptions {
    /// Film major version from the manifest. Only version 41 is supported.
    pub major_version: i32,
    /// Optional match identity retained in portable exports.
    pub match_id: Option<String>,
    /// Manifest duration, in microseconds; otherwise uses the last packet time.
    pub duration_us: Option<u64>,
    /// Retain packet indexes and accepted bit ranges, for unknown-data inspection.
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
/// These describe wire encodings, not playlists or map categories. The rule that
/// selects the per-axis widths is not yet decoded; other layouts remain unsupported.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoordinateLayout {
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

/// Recorded raw position. World units and origin are not calibrated.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    /// X, Y, Z extraction windows.
    pub raw: [u32; 3],
    /// Whether this value comes from a spawn rather than a position update.
    pub spawn: bool,
    /// Input axes in the same checked input chain, when present.
    pub input: Option<InputAxes>,
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
    /// Film-relative time in microseconds.
    pub time_us: u64,
    /// Summary kind, including unrecognized numeric codes.
    pub kind: SummaryKind,
    /// Raw metadata byte (medal code for medal events).
    pub metadata: u8,
    /// Raw medal flag.
    pub medal_flag: u8,
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
/// A checked clock observation. Round signature changes are retained explicitly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClockSample {
    /// Film-relative time.
    pub time_us: u64,
    /// Raw 26-bit signature after three uninterpreted leading bits.
    pub signature: u32,
    /// Eight-bit wrapping counter.
    pub counter: u8,
    /// Original location.
    pub source: SourceSpan,
}
/// Controlled grenade track; general Ranked projectile grammar remains unsupported.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectileTrack {
    /// Checked identity in the two controlled paths.
    pub id: u16,
    /// Owner roster index, established only under the single-player control guards.
    pub player: u8,
    /// Thrower's life, independent of projectile lifetime.
    pub life: u16,
    /// Projectile spawn time.
    pub start_us: u64,
    /// Checked terminal-event time, not a decoded explosion center.
    pub end_us: u64,
    /// Recorded coordinates, including spawn; never a simulated ballistic arc.
    pub positions: Vec<Sample<[u32; 3]>>,
    /// Terminal-event source.
    pub terminal: SourceSpan,
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
pub struct Film {
    /// Portable JSON schema version, separate from the Halo film major version.
    pub schema_version: u32,
    /// Halo film major version.
    pub major_version: i32,
    /// Optional manifest match identity.
    pub match_id: Option<String>,
    /// Film duration in microseconds.
    pub duration_us: u64,
    /// Raw origin timestamp; sample times are relative to this value.
    pub origin_timestamp_us: u64,
    /// Component names; runtime registry bytes are not treated as stable IDs.
    pub registry: FilmRegistry,
    /// Strictly indexed replication packets, optionally omitted for compact export.
    pub packets: Vec<FilmPacket>,
    /// Roster/lives and all accepted player observations.
    pub players: Vec<PlayerTrack>,
    /// Independent summary events.
    pub summary_events: Vec<SummaryEvent>,
    /// Checked clocks, including changes between rounds.
    pub clocks: Vec<ClockSample>,
    /// Only the established controlled grenade tracks.
    pub projectiles: Vec<ProjectileTrack>,
    /// Unsupported candidates, checked source ranges and scope limitations.
    pub diagnostics: DecodeDiagnostics,
}
