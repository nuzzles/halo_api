//! Explicit reference metadata; not recorded damage, ammunition or vehicle state.
//! Damage effects: what each one belongs to and how much it deals.

mod data;
use super::Catalog;
use crate::theater::film::TagId;
use data::{DAMAGE, SPARTAN};

/// What dealt a hit, from where its damage effect sits in the game's tags.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum DamageCause {
    /// A weapon's projectile, or a projectile with no weapon on record.
    Weapon,
    /// A melee strike, a sword or hammer swing included.
    Melee,
    /// A grenade.
    Grenade,
    /// Equipment.
    Equipment,
    /// A vehicle.
    Vehicle,
    /// A physics model: something ran into something.
    Collision,
    /// The game's globals: falling, kill volumes and the like.
    World,
    /// A model blowing up.
    Explosion,
    /// Not in the catalogue.
    Unknown,
}

/// A damage effect's numbers, from its tag and from what the multiplayer
/// Spartan's shield and body armor multiply it by.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct DamageAmount {
    /// The lower bound.
    pub lower: f64,
    /// The upper bound's minimum and maximum.
    pub upper: [f64; 2],
    /// The Spartan's shield multiplier.
    pub shield_scale: f64,
    /// The Spartan's body multiplier.
    pub body_scale: f64,
    /// The effect's `DamageDefinitionFlags`.
    pub flags: u32,
    /// The area's outer radius in engine units; 0 for a direct hit.
    pub radius: f64,
}

impl DamageAmount {
    /// The flag set on an effect that can headshot.
    pub const CAN_HEADSHOT: u32 = 1 << 1;
    /// The flag set on an effect that goes straight to the body.
    pub const SKIPS_SHIELDS: u32 = 1 << 9;
    /// The flag set on an effect whose damage beyond the shield is lost.
    pub const NO_SPILL: u32 = 1 << 23;

    /// One amount whatever the distance or the roll: a direct hit with
    /// equal bounds.
    pub fn fixed(&self) -> bool {
        self.radius == 0.0 && self.lower == self.upper[0] && self.lower == self.upper[1]
    }

    /// Whether `flag` is set.
    pub fn has(&self, flag: u32) -> bool {
        self.flags & flag != 0
    }
}

/// One catalogued damage effect.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct DamageEffect {
    /// The effect's tag.
    pub tag: TagId,
    /// What it belongs to.
    pub cause: DamageCause,
    /// The weapon (or grenade) that deals it, when exactly one does.
    pub weapon: Option<TagId>,
    /// Its numbers; None for an effect that deals no damage.
    pub amount: Option<DamageAmount>,
    /// A blast that grenades spawn among other projectiles, so that the
    /// effect alone doesn't say what set it off.
    pub grenade_blast: bool,
    projectiles: &'static [u32],
}

impl DamageEffect {
    /// The projectiles that deal it.
    pub fn projectiles(&self) -> impl Iterator<Item = TagId> + '_ {
        self.projectiles.iter().map(|&p| TagId(p))
    }
}

pub(super) type DamageRow = DamageEffect;

pub(super) const fn amount(
    lower: f64,
    upper: [f64; 2],
    shield_scale: f64,
    body_scale: f64,
    flags: u32,
    radius: f64,
) -> DamageAmount {
    DamageAmount {
        lower,
        upper,
        shield_scale,
        body_scale,
        flags,
        radius,
    }
}

pub(super) const fn row(
    tag: u32,
    cause: DamageCause,
    weapon: u32,
    amount: Option<DamageAmount>,
    grenade_blast: bool,
    projectiles: &'static [u32],
) -> DamageEffect {
    DamageEffect {
        tag: TagId(tag),
        cause,
        weapon: if weapon == 0 {
            None
        } else {
            Some(TagId(weapon))
        },
        amount,
        grenade_blast,
        projectiles,
    }
}

impl Catalog {
    pub fn damage_effects(self) -> &'static [DamageEffect] {
        DAMAGE
    }

    pub fn damage_effect(self, tag: TagId) -> Option<&'static DamageEffect> {
        let rows = self.damage_effects();
        rows.binary_search_by_key(&tag, |d| d.tag)
            .ok()
            .map(|i| &rows[i])
    }

    /// Reference multiplayer Spartan body/shield vitality, not a film observation.
    pub fn spartan_vitality(self) -> (f64, f64) {
        SPARTAN
    }
}

#[cfg(test)]
mod tests;
