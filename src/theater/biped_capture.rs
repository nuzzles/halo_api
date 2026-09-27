//! Read-only companion capture following an anchored biped position.
use super::{
    DecodedBodyVitality, DecodedShieldVitality, FilmBitReader, read_body_vitality,
    read_shield_vitality,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct BipedCompanions {
    /// i1 packed direction and logarithmic magnitude. None includes keep/absent paths.
    pub velocity: Option<[u32; 2]>,
    /// i2 packed direction; mode/roll are carried separately for dynamic grammar.
    pub forward: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chassis: Option<super::NativeChassisOrientation>,
    /// i21 primary yaw/pitch, retained even if a later flag is truncated.
    pub aim: Option<[u32; 2]>,
    pub aim_flag_0: bool,
    pub aim_flag_1: bool,
    pub aim_b: Option<[u32; 2]>,
    pub aim_flag_2: bool,
    pub body: Option<DecodedBodyVitality>,
    pub shield: Option<DecodedShieldVitality>,
}

impl BipedCompanions {
    /// Native primary aim heading. Absence remains distinct from a zero angle.
    pub fn aim_heading_degrees(&self) -> Option<f32> {
        Some(super::biped_aim::aim_heading_from_raw(self.aim?[0]))
    }

    /// Native primary aim pitch, using the recorded 11-bit midpoint convention.
    pub fn aim_pitch_degrees(&self) -> Option<f32> {
        Some(super::biped_aim::aim_pitch_from_raw(self.aim?[1]))
    }

    /// Native checked velocity projection. Missing or invalid directions remain
    /// absent; the recorded quantized fields are preserved in `velocity`.
    pub fn velocity_vector(&self) -> Option<[f32; 3]> {
        let [code, scale] = self.velocity?;
        let direction = super::decode_native_direction(code, 19)?;
        let magnitude = super::decode_native_velocity_magnitude(u64::from(scale), 10);
        Some(direction.map(|v| v * magnitude))
    }
}

/// Mirrors scanRecordDirs for the ti35 biped grammar. `vector_end` is before the
/// reference's two-bit i0 tail. Stops at unsupported indices or truncated fields;
/// successful earlier observations survive. This does not establish a record end.
pub fn scan_biped_companions(data: &[u8], vector_end: usize, mask: &[u8]) -> BipedCompanions {
    scan_biped_companions_with_grammar(data, vector_end, mask, None, false)
}
/// Capture vehicle companions using the same orientation component decoder as the
/// full record walker. `dynamic_forward_level` is the registry's i2 level; None
/// selects the ordinary biped grammar. Dynamic angular velocity has its own gate.
pub fn scan_biped_companions_with_grammar(
    data: &[u8],
    vector_end: usize,
    mask: &[u8],
    dynamic_forward_level: Option<u32>,
    dynamic_angular: bool,
) -> BipedCompanions {
    let Some(start) = vector_end.checked_add(2) else {
        return BipedCompanions::default();
    };
    scan_companions_at(
        data,
        start,
        &mask[mask.len().min(1)..],
        dynamic_forward_level,
        dynamic_angular,
    )
    .0
}

/// Shared component traversal, including aim records that have no i0 prefix.
/// The second result ends the atomic primary aim fields, before optional tails.
pub(super) fn scan_companions_at(
    data: &[u8],
    start: usize,
    mask: &[u8],
    dynamic_forward_level: Option<u32>,
    dynamic_angular: bool,
) -> (BipedCompanions, Option<usize>) {
    let mut out = BipedCompanions::default();
    let mut aim_end = None;
    let Some(mut c) = FilmBitReader::new(data, start) else {
        return (out, aim_end);
    };
    for &index in mask {
        let ok = (|| -> Option<()> {
            match index {
                1 => {
                    if c.bit()? {
                        c.skip(96)?;
                    } else if !c.bit()? {
                        let dir = c.read(19)? as u32;
                        let scale = c.read(10)? as u32;
                        out.velocity = Some([dir, scale]);
                    }
                }
                2 if dynamic_forward_level.is_some() => {
                    let super::ComponentDecode::Decoded(component) = super::decode_component(
                        data,
                        c.position,
                        "object-forward-and-up-dynamic-precision-component",
                        dynamic_forward_level.unwrap(),
                        40,
                    ) else {
                        return None;
                    };
                    let orientation = super::NativeChassisOrientation::from_component(&component)?;
                    out.forward = orientation.direction;
                    out.chassis = Some(orientation);
                    c.position = usize::try_from(component.end_bit).ok()?;
                }
                2 => {
                    if !c.bit()? {
                        out.forward = Some(c.read(19)? as u32);
                    }
                    c.skip(8)?;
                }
                3 if dynamic_angular => {
                    let super::ComponentDecode::Decoded(component) = super::decode_component(
                        data,
                        c.position,
                        "object-angular-velocity-dynamic-precision-component",
                        0,
                        40,
                    ) else {
                        return None;
                    };
                    c.position = usize::try_from(component.end_bit).ok()?;
                }
                3 => {
                    if !c.bit()? {
                        c.skip(27)?;
                    }
                }
                4 => {
                    out.body = Some(read_body_vitality(&mut c)?);
                }
                5 => {
                    // Preserve the reference scanner's 29-bit minimum guard.
                    if c.position.checked_add(29)? > data.len().saturating_mul(8) {
                        return None;
                    }
                    out.shield = Some(read_shield_vitality(&mut c)?);
                }
                21 => {
                    // The first 24 bits are an atomic publication in Go.
                    let mut head = c;
                    let flag = head.bit()?;
                    let yaw = head.read(12)? as u32;
                    let pitch = head.read(11)? as u32;
                    c = head;
                    out.aim_flag_0 = flag;
                    out.aim = Some([yaw, pitch]);
                    aim_end = Some(c.position);
                    out.aim_flag_1 = c.bit()?;
                    if !flag {
                        let yaw = c.read(12)? as u32;
                        let pitch = c.read(11)? as u32;
                        let flag = c.bit()?;
                        out.aim_b = Some([yaw, pitch]);
                        out.aim_flag_2 = flag;
                    }
                }
                _ => return None,
            }
            Some(())
        })();
        if ok.is_none() || index == 21 {
            break;
        }
    }
    (out, aim_end)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use serde_json::{Value, json};
    use std::io::Read;
    pub(crate) fn assert_native(out: &BipedCompanions, dirs: &Value, vit: &Value) {
        for (has, names, value) in [
            ("HasVel", ["VelRaw", "VelScale"], out.velocity),
            ("HasYaw", ["YawRaw", "PitchRaw"], out.aim),
            ("HasAimB", ["YawRawB", "PitchRawB"], out.aim_b),
        ] {
            assert_eq!(value.is_some(), dirs[has].as_bool().unwrap(), "{has}");
            let value = value.unwrap_or([0; 2]);
            for i in 0..2 {
                assert_eq!(json!(value[i]), dirs[names[i]], "{}", names[i]);
            }
        }
        assert_eq!(json!(out.forward.is_some()), dirs["HasAim"]);
        assert_eq!(json!(out.forward.unwrap_or(0)), dirs["AimRaw"]);
        for (name, value) in [
            ("AimFlag0", out.aim_flag_0),
            ("AimFlag1", out.aim_flag_1),
            ("AimFlag2", out.aim_flag_2),
        ] {
            assert_eq!(json!(value), dirs[name], "{name}");
        }
        let orientation = out.chassis.clone().unwrap_or_default();
        assert_eq!(json!(orientation.roll.is_some()), dirs["HasRoll"]);
        assert_eq!(json!(orientation.roll.unwrap_or(0)), dirs["RollRaw"]);
        assert_eq!(json!(orientation.mode), dirs["FwdMode"]);
        assert_eq!(json!(orientation.direction_default), dirs["AimDefault"]);
        assert_eq!(json!(out.body.is_some()), vit["HasBody"]);
        assert_eq!(json!(out.shield.is_some()), vit["HasShield"]);
        if let Some(b) = &out.body {
            let e = &vit["Body"];
            assert_eq!(json!(b.quantum), e["Q"]);
            assert_eq!(
                b.health.to_bits(),
                (e["Health"].as_f64().unwrap() as f32).to_bits()
            );
            for (i, name) in ["F5c", "F5d", "F5e"].iter().enumerate() {
                assert_eq!(json!(b.flags[i]), e[name]);
            }
        }
        if let Some(s) = &out.shield {
            let e = &vit["Shield"];
            assert_eq!(json!(s.quantum), e["Q"]);
            assert_eq!(
                s.shield.to_bits(),
                (e["Shield"].as_f64().unwrap() as f32).to_bits()
            );
            assert_eq!(json!(s.regen_present), e["RegenPresent"]);
            assert_eq!(json!(s.block_64), e["Block64"]);
            for i in 0..2 {
                assert_eq!(json!(s.regen[i].is_some()), e[format!("HasRegen{i}")]);
                assert_eq!(json!(s.regen[i].unwrap_or(0)), e[format!("Regen{i}")]);
            }
            for (i, name) in ["F66", "F67", "F69", "F68"].iter().enumerate() {
                assert_eq!(json!(s.flags[i]), e[name]);
            }
        }
    }
    #[test]
    fn companion_capture_matches_native_go_including_truncation() {
        let mut json = String::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/biped-scan-levelup-v41.json.zlib")[..],
        )
        .read_to_string(&mut json)
        .unwrap();
        let oracle: Value = serde_json::from_str(&json).unwrap();
        let rows = oracle["companions"].as_array().unwrap();
        assert_eq!(rows.len(), 4096);
        for row in rows {
            let hex = row["hex"].as_str().unwrap();
            let bytes: Vec<u8> = (0..hex.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                .collect();
            let mask: Vec<u8> = serde_json::from_value(row["mask"].clone()).unwrap();
            let out = scan_biped_companions(&bytes, row["at"].as_u64().unwrap() as usize, &mask);
            assert_native(&out, &row["dirs"], &row["vitals"]);
        }
    }
}
