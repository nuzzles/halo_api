//! v41 profile resolution from recorded version/build and the pinned map catalog.
use super::{
    CoordinateBounds, DecodeError, FilmIdentity, FilmRegistry, FrameEncoding, PositionEncoding,
    RecordIdLayout,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilmMapBounds {
    pub module: String,
    pub min: [f32; 3],
    pub max: [f32; 3],
    #[serde(rename = "axisWidths")]
    pub axis_widths: [usize; 3],
    #[serde(default)]
    pub region: u32,
    #[serde(default, rename = "regionIndexBits")]
    pub region_index_bits: usize,
}

impl FilmMapBounds {
    /// Native catalog default: an omitted or zero region width means one bit.
    pub fn effective_region_index_bits(&self) -> usize {
        self.region_index_bits.max(1)
    }

    /// Project catalog metadata without validating it for a particular reader.
    pub fn absolute_precision(&self) -> super::WorldObjectPrecision {
        super::WorldObjectPrecision {
            index_bits: self.effective_region_index_bits(),
            axis_bits: self.axis_widths,
            region: self.region,
        }
    }

    /// Native catalog-imposed i0 layout. This preserves invalid axis widths;
    /// callers decide whether to accept it through `I0Layout::valid`.
    pub fn i0_layout(&self) -> super::I0Layout {
        let precision = self.absolute_precision();
        super::I0Layout {
            gate_bits: super::I0_SPINE_BITS
                .wrapping_add(super::I0_USE_DEFAULT_BITS)
                .wrapping_add(precision.index_bits as i64),
            axis_widths: precision.axis_bits.map(|width| width as u64),
            region: precision.region,
        }
    }

    /// Project the recorded catalog endpoints, preserving their float bits.
    /// This does not infer bounds, sort endpoints, or validate map geometry.
    pub fn quantization_range(&self) -> super::FilmQuantizationRange {
        std::array::from_fn(|axis| [self.min[axis], self.max[axis]])
    }

    pub fn coordinate_bounds(&self) -> CoordinateBounds {
        CoordinateBounds {
            min: self.min,
            max: self.max,
        }
    }
    pub fn position_encoding(&self) -> PositionEncoding {
        PositionEncoding {
            bodies: Default::default(),
            index_bits: self.effective_region_index_bits(),
            world_axis_bits: Some(self.axis_widths),
            handle_bits: 1,
            traversal_axis_bits: [6; 3],
            default_axis_bits: [22; 3],
            region_axis_bits: [(self.region, self.axis_widths)].into(),
            delta_axis_bits: [14; 3],
            calibrated_skip: false,
            baseline_scope: false,
            full_precision: false,
            writer_absolute: false,
            delta_handle_tail: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilmMapCatalog {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    pub source: String,
    pub maps: BTreeMap<String, FilmMapBounds>,
}

/// Normalize the map display name with the reference's variant suffix rules.
pub fn normalize_film_map_name(name: &str) -> String {
    // Go strings.ToLower applies simple per-rune mappings. String::to_lowercase
    // adds contextual final sigma and expands dotted I, changing native keys.
    let mut name: String = name
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .map(|c| c.to_lowercase().next().unwrap_or(c))
        .collect();
    for suffix in [" - ranked", " heavies"] {
        if let Some(trimmed) = name.strip_suffix(suffix) {
            name = trimmed.to_owned();
        }
    }
    name
}

pub fn film_map_catalog() -> FilmMapCatalog {
    serde_json::from_str(include_str!("reference/map_quant_bounds.json"))
        .expect("pinned map catalog is validated")
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilmProfileIssue {
    UnknownFormat(u32),
    UnknownBuild(String),
    MissingIdentity,
    UnknownMap(String),
    MissingMap,
}

/// Effective component corruption checks and whether identification declared them.
/// Standalone profile composition preserves the inherited value when identification
/// is absent. LegacyFilm contexts start from the invariant false and retain that decision
/// across later custom profile replacements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilmCorruptionControl {
    pub enabled: bool,
    pub declared: bool,
}

impl FilmCorruptionControl {
    pub fn from_recorded(recorded: Option<bool>, inherited: bool) -> Self {
        Self {
            enabled: recorded.unwrap_or(inherited),
            declared: recorded.is_some(),
        }
    }

    pub fn uses_fallback(self) -> bool {
        !self.declared
    }

    /// Reapply the film's decision after a custom profile replaces the encoding.
    pub fn apply(self, encoding: &mut FrameEncoding) {
        encoding.corruption_check = self.enabled;
    }
}

impl super::LegacyFilm {
    /// FilmContext corruption policy, including the named missing-identity fallback.
    pub fn corruption_control(&self) -> FilmCorruptionControl {
        FilmCorruptionControl::from_recorded(
            self.identity.as_ref().map(|i| i.corruption_checks),
            false,
        )
    }
}

/// Native scan invariant used when no declared/calibrated MPP layout is installed.
pub const NATIVE_MPP_DEFAULT_WIDTHS: [usize; 2] = [9, 5];

/// Format membership and declared MPP widths are separate native facts. Historical
/// formats remain known metadata even though their MPP widths are undetermined;
/// this does not enable decoding any major version other than 41.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilmMppResolution {
    pub format_version: u32,
    pub widths: Option<[usize; 2]>,
    pub unknown_format: bool,
}

pub fn resolve_film_mpp(format_version: u32) -> FilmMppResolution {
    FilmMppResolution {
        format_version,
        widths: (format_version == 27).then_some([9, 5]),
        unknown_format: !matches!(format_version, 20 | 21 | 24 | 25 | 27),
    }
}

impl FilmMppResolution {
    /// Equivalent to native width selection followed by conditional installation.
    /// Invalid or absent calibration leaves the caller's scan profile untouched.
    pub fn creation_widths(
        self,
        calibrated: Option<[usize; 2]>,
        inherited: [usize; 2],
    ) -> [usize; 2] {
        self.widths
            .or(calibrated.filter(|w| w.iter().all(|&v| v > 0)))
            .unwrap_or(inherited)
    }
}

/// Native `UnknownBuildExpvarPairs` publication for a rejected build.
///
/// Callers publish this once per rejection; this helper does not maintain global
/// counters. Each non-ASCII scalar becomes one underscore (not one per byte).
/// An absent identification section uses the empty build name.
pub fn unknown_build_metric_pairs(build: &str) -> [(String, i64); 1] {
    let suffix = if build.is_empty() {
        "sans_section".to_owned()
    } else {
        build
            .chars()
            .map(|c| match c {
                'a'..='z' | '0'..='9' | '_' => c,
                'A'..='Z' => c.to_ascii_lowercase(),
                _ => '_',
            })
            .collect()
    };
    [(format!("filmdec_unknown_build_{suffix}"), 1)]
}

/// Native unknown-format counter; zero represents a missing format header.
/// Signed values preserve the reference helper's integer input contract.
pub fn unknown_format_metric_pairs(format: i64) -> [(String, i64); 1] {
    [(format!("filmdec_unknown_format_{format}"), 1)]
}

/// Partial resolution keeps unavailable context explicit; no nearest-build or
/// other-map fallback is used. Record-ID widths still require film evidence.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct V41FilmProfile {
    pub build: Option<String>,
    pub format_version: u32,
    pub mpp_widths: Option<[usize; 2]>,
    pub personalization_bytes: Option<usize>,
    pub personalization_delta_bits: Option<i32>,
    pub corruption_checks: Option<bool>,
    pub map_name: Option<String>,
    pub map: Option<FilmMapBounds>,
    pub delta_quantum: f32,
    pub issues: Vec<FilmProfileIssue>,
}

pub fn resolve_v41_profile(
    registry: &FilmRegistry,
    identity: Option<&FilmIdentity>,
    map_name: Option<&str>,
) -> Result<V41FilmProfile, DecodeError> {
    if registry.major_version != 41 {
        return Err(DecodeError::UnsupportedVersion(
            registry.major_version as i32,
        ));
    }
    let mut issues = Vec::new();
    let mpp = resolve_film_mpp(registry.format_version);
    if mpp.unknown_format {
        issues.push(FilmProfileIssue::UnknownFormat(registry.format_version));
    }
    let mpp_widths = mpp.widths;
    let personalization_bytes = match identity.map(|i| i.build.as_str()) {
        Some("HI_1_13_0" | "HI_1_12_0") => Some(1852),
        Some(build) => {
            issues.push(FilmProfileIssue::UnknownBuild(build.into()));
            None
        }
        None => {
            issues.push(FilmProfileIssue::MissingIdentity);
            None
        }
    };
    let map_name = map_name.map(normalize_film_map_name);
    let map = map_name
        .as_ref()
        .and_then(|name| film_map_catalog().maps.remove(name));
    if map.is_none() {
        issues.push(
            map_name
                .as_ref()
                .map_or(FilmProfileIssue::MissingMap, |name| {
                    FilmProfileIssue::UnknownMap(name.clone())
                }),
        );
    }
    Ok(V41FilmProfile {
        build: identity.map(|i| i.build.clone()),
        format_version: registry.format_version,
        mpp_widths,
        personalization_bytes,
        personalization_delta_bits: personalization_bytes.map(|n| (n as i32 - 1852) * 8),
        corruption_checks: identity.map(|i| i.corruption_checks),
        map_name,
        map,
        delta_quantum: super::NATIVE_DELTA_QUANTUM,
        issues,
    })
}

impl V41FilmProfile {
    /// Native map-context warning for unavailable recorded profile keys. Missing
    /// map bounds alone do not trigger it; a source without a registry is silent.
    pub fn warn_incomplete(&self, registry_present: bool) {
        if !registry_present {
            return;
        }
        let mut errors = Vec::new();
        if self
            .issues
            .iter()
            .any(|i| matches!(i, FilmProfileIssue::UnknownFormat(_)))
        {
            errors.push(format!(
                "filmdec: version de format de chunk_00 absente de la table de profil : {}",
                self.format_version
            ));
        }
        if self.issues.iter().any(|i| {
            matches!(
                i,
                FilmProfileIssue::UnknownBuild(_) | FilmProfileIssue::MissingIdentity
            )
        }) {
            // Recorded build keys use the identification grammar's ASCII token.
            errors.push(format!(
                "filmdec: build absent de la table de profil : {:?}",
                self.build.as_deref().unwrap_or_default()
            ));
        }
        if errors.is_empty() {
            return;
        }
        tracing::warn!(
            err = errors.join("\n"),
            format = self.format_version,
            build = self.build.as_deref().unwrap_or_default(),
            "profil du film INCOMPLET — une cle ecrite dans le film est absente de la table de profil ; le film reste decode, les replis existants decident et se comptent"
        );
    }
    /// Compose widths for an independently established ID layout and map.
    /// Unresolved format/identification retains native scan invariants, with the
    /// unavailable declarations and their diagnostics preserved on this profile.
    /// Creation scans may subsequently install widths calibrated from the film.
    pub fn frame_encoding(&self, ids: RecordIdLayout) -> Option<FrameEncoding> {
        let mut capture =
            super::PositionCaptureEncoding::new(self.map.as_ref()?, self.delta_quantum);
        // Native map installation changes only the world-object descriptor.
        // The observer retains Movement.Range = QuantRangeCEBiped; dedicated
        // map-position scans receive the actual map bounds separately.
        capture.min_bits = super::NATIVE_QUANT_RANGE_CE_BIPED.map(|axis| axis[0].to_bits());
        capture.max_bits = super::NATIVE_QUANT_RANGE_CE_BIPED.map(|axis| axis[1].to_bits());
        capture.region_index_bits = capture.region_index_bits.max(1);
        let encoding = FrameEncoding {
            keyframe_layout: Default::default(),
            keyframe_simulation_complete: Some(true),
            native_id_low_bits: None,
            component_widths: Default::default(),
            new_record: Default::default(),
            position_capture: Some(capture),
            ids,
            mpp_widths: self.mpp_widths.unwrap_or(NATIVE_MPP_DEFAULT_WIDTHS),
            position: Some(self.map.as_ref()?.position_encoding()),
            extra_fields: false,
            corruption_check: self.corruption_checks.unwrap_or(false),
        };
        encoding.valid().then_some(encoding)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_map_entry_projections() {
        use std::io::Read;
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/map-entry-v41.json.zlib")[..])
            .read_to_end(&mut raw)
            .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(rows.len(), 256);
        let mut invalid = 0;
        let mut historical_default = 0;
        for (i, row) in rows.into_iter().enumerate() {
            let entry: FilmMapBounds = serde_json::from_value(row["entry"].clone()).unwrap();
            assert_eq!(
                serde_json::json!(entry.effective_region_index_bits()),
                row["index"]
            );
            let precision = entry.absolute_precision();
            assert_eq!(
                serde_json::to_value(precision).unwrap(),
                row["precision"],
                "case {i}"
            );
            assert_eq!(
                super::super::WorldObjectPrecision::from_map(&entry),
                precision
            );
            let layout = entry.i0_layout();
            assert_eq!(
                serde_json::to_value(&layout).unwrap(),
                row["layout"],
                "case {i}"
            );
            assert_eq!(serde_json::json!(layout.valid()), row["valid"]);
            let range = entry
                .quantization_range()
                .map(|axis| axis.map(f32::to_bits));
            assert_eq!(
                serde_json::json!(range),
                row["range_bits"],
                "range bits {i}"
            );
            let encoding = entry.position_encoding();
            assert_eq!(encoding.index_bits, precision.index_bits);
            assert_eq!(encoding.world_axis_bits, Some(precision.axis_bits));
            assert_eq!(
                encoding.region_axis_bits[&precision.region],
                precision.axis_bits
            );
            invalid += usize::from(!layout.valid());
            historical_default += usize::from(entry.region_index_bits == 0);
        }
        assert!(invalid > 0 && invalid < 256 && historical_default > 0);
    }

    #[test]
    fn native_map_name_normalization() {
        use std::io::Read;
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/map-names-v41.json.zlib")[..])
            .read_to_end(&mut raw)
            .unwrap();
        let cases: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        assert!(cases.len() > 3000);
        for case in cases {
            let input = case["input"].as_str().unwrap();
            assert_eq!(
                normalize_film_map_name(input),
                case["output"].as_str().unwrap(),
                "{input:?}"
            );
        }
    }

    #[test]
    fn native_mpp_resolution_and_creation_installation() {
        use std::io::Read;
        #[derive(Deserialize)]
        struct Case {
            format: u32,
            calibrated: [usize; 2],
            inherited: [usize; 2],
            widths: [usize; 2],
            read: bool,
            unknown: bool,
            effective: [usize; 2],
        }
        let mut bytes = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/mpp-resolution-v41.json.zlib")[..],
        )
        .read_to_end(&mut bytes)
        .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(cases.len(), 512);
        let (mut calibrated, mut inherited, mut known_without_widths) = (0, 0, 0);
        for (i, c) in cases.into_iter().enumerate() {
            let result = resolve_film_mpp(c.format);
            assert_eq!(result.widths.is_some(), c.read, "read {i}");
            assert_eq!(result.widths.unwrap_or([0, 0]), c.widths, "widths {i}");
            assert_eq!(result.unknown_format, c.unknown, "membership {i}");
            assert_eq!(
                result.creation_widths(Some(c.calibrated), c.inherited),
                c.effective,
                "effective {i}"
            );
            assert_eq!(
                result.creation_widths(None, c.inherited),
                result.widths.unwrap_or(c.inherited)
            );
            assert_eq!(
                serde_json::from_value::<FilmMppResolution>(serde_json::to_value(result).unwrap())
                    .unwrap(),
                result
            );
            calibrated += usize::from(
                !c.read && c.calibrated.iter().all(|&w| w > 0) && c.effective != c.inherited,
            );
            inherited += usize::from(!c.read && c.calibrated.contains(&0));
            known_without_widths += usize::from(!c.read && !c.unknown);
            let registry = FilmRegistry {
                archetypes: Vec::new(),
                major_version: 41,
                format_version: c.format,
                end_byte: 0,
                truncated: false,
            };
            let profile = resolve_v41_profile(&registry, None, Some("bazaar")).unwrap();
            assert_eq!(
                profile
                    .issues
                    .contains(&FilmProfileIssue::UnknownFormat(c.format)),
                c.unknown
            );
            assert_eq!(
                profile
                    .frame_encoding(RecordIdLayout {
                        low_bits: 13,
                        base: 0
                    })
                    .unwrap()
                    .mpp_widths,
                [9, 5]
            );
        }
        assert_eq!(
            (calibrated, inherited, known_without_widths),
            (154, 217, 256)
        );
    }

    #[test]
    fn native_corruption_context_composition() {
        use std::io::Read;
        #[derive(Deserialize)]
        struct Case {
            read: bool,
            declared: bool,
            inherited: bool,
            direct: bool,
            direct_read: bool,
            states: Vec<bool>,
            previous: Vec<bool>,
            fallback: bool,
        }
        let mut bytes = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/corruption-context-v41.json.zlib")[..],
        )
        .read_to_end(&mut bytes)
        .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(cases.len(), 256);
        let mut differing_fallbacks = 0;
        for (i, c) in cases.into_iter().enumerate() {
            let recorded = c.read.then_some(c.declared);
            let direct = FilmCorruptionControl::from_recorded(recorded, c.inherited);
            assert_eq!(
                (direct.enabled, direct.declared),
                (c.direct, c.direct_read),
                "direct {i}"
            );
            let context = FilmCorruptionControl::from_recorded(recorded, false);
            assert_eq!(context.uses_fallback(), c.fallback);
            let mut encoding = FrameEncoding {
                keyframe_layout: Default::default(),
                keyframe_simulation_complete: None,
                native_id_low_bits: None,
                component_widths: Default::default(),
                new_record: Default::default(),
                position_capture: None,
                ids: RecordIdLayout {
                    low_bits: 11,
                    base: 0,
                },
                mpp_widths: [9, 5],
                position: None,
                extra_fields: false,
                corruption_check: context.enabled,
            };
            for j in 0..4 {
                assert_eq!(encoding.corruption_check, c.previous[j], "previous {i}/{j}");
                encoding.corruption_check = (i + j) % 2 == 0;
                context.apply(&mut encoding);
                assert_eq!(encoding.corruption_check, c.states[j], "state {i}/{j}");
            }
            differing_fallbacks += usize::from(!c.read && direct.enabled != context.enabled);
            assert_eq!(
                serde_json::from_value::<FilmCorruptionControl>(
                    serde_json::to_value(context).unwrap()
                )
                .unwrap(),
                context
            );
        }
        assert_eq!(differing_fallbacks, 18);
    }

    #[test]
    fn unknown_build_publication_matches_native() {
        use std::io::Read;
        let mut bytes = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/unknown-build-v41.json.zlib")[..])
            .read_to_end(&mut bytes)
            .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(rows.len(), 1166);
        for row in rows {
            let pairs = if let Some(build) = row["build"].as_str() {
                unknown_build_metric_pairs(build)
            } else {
                unknown_format_metric_pairs(row["format"].as_i64().unwrap())
            };
            assert_eq!(serde_json::json!(pairs), row["pairs"]);
            let encoded = serde_json::to_string(&pairs).unwrap();
            assert_eq!(
                serde_json::from_str::<[(String, i64); 1]>(&encoded).unwrap(),
                pairs
            );
        }
    }
    #[test]
    fn native_incomplete_profile_warning() {
        use std::io::Read;
        let mut bootstrap = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/bootstrap-v41.zlib")[..])
            .read_to_end(&mut bootstrap)
            .unwrap();
        let mut registry = crate::theater::parse_registry(&bootstrap).unwrap();
        let mut identity = crate::theater::decode_film_identity(&bootstrap, &registry)
            .unwrap()
            .unwrap();
        let rows: Vec<serde_json::Value> =
            serde_json::from_str(include_str!("fixtures/profile-warning-v41.json")).unwrap();
        assert_eq!(rows.len(), 128);
        for (i, row) in rows.iter().enumerate() {
            registry.format_version = row["format"].as_u64().unwrap() as u32;
            identity.build = row["build"].as_str().unwrap().into();
            let profile = resolve_v41_profile(
                &registry,
                (!identity.build.is_empty()).then_some(&identity),
                None,
            )
            .unwrap();
            let logs = crate::theater::log_test_support::capture_logs(|| {
                profile.warn_incomplete(row["present"].as_bool().unwrap())
            });
            assert_eq!(
                serde_json::to_value(logs).unwrap(),
                row["logs"],
                "profile warning {i}"
            );
        }
    }
    #[test]
    fn resolved_profiles_match_go_reference() {
        use std::io::Read;
        let mut bootstrap = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/bootstrap-v41.zlib")[..])
            .read_to_end(&mut bootstrap)
            .unwrap();
        let registry = crate::theater::parse_registry(&bootstrap).unwrap();
        let mut identity = crate::theater::decode_film_identity(&bootstrap, &registry)
            .unwrap()
            .unwrap();
        let rows: Vec<serde_json::Value> =
            serde_json::from_str(include_str!("fixtures/profile-levelup-v41.json")).unwrap();
        assert_eq!(rows.len(), 237);
        for row in rows {
            identity.build = row["build"].as_str().unwrap().into();
            let present = row["identity_read"].as_bool().unwrap();
            let profile =
                resolve_v41_profile(&registry, present.then_some(&identity), row["map"].as_str())
                    .unwrap();
            assert_eq!(
                profile.issues,
                if present {
                    Vec::new()
                } else {
                    vec![FilmProfileIssue::MissingIdentity]
                }
            );
            assert_eq!(
                profile.personalization_bytes.is_some(),
                row["personalization_known"].as_bool().unwrap()
            );
            assert_eq!(profile.corruption_checks.is_some(), present);
            let encoding = profile
                .frame_encoding(RecordIdLayout {
                    low_bits: 13,
                    base: 0,
                })
                .unwrap();
            let position = encoding.position.unwrap();
            assert_eq!(
                serde_json::to_value(&encoding.position_capture).unwrap(),
                row["capture"],
                "native position capture context for {} / {}",
                row["map"],
                row["build"]
            );
            let region = row["region"].as_u64().unwrap() as u32;
            assert_eq!(serde_json::json!(encoding.mpp_widths), row["mpp"]);
            assert_eq!(
                serde_json::json!(profile.personalization_bytes.unwrap_or(0)),
                row["personalization"]
            );
            assert_eq!(
                serde_json::json!(profile.personalization_delta_bits.unwrap_or(0)),
                row["personalization_delta"]
            );
            assert_eq!(
                position.index_bits as u64,
                row["index_bits"].as_u64().unwrap()
            );
            assert_eq!(
                serde_json::json!(position.region_axis_bits[&region]),
                row["axes"]
            );
            assert_eq!(
                serde_json::json!(position.traversal_axis_bits),
                row["traversal_axes"]
            );
            assert_eq!(
                position.handle_bits as u64,
                row["handle_bits"].as_u64().unwrap()
            );
            assert_eq!(
                position.delta_axis_bits,
                [row["delta_axis"].as_u64().unwrap() as usize; 3]
            );
            assert_eq!(
                profile.delta_quantum,
                row["delta_quantum"].as_f64().unwrap() as f32
            );
            assert_eq!(
                position.full_precision,
                row["full_precision"].as_bool().unwrap()
            );
            assert_eq!(
                position.delta_handle_tail,
                row["delta_handle_tail"].as_bool().unwrap()
            );
        }
    }
    #[test]
    fn catalog_widths_agree_with_map_bounds() {
        let catalog = film_map_catalog();
        assert_eq!(catalog.schema_version, 1);
        assert_eq!(catalog.maps.len(), 79);
        for (name, entry) in catalog.maps {
            assert_eq!(
                entry.coordinate_bounds().axis_bits(),
                Some(entry.axis_widths),
                "{name}"
            );
            assert_eq!(
                super::super::position_axis_widths(
                    std::array::from_fn(|axis| [entry.min[axis], entry.max[axis]]),
                    super::super::OBJECT_POSITION_PRECISION_LEVEL,
                )
                .map(|w| w as usize),
                entry.axis_widths,
                "native precision law {name}",
            );
            assert!(entry.position_encoding().index_bits >= 1);
        }
        assert_eq!(
            normalize_film_map_name("  Live   FIRE - Ranked "),
            "live fire"
        );
        assert_eq!(
            normalize_film_map_name("Fragmentation Heavies"),
            "fragmentation"
        );
    }
    #[test]
    fn unknown_context_is_not_replaced_by_a_nearby_profile() {
        let registry = FilmRegistry {
            archetypes: Vec::new(),
            major_version: 41,
            format_version: 28,
            end_byte: 0,
            truncated: false,
        };
        let profile = resolve_v41_profile(&registry, None, Some("unknown-map")).unwrap();
        assert_eq!(
            profile.issues,
            vec![
                FilmProfileIssue::UnknownFormat(28),
                FilmProfileIssue::MissingIdentity,
                FilmProfileIssue::UnknownMap("unknown-map".into())
            ]
        );
        assert!(
            profile
                .frame_encoding(RecordIdLayout {
                    low_bits: 13,
                    base: 0
                })
                .is_none()
        );
    }
}
