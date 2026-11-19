//! Known Halo Infinite weapon definitions and metadata.

use crate::theater::film::ids::WeaponId;

mod defaults;
pub use defaults::{AmmoKind, WeaponDefaults};

pub mod known_weapons;
pub use known_weapons::KnownWeapon;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WeaponDefinition {
    pub weapon: KnownWeapon,
    pub id: WeaponId,
    pub name: &'static str,
}

/// A classified 64-bit weapon identity. The unknown variant retains the exact source ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WeaponIdentity {
    Known(KnownWeapon),
    Unknown(WeaponId),
}

impl WeaponIdentity {
    pub const fn id(self) -> WeaponId {
        match self {
            Self::Known(known) => known.definition().id,
            Self::Unknown(id) => id,
        }
    }

    pub const fn name(self) -> Option<&'static str> {
        match self {
            Self::Known(known) => Some(known.definition().name),
            Self::Unknown(_) => None,
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        // Accept names in older exports, but always emit the configured spelling.
        let name = match name {
            "Bandit Evo" => "Bandit EVO",
            name => name,
        };
        known_weapons::ALL
            .iter()
            .find(|entry| entry.name == name)
            .map(|entry| Self::Known(entry.weapon))
    }
}

impl From<WeaponId> for WeaponIdentity {
    fn from(id: WeaponId) -> Self {
        known_weapons::ALL
            .iter()
            .find(|entry| entry.id == id)
            .map_or(Self::Unknown(id), |entry| Self::Known(entry.weapon))
    }
}

impl From<WeaponIdentity> for WeaponId {
    fn from(weapon: WeaponIdentity) -> Self {
        weapon.id()
    }
}

impl std::fmt::Display for WeaponIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.id().fmt(f)
    }
}
