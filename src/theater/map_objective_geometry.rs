//! Map-objective volumes with the reference's orthonormal frame and inclusive bounds.
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct ObjectiveVec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}
impl ObjectiveVec3 {
    fn sub(self, b: Self) -> Self {
        Self {
            x: self.x - b.x,
            y: self.y - b.y,
            z: self.z - b.z,
        }
    }
    fn scale(self, k: f64) -> Self {
        Self {
            x: self.x * k,
            y: self.y * k,
            z: self.z * k,
        }
    }
    fn dot(self, b: Self) -> f64 {
        self.x * b.x + self.y * b.y + self.z * b.z
    }
    fn cross(self, b: Self) -> Self {
        Self {
            x: self.y * b.z - self.z * b.y,
            y: self.z * b.x - self.x * b.z,
            z: self.x * b.y - self.y * b.x,
        }
    }
    fn normalized(self) -> Option<Self> {
        let n = self.dot(self).sqrt();
        if n < 1e-6 {
            None
        } else {
            Some(self.scale(1. / n))
        }
    }
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ObjectiveShapeRaw {
    pub family: i32,
    pub s5: i64,
    pub s6: i64,
    pub s7: i64,
    pub s8: i64,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ObjectiveShape {
    pub family: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub half_x: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub half_y: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub radius: Option<f64>,
    pub up_z: f64,
    pub down_z: f64,
    pub forward: ObjectiveVec3,
    pub up: ObjectiveVec3,
    pub raw: ObjectiveShapeRaw,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectiveVolumeError {
    NoShape,
    DegenerateFrame,
    MissingExtent,
}
#[derive(Debug, Clone, PartialEq)]
pub struct ObjectiveVolume {
    pub center: ObjectiveVec3,
    fwd: ObjectiveVec3,
    right: ObjectiveVec3,
    up: ObjectiveVec3,
    family: String,
    half_x: f64,
    half_y: f64,
    radius: f64,
    up_z: f64,
    down_z: f64,
}
impl ObjectiveVolume {
    pub fn new(
        center: ObjectiveVec3,
        shape: Option<&ObjectiveShape>,
    ) -> Result<Self, ObjectiveVolumeError> {
        let s = shape.ok_or(ObjectiveVolumeError::NoShape)?;
        let up =
            s.up.normalized()
                .ok_or(ObjectiveVolumeError::DegenerateFrame)?;
        let fwd = s
            .forward
            .sub(up.scale(s.forward.dot(up)))
            .normalized()
            .ok_or(ObjectiveVolumeError::DegenerateFrame)?;
        let mut v = Self {
            center,
            fwd,
            right: up.cross(fwd),
            up,
            family: s.family.clone(),
            half_x: 0.,
            half_y: 0.,
            radius: 0.,
            up_z: s.up_z,
            down_z: s.down_z,
        };
        match s.family.as_str() {
            "box" => {
                v.half_x = s.half_x.ok_or(ObjectiveVolumeError::MissingExtent)?;
                v.half_y = s.half_y.ok_or(ObjectiveVolumeError::MissingExtent)?;
            }
            "cylinder" => v.radius = s.radius.ok_or(ObjectiveVolumeError::MissingExtent)?,
            _ => {}
        }
        Ok(v)
    }
    pub fn contains(&self, p: ObjectiveVec3) -> bool {
        let d = p.sub(self.center);
        let z = d.dot(self.up);
        if z > self.up_z || z < -self.down_z {
            return false;
        }
        let x = d.dot(self.fwd);
        let y = d.dot(self.right);
        match self.family.as_str() {
            "box" => x.abs() <= self.half_x && y.abs() <= self.half_y,
            "cylinder" => x * x + y * y <= self.radius * self.radius,
            _ => false,
        }
    }
    pub fn distance_to(&self, p: ObjectiveVec3) -> f64 {
        let d = p.sub(self.center);
        let z = d.dot(self.up);
        let dz = (z - self.up_z).max(-self.down_z - z).max(0.);
        let x = d.dot(self.fwd);
        let y = d.dot(self.right);
        match self.family.as_str() {
            "box" => {
                let dx = (x.abs() - self.half_x).max(0.);
                let dy = (y.abs() - self.half_y).max(0.);
                (dx * dx + dy * dy + dz * dz).sqrt()
            }
            "cylinder" => (x.hypot(y) - self.radius).max(0.).hypot(dz),
            _ => f64::INFINITY,
        }
    }
    pub fn translated(&self, d: ObjectiveVec3) -> Self {
        let mut v = self.clone();
        v.center = ObjectiveVec3 {
            x: v.center.x + d.x,
            y: v.center.y + d.y,
            z: v.center.z + d.z,
        };
        v
    }
}
