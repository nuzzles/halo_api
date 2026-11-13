//! Explicit reference metadata; not recorded damage, ammunition or vehicle state.
//! How weapons fire, what projectiles do, and how big vehicles are.

mod data;
use super::Catalog;
use crate::theater::film::TagId;
use data::{FIRE, PROJECTILES, VEHICLE_BOUNDS};

/// A weapon's first barrel and magazine.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct FireStats {
    /// The weapon's tag.
    pub tag: TagId,
    /// Rounds a second at the barrel's top rate.
    pub rounds_per_second: f64,
    /// Shots per trigger pull; 0 for an automatic.
    pub burst: i32,
    /// Seconds between bursts.
    pub recovery: f64,
    /// Rounds loaded, carried at the start and carried at most; None for a
    /// weapon without a magazine.
    pub magazine: Option<[i32; 3]>,
    /// The projectile it fires.
    pub projectile: Option<TagId>,
}

/// A projectile's flight and its impact damage effect.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct ProjectileStats {
    /// The projectile's tag.
    pub tag: TagId,
    /// Initial velocity, engine units a second.
    pub velocity: f64,
    /// Maximum range, engine units.
    pub range: f64,
    /// The damage effect of a direct hit.
    pub impact: Option<TagId>,
}

pub(super) type FireRow = FireStats;
pub(super) type ProjectileRow = ProjectileStats;

impl Catalog {
    pub fn firing_catalogue(self) -> &'static [FireStats] {
        FIRE
    }

    pub fn fire_stats(self, tag: TagId) -> Option<&'static FireStats> {
        let rows = self.firing_catalogue();
        rows.binary_search_by_key(&tag, |r| r.tag)
            .ok()
            .map(|i| &rows[i])
    }

    pub fn projectile_catalogue(self) -> &'static [ProjectileStats] {
        PROJECTILES
    }

    pub fn projectile_stats(self, tag: TagId) -> Option<&'static ProjectileStats> {
        let rows = self.projectile_catalogue();
        rows.binary_search_by_key(&tag, |r| r.tag)
            .ok()
            .map(|i| &rows[i])
    }

    /// Reference render-model extents, not a collision shape or observed position.
    /// Each array is x min/max, y min/max, z min/max in engine units.
    pub fn vehicle_models(self) -> &'static [(TagId, [f64; 6])] {
        VEHICLE_BOUNDS
    }

    pub fn vehicle_bounds(self, tag: TagId) -> Option<[f64; 6]> {
        let rows = self.vehicle_models();
        rows.binary_search_by_key(&tag, |r| r.0)
            .ok()
            .map(|i| rows[i].1)
    }
}

#[cfg(test)]
mod tests;
