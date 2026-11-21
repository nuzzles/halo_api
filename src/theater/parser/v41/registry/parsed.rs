//! Validated registry source and conversion to film setup.
use super::{KNOWN_REGISTRY_FINGERPRINT, source};

/// Structural type-1 read kept inside the parser until public setup is built.
#[derive(Debug)]
pub(crate) struct ParsedRegistry {
    pub(crate) index: usize,
    pub(crate) start_ms: i64,
    pub(crate) data: Vec<u8>,
    pub(crate) body: crate::theater::parser::v41::registry::source::FilmRegistryRead,
    pub(crate) registry: crate::theater::film::Registry,
}

impl From<ParsedRegistry> for crate::theater::film::Registry {
    fn from(parsed: ParsedRegistry) -> Self {
        parsed.registry
    }
}

impl TryFrom<(&source::FilmRegistryRead, &[u8])> for crate::theater::film::Registry {
    type Error = crate::theater::film::ParseError;

    fn try_from((body, data): (&source::FilmRegistryRead, &[u8])) -> Result<Self, Self::Error> {
        use super::source::{
            BootstrapIdentificationFieldKind, GameOptionsEntry, GameOptionsFieldKind,
        };
        use crate::theater::film::registry::{
            EntitySchema, EntitySchemaKind, RegistryComponentDefinition, RegistryGameMode,
            RegistryMap, RegistryOptions, RegistryVersion,
        };

        let version = body
            .header
            .map(|[film, format]| RegistryVersion { film, format })
            .ok_or(crate::theater::film::ParseError::TruncatedRegistryHeader)?;
        let recognized_layout = version.film == 41
            && version.format == 27
            && body.registry.archetypes.len() == 50
            && body.registry.fingerprint() == Some(KNOWN_REGISTRY_FINGERPRINT);
        let schemas = body
            .registry
            .archetypes
            .iter()
            .map(|archetype| EntitySchema {
                index: crate::theater::film::EntitySchemaId(archetype.index as u32),
                kind: if recognized_layout {
                    known_schema_kind(archetype.index)
                } else {
                    EntitySchemaKind::Unknown
                },
                components: archetype
                    .components
                    .iter()
                    .map(|component| RegistryComponentDefinition {
                        name: component.name.clone(),
                        precision_level: component.precision_level,
                    })
                    .collect(),
            })
            .collect();
        let identification = body
            .bootstrap_header
            .as_ref()
            .and_then(|header| header.identification.as_ref());
        let options = identification
            .and_then(|id| id.game_options.as_ref())
            .and_then(|read| {
                let value = |kind, width| {
                    read.entries.iter().find_map(|entry| match entry {
                        GameOptionsEntry::Field {
                            kind: recorded,
                            bits,
                        } if *recorded == kind && bits.width == width => bits
                            .raw
                            .as_ref()
                            .filter(|raw| raw.bit_len == width)
                            .map(|raw| raw.low_u64()),
                        _ => None,
                    })
                };
                Some((
                    value(GameOptionsFieldKind::GameMode, 3)? as u8,
                    value(GameOptionsFieldKind::RecorderSimulation, 3)? as u8,
                    crate::theater::film::TagId(value(GameOptionsFieldKind::LevelTag, 32)? as u32),
                ))
            });
        let map = identification.and_then(|id| {
            let bytes = |kind| {
                id.fields.iter().find_map(|field| {
                    if field.kind != kind || field.width != 128 {
                        return None;
                    }
                    let raw = field.raw.as_ref()?;
                    if raw.bit_len != 128 {
                        return None;
                    }
                    raw.bytes.as_slice().try_into().ok()
                })
            };
            let asset_id = crate::theater::film::MapAssetId(bytes(
                BootstrapIdentificationFieldKind::MapAssetId,
            )?);
            Some(RegistryMap {
                kind: map_kind(asset_id),
                asset_id,
                version_id: crate::theater::film::MapVersionId(bytes(
                    BootstrapIdentificationFieldKind::MapVersionId,
                )?),
            })
        });
        let map = map.ok_or(crate::theater::film::ParseError::MissingRegistryMap)?;
        let (game_mode, recorder_simulation, level_tag) =
            options.ok_or(crate::theater::film::ParseError::IncompleteRegistryOptions)?;
        let options = RegistryOptions {
            game_mode: RegistryGameMode::try_from(game_mode)?,
            recorder_simulation,
            level_tag,
        };
        let game_options = identification
            .and_then(|id| id.game_options.as_ref())
            .filter(|read| read.stop == source::GameOptionsStop::Complete)
            .ok_or(crate::theater::film::ParseError::IncompleteRegistryPlayers)?;
        let players = super::super::replication::stream::roster::initial(
            data,
            game_options.end_bit,
            &super::super::context::players::PlayerContext::recorded(body),
        )
        .ok_or(crate::theater::film::ParseError::IncompleteRegistryPlayers)?;
        Ok(Self {
            settings: super::settings::project(body),
            players,
            version,
            schemas,
            options,
            map,
        })
    }
}

