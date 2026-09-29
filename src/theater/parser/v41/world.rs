//! Reference world state for production frame walking and rollback.
use super::{AnticipatedBindings, AnticipatedDeclaration};
pub(crate) use crate::theater::film::chunks::replication::replication_stream::models::world::{
    FilmViewAdmission, NewBindingRefusal,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct FilmWorldSlot {
    pub archetype: u32,
    pub full_id: u32,
    pub soft: bool,
    pub generation_any: bool,
    pub position: Option<[f32; 3]>,
    /// None is an unknown owning view, which does not reject a delta.
    pub view: Option<i8>,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct FilmWorld {
    pub slots: BTreeMap<u32, FilmWorldSlot>,
    pub current_view: i8,
    pub keyframe_namespace: Option<u8>,
    pub current_chunk: i64,
    pub anticipated: Option<AnticipatedBindings>,
    pub anticipations: BTreeMap<u32, usize>,
}
impl FilmWorld {
    /// Reference frame_infer.go admission, distinct from unconditional setup/restore.
    /// Refusal does not stop the record walk or discard the decoded NEW fields.
    pub(crate) fn bind_reference_new(
        &mut self,
        id: u32,
        archetype: u32,
        record_bit: i64,
    ) -> Option<NewBindingRefusal> {
        let slot = id & 0x3fff_ffff;
        if let Some(existing) = self.slots.get(&slot)
            && !existing.soft
            && existing.archetype != archetype
        {
            return Some(NewBindingRefusal {
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
        self.slots.insert(
            id & 0x3fff_ffff,
            FilmWorldSlot {
                archetype,
                full_id: id,
                soft: false,
                generation_any: false,
                position: None,
                view: Some(self.current_view),
            },
        );
    }
    pub(crate) fn bind_wildcard(&mut self, slot: u32, archetype: u32) {
        let slot = slot & 0x3fff_ffff;
        self.slots.insert(
            slot,
            FilmWorldSlot {
                archetype,
                full_id: slot,
                soft: false,
                generation_any: true,
                position: None,
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
    pub(crate) fn unbind(&mut self, slot: u32) {
        self.slots.remove(&slot);
    }
    pub(crate) fn archetype(&self, slot: u32) -> Option<u32> {
        self.slots.get(&slot).map(|s| s.archetype)
    }
    pub(crate) fn view_owns(&self, slot: u32) -> bool {
        self.slots
            .get(&(slot & 0x3fff_ffff))
            .is_some_and(|s| s.view.is_none_or(|v| v == self.current_view))
    }
    pub(crate) fn set_position(&mut self, slot: u32, position: [f32; 3]) {
        if let Some(s) = self.slots.get_mut(&slot) {
            s.position = Some(position);
        }
    }
    pub(crate) fn position(&self, slot: u32) -> Option<[f32; 3]> {
        self.slots.get(&slot).and_then(|s| s.position)
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
    pub(crate) fn admit_delta(&mut self, id: u32, tables_enabled: bool) -> FilmViewAdmission {
        if !tables_enabled {
            return FilmViewAdmission::Allowed;
        }
        let slot = id & 0x3fff_ffff;
        let anticipated = if self.archetype(slot).is_none() {
            let Some(d) = self.bind_anticipated(id) else {
                return FilmViewAdmission::Unbound;
            };
            Some(d)
        } else {
            None
        };
        if !self.view_owns(slot) {
            return FilmViewAdmission::OtherView;
        }
        anticipated.map_or(FilmViewAdmission::Allowed, FilmViewAdmission::Anticipated)
    }
}
