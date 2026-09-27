//! Pinned external named-zone catalogs, including original unclipped polygons.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const MAP_CALLOUTS_SCHEMA_VERSION: i64 = 1;
pub const CALLOUTS_PROVENANCE_RAW: &str = "brut";
pub const CALLOUTS_PROVENANCE_CLIPPED: &str = "decoupe";
pub const CALLOUTS_PROVENANCE_MVAR: &str = "mvar";
fn empty<T>(v: &Option<Vec<T>>) -> bool {
    v.as_ref().is_none_or(Vec::is_empty)
}
fn empty_map<T>(v: &Option<BTreeMap<String, T>>) -> bool {
    v.as_ref().is_none_or(BTreeMap::is_empty)
}
fn no(v: &bool) -> bool {
    !*v
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CalloutZone {
    pub volume_index: i64,
    pub name: String,
    pub en: String,
    pub fr: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub z_bottom: f64,
    pub z_top: f64,
    #[serde(skip_serializing_if = "no")]
    pub big: bool,
    #[serde(skip_serializing_if = "empty")]
    pub polygon: Option<Vec<[f64; 2]>>,
    #[serde(skip_serializing_if = "empty")]
    pub parts: Option<Vec<Option<Vec<[f64; 2]>>>>,
    #[serde(skip_serializing_if = "empty")]
    pub holes: Option<Vec<Option<Vec<[f64; 2]>>>>,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CalloutRawZone {
    pub volume_index: i64,
    pub polygon: Option<Vec<[f64; 2]>>,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MapCalloutsEntry {
    pub module: String,
    pub provenance: String,
    pub zones: Option<Vec<CalloutZone>>,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MapCalloutsCatalog {
    pub schema_version: i64,
    pub title_slug: String,
    pub source: String,
    pub maps: Option<BTreeMap<String, MapCalloutsEntry>>,
    #[serde(skip_serializing_if = "empty_map")]
    pub brut: Option<BTreeMap<String, Option<Vec<CalloutRawZone>>>>,
    #[serde(skip_serializing_if = "empty_map")]
    pub maps_by_id: Option<BTreeMap<String, MapCalloutsEntry>>,
}
#[derive(Debug, thiserror::Error)]
pub enum CalloutsCatalogError {
    #[error("invalid callout catalog JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("callout catalog schema {0}; expected 1")]
    Schema(i64),
    #[error("callout catalog I/O: {0}")]
    Io(#[from] std::io::Error),
    #[error("unknown callout map: {0}")]
    UnknownMap(String),
}
impl CalloutsCatalogError {
    pub fn is_unknown_map(&self) -> bool {
        matches!(self, Self::UnknownMap(_))
    }
}
impl MapCalloutsCatalog {
    /// Exact module identity; native lookup does not trim or normalize case.
    pub fn lookup(&self, module: &str) -> Result<&MapCalloutsEntry, CalloutsCatalogError> {
        lookup_callouts(Some(self), module, false)
    }
    /// Forge asset identities occupy a separate namespace from native modules.
    pub fn lookup_by_id(&self, id: &str) -> Result<&MapCalloutsEntry, CalloutsCatalogError> {
        lookup_callouts(Some(self), id, true)
    }
}
/// Also supports the native nil receiver. Empty names never match an entry.
pub fn lookup_callouts<'a>(
    catalog: Option<&'a MapCalloutsCatalog>,
    key: &str,
    by_id: bool,
) -> Result<&'a MapCalloutsEntry, CalloutsCatalogError> {
    if !key.is_empty()
        && let Some(catalog) = catalog
    {
        let maps = if by_id {
            &catalog.maps_by_id
        } else {
            &catalog.maps
        };
        if let Some(entry) = maps.as_ref().and_then(|maps| maps.get(key)) {
            return Ok(entry);
        }
    }
    Err(CalloutsCatalogError::UnknownMap(key.into()))
}
/// Native file decoding semantics, including repeated fields, null and reused
/// arrays. Ordinary Deserialize on the public DTOs is for canonical exports.
pub fn parse_map_callouts(bytes: &[u8]) -> Result<MapCalloutsCatalog, CalloutsCatalogError> {
    let value = super::callouts_json::decode(bytes)?;
    let catalog: MapCalloutsCatalog = serde_json::from_value(value)?;
    if catalog.schema_version != MAP_CALLOUTS_SCHEMA_VERSION {
        return Err(CalloutsCatalogError::Schema(catalog.schema_version));
    }
    Ok(catalog)
}
#[cfg(not(target_arch = "wasm32"))]
pub fn load_map_callouts(
    path: impl AsRef<std::path::Path>,
) -> Result<MapCalloutsCatalog, CalloutsCatalogError> {
    parse_map_callouts(&std::fs::read(path)?)
}
