//! Reference object-tag catalogue. Selection is explicit; metadata is never
//! used to admit canonical wire parsing or equate tags with 64-bit weapon IDs.
//! Weapons, vehicles, equipment and the other objects a film names by tag.

mod data;
use super::Catalog;
use crate::theater::film::ids::{TagId, WeaponVariantId};
pub use data::{Equipment, Vehicle, Weapon as TagWeapon};
use data::{TAGS, VARIANTS};

/// What kind of object a tag is, from the group the game files it under.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum ObjectKind {
    /// A weapon, including vehicles' guns and turrets.
    Weapon,
    /// A vehicle or a standalone turret.
    Vehicle,
    /// Equipment, power-ups and grenades as pickups.
    Equipment,
    /// A grenade as the inventory holds it.
    Grenade,
    /// A projectile: a round in flight, a thrown grenade.
    Projectile,
    /// A placed object: props, Forge blocks, zones, spawn points.
    Crate,
    /// A device with moving parts: doors, lifts, terminals.
    Machine,
    /// Static scenery and volumes.
    Scenery,
    /// A control: a switch or a panel.
    Control,
}

/// A grenade. The film's grenade type is one more than its index in the
/// game's grenade list: 1 frag, 2 plasma, 3 dynamo, 4 spike.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[non_exhaustive]
pub enum Grenade {
    /// M9 Frag Grenade.
    Frag,
    /// Plasma Grenade.
    Plasma,
    /// Dynamo Grenade.
    Dynamo,
    /// Spike Grenade.
    Spike,
}

impl Grenade {
    /// The grenade with the film's one-based grenade type.
    pub fn from_type(t: u32) -> Option<Grenade> {
        Some(match t {
            1 => Grenade::Frag,
            2 => Grenade::Plasma,
            3 => Grenade::Dynamo,
            4 => Grenade::Spike,
            _ => return None,
        })
    }

    /// The film's one-based grenade type.
    pub fn type_index(self) -> u32 {
        self as u32 + 1
    }

    /// The game's English name.
    pub fn name(self) -> &'static str {
        match self {
            Grenade::Frag => "M9 Frag Grenade",
            Grenade::Plasma => "Plasma Grenade",
            Grenade::Dynamo => "Dynamo Grenade",
            Grenade::Spike => "Spike Grenade",
        }
    }
}

/// Something a player can hold, use or ride.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum Item {
    /// A weapon.
    Weapon(TagWeapon),
    /// A vehicle.
    Vehicle(Vehicle),
    /// A grenade.
    Grenade(Grenade),
    /// Equipment or a power-up.
    Equipment(Equipment),
}

impl Item {
    /// The game's English name.
    pub fn name(self) -> &'static str {
        match self {
            Item::Weapon(w) => w.name(),
            Item::Vehicle(v) => v.name(),
            Item::Grenade(g) => g.name(),
            Item::Equipment(e) => e.name(),
        }
    }
}

/// What the game's files say about one tag.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct TagInfo {
    /// The tag.
    pub tag: TagId,
    /// Its kind.
    pub kind: ObjectKind,
    /// Its English name, or its nearest ancestor's when it has none of its
    /// own.
    pub name: Option<&'static str>,
    /// The item it is, for weapons, vehicles, grenades and equipment.
    pub item: Option<Item>,
    /// The tag it derives from (a weapon variant's base weapon).
    pub parent: Option<TagId>,
    /// For a vehicle part (a turret, a gunner seat), the vehicle whose model
    /// attaches it.
    pub part_of: Option<TagId>,
    /// Whether the mode scripts count the weapon in the power weapon tier.
    pub power_weapon: bool,
}

const fn row(
    tag: u32,
    kind: ObjectKind,
    name: &'static str,
    item: Option<Item>,
    parent: u32,
    part_of: u32,
    power_weapon: bool,
) -> TagInfo {
    TagInfo {
        tag: TagId(tag),
        kind,
        name: if name.is_empty() { None } else { Some(name) },
        item,
        parent: if parent == 0 {
            None
        } else {
            Some(TagId(parent))
        },
        part_of: if part_of == 0 {
            None
        } else {
            Some(TagId(part_of))
        },
        power_weapon,
    }
}

impl Catalog {
    /// All entries in tag order, including unnamed and unclassified tags.
    pub fn tags(self) -> &'static [TagInfo] {
        TAGS
    }

    pub fn tag_info(self, tag: TagId) -> Option<&'static TagInfo> {
        let tags = self.tags();
        tags.binary_search_by_key(&tag, |t| t.tag)
            .ok()
            .map(|i| &tags[i])
    }

    pub fn item(self, tag: TagId) -> Option<Item> {
        self.tag_info(tag)?.item
    }

    /// Every variant string hash/name from the selected reference revision.
    pub fn weapon_variants(self) -> impl ExactSizeIterator<Item = (WeaponVariantId, &'static str)> {
        let variants = VARIANTS;
        variants
            .iter()
            .map(|&(id, name)| (WeaponVariantId(id), name))
    }

    pub fn weapon_variant_name(self, id: WeaponVariantId) -> Option<&'static str> {
        let variants = VARIANTS;
        variants
            .binary_search_by_key(&id.0, |v| v.0)
            .ok()
            .map(|i| variants[i].1)
    }
}

impl TagWeapon {
    pub fn from_tag(catalog: Catalog, tag: TagId) -> Option<Self> {
        match catalog.item(tag)? {
            Item::Weapon(w) => Some(w),
            _ => None,
        }
    }
}
impl Vehicle {
    pub fn from_tag(catalog: Catalog, tag: TagId) -> Option<Self> {
        match catalog.item(tag)? {
            Item::Vehicle(v) => Some(v),
            _ => None,
        }
    }
}
impl Equipment {
    pub fn from_tag(catalog: Catalog, tag: TagId) -> Option<Self> {
        match catalog.item(tag)? {
            Item::Equipment(e) => Some(e),
            _ => None,
        }
    }
}
impl Grenade {
    pub fn from_tag(catalog: Catalog, tag: TagId) -> Option<Self> {
        match catalog.item(tag)? {
            Item::Grenade(g) => Some(g),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests;
