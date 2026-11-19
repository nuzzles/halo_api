//! Catalogue data captured alongside a recorded object tag in a finished film.
//! These are reference values, not observed ammunition, damage, or positions.

use crate::theater::film::TagId;
use crate::theater::film::catalog::items::Grenade;

/// Reference metadata for the exact tag recorded in a replication field.
/// Each known tag belongs to one of these catalogue tables.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum TagReference {
    /// A firing tag, with a link to its projectile when one is recorded in the catalogue.
    Firing(FiringReference),
    /// A projectile tag, with an optional link to its damage effect.
    Projectile(ProjectileReference),
    /// A damage-effect tag.
    Damage(DamageReference),
    /// Render-model extents in engine units: x min/max, y min/max, z min/max.
    VehicleBounds([f64; 6]),
}

/// A catalogue link to another tag. An unknown target retains its exact ID.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum TagLink<T> {
    /// The linked tag has reference details in the bundled catalogue.
    Known(T),
    /// The link exists, but the bundled catalogue has no details for its target.
    Unknown(TagId),
}

/// The first barrel and magazine in the game's reference data.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FiringReference {
    pub rounds_per_second: f64,
    pub burst: i32,
    pub recovery: f64,
    /// Loaded, carried at start, and carried at most; not observed ammunition.
    pub magazine: Option<[i32; 3]>,
    /// Projectile link, if this firing entry names one.
    pub projectile: Option<TagLink<ProjectileReference>>,
}

/// Flight and direct-hit effect of a catalogued projectile.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ProjectileReference {
    pub tag: TagId,
    pub velocity: f64,
    pub range: f64,
    /// Direct-hit damage-effect link, if this projectile entry names one.
    pub impact: Option<TagLink<DamageReference>>,
}

/// One catalogued damage effect, not damage observed in this film.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DamageReference {
    pub tag: TagId,
    /// Cause and any single associated tag recorded by the catalogue.
    pub cause: DamageCause,
    /// Reference damage numbers, when this effect defines them.
    pub amount: Option<DamageAmount>,
    /// This effect is also marked as a grenade blast; it can have a weapon cause.
    pub grenade_blast: bool,
    /// Projectile tags that can produce this effect, independently of `cause`.
    pub projectiles: Vec<TagId>,
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DamageAmount {
    pub lower: f64,
    pub upper: [f64; 2],
    pub shield_scale: f64,
    pub body_scale: f64,
    pub flags: u32,
    pub radius: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DamageCause {
    /// A weapon effect; some effects have no single weapon tag on record.
    Weapon { weapon: Option<TagId> },
    /// A melee effect, optionally associated with a weapon tag.
    Melee { weapon: Option<TagId> },
    /// A known grenade type and the exact grenade or grenade-projectile tag
    /// linked by this effect.
    Grenade { kind: Grenade, source: TagId },
    /// An equipment effect without a single associated item tag.
    Equipment,
    /// A vehicle effect without a single associated vehicle tag.
    Vehicle,
    /// A physics collision effect.
    Collision,
    /// An effect from game globals such as a kill volume or falling.
    World,
    /// A model explosion effect.
    Explosion,
    /// The catalogue does not identify a cause.
    Unknown,
}
