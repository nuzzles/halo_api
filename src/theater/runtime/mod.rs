//! Stateful playback of a film's recorded ECS observations.
//!
//! Controls and recorded events remain in the observation feed; they do not
//! imply entity state changes. Highlights are the film's guarded candidates.

use crate::theater::Film;
pub use crate::theater::film::replication::ReplicationValue;
use crate::theater::film::{ComponentSlot, EntityId, EntitySchemaId, EntitySchemaKind, KeyframeId};
pub use crate::theater::film::{
    ReplicationChange, ReplicationControl, ReplicationField, ReplicationObservation,
    SummaryHighlight,
};
use crate::theater::film::{ReplicationComponentChange, ReplicationStream};
use std::collections::BTreeMap;
use std::sync::Arc;

/// One complete component body as recorded at a packet timestamp. Values may
/// encode deltas or gated fields; they are not necessarily absolute state.
#[derive(Debug, Clone, PartialEq)]
pub struct ComponentObservation {
    pub timestamp_us: u64,
    pub fields: Vec<ReplicationField>,
}

/// Component slot observed on a live entity or keyframe. Bodies retain source
/// order; the latest body is available through [`Self::fields`].
#[derive(Debug, Clone, PartialEq)]
pub struct ComponentState {
    pub name: String,
    pub observations: Vec<ComponentObservation>,
}

impl ComponentState {
    /// Fields from the latest complete recorded body, possibly encoded deltas.
    pub fn fields(&self) -> &[ReplicationField] {
        self.observations.last().map_or(&[], |body| &body.fields)
    }
}

/// Entity-level fields from one complete creation or update record.
#[derive(Debug, Clone, PartialEq)]
pub struct EntityFieldObservation {
    pub timestamp_us: u64,
    pub fields: Vec<ReplicationField>,
}

/// ECS-shaped view of observed live entity or keyframe data. Encoded delta
/// bodies remain raw; this does not reconstruct every absolute gameplay value.
/// `Id` keeps the two recorded ID domains distinct.
#[derive(Debug, Clone, PartialEq)]
pub struct EntityState<Id = EntityId> {
    /// Exact recorded ID in this state's domain.
    pub id: Id,
    pub schema: EntitySchemaId,
    /// Schema meaning captured in the finished film.
    pub schema_kind: EntitySchemaKind,
    pub default_fields: Vec<ReplicationField>,
    /// Nonempty entity-level field bodies in source order.
    pub field_observations: Vec<EntityFieldObservation>,
    pub components: BTreeMap<ComponentSlot, ComponentState>,
    pub created_at_us: Option<u64>,
    pub updated_at_us: u64,
}

impl<Id> EntityState<Id> {
    /// Fields from the latest entity-level record that carried fields.
    pub fn fields(&self) -> &[ReplicationField] {
        self.field_observations
            .last()
            .map_or(&[], |body| &body.fields)
    }
}

/// Live entity and separate keyframe observations through the requested time.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct WorldSnapshot {
    /// Last complete recorded session roster; None until a roster packet is observed.
    pub roster: Option<Vec<crate::theater::film::RosterPlayer>>,
    /// Last complete recorded bot metadata, with no inferred entity ownership.
    pub bots: Option<Vec<crate::theater::film::BotPlayer>>,
    /// Live entities keyed by their exact recorded IDs.
    pub entities: BTreeMap<EntityId, EntityState>,
    /// Keyframe baselines keyed by their exact recorded IDs. No link to a live
    /// entity is assumed from the keyframe ID.
    pub keyframes: BTreeMap<KeyframeId, EntityState<KeyframeId>>,
}

/// A shared full film or replication-only film and an independent playback cursor.
/// The original input type is retained, without cloning its decoded observations.
#[derive(Debug)]
pub struct FilmRuntime<F = Film> {
    film: Arc<F>,
    cursor: usize,
    timestamp_us: Option<u64>,
    world: WorldSnapshot,
}

