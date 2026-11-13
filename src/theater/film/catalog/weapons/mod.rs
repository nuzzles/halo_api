//! Known Halo Infinite weapon definitions and metadata.

use crate::theater::film::ids::WeaponId;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

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

/// A classified film weapon. The unknown variant retains the exact source ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Weapon {
    Known(KnownWeapon),
    Unknown(WeaponId),
}

impl Weapon {
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

impl From<WeaponId> for Weapon {
    fn from(id: WeaponId) -> Self {
        known_weapons::ALL
            .iter()
            .find(|entry| entry.id == id)
            .map_or(Self::Unknown(id), |entry| Self::Known(entry.weapon))
    }
}

impl From<Weapon> for WeaponId {
    fn from(weapon: Weapon) -> Self {
        weapon.id()
    }
}

impl std::fmt::Display for Weapon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.id().fmt(f)
    }
}

// Preserve the numeric JSON shape of existing film and runtime APIs.
impl Serialize for Weapon {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.id().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Weapon {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(WeaponId::deserialize(deserializer)?.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalogue_classifies_known_and_preserves_unknown() {
        assert_eq!(known_weapons::ALL.len(), 38);
        let ranked_stalker = Weapon::from(WeaponId(0xadb7822542c9679f));
        assert_eq!(
            ranked_stalker,
            Weapon::Known(KnownWeapon::StalkerRifleRanked)
        );
        assert_eq!(ranked_stalker.name(), Some("Stalker Rifle (Ranked)"));
        let bandit = Weapon::Known(KnownWeapon::BanditEvo);
        assert_eq!(bandit.id(), WeaponId(0x6acdc44d42c9679f));
        assert_eq!(bandit.name(), Some("Bandit EVO"));
        assert_eq!(Weapon::from_name("Bandit Evo"), Some(bandit));
        assert_eq!(Weapon::from_name("Bandit EVO"), Some(bandit));
        assert_eq!(Weapon::from(bandit.id()), bandit);
        assert_eq!(
            serde_json::to_string(&bandit).unwrap(),
            "7696023174358067103"
        );
        assert_eq!(
            serde_json::from_str::<Weapon>("7696023174358067103").unwrap(),
            bandit
        );
        assert_eq!(
            WeaponId::from_be_bytes(bandit.id().to_be_bytes()),
            bandit.id()
        );
        assert_eq!(
            Weapon::Known(KnownWeapon::Unarmed).to_string(),
            "0x00007ca942c9679f"
        );
        for id in [
            0xb6dbead842c9679f,
            0xc1e1bab042c9679f,
            0x3ad55da442c9679f,
            0x123456789abcdef0,
        ] {
            let weapon = Weapon::from(WeaponId(id));
            assert_eq!(weapon, Weapon::Unknown(WeaponId(id)));
            assert_eq!(weapon.name(), None);
            assert_eq!(
                serde_json::from_str::<Weapon>(&serde_json::to_string(&weapon).unwrap()).unwrap(),
                weapon
            );
        }
        let mut ids: Vec<_> = known_weapons::ALL.iter().map(|entry| entry.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), known_weapons::ALL.len());
        for entry in known_weapons::ALL {
            assert_eq!(Weapon::from(entry.id), Weapon::Known(entry.weapon));
            assert_eq!(
                Weapon::from_name(entry.name),
                Some(Weapon::Known(entry.weapon))
            );
        }
    }
}
