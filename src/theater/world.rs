//! Reference world state for production frame walking and rollback.
use super::{AnticipatedBindings, AnticipatedDeclaration};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A decoded NEW contradicted an existing hard parser binding. Its raw record
/// remains retained; only the proposed binding mutation is refused.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeNewBindingRefusal {
    pub record_bit: i64,
    pub id: u32,
    pub slot: u32,
    pub existing_archetype: u32,
    pub proposed_archetype: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilmWorldSlot {
    pub archetype: u32,
    pub full_id: u32,
    pub soft: bool,
    pub generation_any: bool,
    pub position: Option<[f32; 3]>,
    /// None is an unknown owning view, which does not reject a delta.
    pub view: Option<i8>,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FilmWorld {
    pub slots: BTreeMap<u32, FilmWorldSlot>,
    pub current_view: i8,
    pub keyframe_namespace: Option<u8>,
    pub current_chunk: i64,
    pub anticipated: Option<AnticipatedBindings>,
    pub anticipations: BTreeMap<u32, usize>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilmWorldSnapshot {
    pub slots: BTreeMap<u32, FilmWorldSlot>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilmViewAdmission {
    Allowed,
    Anticipated(AnticipatedDeclaration),
    Unbound,
    OtherView,
}
impl FilmWorld {
    /// Native frame_infer.go admission, distinct from unconditional setup/restore.
    /// Refusal does not stop the record walk or discard the decoded NEW fields.
    pub(crate) fn bind_native_new(
        &mut self,
        id: u32,
        archetype: u32,
        record_bit: i64,
    ) -> Option<NativeNewBindingRefusal> {
        let slot = id & 0x3fff_ffff;
        if let Some(existing) = self.slots.get(&slot)
            && !existing.soft
            && existing.archetype != archetype
        {
            return Some(NativeNewBindingRefusal {
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

    pub fn bind_full(&mut self, id: u32, archetype: u32) {
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
    pub fn bind_soft(&mut self, id: u32, archetype: u32) {
        self.bind_full(id, archetype);
        self.slots.get_mut(&(id & 0x3fff_ffff)).unwrap().soft = true;
    }
    pub fn bind_wildcard(&mut self, slot: u32, archetype: u32) {
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
    pub fn bind_keyframe(&mut self, namespace: u32, slot: u32, archetype: u32) {
        let namespace = (namespace & 3) as u8;
        let first = *self.keyframe_namespace.get_or_insert(namespace);
        self.bind_wildcard(slot, archetype);
        self.slots.get_mut(&(slot & 0x3fff_ffff)).unwrap().view = (namespace == first).then_some(0);
    }
    pub fn bind_datum(&mut self, slot: u32, archetype: u32) {
        self.bind_wildcard(slot, archetype);
        let s = self.slots.get_mut(&(slot & 0x3fff_ffff)).unwrap();
        s.soft = true;
        s.view = None;
    }
    pub fn unbind(&mut self, slot: u32) {
        self.slots.remove(&slot);
    }
    pub fn archetype(&self, slot: u32) -> Option<u32> {
        self.slots.get(&slot).map(|s| s.archetype)
    }
    pub fn hard_bound(&self, slot: u32) -> bool {
        self.slots.get(&slot).is_some_and(|s| !s.soft)
    }
    pub fn view_owns(&self, slot: u32) -> bool {
        self.slots
            .get(&(slot & 0x3fff_ffff))
            .is_some_and(|s| s.view.is_none_or(|v| v == self.current_view))
    }
    pub fn generation_matches(&self, id: u32, strict: bool) -> bool {
        !strict
            || self
                .slots
                .get(&(id & 0x3fff_ffff))
                .is_some_and(|s| s.generation_any || s.full_id == id)
    }
    pub fn set_position(&mut self, slot: u32, position: [f32; 3]) {
        if let Some(s) = self.slots.get_mut(&slot) {
            s.position = Some(position);
        }
    }
    pub fn position(&self, slot: u32) -> Option<[f32; 3]> {
        self.slots.get(&slot).and_then(|s| s.position)
    }
    pub fn bind_anticipated(&mut self, id: u32) -> Option<AnticipatedDeclaration> {
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
    pub fn admit_delta(&mut self, id: u32, tables_enabled: bool) -> FilmViewAdmission {
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
    /// Native rollback restores slots only; namespace, view and counters remain current.
    pub fn snapshot(&self) -> FilmWorldSnapshot {
        FilmWorldSnapshot {
            slots: self.slots.clone(),
        }
    }
    pub fn restore(&mut self, snapshot: FilmWorldSnapshot) {
        self.slots = snapshot.slots;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
    use std::io::Read;
    #[test]
    fn mutations_generation_views_positions_and_rollback_match_go() {
        let mut text = String::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/world-levelup-v41.json.zlib")[..])
            .read_to_string(&mut text)
            .unwrap();
        let oracle: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(oracle["initial"].as_array().unwrap().len(), 2);
        for initial in oracle["initial"].as_array().unwrap() {
            let world = FilmWorld::default();
            assert_eq!(json!(world.slots), initial["slots"]);
            assert_eq!(json!(world.current_view), initial["current_view"]);
            assert_eq!(json!(world.current_chunk), initial["current_chunk"]);
            assert_eq!(
                json!(world.keyframe_namespace.map(i64::from).unwrap_or(-1)),
                initial["namespace"]
            );
            assert_eq!(json!(world.anticipations), initial["anticipations"]);
            assert_eq!(json!(world.anticipated.is_none()), initial["table_absent"]);
            // Rust passes the registry separately to walkers; it has no
            // registry pointer or one-time logging latch on FilmWorld.
            assert_eq!(initial["registry_retained"], true);
            assert_eq!(initial["logged"], false);
        }
        let declarations: Vec<(u32, i64, u32)> =
            serde_json::from_value(oracle["declarations"].clone()).unwrap();
        let table = AnticipatedBindings::from_declarations(declarations);
        assert_eq!(oracle["runs"].as_array().unwrap().len(), 128);
        for steps in oracle["runs"].as_array().unwrap() {
            let mut world = FilmWorld {
                anticipated: Some(table.clone()),
                ..Default::default()
            };
            let mut snapshot = world.snapshot();
            for step in steps.as_array().unwrap() {
                let id = step["id"].as_u64().unwrap() as u32;
                let slot = id & 0x3fff_ffff;
                let ti = step["ti"].as_u64().unwrap() as u32;
                let mut anticipated = None;
                match step["kind"].as_u64().unwrap() {
                    0 => snapshot = world.snapshot(),
                    1 => {
                        world.restore(snapshot);
                        snapshot = world.snapshot();
                    }
                    2 => world.bind_full(id, ti),
                    3 => world.bind_soft(id, ti),
                    4 => world.bind_wildcard(id, ti),
                    5 => world.bind_keyframe(step["ns"].as_u64().unwrap() as u32, id, ti),
                    6 => world.bind_datum(id, ti),
                    7 => world.unbind(slot),
                    8 => world.set_position(
                        slot,
                        serde_json::from_value(step["position"].clone()).unwrap(),
                    ),
                    9 => anticipated = world.bind_anticipated(id),
                    10 => world.current_view = step["view"].as_i64().unwrap() as i8,
                    11 => world.current_chunk = step["chunk"].as_i64().unwrap(),
                    12 => world.anticipated = None,
                    13 => world.anticipated = Some(table.clone()),
                    _ => unreachable!(),
                }
                assert_eq!(json!(anticipated.is_some()), step["anticipated_ok"]);
                assert_eq!(
                    json!(anticipated.map_or(0, |d| d.archetype)),
                    step["anticipated_ti"]
                );
                let admission = match world.admit_delta(id, step["tables"].as_bool().unwrap()) {
                    FilmViewAdmission::Allowed | FilmViewAdmission::Anticipated(_) => "allowed",
                    FilmViewAdmission::Unbound => "unbound",
                    FilmViewAdmission::OtherView => "other_view",
                };
                assert_eq!(admission, step["admission"].as_str().unwrap());
                assert_eq!(json!(world.current_chunk), step["current_chunk"]);
                assert_eq!(json!(world.current_view), step["current_view"]);
                assert_eq!(
                    json!(world.keyframe_namespace.map_or(-1, i32::from)),
                    step["namespace"]
                );
                assert_eq!(json!(world.anticipations), step["anticipations"]);
                let slots:BTreeMap<String,Value>=world.slots.iter().map(|(slot,s)|(slot.to_string(),json!({"TypeIndex":s.archetype,"FullID":s.full_id,"Soft":s.soft,"GenAny":s.generation_any,"Pos":s.position.unwrap_or([0.0;3]),"PosValid":s.position.is_some(),"Vue":s.view.unwrap_or(-1)}))).collect();
                let mut expected_slots = step["slots"].clone();
                for value in expected_slots.as_object_mut().unwrap().values_mut() {
                    let position: [f32; 3] = serde_json::from_value(value["Pos"].clone()).unwrap();
                    value["Pos"] = json!(position);
                }
                assert_eq!(json!(slots), expected_slots, "{step}");
                for q in step["checks"].as_array().unwrap() {
                    let id = q["id"].as_u64().unwrap() as u32;
                    let slot = id & 0x3fff_ffff;
                    assert_eq!(json!(world.archetype(slot).unwrap_or(0)), q["archetype"]);
                    assert_eq!(json!(world.archetype(slot).is_some()), q["bound"]);
                    assert_eq!(json!(world.hard_bound(slot)), q["hard"]);
                    assert_eq!(json!(world.view_owns(id)), q["owns"]);
                    assert_eq!(json!(world.generation_matches(id, true)), q["strict"]);
                    assert_eq!(json!(world.generation_matches(id, false)), q["loose"]);
                    assert_eq!(json!(world.position(slot).is_some()), q["position_valid"]);
                    let position: [f32; 3] = serde_json::from_value(q["position"].clone()).unwrap();
                    assert_eq!(world.position(slot).unwrap_or([0.0; 3]), position);
                }
            }
            assert_eq!(
                serde_json::from_value::<FilmWorld>(json!(world)).unwrap(),
                world
            );
        }
    }
}
