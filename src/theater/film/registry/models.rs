//! Reference registry models.

use crate::theater::film::{ComponentSlot, EntitySchemaId, MapAssetId, MapVersionId, TagId};

/// Typed setup retained by a parsed film. Source bytes and partial structural
/// reads stay in the parser's `ParsedRegistry`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Registry {
    /// Named recorded settings; absent values do not imply game defaults.
    pub settings: crate::theater::film::MatchSettings,
    /// Complete initial table in source-slot order, including vacant slots.
    /// Film parsing fails if this table cannot be decoded.
    pub players: Vec<crate::theater::film::InitialPlayer>,
    /// Recorded film and format versions. Parsing rejects an incomplete header.
    pub version: RegistryVersion,
    /// Recorded entity schemas in index order.
    pub schemas: Vec<EntitySchema>,
    /// Recorded game mode, recorder simulation, and level identity.
    pub options: RegistryOptions,
    /// Recorded map identity and its catalogue classification.
    pub map: RegistryMap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RegistryVersion {
    pub film: u32,
    pub format: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EntitySchema {
    pub index: EntitySchemaId,
    /// Known meaning of this film-local schema, when its registry layout is verified.
    pub kind: EntitySchemaKind,
    /// Component definitions recorded in the film, regardless of whether the
    /// schema's meaning is known.
    pub components: Vec<RegistryComponentDefinition>,
}

/// Entity-definition identities for the verified v41/27 registry layout.
/// These are labels from the game executable, not fields recorded in the film.
/// Indices 2 and 10 both name `managed-engine-object` and share one kind;
/// [`EntitySchema::index`] distinguishes them. Index 8 has no known name.
/// Source: <https://github.com/OpenSpartan/libfilmshell/blob/main/src/format.rs>
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum EntitySchemaKind {
    ForgeEngineGlobals,
    CampaignEngineGlobals,
    ManagedEngineObject,
    GameKeyframingSystem,
    GameTimeSystem,
    /// The game-engine-player schema; its fields include respawn and loadout state.
    PlayerState,
    /// The game's `game-engine-statborg` schema.
    StatTracker,
    ForgeSimEntity,
    /// The managed-engine-player schema, distinct from a physical biped.
    Player,
    /// Objective logic and presentation, not necessarily a loose flag.
    ManagedObjective,
    ManagedNavpoint,
    ManagedNetworkedProperty,
    EngineCrew,
    NavPointScripting,
    GameComposer,
    /// The game's `game_music` schema.
    MusicState,
    /// The game's `scripted_effects` schema.
    EffectState,
    /// The game's `scripted_sound_placements` schema.
    SoundPlacement,
    SpawnFilter,
    GameFlock,
    GamePhysics,
    SelectableSpawnFilterManager,
    Persistence,
    GamePowerframe,
    SupplyLinesBlocked,
    SupplyLinesItems,
    GameNarrativeMoment,
    ManagedRespawnManager,
    TacMapIcon,
    TacMapMission,
    TacMapAreaOfInterest,
    TacMapDisplayAsset,
    TacMapSettings,
    /// Physical biped entity; this is distinct from the managed player schema.
    Biped,
    Item,
    Equipment,
    Generic,
    GenericGarbage,
    Vehicle,
    Projectile,
    Weapon,
    Device,
    Asset,
    GameMatchflow,
    StateBroker,
    ManagedParticipant,
    ForgeEnginePlayerData,
    ManagedRadialMenu,
    /// Unnamed or unverified schema; its index and components remain available.
    Unknown,
}

impl EntitySchema {
    /// Resolve a component slot only within this entity schema.
    pub fn component(&self, slot: ComponentSlot) -> Option<&RegistryComponentDefinition> {
        self.components.get(slot.index())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RegistryComponentDefinition {
    pub name: String,
    /// The recorded encoding parameter; its interpretation is component-specific.
    pub precision_level: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RegistryOptions {
    /// Broad engine game category, not the Slayer/Oddball/Infection ruleset.
    pub game_mode: RegistryGameMode,
    pub recorder_simulation: u8,
    pub level_tag: TagId,
}

/// Broad game category recorded in the registry's three-bit game-options field.
/// Only code 2, observed across multiplayer films, is supported.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RegistryGameMode {
    Multiplayer = 2,
}

impl RegistryGameMode {
    pub const fn raw(self) -> u8 {
        self as u8
    }
}

impl TryFrom<u8> for RegistryGameMode {
    type Error = crate::theater::film::ParseError;

    fn try_from(code: u8) -> Result<Self, Self::Error> {
        match code {
            2 => Ok(Self::Multiplayer),
            code => Err(Self::Error::UnsupportedGameMode(code)),
        }
    }
}

impl serde::Serialize for RegistryGameMode {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u8(self.raw())
    }
}

impl<'de> serde::Deserialize<'de> for RegistryGameMode {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::try_from(<u8 as serde::Deserialize>::deserialize(deserializer)?)
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RegistryMap {
    /// Exact asset identity recorded in the film.
    pub asset_id: MapAssetId,
    /// Exact version identity recorded in the film.
    pub version_id: MapVersionId,
    /// Catalogue classification of the asset; not a recorded field or build identity.
    pub kind: RegistryMapKind,
}

/// Classification of a recorded map asset against the bundled catalogue.
/// Multiple asset IDs can share a map kind. Unknown assets never imply Forge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RegistryMapKind {
    Aquarius,
    Bazaar,
    Behemoth,
    Breaker,
    Catalyst,
    Chasm,
    Cliffhanger,
    Deadlock,
    Forbidden,
    Forest,
    Fragmentation,
    Highpower,
    Illusion,
    LaunchSite,
    LiveFire,
    Oasis,
    Prism,
    Recharge,
    Scarr,
    Streets,
    Forge,
    Unknown,
}
