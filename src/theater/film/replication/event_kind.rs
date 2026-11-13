use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Event identity for established v41 codes. Other codes retain their exact value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReplicationEventKind {
    Damage,
    ProjectileDetonation,
    Pickup,
    Zoom,
    VehicleExit,
    Teleport,
    WeaponFire,
    WeaponReload,
    Throw,
    Melee,
    AiDialogue,
    GameMessage,
    Kill,
    TranslocatorTeleportEffects,
    Unknown(u8),
}

impl ReplicationEventKind {
    pub const fn code(self) -> u8 {
        match self {
            Self::Damage => 0,
            Self::ProjectileDetonation => 5,
            Self::Pickup => 9,
            Self::Zoom => 21,
            Self::VehicleExit => 22,
            Self::Teleport => 32,
            Self::WeaponFire => 36,
            Self::WeaponReload => 38,
            Self::Throw => 39,
            Self::Melee => 40,
            Self::AiDialogue => 75,
            Self::GameMessage => 82,
            Self::Kill => 85,
            Self::TranslocatorTeleportEffects => 117,
            Self::Unknown(code) => code,
        }
    }
}

impl From<u8> for ReplicationEventKind {
    fn from(code: u8) -> Self {
        match code {
            0 => Self::Damage,
            5 => Self::ProjectileDetonation,
            9 => Self::Pickup,
            21 => Self::Zoom,
            22 => Self::VehicleExit,
            32 => Self::Teleport,
            36 => Self::WeaponFire,
            38 => Self::WeaponReload,
            39 => Self::Throw,
            40 => Self::Melee,
            75 => Self::AiDialogue,
            82 => Self::GameMessage,
            85 => Self::Kill,
            117 => Self::TranslocatorTeleportEffects,
            _ => Self::Unknown(code),
        }
    }
}

impl Serialize for ReplicationEventKind {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.code().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ReplicationEventKind {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(u8::deserialize(deserializer)?.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_and_unknown_codes_round_trip_as_numbers() {
        for code in [0, 5, 7, 9, 21, 22, 32, 36, 38, 39, 40, 75, 82, 85, 117, 255] {
            let kind = ReplicationEventKind::from(code);
            assert_eq!(kind.code(), code);
            assert_eq!(
                serde_json::from_str::<ReplicationEventKind>(&code.to_string()).unwrap(),
                kind
            );
            assert_eq!(serde_json::to_string(&kind).unwrap(), code.to_string());
        }
        assert_eq!(ReplicationEventKind::from(85), ReplicationEventKind::Kill);
        assert_eq!(
            ReplicationEventKind::from(7),
            ReplicationEventKind::Unknown(7)
        );
    }
}
