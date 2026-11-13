//! Chronological schema declarations and view admission needed to decode records.
use crate::theater::parser::v41::chunks::registry::source::FilmRegistry;
use crate::theater::parser::v41::chunks::replication::bindings::{
    AnticipatedBindings, AnticipatedDeclaration,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct BindingConflict {
    pub record_bit: i64,
    pub id: u32,
    pub slot: u32,
    pub existing_archetype: u32,
    pub proposed_archetype: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum DeltaAdmission {
    Allowed,
    Anticipated(AnticipatedDeclaration),
    Unbound,
    OtherView,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct DecodeBinding {
    pub archetype: u32,
    pub full_id: u32,
    pub soft: bool,
    pub generation_any: bool,
    /// None is an unknown owning view, which does not reject a delta.
    pub view: Option<i8>,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct ReplicationDecodeState {
    pub slots: BTreeMap<u32, DecodeBinding>,
    /// Ephemeral contextual recovery evidence, drained by runtime per packet.
    #[serde(skip)]
    pub(crate) schema_recoveries: Vec<(
        u32,
        i64,
        super::replication_stream::frame::entities::RecoveredSchema,
    )>,
    pub current_view: i8,
    pub keyframe_namespace: Option<u8>,
    pub current_chunk: usize,
    pub anticipated: Option<AnticipatedBindings>,
    pub anticipations: BTreeMap<u32, usize>,
    #[serde(skip)]
    pub(crate) pending_frame_deletes: Option<Vec<(u32, DecodeBinding)>>,
    #[serde(skip)]
    pub(crate) deferred_frame_deletes: BTreeMap<u32, (u32, DecodeBinding, u64)>,
    #[serde(skip)]
    pub(crate) frame_sequence: u64,
}
impl ReplicationDecodeState {
    pub(crate) fn begin_frame_deletes(&mut self) {
        self.frame_sequence += 1;
        // Later reuse of the slot cannot corroborate the rejected frame.
        self.deferred_frame_deletes
            .retain(|_, (_, _, sequence)| *sequence + 1 >= self.frame_sequence);
        self.pending_frame_deletes = Some(Vec::new());
    }

    pub(crate) fn finish_frame_deletes(
        &mut self,
        defer_id: Option<u32>,
        closed: bool,
        registry: &FilmRegistry,
    ) {
        if let Some(deletes) = self.pending_frame_deletes.take() {
            for (deleted_id, previous) in deletes {
                // An unclosed packet cannot establish a replacement biped generation.
                // A later admitted NEW in this packet still supersedes the old schema.
                if !closed && previous.archetype == 35 {
                    self.slots
                        .entry(deleted_id & 0x3fff_ffff)
                        .or_insert(previous);
                    continue;
                }
                // Only the single-component high-frequency schema can be retried;
                // composite schemas can decode plausible but wrong successors.
                if Some(deleted_id) == defer_id
                    && registry
                        .archetype(previous.archetype as usize)
                        .is_some_and(|archetype| {
                            archetype.components.len() == 1
                                && archetype.components[0].name() == Some("high-frequency")
                        })
                {
                    // A rejected successor cannot prove the old generation survived.
                    // Only its own later delta can restore this binding.
                    self.deferred_frame_deletes.insert(
                        deleted_id & 0x3fff_ffff,
                        (deleted_id, previous, self.frame_sequence),
                    );
                }
            }
        }
    }

    pub(crate) fn unbind_frame_delete(&mut self, id: u32) {
        let slot = id & 0x3fff_ffff;
        if let Some(deletes) = self.pending_frame_deletes.as_mut()
            && let Some(previous) = self.slots.get(&slot)
            && !previous.generation_any
            && previous.full_id != id
            && !deletes.iter().any(|(old_id, _)| *old_id == id)
        {
            deletes.push((id, previous.clone()));
        }
        self.unbind(slot);
    }

    pub(crate) fn deferred_mismatched_delete(&self, id: u32) -> bool {
        self.deferred_frame_deletes
            .get(&(id & 0x3fff_ffff))
            .is_some_and(|(deleted_id, _, _)| *deleted_id == id)
    }

    /// Reference frame_infer.go admission, distinct from unconditional setup/restore.
    /// Refusal does not stop the record walk or discard the decoded NEW fields.
    pub(crate) fn bind_reference_new(
        &mut self,
        id: u32,
        archetype: u32,
        record_bit: i64,
    ) -> Option<BindingConflict> {
        let slot = id & 0x3fff_ffff;
        if let Some(existing) = self.slots.get(&slot)
            && !existing.soft
            && existing.archetype != archetype
        {
            return Some(BindingConflict {
                record_bit,
                id,
                slot,
                existing_archetype: existing.archetype,
                proposed_archetype: archetype,
            });
        }
        self.bind_full(id, archetype);
        None
    }

    pub(crate) fn bind_full(&mut self, id: u32, archetype: u32) {
        self.deferred_frame_deletes.remove(&(id & 0x3fff_ffff));
        self.slots.insert(
            id & 0x3fff_ffff,
            DecodeBinding {
                archetype,
                full_id: id,
                soft: false,
                generation_any: false,
                view: Some(self.current_view),
            },
        );
    }
    pub(crate) fn bind_wildcard(&mut self, slot: u32, archetype: u32) {
        let slot = slot & 0x3fff_ffff;
        self.deferred_frame_deletes.remove(&slot);
        self.slots.insert(
            slot,
            DecodeBinding {
                archetype,
                full_id: slot,
                soft: false,
                generation_any: true,
                view: Some(0),
            },
        );
    }
    pub(crate) fn bind_keyframe(&mut self, namespace: u32, slot: u32, archetype: u32) {
        let namespace = (namespace & 3) as u8;
        let first = *self.keyframe_namespace.get_or_insert(namespace);
        self.bind_wildcard(slot, archetype);
        self.slots.get_mut(&(slot & 0x3fff_ffff)).unwrap().view = (namespace == first).then_some(0);
    }
    pub(crate) fn bind_datum(&mut self, slot: u32, archetype: u32) {
        self.bind_wildcard(slot, archetype);
        let s = self.slots.get_mut(&(slot & 0x3fff_ffff)).unwrap();
        s.soft = true;
        s.view = None;
    }
    pub(crate) fn bind_inferred(&mut self, id: u32, archetype: u32) {
        self.bind_full(id, archetype);
        let slot = self.slots.get_mut(&(id & 0x3fff_ffff)).unwrap();
        slot.soft = true;
    }
    pub(crate) fn unbind(&mut self, slot: u32) {
        self.slots.remove(&slot);
        self.deferred_frame_deletes.remove(&slot);
    }
    pub(crate) fn archetype(&self, slot: u32) -> Option<u32> {
        self.slots.get(&slot).map(|s| s.archetype)
    }
    pub(crate) fn view_owns(&self, slot: u32) -> bool {
        self.slots
            .get(&(slot & 0x3fff_ffff))
            .is_some_and(|s| s.view.is_none_or(|v| v == self.current_view))
    }
    pub(crate) fn bind_anticipated(&mut self, id: u32) -> Option<AnticipatedDeclaration> {
        let slot = id & 0x3fff_ffff;
        if self.slots.contains_key(&slot) {
            return None;
        }
        let d = self.anticipated.as_ref()?.after(id, self.current_chunk)?;
        self.bind_datum(slot, d.archetype);
        *self.anticipations.entry(d.archetype).or_default() += 1;
        Some(d)
    }
    /// Production rejection checks view ownership, not strict generation matching.
    /// A rejection ends the entity view at the header boundary without reading its body.
    pub(crate) fn admit_delta(&mut self, id: u32, tables_enabled: bool) -> DeltaAdmission {
        if !tables_enabled {
            return DeltaAdmission::Allowed;
        }
        let slot = id & 0x3fff_ffff;
        if !self.slots.contains_key(&slot)
            && let Some((_, previous, _)) = self
                .deferred_frame_deletes
                .get(&slot)
                .filter(|(_, previous, _)| previous.full_id == id)
                .cloned()
        {
            self.deferred_frame_deletes.remove(&slot);
            self.slots.insert(slot, previous);
        }
        if self
            .slots
            .get(&slot)
            .is_some_and(|binding| binding.soft && !binding.generation_any && binding.full_id != id)
        {
            self.unbind(slot);
        }
        let anticipated = if self.archetype(slot).is_none() {
            let Some(d) = self.bind_anticipated(id) else {
                return DeltaAdmission::Unbound;
            };
            Some(d)
        } else {
            None
        };
        if !self.view_owns(slot) {
            return DeltaAdmission::OtherView;
        }
        anticipated.map_or(DeltaAdmission::Allowed, DeltaAdmission::Anticipated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theater::parser::packet::ByteRange;
    use crate::theater::parser::v41::chunks::registry::source::{FilmArchetype, RegistryComponent};

    fn registry() -> FilmRegistry {
        let range = ByteRange { start: 0, end: 0 };
        let mut archetypes = (0..5)
            .map(|index| FilmArchetype {
                index,
                components: vec![],
                source: range,
                padding: range,
            })
            .collect::<Vec<_>>();
        let mut name_bytes = vec![0; 256];
        name_bytes[.."high-frequency".len()].copy_from_slice(b"high-frequency");
        archetypes[4].components.push(RegistryComponent {
            name_bytes,
            precision_level: 0,
            source: range,
        });
        FilmRegistry { archetypes }
    }

    #[test]
    fn unclosed_delete_of_another_generation_preserves_a_biped() {
        let live = (1 << 30) | 123;
        let other = (2 << 30) | 123;
        for (deleted, closed, survives) in [
            (other, false, true),
            (other, true, false),
            (live, false, false),
            (live, true, false),
        ] {
            let mut state = ReplicationDecodeState::default();
            state.bind_full(live, 35);
            state.begin_frame_deletes();
            state.unbind_frame_delete(deleted);
            state.finish_frame_deletes(None, closed, &registry());
            assert_eq!(
                state.slots.get(&123).map(|b| b.full_id),
                survives.then_some(live)
            );
        }

        let mut state = ReplicationDecodeState::default();
        state.bind_full(live, 35);
        state.begin_frame_deletes();
        state.unbind_frame_delete(other);
        state.bind_full(other, 3);
        state.finish_frame_deletes(None, false, &registry());
        assert_eq!(state.slots[&123].full_id, other);
        assert_eq!(state.slots[&123].archetype, 3);
    }

    #[test]
    fn inferred_schema_does_not_cross_generation_reuse() {
        let mut state = ReplicationDecodeState::default();
        let first = (1 << 30) | 191;
        let next = (2 << 30) | 191;
        state.bind_inferred(first, 35);
        assert_eq!(state.admit_delta(first, true), DeltaAdmission::Allowed);
        assert_eq!(state.admit_delta(next, true), DeltaAdmission::Unbound);
        assert!(state.archetype(191).is_none());
    }

    #[test]
    fn unclosed_cross_generation_delete_recovers_prior_binding_on_matching_delta() {
        let mut state = ReplicationDecodeState::default();
        let live = (1 << 30) | 123;
        state.bind_full(live, 4);
        state.begin_frame_deletes();
        state.unbind_frame_delete((2 << 30) | 123);
        state.finish_frame_deletes(Some((2 << 30) | 123), false, &registry());
        assert_eq!(state.archetype(123), None);
        state.begin_frame_deletes();
        assert_eq!(state.admit_delta(live, true), DeltaAdmission::Allowed);
        assert_eq!(state.archetype(123), Some(4));
        state.begin_frame_deletes();
        state.unbind_frame_delete((2 << 30) | 123);
        state.finish_frame_deletes(None, true, &registry());
        assert_eq!(state.archetype(123), None);
    }

    #[test]
    fn new_binding_supersedes_a_deferred_delete() {
        let mut state = ReplicationDecodeState::default();
        let live = (1 << 30) | 123;
        let deleted = (2 << 30) | 123;
        state.bind_full(live, 4);
        state.begin_frame_deletes();
        state.unbind_frame_delete(deleted);
        state.finish_frame_deletes(Some(deleted), false, &registry());
        state.begin_frame_deletes();
        assert_eq!(state.bind_reference_new(deleted, 35, 0), None);
        assert!(!state.deferred_mismatched_delete(deleted));
        assert_eq!(state.admit_delta(live, true), DeltaAdmission::Allowed);
        assert_eq!(state.archetype(123), Some(35));
    }

    #[test]
    fn deferred_delete_expires_after_the_next_frame() {
        let mut state = ReplicationDecodeState::default();
        let live = (1 << 30) | 123;
        let deleted = (2 << 30) | 123;
        state.bind_full(live, 4);
        state.begin_frame_deletes();
        state.unbind_frame_delete(deleted);
        state.finish_frame_deletes(Some(deleted), false, &registry());
        state.begin_frame_deletes();
        assert!(state.deferred_mismatched_delete(deleted));
        state.finish_frame_deletes(None, true, &registry());
        state.begin_frame_deletes();
        assert!(!state.deferred_mismatched_delete(deleted));
        assert_eq!(state.admit_delta(live, true), DeltaAdmission::Unbound);
    }

    #[test]
    fn rejected_delete_does_not_restore_an_unrelated_schema() {
        let mut state = ReplicationDecodeState::default();
        let live = (1 << 30) | 123;
        let deleted = (2 << 30) | 123;
        state.bind_full(live, 3);
        state.begin_frame_deletes();
        state.unbind_frame_delete(deleted);
        state.finish_frame_deletes(Some(deleted), false, &registry());
        state.begin_frame_deletes();
        assert!(!state.deferred_mismatched_delete(deleted));
        assert_eq!(state.admit_delta(live, true), DeltaAdmission::Unbound);
    }
}