fn map_kind(asset_id: crate::theater::film::MapAssetId) -> crate::theater::film::RegistryMapKind {
    use crate::theater::film::RegistryMapKind as Kind;
    use crate::theater::film::catalog::maps::KnownMap;

    // The registry stores the UUID's first three fields in little-endian order.
    match KnownMap::from_asset_id(uuid::Uuid::from_bytes_le(asset_id.bytes())) {
        Some(KnownMap::Aquarius) => Kind::Aquarius,
        Some(KnownMap::Bazaar) => Kind::Bazaar,
        Some(KnownMap::Behemoth) => Kind::Behemoth,
        Some(KnownMap::Breaker) => Kind::Breaker,
        Some(KnownMap::Catalyst) => Kind::Catalyst,
        Some(KnownMap::Chasm) => Kind::Chasm,
        Some(KnownMap::Cliffhanger) => Kind::Cliffhanger,
        Some(KnownMap::Deadlock) => Kind::Deadlock,
        Some(KnownMap::Forbidden) => Kind::Forbidden,
        Some(KnownMap::Forest) => Kind::Forest,
        Some(KnownMap::Fragmentation) => Kind::Fragmentation,
        Some(KnownMap::Highpower) => Kind::Highpower,
        Some(KnownMap::Illusion) => Kind::Illusion,
        Some(KnownMap::LaunchSite) => Kind::LaunchSite,
        Some(KnownMap::LiveFire) => Kind::LiveFire,
        Some(KnownMap::Oasis) => Kind::Oasis,
        Some(KnownMap::Prism) => Kind::Prism,
        Some(KnownMap::Recharge) => Kind::Recharge,
        Some(KnownMap::Scarr) => Kind::Scarr,
        Some(KnownMap::Streets) => Kind::Streets,
        Some(KnownMap::Forge) => Kind::Forge,
        None => Kind::Unknown,
    }
}

/// Semantic classifications established for the recognized registry layout.
const fn known_schema_kind(index: usize) -> crate::theater::film::EntitySchemaKind {
    use crate::theater::film::EntitySchemaKind;
    match index {
        0 => EntitySchemaKind::ForgeEngineGlobals,
        1 => EntitySchemaKind::CampaignEngineGlobals,
        2 | 10 => EntitySchemaKind::ManagedEngineObject,
        3 => EntitySchemaKind::GameKeyframingSystem,
        4 => EntitySchemaKind::GameTimeSystem,
        5 => EntitySchemaKind::PlayerState,
        6 => EntitySchemaKind::StatTracker,
        7 => EntitySchemaKind::ForgeSimEntity,
        9 => EntitySchemaKind::Player,
        11 => EntitySchemaKind::ManagedObjective,
        12 => EntitySchemaKind::ManagedNavpoint,
        13 => EntitySchemaKind::ManagedNetworkedProperty,
        14 => EntitySchemaKind::EngineCrew,
        15 => EntitySchemaKind::NavPointScripting,
        16 => EntitySchemaKind::GameComposer,
        17 => EntitySchemaKind::MusicState,
        18 => EntitySchemaKind::EffectState,
        19 => EntitySchemaKind::SoundPlacement,
        20 => EntitySchemaKind::SpawnFilter,
        21 => EntitySchemaKind::GameFlock,
        22 => EntitySchemaKind::GamePhysics,
        23 => EntitySchemaKind::SelectableSpawnFilterManager,
        24 => EntitySchemaKind::Persistence,
        25 => EntitySchemaKind::GamePowerframe,
        26 => EntitySchemaKind::SupplyLinesBlocked,
        27 => EntitySchemaKind::SupplyLinesItems,
        28 => EntitySchemaKind::GameNarrativeMoment,
        29 => EntitySchemaKind::ManagedRespawnManager,
        30 => EntitySchemaKind::TacMapIcon,
        31 => EntitySchemaKind::TacMapMission,
        32 => EntitySchemaKind::TacMapAreaOfInterest,
        33 => EntitySchemaKind::TacMapDisplayAsset,
        34 => EntitySchemaKind::TacMapSettings,
        35 => EntitySchemaKind::Biped,
        36 => EntitySchemaKind::Item,
        37 => EntitySchemaKind::Equipment,
        38 => EntitySchemaKind::Generic,
        39 => EntitySchemaKind::GenericGarbage,
        40 => EntitySchemaKind::Vehicle,
        41 => EntitySchemaKind::Projectile,
        42 => EntitySchemaKind::Weapon,
        43 => EntitySchemaKind::Device,
        44 => EntitySchemaKind::Asset,
        45 => EntitySchemaKind::GameMatchflow,
        46 => EntitySchemaKind::StateBroker,
        47 => EntitySchemaKind::ManagedParticipant,
        48 => EntitySchemaKind::ForgeEnginePlayerData,
        49 => EntitySchemaKind::ManagedRadialMenu,
        _ => EntitySchemaKind::Unknown,
    }
}
