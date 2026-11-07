//! Inventory identities and desired selection, interpreted from complete records.
use super::*;

/// Recorded asset identity: high 32-bit family followed by low 32-bit variant.
/// This is not an entity instance or a name-table index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct WeaponSample {
    pub timestamp_us: u64,
    /// Desired principal asset; draw-animation completion is unavailable.
    pub active_weapon_id: Option<Weapon>,
    /// Sole other carried asset when all four slots are known.
    pub secondary_weapon_id: Option<Weapon>,
    /// Some(true) means all other slots are explicitly empty; None means unknown.
    pub secondary_weapon_empty: Option<bool>,
    /// Magazine readings transmitted at this timestamp, not carried estimates.
    pub active_weapon_ammo: Option<u32>,
    pub secondary_weapon_ammo: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WeaponObservation {
    pub timestamp_us: u64,
    pub entity_id: u32,
    pub source: PoseSource,
    pub change: WeaponChange,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub enum WeaponChange {
    Slot {
        slot: usize,
        asset_id: Option<Weapon>,
    },
    Ammo {
        slot: usize,
        magazine: u32,
    },
    Selection {
        principal: Option<usize>,
        offhand: Option<usize>,
    },
    UnknownSelection,
}

pub(super) fn slot_indices(registry: &FilmRegistry) -> Vec<usize> {
    registry
        .archetypes
        .get(35)
        .map(|arch| {
            arch.components
                .iter()
                .enumerate()
                .filter_map(|(i, c)| (c.name() == Some("weapon-state-type-info")).then_some(i))
                .collect()
        })
        .unwrap_or_default()
}

pub(super) fn ammo_indices(registry: &FilmRegistry) -> Vec<usize> {
    registry
        .archetypes
        .get(35)
        .map(|arch| {
            arch.components
                .iter()
                .enumerate()
                .filter_map(|(i, c)| (c.name() == Some("weapon-state-ammo")).then_some(i))
                .collect()
        })
        .unwrap_or_default()
}

fn selected(fields: &[ComponentField], name: &str) -> Option<Option<usize>> {
    match value(fields, &format!("{name}.gate"), 1)? {
        1 => Some(None),
        0 => Some(Some(value(fields, name, 2)? as usize)),
        _ => None,
    }
}

pub(super) fn observe(
    record: &EntityRecord,
    indices: &[usize],
    ammo_indices: &[usize],
    time: u64,
    source: PoseSource,
) -> Vec<WeaponObservation> {
    if indices.len() != 4 || record.stop != EntityViewStop::Complete {
        return Vec::new();
    }
    let Some(entity_id) = record.header.id else {
        return Vec::new();
    };
    let mut observations: Vec<_> = record
        .components
        .iter()
        .filter_map(|c| {
            if c.status != ComponentReadStatus::Complete {
                return None;
            }
            let change = if c.name == "weapon-state-type-info" {
                let slot = indices.iter().position(|i| *i == c.index)?;
                // Nested frame-configuration fields reuse "present"; the
                // identity gate is the first field in this component grammar.
                let asset_id = c.weapon_id;
                WeaponChange::Slot { slot, asset_id }
            } else if c.name == "weapon-state-ammo" && ammo_indices.len() == 4 {
                let slot = ammo_indices.iter().position(|i| *i == c.index)?;
                // A closed gate means no magazine update, not zero.
                if value(&c.fields, "magazine.gate", 1)? != 0 {
                    return None;
                }
                WeaponChange::Ammo {
                    slot,
                    magazine: value(&c.fields, "magazine", 8)? as u32,
                }
            } else if c.name == "biped-desired-weapon-set" {
                // The first R3 ("selection") is a request counter, NOT a slot.
                match (selected(&c.fields, "slot_a"), selected(&c.fields, "slot_b")) {
                    (Some(principal), Some(offhand))
                        if principal.is_none() || principal != offhand =>
                    {
                        WeaponChange::Selection { principal, offhand }
                    }
                    _ => WeaponChange::UnknownSelection,
                }
            } else {
                return None;
            };
            Some(WeaponObservation {
                timestamp_us: time,
                entity_id,
                source: PoseSource {
                    start_bit: c.start_bit,
                    end_bit: c.end_bit,
                    ..source
                },
                change,
            })
        })
        .collect();
    // Ammo components precede identities in the biped schema. Apply identity
    // changes first so a pickup resets the old slot ammo before its reading.
    observations.sort_by_key(|o: &WeaponObservation| match o.change {
        WeaponChange::Slot { .. } => 0,
        WeaponChange::Ammo { .. } => 1,
        _ => 2,
    });
    observations
}

#[derive(Default)]
struct Inventory {
    // Outer None is unknown; Some(None) is an explicitly empty slot.
    slots: [Option<Option<Weapon>>; 4],
    magazines: [Option<u32>; 4],
    selection: Option<(Option<usize>, Option<usize>)>,
}
impl Inventory {
    fn apply(&mut self, change: WeaponChange) {
        match change {
            WeaponChange::Slot { slot, asset_id } => {
                if self.slots[slot] != Some(asset_id) {
                    self.magazines[slot] = None;
                }
                self.slots[slot] = Some(asset_id);
            }
            WeaponChange::Ammo { slot, magazine } => self.magazines[slot] = Some(magazine),
            WeaponChange::Selection { principal, offhand } => {
                self.selection = Some((principal, offhand))
            }
            WeaponChange::UnknownSelection => self.selection = None,
        }
    }
    fn sample(&self, timestamp_us: u64, touched_ammo: &[bool; 4]) -> WeaponSample {
        let mut sample = WeaponSample {
            timestamp_us,
            active_weapon_id: None,
            secondary_weapon_id: None,
            secondary_weapon_empty: None,
            active_weapon_ammo: None,
            secondary_weapon_ammo: None,
        };
        // Off-hand selection is dual wield, not the alternate carried weapon.
        if let Some((Some(slot), None)) = self.selection {
            sample.active_weapon_id = self.slots[slot].flatten();
            if sample.active_weapon_id.is_some() && touched_ammo[slot] {
                sample.active_weapon_ammo = self.magazines[slot];
            }
            if sample.active_weapon_id.is_some() && self.slots.iter().all(Option::is_some) {
                let mut others = self
                    .slots
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| *i != slot)
                    .filter_map(|(i, id)| id.flatten().map(|id| (i, id)));
                let other = others.next();
                if others.next().is_none() {
                    sample.secondary_weapon_id = other.map(|(_, id)| id);
                    sample.secondary_weapon_empty = Some(other.is_none());
                    if let Some((other_slot, _)) = other
                        && touched_ammo[other_slot]
                    {
                        sample.secondary_weapon_ammo = self.magazines[other_slot];
                    }
                }
            }
        }
        sample
    }
}

