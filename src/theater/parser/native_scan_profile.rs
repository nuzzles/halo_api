//! Native scan settings, including Go map aliasing across by-value profile copies.
use super::*;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

/// An allocated native calibration map. None at the grammar field represents a
/// nil map; clones of this handle share subsequent entry mutations, like Go maps.
#[derive(Debug, Clone, Default)]
pub struct NativeSharedWidths(Arc<Mutex<BTreeMap<String, i64>>>);
impl NativeSharedWidths {
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
impl PartialEq for NativeSharedWidths {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0) || self.snapshot() == other.snapshot()
    }
}

impl Serialize for NativeSharedWidths {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.snapshot().serialize(s)
    }
}
impl<'de> Deserialize<'de> for NativeSharedWidths {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Ok(Self::from_map(BTreeMap::deserialize(d)?))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeScanGrammar {
    pub corruption_check: bool,
    pub new_record_tail_bits: i64,
    pub default_state_by_archetype: bool,
    pub simulation_complete: bool,
    pub baseline_scope: bool,
    pub writer_absolute: bool,
    pub mobility_action_body: bool,
    pub ability_anchor_body: bool,
    pub chain_inference: bool,
    pub calibrated_widths: Option<NativeSharedWidths>,
    pub generation_strict: bool,
    pub view_tables: bool,
    pub view_classes: bool,
    pub stub_widths: Option<NativeSharedWidths>,
}
impl Default for NativeScanGrammar {
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
/// Raw native keyframe dimensions. Header movement is signed; size words are
/// cast to native uint only when a full-state body actually reads them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct NativeKeyframeLayout {
    pub header_bits: i64,
    pub size_word_bits: i64,
}
impl Default for NativeKeyframeLayout {
    fn default() -> Self {
        Self {
            header_bits: 108,
            size_word_bits: 32,
        }
    }
}
impl NativeKeyframeLayout {
    /// Address-sized representation for legacy consumers; admission remains at use.
    pub(crate) fn bounded(self) -> Option<KeyframeLayout> {
        Some(KeyframeLayout {
            header_bits: self.header_bits.try_into().ok()?,
            size_word_bits: self.size_word_bits.try_into().ok()?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct NativeScanProfile {
    pub movement: NativeMovementProfile,
    pub keyframe: NativeKeyframeLayout,
    pub mpp: FilmMppWidths,
    pub grammar: NativeScanGrammar,
}
impl NativeScanProfile {
    /// Freeze allocated width maps instead of sharing subsequent native mutations.
    /// Ordinary Clone deliberately preserves Go map aliasing.
    pub(crate) fn snapshot(&self) -> Self {
        let mut value = self.clone();
        value.grammar.calibrated_widths = self
            .grammar
            .calibrated_widths
            .as_ref()
            .map(|map| NativeSharedWidths::from_map(map.snapshot()));
        value.grammar.stub_widths = self
            .grammar
            .stub_widths
            .as_ref()
            .map(|map| NativeSharedWidths::from_map(map.snapshot()));
        value
    }
}

#[cfg(test)]
impl NativeSharedWidths {
    pub fn insert(&self, name: String, width: i64) -> Option<i64> {
        self.0
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(name, width)
    }
}

impl Eq for NativeSharedWidths {}
