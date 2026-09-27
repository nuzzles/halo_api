//! Published native vehicle lives, occupancy and coverage.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
fn zero(v: &f32) -> bool {
    *v == 0.0
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReplayVehicleTrack {
    #[serde(rename = "slot")]
    pub slot: u32,
    #[serde(rename = "gen")]
    pub r#gen: u32,
    #[serde(rename = "chassis", default, skip_serializing_if = "String::is_empty")]
    pub chassis: String,
    #[serde(rename = "family", default, skip_serializing_if = "String::is_empty")]
    pub family: String,
    #[serde(rename = "t0")]
    pub t0: i64,
    #[serde(rename = "t1")]
    pub t1: i64,
    #[serde(rename = "t1max")]
    pub t1_max: i64,
    #[serde(rename = "end")]
    pub end: String,
    #[serde(rename = "tEnd", default, skip_serializing_if = "Option::is_none")]
    pub t_end: Option<i64>,
    #[serde(rename = "spawn", default, skip_serializing_if = "Option::is_none")]
    pub spawn: Option<ReplayVehicleSpawn>,
    #[serde(rename = "samples", default, skip_serializing_if = "Vec::is_empty")]
    pub samples: Vec<ReplayVehicleSample>,
    #[serde(rename = "rides", default, skip_serializing_if = "Vec::is_empty")]
    pub rides: Vec<ReplayVehicleRide>,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReplayVehicleSpawn {
    #[serde(rename = "x")]
    pub x: f32,
    #[serde(rename = "y")]
    pub y: f32,
    #[serde(rename = "z", default, skip_serializing_if = "zero")]
    pub z: f32,
    #[serde(rename = "h", default, skip_serializing_if = "Option::is_none")]
    pub h: Option<f32>,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReplayVehicleSample {
    #[serde(rename = "t")]
    pub t: i64,
    #[serde(rename = "x")]
    pub x: f32,
    #[serde(rename = "y")]
    pub y: f32,
    #[serde(rename = "z", default, skip_serializing_if = "zero")]
    pub z: f32,
    #[serde(rename = "h", default, skip_serializing_if = "zero")]
    pub h: f32,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReplayVehicleRide {
    #[serde(rename = "t0")]
    pub t0: i64,
    #[serde(rename = "t1")]
    pub t1: i64,
    #[serde(rename = "slot")]
    pub slot: u32,
    #[serde(rename = "xuid", default, skip_serializing_if = "String::is_empty")]
    pub xuid: String,
    #[serde(rename = "seat", default, skip_serializing_if = "Option::is_none")]
    pub seat: Option<i64>,
    #[serde(rename = "src")]
    pub src: String,
    #[serde(rename = "aim", default, skip_serializing_if = "Vec::is_empty")]
    pub aim: Vec<ReplayVehicleAim>,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReplayVehicleAim {
    #[serde(rename = "t")]
    pub t: i64,
    #[serde(rename = "h", default, skip_serializing_if = "zero")]
    pub h: f32,
    #[serde(rename = "p", default, skip_serializing_if = "zero")]
    pub p: f32,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReplayVehicleCoverage {
    #[serde(rename = "scanned")]
    pub scanned: bool,
    #[serde(rename = "lives")]
    pub lives: i64,
    #[serde(rename = "published")]
    pub published: i64,
    #[serde(rename = "noPosition")]
    pub no_position: i64,
    #[serde(rename = "merged")]
    pub merged: i64,
    #[serde(rename = "withSpawn")]
    pub with_spawn: i64,
    #[serde(rename = "withChassis")]
    pub with_chassis: i64,
    #[serde(rename = "familyResolved")]
    pub family_resolved: i64,
    #[serde(rename = "familyUnknown")]
    pub family_unknown: i64,
    #[serde(
        rename = "unknownChassis",
        default,
        skip_serializing_if = "BTreeMap::is_empty"
    )]
    pub unknown_chassis: BTreeMap<String, i64>,
    #[serde(rename = "samples")]
    pub samples: i64,
    #[serde(rename = "withHeading")]
    pub with_heading: i64,
    #[serde(rename = "deathsRead")]
    pub deaths_read: i64,
    #[serde(rename = "deathsMatched")]
    pub deaths_matched: i64,
    #[serde(rename = "deathsUnmatched")]
    pub deaths_unmatched: i64,
    #[serde(rename = "deathsTailDesync")]
    pub deaths_tail_desync: i64,
    #[serde(rename = "endDestroyed")]
    pub end_destroyed: i64,
    #[serde(rename = "endFilmEnd")]
    pub end_film_end: i64,
    #[serde(rename = "endUnknown")]
    pub end_unknown: i64,
    #[serde(rename = "samplesAfterEnd")]
    pub samples_after_end: i64,
    #[serde(rename = "rides")]
    pub rides: i64,
    #[serde(rename = "vehiclesRidden")]
    pub vehicles_ridden: i64,
    #[serde(rename = "ridesNamed")]
    pub rides_named: i64,
    #[serde(rename = "ridesRead")]
    pub rides_read: i64,
    #[serde(rename = "ridesProximity")]
    pub rides_proximity: i64,
    #[serde(rename = "ridesWithSeat")]
    pub rides_with_seat: i64,
    #[serde(rename = "aimReads")]
    pub aim_reads: i64,
    #[serde(rename = "ridesWithAim")]
    pub rides_with_aim: i64,
    #[serde(rename = "aimSamples")]
    pub aim_samples: i64,
    #[serde(rename = "aimRideFrames")]
    pub aim_ride_frames: i64,
    #[serde(rename = "ambiguous")]
    pub ambiguous: i64,
    #[serde(rename = "shots")]
    pub shots: i64,
    #[serde(rename = "shotsAmbiguous")]
    pub shots_ambiguous: i64,
    #[serde(rename = "shotsUnplaced")]
    pub shots_unplaced: i64,
    #[serde(rename = "shotsNoRide")]
    pub shots_no_ride: i64,
    #[serde(rename = "shotsVehicleWeapon")]
    pub shots_vehicle_weapon: i64,
    #[serde(rename = "cycleLocations")]
    pub cycle_locations: i64,
    #[serde(rename = "cycles")]
    pub cycles: i64,
    #[serde(rename = "cycleGaps")]
    pub cycle_gaps: i64,
    #[serde(rename = "cycleMissing")]
    pub cycle_missing: i64,
}
