use serde::{Deserialize, Serialize};

/// Exact 32-bit identity of one replicated entity instance. The upper bits
/// distinguish generations; it is not an asset or catalog lookup key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EntityId(pub u32);

impl EntityId {
    pub const fn raw(self) -> u32 {
        self.0
    }
    pub const fn slot(self) -> u32 {
        self.0 & 0x3fff_ffff
    }
    pub const fn generation(self) -> u32 {
        self.0 >> 30
    }
}

impl From<u32> for EntityId {
    fn from(raw: u32) -> Self {
        Self(raw)
    }
}

impl std::fmt::Display for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Exact ID of a keyframe baseline. Its upper bits select a keyframe
/// namespace, not a live entity generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct KeyframeId(pub u32);

impl KeyframeId {
    pub const fn raw(self) -> u32 {
        self.0
    }
    pub const fn slot(self) -> u32 {
        self.0 & 0x3fff_ffff
    }
}

/// An index into this film's registry archetypes, not a globally named game asset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ArchetypeId(pub u32);

impl ArchetypeId {
    pub const fn raw(self) -> u32 {
        self.0
    }

    /// Resolve this film-local index; unknown indices remain representable.
    pub fn resolve(
        self,
        registry: &crate::theater::film::Registry,
    ) -> Option<&crate::theater::film::RegistryArchetype> {
        registry
            .components
            .iter()
            .find(|archetype| archetype.index == self)
    }
}

/// Component position within one archetype's recorded schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ComponentSlot(pub usize);

impl ComponentSlot {
    pub const fn index(self) -> usize {
        self.0
    }
}

/// Player index recorded by a control payload. It is not a roster position or XUID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ControlPlayerIndex(pub u8);

impl ControlPlayerIndex {
    pub const fn raw(self) -> u8 {
        self.0
    }
}

/// Exact Xbox user identity recorded in a summary highlight.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Xuid(pub u64);

impl Xuid {
    pub const fn raw(self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for Xuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<u32> for ArchetypeId {
    fn from(raw: u32) -> Self {
        Self(raw)
    }
}

impl std::fmt::Display for ArchetypeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod replication_id_tests {
    use super::*;
    use crate::theater::film::{Registry, RegistryArchetype};

    #[test]
    fn entity_identity_keeps_generation_and_numeric_wire_form() {
        let id = EntityId(0x8000_0009);
        assert_eq!(id.slot(), 9);
        assert_eq!(id.generation(), 2);
        assert_ne!(id, EntityId(9));
        assert_eq!(serde_json::to_string(&id).unwrap(), "2147483657");
        assert_eq!(serde_json::from_str::<EntityId>("2147483657").unwrap(), id);
        let keyframe = KeyframeId(id.raw());
        assert_eq!(keyframe.slot(), id.slot());
        assert_eq!(serde_json::to_string(&keyframe).unwrap(), "2147483657");
    }

    #[test]
    fn archetype_resolves_only_against_its_film_registry() {
        let registry = Registry {
            version: crate::theater::film::RegistryVersion {
                film: 41,
                format: 27,
            },
            components: vec![RegistryArchetype {
                index: ArchetypeId(3),
                components: vec![],
            }],
            options: crate::theater::film::RegistryOptions {
                game_mode: crate::theater::film::RegistryGameMode::Multiplayer,
                recorder_simulation: 0,
                level_tag: crate::theater::film::TagId(0),
            },
            map: crate::theater::film::RegistryMap {
                asset_id: crate::theater::film::MapAssetId([0; 16]),
                version_id: crate::theater::film::MapVersionId([0; 16]),
            },
        };
        assert_eq!(
            ArchetypeId(3).resolve(&registry).map(|a| a.index),
            Some(ArchetypeId(3))
        );
        assert!(ArchetypeId(4).resolve(&registry).is_none());
        assert_eq!(ArchetypeId(4).raw(), 4);
    }
}

/// Exact recorded 128-bit map asset identity. Byte order is retained as read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MapAssetId(pub [u8; 16]);

impl MapAssetId {
    pub const fn bytes(self) -> [u8; 16] {
        self.0
    }
}

/// Exact recorded 128-bit map version identity, distinct from the map asset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MapVersionId(pub [u8; 16]);

impl MapVersionId {
    pub const fn bytes(self) -> [u8; 16] {
        self.0
    }
}

/// A 32-bit game asset tag. This is a different namespace from `weapon::WeaponId`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TagId(pub u32);

impl TagId {
    pub const fn raw(self) -> u32 {
        self.0
    }
}

impl From<u32> for TagId {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for TagId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:08x}", self.0)
    }
}

/// A 32-bit weapon variant string hash, distinct from an asset tag or weapon ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WeaponVariantId(pub u32);

impl WeaponVariantId {
    pub const fn raw(self) -> u32 {
        self.0
    }
}

/// Recorded high 32-bit weapon family followed by low 32-bit variant.
/// This identity is retained even when the reference catalogue has no name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WeaponId(pub u64);

impl WeaponId {
    pub const fn from_parts(family: u32, variant: u32) -> Self {
        Self((family as u64) << 32 | variant as u64)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }

    pub const fn from_be_bytes(bytes: [u8; 8]) -> Self {
        Self(u64::from_be_bytes(bytes))
    }

    pub const fn to_be_bytes(self) -> [u8; 8] {
        self.0.to_be_bytes()
    }
}

impl std::fmt::Display for WeaponId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:016x}", self.0)
    }
}
