//! Native external Forge geometry ingestion. This is catalog data, not film bytes.
use super::{
    ReplayMapObject, replay_geometry_csv::read_geometry_csv, replay_geometry_float::geometry_float,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path};

pub const REPLAY_MAP_OBJECTS_FILE: &str = "map_objects.csv";
pub const REPLAY_OBJECT_TYPES_FILE: &str = "forge_object_types.csv";
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReplayGeometryFile {
    Types,
    Map,
}
#[derive(Debug)]
pub enum ReplayGeometryFailure {
    Csv(super::GeometryCsvError),
    Io(std::io::Error),
}
#[derive(Debug)]
pub struct ReplayGeometryError {
    pub file: ReplayGeometryFile,
    pub cause: ReplayGeometryFailure,
}
impl std::fmt::Display for ReplayGeometryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "geometry {:?}: ", self.file)?;
        match &self.cause {
            ReplayGeometryFailure::Csv(e) => e.fmt(f),
            ReplayGeometryFailure::Io(e) => e.fmt(f),
        }
    }
}
impl std::error::Error for ReplayGeometryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(match &self.cause {
            ReplayGeometryFailure::Csv(e) => e,
            ReplayGeometryFailure::Io(e) => e,
        })
    }
}
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct LoadedReplayGeometry {
    pub objects: Vec<ReplayMapObject>,
    pub skipped: usize,
}
impl Serialize for LoadedReplayGeometry {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if self.objects.iter().any(|o| {
            [o.x, o.y, o.z, o.dx, o.dy, o.yaw]
                .iter()
                .any(|v| !v.is_finite())
        }) {
            return Err(serde::ser::Error::custom(
                "nonfinite geometry cannot be published as JSON",
            ));
        }
        #[derive(Serialize)]
        struct Output<'a> {
            objects: &'a [ReplayMapObject],
            skipped: usize,
        }
        Output {
            objects: &self.objects,
            skipped: self.skipped,
        }
        .serialize(serializer)
    }
}
fn type_id(s: &[u8]) -> Option<i64> {
    let rest = if matches!(s.first(), Some(b'+' | b'-')) {
        &s[1..]
    } else {
        s
    };
    if rest.is_empty() || !rest.iter().all(u8::is_ascii_digit) {
        return None;
    }
    std::str::from_utf8(s).ok()?.parse().ok()
}
fn extents(types: &[u8]) -> Result<BTreeMap<i64, [f32; 2]>, ReplayGeometryError> {
    let csv = read_geometry_csv(types).map_err(|e| ReplayGeometryError {
        file: ReplayGeometryFile::Types,
        cause: ReplayGeometryFailure::Csv(e),
    })?;
    let mut out = BTreeMap::new();
    for row in &csv.rows {
        let Some(id) = type_id(csv.field(row, b"type_id")) else {
            continue;
        };
        if csv.field(row, b"geom") != b"ok" {
            continue;
        }
        let dx = geometry_float(csv.field(row, b"dx"));
        let dy = geometry_float(csv.field(row, b"dy"));
        if dx <= 0. || dy <= 0. {
            continue;
        }
        out.insert(id, [dx, dy]);
    }
    Ok(out)
}
fn objects(
    map: Option<&[u8]>,
    sizes: &BTreeMap<i64, [f32; 2]>,
) -> Result<LoadedReplayGeometry, ReplayGeometryError> {
    let Some(map) = map else {
        return Ok(LoadedReplayGeometry::default());
    };
    let csv = read_geometry_csv(map).map_err(|e| ReplayGeometryError {
        file: ReplayGeometryFile::Map,
        cause: ReplayGeometryFailure::Csv(e),
    })?;
    let mut out = LoadedReplayGeometry::default();
    let round = |v: f32| ((f64::from(v) * 100.).round() / 100.) as f32;
    for row in &csv.rows {
        let Some(id) = type_id(csv.field(row, b"type_id")) else {
            continue;
        };
        let Some(size) = sizes.get(&id) else {
            out.skipped += 1;
            continue;
        };
        out.objects.push(ReplayMapObject {
            type_id: id,
            x: round(geometry_float(csv.field(row, b"x"))),
            y: round(geometry_float(csv.field(row, b"y"))),
            z: round(geometry_float(csv.field(row, b"z"))),
            dx: round(size[0]),
            dy: round(size[1]),
            yaw: round(geometry_float(csv.field(row, b"yaw_deg"))),
        });
    }
    Ok(out)
}
/// Parse native CSV assets in memory, preserving order and skipped-type counts.
/// `None` means a missing map asset, accepted only after the type catalog parses.
/// Empty bytes are an empty file and produce a missing-header error. IEEE NaN,
/// infinity and signed zero are retained in memory. Serialization of the returned
/// wrapper rejects nonfinite values, as native JSON does.
pub fn parse_replay_geometry_csv(
    map: Option<&[u8]>,
    types: &[u8],
) -> Result<LoadedReplayGeometry, ReplayGeometryError> {
    objects(map, &extents(types)?)
}
/// Native LoadGeometry: read the title catalog first, then the map-specific file.
/// Missing map assets are nominal; missing type catalogs and other I/O errors fail.
pub fn load_replay_geometry(
    map_dir: impl AsRef<Path>,
    types_dir: impl AsRef<Path>,
) -> Result<LoadedReplayGeometry, ReplayGeometryError> {
    let io_error = |file, e| ReplayGeometryError {
        file,
        cause: ReplayGeometryFailure::Io(e),
    };
    let types = std::fs::read(types_dir.as_ref().join(REPLAY_OBJECT_TYPES_FILE))
        .map_err(|e| io_error(ReplayGeometryFile::Types, e))?;
    let sizes = extents(&types)?;
    match std::fs::read(map_dir.as_ref().join(REPLAY_MAP_OBJECTS_FILE)) {
        Ok(map) => objects(Some(&map), &sizes),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => objects(None, &sizes),
        Err(e) => Err(io_error(ReplayGeometryFile::Map, e)),
    }
}
