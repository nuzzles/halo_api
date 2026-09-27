//! Native background sidecar metadata and loading. External images remain external inputs.
use super::callouts_json::Shape;
use super::{MapBackgroundCalibration, MapBackgroundTime};
use serde::{Deserialize, Serialize};
pub const MAP_BACKGROUND_SCHEMA_VERSION: i64 = 1;
fn zero(v: &i64) -> bool {
    *v == 0
}
fn empty(v: &Option<Vec<String>>) -> bool {
    v.as_ref().is_none_or(Vec::is_empty)
}
fn finite<S: serde::Serializer>(v: &f64, s: S) -> Result<S::Ok, S::Error> {
    if !v.is_finite() {
        return Err(serde::ser::Error::custom("nonfinite background statistic"));
    }
    s.serialize_f64(*v)
}
fn optional_finite<S: serde::Serializer>(v: &Option<f64>, s: S) -> Result<S::Ok, S::Error> {
    match v {
        Some(v) => finite(v, s),
        None => s.serialize_none(),
    }
}
static STRINGS: Shape = Shape::Slice(&Shape::Text);
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MapBackgroundStats {
    #[serde(rename = "anchors")]
    pub anchors: i64,
    #[serde(rename = "anchorsInFrame")]
    pub anchors_in_frame: i64,
    #[serde(rename = "anchorsWithGround")]
    pub anchors_with_ground: i64,
    #[serde(
        rename = "anchorMedianGapM",
        skip_serializing_if = "Option::is_none",
        serialize_with = "optional_finite"
    )]
    pub anchor_median_gap_m: Option<f64>,
    #[serde(rename = "instancesDrawn")]
    pub instances_drawn: i64,
    #[serde(rename = "instancesScenery")]
    pub instances_scenery: i64,
    #[serde(rename = "playLevelZ", serialize_with = "finite")]
    pub play_level_z: f64,
    #[serde(rename = "boundaryApplied")]
    pub boundary_applied: bool,
    #[serde(rename = "boundaryPlanes")]
    pub boundary_planes: i64,
    #[serde(rename = "boundaryCellsCleared")]
    pub boundary_cells_cleared: i64,
    #[serde(rename = "waterVolumes")]
    pub water_volumes: i64,
    #[serde(rename = "waterCells")]
    pub water_cells: i64,
    #[serde(rename = "coveredShare", serialize_with = "finite")]
    pub covered_share: f64,
    #[serde(rename = "covered")]
    pub covered: bool,
    #[serde(rename = "cellsSubstituted", skip_serializing_if = "zero")]
    pub cells_substituted: i64,
    #[serde(rename = "cellsClipped", skip_serializing_if = "zero")]
    pub cells_clipped: i64,
    #[serde(rename = "cellsAssumedFloor", skip_serializing_if = "zero")]
    pub cells_assumed_floor: i64,
    #[serde(rename = "forgeObjects", skip_serializing_if = "zero")]
    pub forge_objects: i64,
    #[serde(rename = "forgeObjectsDrawn", skip_serializing_if = "zero")]
    pub forge_objects_drawn: i64,
    #[serde(rename = "forgeObjectsWithoutModel", skip_serializing_if = "zero")]
    pub forge_objects_without_model: i64,
    #[serde(rename = "forgeDeathVolumes", skip_serializing_if = "zero")]
    pub forge_death_volumes: i64,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MapBackground {
    #[serde(rename = "schemaVersion")]
    pub schema_version: i64,
    #[serde(rename = "module")]
    pub module: String,
    #[serde(rename = "mapNames", skip_serializing_if = "empty")]
    pub map_names: Option<Vec<String>>,
    #[serde(rename = "image")]
    pub image: String,
    #[serde(rename = "source")]
    pub source: String,
    #[serde(rename = "generatedAt")]
    pub generated_at: MapBackgroundTime,
    #[serde(rename = "style")]
    pub style: String,
    #[serde(rename = "calibration")]
    pub calibration: MapBackgroundCalibration,
    #[serde(rename = "stats")]
    pub stats: MapBackgroundStats,
    #[serde(rename = "degradations", skip_serializing_if = "empty")]
    pub degradations: Option<Vec<String>>,
}
static STATS: Shape = Shape::Struct(&[
    ("anchors", &Shape::Int),
    ("anchorsInFrame", &Shape::Int),
    ("anchorsWithGround", &Shape::Int),
    ("anchorMedianGapM", &Shape::NullableFloat),
    ("instancesDrawn", &Shape::Int),
    ("instancesScenery", &Shape::Int),
    ("playLevelZ", &Shape::Float),
    ("boundaryApplied", &Shape::Bool),
    ("boundaryPlanes", &Shape::Int),
    ("boundaryCellsCleared", &Shape::Int),
    ("waterVolumes", &Shape::Int),
    ("waterCells", &Shape::Int),
    ("coveredShare", &Shape::Float),
    ("covered", &Shape::Bool),
    ("cellsSubstituted", &Shape::Int),
    ("cellsClipped", &Shape::Int),
    ("cellsAssumedFloor", &Shape::Int),
    ("forgeObjects", &Shape::Int),
    ("forgeObjectsDrawn", &Shape::Int),
    ("forgeObjectsWithoutModel", &Shape::Int),
    ("forgeDeathVolumes", &Shape::Int),
]);
static CALIBRATION: Shape = Shape::Struct(&[
    ("metersPerPixel", &Shape::Float),
    ("originX", &Shape::Float),
    ("originY", &Shape::Float),
    ("widthPx", &Shape::Int),
    ("heightPx", &Shape::Int),
    ("convention", &Shape::Text),
]);
static BACKGROUND: Shape = Shape::Struct(&[
    ("schemaVersion", &Shape::Int),
    ("module", &Shape::Text),
    ("mapNames", &STRINGS),
    ("image", &Shape::Text),
    ("source", &Shape::Text),
    ("generatedAt", &Shape::Timestamp),
    ("style", &Shape::Text),
    ("calibration", &CALIBRATION),
    ("stats", &STATS),
    ("degradations", &STRINGS),
]);

#[derive(Debug, thiserror::Error)]
pub enum MapBackgroundError {
    #[error("invalid map background JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("map background schema {0}; expected 1")]
    Schema(i64),
    #[error("map background I/O: {0}")]
    Io(#[from] std::io::Error),
}
/// Native LoadMapBackground decoding; ordinary DTO Deserialize is for canonical
/// JSON and internal timestamp fields, not arbitrary native JSON field updates.
pub fn parse_map_background(bytes: &[u8]) -> Result<MapBackground, MapBackgroundError> {
    let data = super::callouts_json::decode_with(bytes, &BACKGROUND)?;
    let background: MapBackground = serde_json::from_value(data)?;
    if background.schema_version != MAP_BACKGROUND_SCHEMA_VERSION {
        return Err(MapBackgroundError::Schema(background.schema_version));
    }
    Ok(background)
}
#[cfg(not(target_arch = "wasm32"))]
pub fn load_map_background(
    path: impl AsRef<std::path::Path>,
) -> Result<MapBackground, MapBackgroundError> {
    parse_map_background(&std::fs::read(path)?)
}
