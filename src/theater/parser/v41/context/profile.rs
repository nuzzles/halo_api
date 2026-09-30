//! Reference data models.
use crate::theater::parser::v41::context::movement::MovementProfile;
use crate::theater::parser::v41::context::values::FilmMppWidths;
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};
/// An allocated reference calibration map. None at the grammar field represents a
/// nil map; clones of this handle share subsequent entry mutations, like Go maps.
#[derive(Debug, Clone, Default)]
pub struct SharedWidths(pub(crate) Arc<Mutex<BTreeMap<String, i64>>>);

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DecodeGrammar {
    pub corruption_check: bool,
    pub new_record_tail_bits: i64,
    pub default_state_by_archetype: bool,
    pub simulation_complete: bool,
    pub baseline_scope: bool,
    pub writer_absolute: bool,
    pub mobility_action_body: bool,
    pub ability_anchor_body: bool,
    pub chain_inference: bool,
    pub calibrated_widths: Option<SharedWidths>,
    pub generation_strict: bool,
    pub view_tables: bool,
    pub view_classes: bool,
    pub stub_widths: Option<SharedWidths>,
}

/// Raw reference keyframe dimensions. Header movement is signed; size words are
/// cast to reference uint only when a full-state body actually reads them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct KeyframeReadLayout {
    pub header_bits: i64,
    pub size_word_bits: i64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Default)]
pub struct DecodeProfile {
    pub movement: MovementProfile,
    pub keyframe: KeyframeReadLayout,
    pub mpp: FilmMppWidths,
    pub grammar: DecodeGrammar,
}

use serde::{Deserialize, Deserializer, Serialize, Serializer};

impl SharedWidths {
    pub(crate) fn from_map(values: BTreeMap<String, i64>) -> Self {
        Self(Arc::new(Mutex::new(values)))
    }
    pub(crate) fn snapshot(&self) -> BTreeMap<String, i64> {
        self.0.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }
    pub(crate) fn get(&self, name: &str) -> Option<i64> {
        self.0
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(name)
            .copied()
    }
}
impl PartialEq for SharedWidths {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0) || self.snapshot() == other.snapshot()
    }
}

impl Serialize for SharedWidths {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.snapshot().serialize(s)
    }
}
impl<'de> Deserialize<'de> for SharedWidths {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Ok(Self::from_map(BTreeMap::deserialize(d)?))
    }
}

impl Default for DecodeGrammar {
    fn default() -> Self {
        Self {
            corruption_check: false,
            new_record_tail_bits: 0,
            default_state_by_archetype: true,
            simulation_complete: false,
            baseline_scope: false,
            writer_absolute: false,
            mobility_action_body: true,
            ability_anchor_body: true,
            chain_inference: false,
            calibrated_widths: None,
            generation_strict: false,
            view_tables: true,
            view_classes: true,
            stub_widths: None,
        }
    }
}
impl Default for KeyframeReadLayout {
    fn default() -> Self {
        Self {
            header_bits: 108,
            size_word_bits: 32,
        }
    }
}

impl Eq for SharedWidths {}