pub(super) fn timeline(
    observations: &mut Vec<WeaponObservation>,
    gaps: Vec<(u64, usize)>,
    lives: &[PlayerLife],
) -> Vec<WeaponSample> {
    let within_life = |o: &WeaponObservation| {
        lives.iter().any(|l| {
            l.entity_id == o.entity_id && l.start_us <= o.timestamp_us && o.timestamp_us < l.end_us
        })
    };
    let mut events = BTreeMap::<u64, Vec<(usize, Option<WeaponChange>)>>::new();
    // Preserve packet order even for coincident clocks. A later gap must win
    // over an earlier observation at the same timestamp. Life boundaries first.
    for l in lives {
        for time in [l.start_us, l.end_us] {
            events.entry(time).or_default().push((0, None));
        }
    }
    for (time, preceding_observations) in gaps {
        events
            .entry(time)
            .or_default()
            .push((2 * preceding_observations + 1, None));
    }
    for (index, o) in observations
        .iter()
        .enumerate()
        .filter(|(_, o)| within_life(o))
    {
        events
            .entry(o.timestamp_us)
            .or_default()
            .push((2 * index + 2, Some(o.change)));
    }
    observations.retain(within_life);
    let mut inventory = Inventory::default();
    events
        .into_iter()
        .map(|(time, mut changes)| {
            changes.sort_by_key(|(order, _)| *order);
            let mut touched_ammo = [false; 4];
            for (_, change) in changes {
                if let Some(change) = change {
                    if let WeaponChange::Ammo { slot, .. } = change {
                        touched_ammo[slot] = true;
                    }
                    inventory.apply(change);
                } else {
                    inventory = Inventory::default();
                    touched_ammo = [false; 4];
                }
            }
            inventory.sample(time, &touched_ammo)
        })
        .collect()
}

#[cfg(test)]
mod tests;
