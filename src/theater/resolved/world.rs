//! World support for ResolvedFilm.
use super::*;
/// Runtime entity view or an independent keyframe namespace. The first native
/// keyframe namespace seeds the entity view, following the decoder's binding policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EntityDomain {
    Runtime,
    Keyframe(u8),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct EntityKey {
    pub domain: EntityDomain,
    pub slot: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentState {
    pub name: String,
    /// Ordered raw fields, including repeated names and quantized values.
    pub fields: Vec<ComponentField>,
    /// False means these are only the known prefix of the latest update.
    pub complete: bool,
    pub source: SourceRef,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityState {
    /// Runtime ID including generation, or None for keyframe-only slot evidence.
    pub id: Option<u32>,
    pub archetype: u32,
    /// NEW timestamp, never a fabricated spawn time for a keyframe observation.
    pub created_at_us: Option<u64>,
    /// Whether the native baseline reader completed; opaque/skipped component
    /// bodies remain individually marked and do not become decoded values.
    pub baseline_read_complete: bool,
    pub components: BTreeMap<usize, Arc<ComponentState>>,
    /// Raw fields of the last NEW/keyframe baseline, including header fields.
    pub initial_fields: Arc<Vec<ComponentField>>,
    pub last_source: SourceRef,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WorldSnapshot {
    pub entities: BTreeMap<EntityKey, Arc<EntityState>>,
}

pub(super) fn apply(world: &mut WorldSnapshot, change: Option<&StateChange>) {
    if let Some(change) = change {
        if let Some(new) = &change.new {
            world.entities.insert(change.key, new.clone());
        } else {
            world.entities.remove(&change.key);
        }
    }
}

pub(super) fn resolve_change(
    film: &Film,
    world: &WorldSnapshot,
    event: &Event,
    first_namespace: &mut Option<u8>,
) -> Option<StateChange> {
    match record(film, event.source)? {
        Record::Entity(record) => {
            if let RecordRef::Entity {
                continuation: true, ..
            } = event.source.record
            {
                let continuation = film
                    .chunk(event.source.chunk)
                    .expect("indexed source chunk")
                    .packets()[event.source.packet]
                    .event_continuation
                    .as_ref()?;
                if continuation.state_policy != NativeContinuationStatePolicy::Applied {
                    return None;
                }
            }
            let packet = &film
                .chunk(event.source.chunk)
                .expect("indexed source chunk")
                .packets()[event.source.packet];
            if record.end_bit > packet.header.payload_size as i64 * 8 {
                return None;
            }
            if let RecordRef::Entity { continuation, .. } = event.source.record
                && frame(packet, continuation)?
                    .header_diagnostics
                    .new_binding_refusals
                    .iter()
                    .any(|refusal| refusal.record_bit == record.header.start_bit)
            {
                return None;
            }
            entity_change(world, event, record)
        }
        Record::Keyframe(record) => {
            let namespace = (record.id >> 30) as u8;
            let first = *first_namespace.get_or_insert(namespace);
            let key = EntityKey {
                domain: if first == namespace {
                    EntityDomain::Runtime
                } else {
                    EntityDomain::Keyframe(namespace)
                },
                slot: record.id & 0x3fff_ffff,
            };
            // Native padded reads remain visible but cannot become recorded state.
            let packet = &film
                .chunk(event.source.chunk)
                .expect("indexed source chunk")
                .packets()[event.source.packet];
            if record.start_bit < 0 || record.end_bit > packet.header.payload_size as i64 * 8 {
                return None;
            }
            let previous = world.entities.get(&key).cloned();
            let mut state = EntityState {
                id: None,
                archetype: record.archetype,
                created_at_us: None,
                baseline_read_complete: record.stop == KeyframeStop::Complete,
                components: BTreeMap::new(),
                initial_fields: Arc::new(record.fields.clone()),
                last_source: event.source,
            };
            for component in &record.attempts {
                let Some([start, end]) = component.field_range else {
                    continue;
                };
                let Some(fields) = record.fields.get(start..end) else {
                    continue;
                };
                state.components.insert(
                    component.index,
                    Arc::new(ComponentState {
                        name: component.name.clone(),
                        fields: fields.to_vec(),
                        complete: component.ported == Some(true),
                        source: event.source,
                    }),
                );
            }
            // Even partial keyframes explicitly replace the old baseline. Its
            // unread components stay absent instead of inheriting stale values.
            if record.stop == KeyframeStop::InvalidArchetype {
                return None;
            }
            Some(StateChange {
                key,
                previous,
                new: Some(Arc::new(state)),
                derivation: "sequential-keyframe-baseline-v1",
            })
        }
        _ => None,
    }
}

fn entity_change(
    world: &WorldSnapshot,
    event: &Event,
    record: &EntityRecord,
) -> Option<StateChange> {
    let id = record.header.id?;
    if record.padded_bits != 0
        || record.header.start_bit < 0
        || !record.diagnostics.new_binding_refusals.is_empty()
    {
        return None;
    }
    let key = EntityKey {
        domain: EntityDomain::Runtime,
        slot: id & 0x3fff_ffff,
    };
    let previous = world.entities.get(&key).cloned();
    if record.header.kind == RecordKind::Delete {
        let existing = previous.as_ref()?;
        if existing.id.is_some_and(|old| old != id) || record.stop != EntityViewStop::Complete {
            return None;
        }
        return Some(StateChange {
            key,
            previous,
            new: None,
            derivation: "recorded-delete-v1",
        });
    }
    // Match native admission: an incomplete NEW does not establish a binding.
    if record.header.kind == RecordKind::New && record.stop != EntityViewStop::Complete {
        return None;
    }
    let mut state = match record.header.kind {
        RecordKind::New => EntityState {
            id: Some(id),
            archetype: record.archetype?,
            created_at_us: Some(event.timestamp_us),
            baseline_read_complete: !record.default_state_fallback,
            components: BTreeMap::new(),
            initial_fields: Arc::new(record.fields.clone()),
            last_source: event.source,
        },
        RecordKind::Delta => {
            let previous = previous.as_ref()?;
            if previous.id.is_some_and(|old| old != id)
                || record.archetype.is_some_and(|a| a != previous.archetype)
            {
                return None;
            }
            let mut state = (**previous).clone();
            state.id = Some(id);
            state
        }
        _ => return None,
    };
    state.last_source = event.source;
    for component in &record.attempts {
        let Some(fields) = record
            .fields
            .get(component.field_start..component.field_end)
        else {
            continue;
        };
        state.components.insert(
            component.span.index,
            Arc::new(ComponentState {
                name: component.span.name.clone(),
                fields: fields.to_vec(),
                complete: component.status == Some(true),
                source: event.source,
            }),
        );
    }
    Some(StateChange {
        key,
        previous,
        new: Some(Arc::new(state)),
        derivation: "sequential-entity-components-v1",
    })
}
