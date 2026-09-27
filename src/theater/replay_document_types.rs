//! Native supporting document types for optional map and presentation data.
use super::*;
use serde::{Deserialize, Serialize};
fn zero_f(v: &f32) -> bool {
    *v == 0.
}
fn false_b(v: &bool) -> bool {
    !*v
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ReplaySurface {
    #[serde(rename = "x0")]
    pub x0: f32,
    #[serde(rename = "y0")]
    pub y0: f32,
    #[serde(rename = "x1")]
    pub x1: f32,
    #[serde(rename = "y1")]
    pub y1: f32,
    #[serde(rename = "z")]
    pub z: f32,
    #[serde(rename = "zb")]
    pub zb: f32,
    #[serde(rename = "poly", skip_serializing_if = "Vec::is_empty")]
    pub poly: Vec<[f32; 2]>,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ReplayMapObject {
    #[serde(rename = "typeId")]
    pub type_id: i64,
    #[serde(rename = "x")]
    pub x: f32,
    #[serde(rename = "y")]
    pub y: f32,
    #[serde(rename = "z", skip_serializing_if = "zero_f")]
    pub z: f32,
    #[serde(rename = "dx", skip_serializing_if = "zero_f")]
    pub dx: f32,
    #[serde(rename = "dy", skip_serializing_if = "zero_f")]
    pub dy: f32,
    #[serde(rename = "yaw", skip_serializing_if = "zero_f")]
    pub yaw: f32,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ReplayWeaponLabel {
    #[serde(rename = "en")]
    pub en: String,
    #[serde(rename = "fr")]
    pub fr: String,
    #[serde(rename = "fx", skip_serializing_if = "String::is_empty")]
    pub fx: String,
    #[serde(rename = "key", skip_serializing_if = "String::is_empty")]
    pub key: String,
    #[serde(rename = "role", skip_serializing_if = "String::is_empty")]
    pub role: String,
    #[serde(rename = "tint", skip_serializing_if = "String::is_empty")]
    pub tint: String,
    #[serde(rename = "img", skip_serializing_if = "String::is_empty")]
    pub img: String,
    #[serde(rename = "tinted", skip_serializing_if = "false_b")]
    pub tinted: bool,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ReplayVehicleLabel {
    #[serde(rename = "img", skip_serializing_if = "String::is_empty")]
    pub img: String,
    #[serde(rename = "tinted", skip_serializing_if = "false_b")]
    pub tinted: bool,
    #[serde(rename = "kind", skip_serializing_if = "String::is_empty")]
    pub kind: String,
    #[serde(rename = "en", skip_serializing_if = "String::is_empty")]
    pub en: String,
    #[serde(rename = "fr", skip_serializing_if = "String::is_empty")]
    pub fr: String,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ReplayNeutralDeath {
    #[serde(rename = "xuid")]
    pub xuid: String,
    #[serde(rename = "feedMs")]
    pub feed_ms: i64,
    #[serde(rename = "kind")]
    pub kind: String,
    #[serde(rename = "img", skip_serializing_if = "String::is_empty")]
    pub img: String,
    #[serde(rename = "tinted", skip_serializing_if = "false_b")]
    pub tinted: bool,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ReplayMapWeaponPads {
    #[serde(rename = "pads")]
    pub pads: Vec<ReplayMapWeaponPadDTO>,
    #[serde(rename = "catalogN")]
    pub catalog_n: i64,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ReplayMapWeaponPadDTO {
    #[serde(rename = "x")]
    pub x: f32,
    #[serde(rename = "y")]
    pub y: f32,
    #[serde(rename = "z", skip_serializing_if = "zero_f")]
    pub z: f32,
    #[serde(rename = "pad")]
    pub pad: i64,
    #[serde(rename = "family")]
    pub family: String,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ReplayWeaponTiersInfo {
    #[serde(rename = "randomStarts")]
    pub random_starts: bool,
}
/// Native geometry framing uses centers only, with no altitude or size expansion.
pub fn replay_geometry_bounds(objects: &[ReplayMapObject]) -> Option<ReplayBounds> {
    let first = objects.first()?;
    let mut b = ReplayBounds {
        min_x: first.x,
        max_x: first.x,
        min_y: first.y,
        max_y: first.y,
        ..Default::default()
    };
    for o in &objects[1..] {
        b.min_x = if b.min_x < o.x { b.min_x } else { o.x };
        b.max_x = if b.max_x > o.x { b.max_x } else { o.x };
        b.min_y = if b.min_y < o.y { b.min_y } else { o.y };
        b.max_y = if b.max_y > o.y { b.max_y } else { o.y };
    }
    Some(b)
}
pub fn replay_surface_bounds(surfaces: &[ReplaySurface]) -> Option<ReplayBounds> {
    let first = surfaces.first()?;
    let mut b = ReplayBounds {
        min_x: first.x0,
        max_x: first.x1,
        min_y: first.y0,
        max_y: first.y1,
        ..Default::default()
    };
    for s in &surfaces[1..] {
        b.min_x = if b.min_x < s.x0 { b.min_x } else { s.x0 };
        b.max_x = if b.max_x > s.x1 { b.max_x } else { s.x1 };
        b.min_y = if b.min_y < s.y0 { b.min_y } else { s.y0 };
        b.max_y = if b.max_y > s.y1 { b.max_y } else { s.y1 };
    }
    Some(b)
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReplayStance<K = String> {
    pub slot: u32,
    pub kind: K,
    pub t0: i64,
    pub t1: i64,
}
