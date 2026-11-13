//! Standard weapon behavior, not an assertion about a particular pickup's ammo.
use super::{KnownWeapon, Weapon};

/// How a weapon accounts for its remaining ammunition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmmoKind {
    /// Discrete rounds, with the standard magazine size and per-shot consumption.
    /// Consumption is absent for burst/continuous fire whose event cadence is ambiguous.
    Magazine {
        capacity: u32,
        rounds_per_shot: Option<u32>,
    },
    /// Remaining battery/energy fraction; never interpreted as magazine rounds.
    Charge,
    /// No ammunition requirement (empty hands).
    None,
}

/// Standard interpretation shared by consumers of Theater weapon state.
/// Recorded values and game-mode respawn configuration always take precedence.
/// A capacity is not a pickup default: a dropped weapon may be partly depleted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WeaponDefaults {
    pub ammo: AmmoKind,
    /// Whether the weapon has a heat mechanic. No unobserved cooling is simulated.
    pub tracks_heat: bool,
}

impl WeaponDefaults {
    /// Known standard behavior. Unknown or unverified variants have no defaults.
    pub fn for_weapon(weapon: Weapon) -> Option<Self> {
        use KnownWeapon::*;
        let Weapon::Known(weapon) = weapon else {
            return None;
        };
        let rounds = |capacity, rounds_per_shot| Self {
            ammo: AmmoKind::Magazine {
                capacity,
                rounds_per_shot,
            },
            tracks_heat: false,
        };
        Some(match weapon {
            BanditEvo | M392Bandit => rounds(15, Some(1)),
            M41Spnkr => rounds(2, Some(1)),
            S7Sniper => rounds(4, Some(1)),
            Br75 => rounds(36, None),
            ShockRifle => rounds(12, None),
            ShockRifleRanked => rounds(12, None),
            StalkerRifle | StalkerRifleRanked | PlasmaPistol | PulseCarbine | Ravager => Self {
                ammo: AmmoKind::Charge,
                tracks_heat: true,
            },
            SentinelBeam | EnergySword | GravityHammer => Self {
                ammo: AmmoKind::Charge,
                tracks_heat: false,
            },
            Unarmed => Self {
                ammo: AmmoKind::None,
                tracks_heat: false,
            },
            _ => return None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn distinct_ammo_kinds_and_unverified_variants() {
        assert_eq!(
            WeaponDefaults::for_weapon(Weapon::Known(KnownWeapon::S7Sniper))
                .unwrap()
                .ammo,
            AmmoKind::Magazine {
                capacity: 4,
                rounds_per_shot: Some(1)
            }
        );
        let stalker =
            WeaponDefaults::for_weapon(Weapon::Known(KnownWeapon::StalkerRifleRanked)).unwrap();
        assert_eq!(stalker.ammo, AmmoKind::Charge);
        assert!(stalker.tracks_heat);
        assert_eq!(
            WeaponDefaults::for_weapon(Weapon::Unknown(super::super::WeaponId(1))),
            None
        );
    }
}