impl<F: AsRef<ReplicationStream>> FilmRuntime<F> {
    /// Prepare playback from an owned or `Arc`-shared [`Film`] or [`crate::theater::FilmReplication`].
    /// Observation timestamps are assumed to be nondecreasing in source order.
    pub fn load(film: impl Into<Arc<F>>) -> Self {
        let film = film.into();
        Self {
            film,
            cursor: 0,
            timestamp_us: None,
            world: WorldSnapshot::default(),
        }
    }

    /// Parsed film used by this playback cursor.
    pub fn film(&self) -> &F {
        self.film.as_ref()
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
        if self
            .timestamp_us
            .is_some_and(|previous| timestamp_us < previous)
        {
            self.rewind();
        }
        let observations = &self.film.as_ref().as_ref().observations;
        while self.cursor < observations.len()
            && observations[self.cursor].timestamp_us <= timestamp_us
        {
            apply(&mut self.world, &observations[self.cursor]);
            self.cursor += 1;
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
        self.world = WorldSnapshot::default();
        self.cursor = 0;
        self.timestamp_us = None;
    }
}

fn component_state(change: &ReplicationComponentChange, timestamp_us: u64) -> ComponentState {
    ComponentState {
        name: change.name.to_string(),
        observations: vec![ComponentObservation {
            timestamp_us,
            fields: change.fields.clone(),
        }],
    }
}

fn components(
    changes: &[ReplicationComponentChange],
    timestamp_us: u64,
) -> BTreeMap<ComponentSlot, ComponentState> {
    changes
        .iter()
        .map(|change| (change.index, component_state(change, timestamp_us)))
        .collect()
}

fn field_observations(
    fields: &[ReplicationField],
    timestamp_us: u64,
) -> Vec<EntityFieldObservation> {
    if fields.is_empty() {
        Vec::new()
    } else {
        vec![EntityFieldObservation {
            timestamp_us,
            fields: fields.to_vec(),
        }]
    }
}

fn apply(world: &mut WorldSnapshot, observation: &ReplicationObservation) {
    let timestamp_us = observation.timestamp_us;
    match &observation.change {
        ReplicationChange::Roster { players } => world.roster = Some(players.clone()),
        ReplicationChange::Bots { players } => world.bots = Some(players.clone()),
        ReplicationChange::EntityCreated {
            id,
            schema,
            schema_kind,
            default_fields,
            fields,
            components: changes,
        } => {
            world.entities.insert(
                *id,
                EntityState {
                    id: *id,
                    schema: *schema,
                    schema_kind: *schema_kind,
                    default_fields: default_fields.clone(),
                    field_observations: field_observations(fields, timestamp_us),
                    components: components(changes, timestamp_us),
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
            state
                .field_observations
                .extend(field_observations(fields, timestamp_us));
            state.updated_at_us = timestamp_us;
            for change in changes {
                if let Some(component) = state.components.get_mut(&change.index) {
                    component.name = change.name.to_string();
                    component.observations.push(ComponentObservation {
                        timestamp_us,
                        fields: change.fields.clone(),
                    });
                } else {
                    state
                        .components
                        .insert(change.index, component_state(change, timestamp_us));
                }
            }
        }
        ReplicationChange::EntityDeleted { id } => {
            world.entities.remove(id);
        }
        ReplicationChange::Keyframe {
            id,
            schema,
            schema_kind,
            default_fields,
            fields,
            components: changes,
        } => {
            world.keyframes.insert(
                *id,
                EntityState {
                    id: *id,
                    schema: *schema,
                    schema_kind: *schema_kind,
                    default_fields: default_fields.clone(),
                    field_observations: field_observations(fields, timestamp_us),
                    components: components(changes, timestamp_us),
                    created_at_us: None,
                    updated_at_us: timestamp_us,
                },
            );
        }
        ReplicationChange::Control { .. } | ReplicationChange::Event { .. } => {}
    }
}
