use serde::{Deserialize, Serialize};

/// Replicated entity handle, split into its index and generation.
/// The film stores these as one 32-bit word: 30 index bits and 2 generation bits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct EntityId {
    index: u32,
    generation: u8,
}

impl EntityId {
    pub const fn from_raw(raw: u32) -> Self {
        Self {
            index: raw & 0x3fff_ffff,
            generation: (raw >> 30) as u8,
        }
    }

    /// Construct an entity handle from its two recorded parts.
    pub const fn from_parts(index: u32, generation: u8) -> Option<Self> {
        if index <= 0x3fff_ffff && generation <= 3 {
            Some(Self { index, generation })
        } else {
            None
        }
    }

    pub const fn raw(self) -> u32 {
        ((self.generation as u32) << 30) | self.index
    }
    pub const fn index(self) -> u32 {
        self.index
    }
    pub const fn generation(self) -> u8 {
        self.generation
    }
}

impl From<u32> for EntityId {
    fn from(raw: u32) -> Self {
        Self::from_raw(raw)
    }
}

impl<'de> Deserialize<'de> for EntityId {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Parts {
            index: u32,
            generation: u8,
        }
        let parts = Parts::deserialize(deserializer)?;
        Self::from_parts(parts.index, parts.generation)
            .ok_or_else(|| serde::de::Error::custom("entity index or generation out of range"))
    }
}

impl std::fmt::Display for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}v{}", self.index, self.generation)
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

/// An index into this film's entity schemas, not a globally named game asset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EntitySchemaId(pub u32);

impl EntitySchemaId {
    pub const fn raw(self) -> u32 {
        self.0
    }
}

/// Component position within one recorded entity schema.
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

/// Five-bit player slot recorded in an event body, not a roster position or XUID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EventPlayerSlot(pub u8);

impl EventPlayerSlot {
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

impl From<u32> for EntitySchemaId {
    fn from(raw: u32) -> Self {
        Self(raw)
    }
}

impl std::fmt::Display for EntitySchemaId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
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

/// A 32-bit weapon asset tag, distinct from the 64-bit family/variant `WeaponId`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WeaponTagId(pub u32);

impl WeaponTagId {
    pub const fn raw(self) -> u32 {
        self.0
    }
}

impl From<WeaponTagId> for TagId {
    fn from(value: WeaponTagId) -> Self {
        Self(value.raw())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weapon_tags_serialize_as_u32_and_convert_to_catalog_tags() {
        for raw in [0, 0x0000_8594, u32::MAX] {
            let id = WeaponTagId(raw);
            assert_eq!(id.raw(), raw);
            assert_eq!(TagId::from(id), TagId(raw));
            let json = serde_json::to_string(&id).unwrap();
            assert_eq!(json, raw.to_string());
            assert_eq!(serde_json::from_str::<WeaponTagId>(&json).unwrap(), id);
        }
        assert!(serde_json::from_str::<WeaponTagId>("4294967296").is_err());
        assert!(serde_json::from_str::<WeaponTagId>("-1").is_err());
    }

    #[test]
    fn event_player_slots_serialize_as_u8_and_preserve_optional_absence() {
        for raw in [0, 7, 31, u8::MAX] {
            let id = EventPlayerSlot(raw);
            assert_eq!(id.raw(), raw);
            let json = serde_json::to_string(&id).unwrap();
            assert_eq!(json, raw.to_string());
            assert_eq!(serde_json::from_str::<EventPlayerSlot>(&json).unwrap(), id);
        }
        assert!(serde_json::from_str::<EventPlayerSlot>("256").is_err());
        assert!(serde_json::from_str::<EventPlayerSlot>("-1").is_err());
        assert_eq!(
            serde_json::from_str::<Option<EventPlayerSlot>>("null").unwrap(),
            None
        );
        assert_eq!(
            serde_json::to_string(&Some(EventPlayerSlot(0))).unwrap(),
            "0"
        );
    }
}
