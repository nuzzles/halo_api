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
    pub fn from_map(values: BTreeMap<String, i64>) -> Self {
        Self(Arc::new(Mutex::new(values)))
    }
    pub fn snapshot(&self) -> BTreeMap<String, i64> {
        self.0.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }
    pub fn insert(&self, name: String, width: i64) -> Option<i64> {
        self.0
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(name, width)
    }
    /// Delete a live override, visible through every clone of this native map.
    /// An allocated empty map remains distinct from an absent map.
    pub fn remove(&self, name: &str) {
        self.0
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(name);
    }
    pub fn get(&self, name: &str) -> Option<i64> {
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
impl Eq for NativeSharedWidths {}
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
    /// Native CadreBits: header plus the two size words, with signed wrapping.
    /// This excludes default-state and component bodies; it is not a record length.
    pub fn frame_bits(self) -> i64 {
        self.header_bits
            .wrapping_add(self.size_word_bits.wrapping_mul(2))
    }

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
    pub fn snapshot(&self) -> Self {
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

    /// Raw replacement does not enable simulation completion or validate widths.
    pub fn set_world_precision(&mut self, descriptor: NativePrecisionDescriptor) {
        self.movement.world_object = descriptor;
    }
    /// Native layout installation: zero axes leave everything unchanged; a gate
    /// of four or less preserves the inherited index width. Other widths, even
    /// implausible ones, are metadata and are retained without truncation on WASM.
    pub fn set_world_precision_from_layout(&mut self, layout: &I0Layout) -> bool {
        if layout.axis_widths.contains(&0) {
            return false;
        }
        self.movement.world_object.axis_bits = layout.axis_widths;
        if layout.gate_bits > 4 {
            self.movement.world_object.index_bits = (layout.gate_bits - 4) as u64;
        }
        self.movement.world_object.region = layout.region;
        self.grammar.simulation_complete = true;
        true
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use serde_json::Value;
    use std::io::Read;
    fn inflate(data: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        flate2::read::ZlibDecoder::new(data)
            .read_to_end(&mut out)
            .unwrap();
        out
    }
    fn descriptor(v: &Value) -> NativePrecisionDescriptor {
        NativePrecisionDescriptor {
            index_bits: v["IndexW"].as_u64().unwrap(),
            axis_bits: serde_json::from_value(v["AxisW"].clone()).unwrap(),
            region: v["Region"].as_u64().unwrap() as u32,
        }
    }
    fn layout(v: &Value) -> I0Layout {
        I0Layout {
            gate_bits: v["GateBits"].as_i64().unwrap(),
            axis_widths: serde_json::from_value(v["AxisW"].clone()).unwrap(),
            region: v["Region"].as_u64().unwrap() as u32,
        }
    }
    pub(crate) fn profile(v: &Value) -> NativeScanProfile {
        let m = &v["Mouvement"];
        let g = &v["Grammaire"];
        let bit = |key: &str| g[key].as_bool().unwrap();
        NativeScanProfile {
            movement: NativeMovementProfile {
                traversal: descriptor(&m["Traversal"]),
                world_object: descriptor(&m["WorldObject"]),
                delta_quantum: m["DeltaQuantum"].as_f64().unwrap() as f32,
                delta_axis_width: m["DeltaAxisWidth"].as_u64().unwrap(),
                range: std::array::from_fn(|i| {
                    [
                        m["Range"][i]["Min"].as_f64().unwrap() as f32,
                        m["Range"][i]["Max"].as_f64().unwrap() as f32,
                    ]
                }),
                full_precision: m["FullPrecision"].as_bool().unwrap(),
                delta_has_handle_tail: m["DeltaHasHandleTail"].as_bool().unwrap(),
                calibrated_skip: m["CalibratedSkip"].as_bool().unwrap(),
                mobility_action_extra_bits: m["MobilityActionExtraBits"].as_i64().unwrap(),
            },
            keyframe: NativeKeyframeLayout {
                header_bits: v["Cadre"]["EnTeteBits"].as_i64().unwrap(),
                size_word_bits: v["Cadre"]["MotDeTailleBits"].as_i64().unwrap(),
            },
            mpp: serde_json::from_value(v["MPP"].clone()).unwrap(),
            grammar: NativeScanGrammar {
                corruption_check: bit("ControleDeCorruption"),
                new_record_tail_bits: g["BitsDeQueueRecordNew"].as_i64().unwrap(),
                default_state_by_archetype: bit("DeserEtatParArchetype"),
                simulation_complete: bit("SimStateComplet"),
                baseline_scope: bit("PorteeBaseline"),
                writer_absolute: bit("GrammaireEcrivainI0"),
                mobility_action_body: bit("CorpsActionMobilite"),
                ability_anchor_body: bit("CorpsAncrageCapacite"),
                chain_inference: bit("InferenceChaine"),
                calibrated_widths: serde_json::from_value(g["LargeursCalibrees"].clone()).unwrap(),
                generation_strict: bit("GenerationStricte"),
                view_tables: bit("TablesParVue"),
                view_classes: bit("ClassesDeVue"),
                stub_widths: serde_json::from_value(g["LargeursBouchon"].clone()).unwrap(),
            },
        }
    }
    #[test]
    fn native_scan_profile_replacement_restoration_and_map_aliases() {
        let oracle: Value = serde_json::from_slice(&inflate(include_bytes!(
            "fixtures/scan-profile-sequence-v41.json.zlib"
        )))
        .unwrap();
        let base = inflate(include_bytes!("fixtures/bootstrap-v41.zlib"));
        let flag = oracle["flag_byte"].as_u64().unwrap() as usize;
        let rows = oracle["rows"].as_array().unwrap();
        assert_eq!(rows.len(), 96);
        for row in rows {
            let i = row["case"].as_u64().unwrap();
            let source = (i % 3 != 0).then(|| {
                let mut b = base.clone();
                if i % 3 == 1 {
                    b[flag] |= 0x80
                } else {
                    b[flag] &= 0x7f
                };
                FilmSource::load(
                    &[b],
                    &[FilmSourceMetadata {
                        index: 0,
                        chunk_type: 1,
                        start_ms: 0,
                    }],
                )
                .unwrap()
            });
            let entry: Option<FilmMapBounds> =
                serde_json::from_value(row["entry"].clone()).unwrap();
            let forced = (!row["forced"].is_null()).then(|| layout(&row["forced"]));
            let mut ctx =
                NativeFilmContext::for_map(source.as_ref(), entry.as_ref(), forced.as_ref())
                    .unwrap();
            let initial = ctx.scan_profile().unwrap();
            assert_eq!(initial, profile(&row["initial"]));
            assert_eq!(
                ctx.corruption_control().unwrap().uses_fallback(),
                row["fallback"].as_bool().unwrap()
            );
            assert_eq!(
                ctx.imposed_layout(),
                (!row["imposed"].is_null()).then(|| layout(&row["imposed"]))
            );
            let replacement = profile(&row["input"]);
            let previous = ctx.set_scan_profile(replacement.clone()).unwrap();
            assert_eq!(previous, profile(&row["previous"]));
            assert_eq!(ctx.scan_profile().unwrap(), profile(&row["installed"]));
            replacement
                .grammar
                .calibrated_widths
                .as_ref()
                .unwrap()
                .insert("probe".into(), 777);
            let mut scalar_copy = ctx.scan_profile().unwrap();
            scalar_copy.mpp.lead = 999;
            assert_eq!(ctx.scan_profile().unwrap(), profile(&row["shared"]));
            assert_ne!(ctx.scan_profile().unwrap().mpp, scalar_copy.mpp);
            let previous_mpp = ctx.set_mpp(FilmMppWidths {
                lead: -7,
                index: 42,
            });
            assert_eq!(
                previous_mpp,
                serde_json::from_value(row["old_mpp"].clone()).unwrap()
            );
            assert_eq!(ctx.scan_profile().unwrap(), profile(&row["after_mpp"]));
            ctx.set_world_precision(descriptor(&row["descriptor"]));
            assert_eq!(ctx.scan_profile().unwrap(), profile(&row["after_raw"]));
            ctx.set_world_precision_from_layout(&layout(&row["layout"]));
            assert_eq!(ctx.scan_profile().unwrap(), profile(&row["after_layout"]));
            ctx.set_scan_profile(initial).unwrap();
            assert_eq!(ctx.scan_profile().unwrap(), profile(&row["restored"]));
            let out = ctx.scan_profile().unwrap();
            let restored: NativeScanProfile =
                serde_json::from_slice(&serde_json::to_vec(&out).unwrap()).unwrap();
            assert_eq!(restored, out);
        }
    }
}
