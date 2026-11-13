//! Stateful playback of a film's finished, typed observations.
//!
//! Controls and recorded events remain in the observation feed; they do not
//! imply entity state changes. Highlights are the film's guarded candidates.

use crate::theater::Film;
use crate::theater::film::ReplicationComponentChange;
pub use crate::theater::film::replication::ReplicationValue;
use crate::theater::film::{ArchetypeId, ComponentSlot, EntityId, KeyframeId};
pub use crate::theater::film::{
    ReplicationChange, ReplicationControl, ReplicationField, ReplicationObservation,
    SummaryHighlight,
};
use std::collections::BTreeMap;
use std::sync::Arc;

/// Latest complete component update for an entity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentState {
    pub name: String,
    pub fields: Vec<ReplicationField>,
    pub weapon_id: Option<crate::theater::film::catalog::weapons::Weapon>,
}

/// Materialized live entity or keyframe baseline state. `Id` keeps their
/// distinct ID domains while sharing the component snapshot shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityState<Id = EntityId> {
    /// Exact recorded ID in this state's domain.
    pub id: Id,
    pub archetype: ArchetypeId,
    pub default_fields: Vec<ReplicationField>,
    /// Fields of the latest entity-level change; component fields are separate.
    pub fields: Vec<ReplicationField>,
    pub components: BTreeMap<ComponentSlot, ComponentState>,
    pub created_at_us: Option<u64>,
    pub updated_at_us: u64,
}

/// Entity state after applying observations through the requested time.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WorldSnapshot {
    /// Live entities keyed by their exact recorded IDs.
    pub entities: BTreeMap<EntityId, EntityState>,
    /// Keyframe baselines keyed by their exact recorded IDs. No link to a live
    /// entity is assumed from the keyframe ID.
    pub keyframes: BTreeMap<KeyframeId, EntityState<KeyframeId>>,
}

/// An owned film and an independent playback cursor.
#[derive(Debug)]
pub struct TheaterRuntime {
    film: Arc<Film>,
    /// Source order is authoritative; monotonic films can advance by prefix.
    monotonic_timestamps: bool,
    cursor: usize,
    timestamp_us: Option<u64>,
    world: WorldSnapshot,
}

impl TheaterRuntime {
    /// Prepare playback from the film's finished observations.
    pub fn load(film: impl Into<Arc<Film>>) -> Self {
        let film = film.into();
        let monotonic_timestamps = film
            .replication_stream
            .observations
            .windows(2)
            .all(|pair| pair[0].timestamp_us <= pair[1].timestamp_us);
        Self {
            film,
            monotonic_timestamps,
            cursor: 0,
            timestamp_us: None,
            world: WorldSnapshot::default(),
        }
    }

    /// Finished replication observations in recorded order.
    pub fn observations(&self) -> &[ReplicationObservation] {
        &self.film.replication_stream.observations
    }

    /// Guarded summary highlights in recorded order.
    pub fn summary_highlights(&self) -> &[SummaryHighlight] {
        &self.film.summary.highlights
    }

    /// Materialized entity state without moving playback.
    pub fn current(&self) -> &WorldSnapshot {
        &self.world
    }

    /// Requested playback time, or None before playback and after rewind.
    pub fn timestamp_us(&self) -> Option<u64> {
        self.timestamp_us
    }

    /// Apply changes through the requested time in observation vector order.
    pub fn advance_to(&mut self, timestamp_us: u64) -> &WorldSnapshot {
        if !self.monotonic_timestamps
            || self
                .timestamp_us
                .is_some_and(|previous| timestamp_us < previous)
        {
            self.rewind();
        }
        let observations = &self.film.replication_stream.observations;
        if self.monotonic_timestamps {
            while self.cursor < observations.len()
                && observations[self.cursor].timestamp_us <= timestamp_us
            {
                apply(&mut self.world, &observations[self.cursor]);
                self.cursor += 1;
            }
        } else {
            for observation in observations {
                if observation.timestamp_us <= timestamp_us {
                    apply(&mut self.world, observation);
                }
            }
        }
        self.timestamp_us = Some(timestamp_us);
        &self.world
    }

    /// Reconstruct state at a timestamp, regardless of the current cursor.
    pub fn seek(&mut self, timestamp_us: u64) -> &WorldSnapshot {
        self.rewind();
        self.advance_to(timestamp_us)
    }

    /// Clear playback state while retaining the film.
    pub fn rewind(&mut self) {
        self.world.entities.clear();
        self.world.keyframes.clear();
        self.cursor = 0;
        self.timestamp_us = None;
    }
}

fn component_state(change: &ReplicationComponentChange) -> ComponentState {
    ComponentState {
        name: change.name.clone(),
        fields: change.fields.clone(),
        weapon_id: change.weapon_id,
    }
}

fn components(changes: &[ReplicationComponentChange]) -> BTreeMap<ComponentSlot, ComponentState> {
    changes
        .iter()
        .map(|change| (change.index, component_state(change)))
        .collect()
}

fn apply(world: &mut WorldSnapshot, observation: &ReplicationObservation) {
    let timestamp_us = observation.timestamp_us;
    match &observation.change {
        ReplicationChange::EntityCreated {
            id,
            archetype,
            default_fields,
            fields,
            components: changes,
        } => {
            world.entities.insert(
                *id,
                EntityState {
                    id: *id,
                    archetype: *archetype,
                    default_fields: default_fields.clone(),
                    fields: fields.clone(),
                    components: components(changes),
                    created_at_us: Some(timestamp_us),
                    updated_at_us: timestamp_us,
                },
            );
        }
        ReplicationChange::EntityUpdated {
            id,
            fields,
            components: changes,
        } => {
            let Some(state) = world.entities.get_mut(id) else {
                return;
            };
            state.fields = fields.clone();
            state.updated_at_us = timestamp_us;
            for change in changes {
                state
                    .components
                    .insert(change.index, component_state(change));
            }
        }
        ReplicationChange::EntityDeleted { id } => {
            world.entities.remove(id);
        }
        ReplicationChange::Keyframe {
            id,
            archetype,
            default_fields,
            fields,
            components: changes,
        } => {
            world.keyframes.insert(
                *id,
                EntityState {
                    id: *id,
                    archetype: *archetype,
                    default_fields: default_fields.clone(),
                    fields: fields.clone(),
                    components: components(changes),
                    created_at_us: None,
                    updated_at_us: timestamp_us,
                },
            );
        }
        ReplicationChange::Control { .. } | ReplicationChange::Event { .. } => {}
    }
}

#[cfg(test)]
mod playback_tests;
