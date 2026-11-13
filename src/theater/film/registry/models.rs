//! Reference registry models.

use crate::theater::film::{ArchetypeId, ComponentSlot, MapAssetId, MapVersionId, TagId};

/// Typed setup retained by a parsed film. Source bytes and partial structural
/// reads stay in the parser's `ParsedRegistry`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Registry {
    /// Recorded film and format versions. Parsing rejects an incomplete header.
    pub version: RegistryVersion,
    /// Accepted component schemas in recorded archetype and slot order.
    pub components: Vec<RegistryArchetype>,
    /// Recorded game mode, recorder simulation, and level identity.
    pub options: RegistryOptions,
    /// Recorded map asset and version identity.
    pub map: RegistryMap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RegistryVersion {
    pub film: u32,
    pub format: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RegistryArchetype {
    pub index: ArchetypeId,
    pub components: Vec<RegistryComponentDefinition>,
}

impl RegistryArchetype {
    /// Resolve a component slot only within this archetype's schema.
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

#[cfg(test)]
mod game_mode_tests {
    use super::RegistryGameMode;

    #[test]
    fn game_mode_accepts_only_multiplayer_and_keeps_numeric_json() {
        let mode = RegistryGameMode::Multiplayer;
        assert_eq!(mode.raw(), 2);
        assert_eq!(serde_json::to_string(&mode).unwrap(), "2");
        assert_eq!(serde_json::from_str::<RegistryGameMode>("2").unwrap(), mode);
        for code in [0, 1, 3, 4, 5, 6, 7] {
            assert!(RegistryGameMode::try_from(code).is_err());
            assert!(serde_json::from_str::<RegistryGameMode>(&code.to_string()).is_err());
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RegistryMap {
    pub asset_id: MapAssetId,
    pub version_id: MapVersionId,
}
