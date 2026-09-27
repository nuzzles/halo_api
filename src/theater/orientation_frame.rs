//! Native packed directions and chassis forward reconstruction from up plus roll.
use serde::{Deserialize, Serialize};

const CUBEMAP: [(u32, u32); 25] = [
    (10, 2),
    (21, 3),
    (42, 5),
    (85, 8),
    (170, 12),
    (341, 17),
    (682, 25),
    (1365, 35),
    (2730, 51),
    (5461, 72),
    (10922, 103),
    (21845, 146),
    (43690, 208),
    (87381, 294),
    (174762, 417),
    (349525, 590),
    (699050, 835),
    (1398101, 1181),
    (2796202, 1671),
    (5592405, 2363),
    (11184810, 3343),
    (22369621, 4728),
    (44739242, 6687),
    (89478485, 9458),
    (178956970, 13376),
];
/// Native cubemap direction, with absence for illegal width or face. Widths 6..30
/// are defined by the binary's table; callers must retain the component's mode.
pub fn decode_native_direction(code: u32, width: u32) -> Option<[f32; 3]> {
    decode_direction(code, width, false)
}

/// Native unchecked projection: invalid widths/faces return the +Z sentinel.
/// Use `decode_native_direction` when validity must remain distinguishable.
pub fn decode_native_direction_or_sentinel(code: u32, width: u32) -> [f32; 3] {
    decode_native_direction(code, width).unwrap_or([0., 0., 1.])
}

/// Native dynamic-precision wrapper. Only the low 32 bits of `packed` are used;
/// invalid widths/faces return the +Z sentinel, as in the reference.
pub fn decode_native_dynamic_direction(packed: u64, width: u32) -> [f32; 3] {
    decode_native_direction_or_sentinel(packed as u32, width)
}

/// Native logarithmic translational speed quantizer in world units per second.
/// Zero maps to 0.03; the largest code maps to 350. Widths >=64 retain the
/// reference's zero-shift/unsigned-wrap behavior, including infinite results.
pub fn decode_native_velocity_magnitude(scale: u64, width: u32) -> f32 {
    let n = 1_u64.checked_shl(width).unwrap_or(0);
    if scale == 0 {
        return 0.03;
    }
    if scale >= n.wrapping_sub(1) {
        return 350.;
    }
    let one_minus = 1.0_f32 - 0.03;
    let step = f64::from(one_minus + 350.).ln() as f32 / n as f32;
    // The reference rounds the exponent before calling math.Exp.
    f64::from(scale as f32 * step + step * 0.5).exp() as f32 - one_minus
}

/// Native 19-bit direction times 10-bit translational speed. Invalid direction
/// codes retain the native sentinel; use the companion query for checked values.
pub fn decode_native_velocity(packed_direction: u64, scale: u64) -> [f32; 3] {
    let direction = decode_native_dynamic_direction(packed_direction, 19);
    let magnitude = decode_native_velocity_magnitude(scale, 10);
    direction.map(|v| v * magnitude)
}

/// Reference validation helper with the second cubemap coordinate forced to zero.
/// This intentionally incomplete reconstruction is not a production direction.
pub fn decode_native_direction_flat(code: u32, width: u32) -> Option<[f32; 3]> {
    decode_direction(code, width, true)
}

fn decode_direction(code: u32, width: u32, flat: bool) -> Option<[f32; 3]> {
    let &(size, n) = CUBEMAP.get(width.checked_sub(6)? as usize)?;
    let face = code / size;
    let rem = code % size;
    let step = 2.0_f32 / (n - 1) as f32;
    let coord = |i: u32| {
        if i * 2 == n - 2 {
            0.0
        } else {
            (i as f32).mul_add(step, -1.0) + step * 0.5
        }
    };
    let (a, b) = (coord(rem / n), if flat { 0. } else { coord(rem % n) });
    let v = match face {
        0 => [1., a, b],
        1 => [a, 1., b],
        2 => [a, b, 1.],
        3 => [-1., a, b],
        4 => [a, -1., b],
        5 => [a, b, -1.],
        _ => return None,
    };
    let n = norm(v);
    Some(if n < 1e-4 { v } else { v.map(|x| x / n) })
}

/// Native inverse direction quantizer, retained for codec validation. This does
/// not provide LegacyFilm re-encoding. Width validity is separate from vector validity:
/// like the reference, the zero vector encodes on the negative X face.
pub fn encode_native_direction(v: [f32; 3], width: u32) -> Option<u32> {
    let &(size, n) = CUBEMAP.get(width.checked_sub(6)? as usize)?;
    let [ax, ay, az] = v.map(f32::abs);
    let (face, c0, c1) = if ay <= ax && az <= ax {
        (if v[0] > 0. { 0 } else { 3 }, v[1] / ax, v[2] / ax)
    } else if az <= ay {
        (if v[1] > 0. { 1 } else { 4 }, v[0] / ay, v[2] / ay)
    } else {
        (if v[2] > 0. { 2 } else { 5 }, v[0] / az, v[1] / az)
    };
    let step = 2.0_f32 / (n - 1) as f32;
    let cell = |c: f32| (((c + 1.) / step) as i64).clamp(0, i64::from(n - 2)) as u32;
    Some(face * size + n * cell(c0) + cell(c1))
}
// Fused operation order follows the pinned Go ARM64 executable. Its initial
// basis norm and final norm have different rounding order; keep both explicit.
fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[2].mul_add(b[2], a[0].mul_add(b[0], a[1] * b[1]))
}
fn norm(v: [f32; 3]) -> f32 {
    f64::from(v[2].mul_add(v[2], v[1].mul_add(v[1], v[0] * v[0]))).sqrt() as f32
}
fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        (-a[2]).mul_add(b[1], a[1] * b[2]),
        (-a[0]).mul_add(b[2], a[2] * b[0]),
        (-a[1]).mul_add(b[0], a[0] * b[1]),
    ]
}
/// Midpoint roll quantization in [-pi,pi], using the native float32 constants.
/// Native shifts >=64 yield zero steps; keep IEEE arithmetic for such inputs.
pub fn native_roll_angle(raw: u32, bits: u32) -> f32 {
    let pi = std::f32::consts::PI;
    let steps = 1_u64.checked_shl(bits).unwrap_or(0) as f32;
    -pi + (raw as f32 + 0.5) * (pi - (-pi)) / steps
}
/// Reconstruct the perpendicular forward vector using the native asymmetric base
/// cross-products and Rodrigues rotation. Input up is normally unit length.
pub fn forward_from_up_roll(up: [f32; 3], roll: f32) -> [f32; 3] {
    let mut v = if dot(up, [1., 0., 0.]).abs() < dot(up, [0., 1., 0.]).abs() {
        cross(up, [1., 0., 0.])
    } else {
        cross([0., 1., 0.], up)
    };
    let n = f64::from(dot(v, v)).sqrt() as f32;
    if n >= 1e-4 {
        v = v.map(|x| x / n);
    }
    let (cos, sin) = if roll == std::f32::consts::PI || roll == -std::f32::consts::PI {
        (-1., 0.)
    } else {
        (f64::from(roll).cos() as f32, f64::from(roll).sin() as f32)
    };
    let k = cross(up, v);
    let ax = dot(v, up) * (1. - cos);
    let mut out = std::array::from_fn(|i| k[i].mul_add(sin, v[i].mul_add(cos, up[i] * ax)));
    let n = norm(out);
    if n > 0. {
        out = out.map(|x| x / n);
    }
    out
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeChassisOrientation {
    pub direction: Option<u32>,
    pub roll: Option<u32>,
    pub mode: u8,
    pub delta: bool,
    pub direction_default: bool,
}
impl NativeChassisOrientation {
    /// Typed projection of the shared dynamic-orientation grammar. Raw vectors
    /// and delta increments remain in the source component; native reconstruction
    /// only uses the absolute direction/roll fields represented here.
    pub fn from_component(component: &super::DecodedComponent) -> Option<Self> {
        if component.name != "object-forward-and-up-dynamic-precision-component" {
            return None;
        }
        let value = |name: &str| {
            component
                .fields
                .iter()
                .find(|f| f.name == name)
                .map(|f| f.raw)
        };
        let raw_vectors = value("raw_vectors")? != 0;
        let delta = value("delta").unwrap_or(0) != 0;
        let mode = if value("config_precision") == Some(1) {
            1
        } else if raw_vectors {
            2
        } else {
            0
        };
        let roll = value("roll").map(|v| v as u32);
        Some(Self {
            direction: value("up_direction").map(|v| v as u32),
            roll,
            mode,
            delta,
            direction_default: roll.is_some() && value("up_direction.gate") == Some(1),
        })
    }
    pub fn up(&self) -> Option<[f32; 3]> {
        if self.direction_default {
            Some([0., 0., 1.])
        } else {
            decode_native_direction(self.direction?, if self.mode == 1 { 30 } else { 19 })
        }
    }
    pub fn forward(&self) -> Option<[f32; 3]> {
        Some(forward_from_up_roll(
            self.up()?,
            native_roll_angle(self.roll?, if self.mode == 1 { 30 } else { 8 }),
        ))
    }
    /// The native replay publishes film-derived heading only for config mode 1.
    pub fn film_heading_degrees(&self) -> Option<f32> {
        native_film_chassis_heading(self.direction, self.roll, self.mode, self.direction_default)
    }
}
/// Native film-heading query from the fields consumed by configuration mode 1.
/// Delta/provenance fields remain in their source; this query does not infer them.
pub fn native_film_chassis_heading(
    direction: Option<u32>,
    roll: Option<u32>,
    mode: u8,
    direction_default: bool,
) -> Option<f32> {
    if mode != 1 {
        return None;
    }
    let up = if direction_default {
        [0., 0., 1.]
    } else {
        decode_native_direction(direction?, 30)?
    };
    let f = forward_from_up_roll(up, native_roll_angle(roll?, 30));
    let mut h = f64::from(f[1]).atan2(f64::from(f[0])) * 180. / std::f64::consts::PI;
    if h < 0. {
        h += 360.;
    }
    Some(h as f32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[derive(Deserialize)]
    struct OfflineAimCase {
        data: Vec<u8>,
        position: i64,
        width: i64,
        value: Option<u32>,
        indices: Vec<i64>,
        mask: u64,
        overflow: bool,
    }

    #[test]
    fn native_offline_aim_masks_and_diagnostic_reads() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/offline-aim-helpers-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let cases: Vec<OfflineAimCase> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(cases.len(), 2048);
        for (i, c) in cases.into_iter().enumerate() {
            let value = std::panic::catch_unwind(|| {
                super::super::native_bits_for_diagnostics(&c.data, c.position, c.width)
            })
            .ok();
            assert_eq!(value, c.value, "diagnostic {i}");
            assert_eq!(
                super::super::native_component_mask(c.indices.iter().copied()),
                (c.mask, c.overflow),
                "mask {i}"
            );
            if let Some(indices) = c
                .indices
                .iter()
                .map(|&v| u8::try_from(v).ok())
                .collect::<Option<Vec<_>>>()
            {
                let record = super::super::BipedPositionRecord {
                    start_bit: 0,
                    position_bit: 0,
                    end_bit: 0,
                    slot: 0,
                    generation: 0,
                    component_indices: indices,
                    quantized: [0; 3],
                    world: [0.; 3],
                    companions: Default::default(),
                };
                assert_eq!(
                    record.compact_component_mask(),
                    (c.mask, c.overflow),
                    "record mask {i}"
                );
            }
        }
    }
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct VelocityCase {
        width: u32,
        scale: u64,
        packed: u64,
        magnitude: u32,
        direction: [u32; 3],
        velocity: [u32; 3],
        has_vel: bool,
        has_yaw: bool,
        yaw: u32,
        pitch: u32,
        checked_velocity: Option<[u32; 3]>,
        heading: Option<u32>,
        angle: Option<u32>,
    }

    #[test]
    fn native_velocity_codecs_and_companion_queries() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/velocity-codec-v41.json.zlib")[..],
        )
        .read_to_end(&mut raw)
        .unwrap();
        let cases: Vec<VelocityCase> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(cases.len(), 8192);
        for (i, c) in cases.into_iter().enumerate() {
            assert_eq!(
                decode_native_velocity_magnitude(c.scale, c.width).to_bits(),
                c.magnitude,
                "magnitude {i}"
            );
            assert_eq!(
                decode_native_dynamic_direction(c.packed, c.width).map(f32::to_bits),
                c.direction,
                "direction {i}"
            );
            assert_eq!(
                decode_native_velocity(c.packed, c.scale).map(f32::to_bits),
                c.velocity,
                "velocity {i}"
            );
            let companions = super::super::BipedCompanions {
                velocity: c.has_vel.then_some([c.packed as u32, c.scale as u32]),
                aim: c.has_yaw.then_some([c.yaw, c.pitch]),
                ..Default::default()
            };
            assert_eq!(
                companions.velocity_vector().map(|v| v.map(f32::to_bits)),
                c.checked_velocity,
                "checked velocity {i}"
            );
            assert_eq!(
                companions.aim_heading_degrees().map(f32::to_bits),
                c.heading,
                "heading {i}"
            );
            assert_eq!(
                companions.aim_pitch_degrees().map(f32::to_bits),
                c.angle,
                "pitch {i}"
            );
        }
    }
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Case {
        mask: Vec<u8>,
        length: usize,
        angular: bool,
        dirs: serde_json::Value,
        vitals: serde_json::Value,
        data: Vec<u8>,
        start: usize,
        level: u32,
        end: usize,
        dynamic: NativeChassisOrientation,
        width: u32,
        code: u32,
        direction: Option<[f32; 3]>,
        flat: Option<[f32; 3]>,
        encoded: Option<u32>,
        sentinel: [f32; 3],
        up: [f32; 3],
        roll: f32,
        forward: [f32; 3],
        raw: u32,
        bits: u32,
        angle: f32,
        orientation: NativeChassisOrientation,
        object_up: Option<[f32; 3]>,
        object_forward: Option<[f32; 3]>,
    }
    #[test]
    fn native_chassis_orientation() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(&include_bytes!("fixtures/orientation-v41.json.zlib")[..])
            .read_to_end(&mut raw)
            .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&raw).unwrap();
        for (i, c) in cases.into_iter().enumerate() {
            let companions = super::super::scan_biped_companions_with_grammar(
                &c.data[..c.length],
                c.start,
                &c.mask,
                Some(c.level),
                c.angular,
            );
            super::super::biped_capture::tests::assert_native(&companions, &c.dirs, &c.vitals);
            let super::super::ComponentDecode::Decoded(component) = super::super::decode_component(
                &c.data,
                c.start,
                "object-forward-and-up-dynamic-precision-component",
                c.level,
                40,
            ) else {
                panic!("decode {i}");
            };
            assert_eq!(
                serde_json::json!(component.end_bit),
                serde_json::json!(c.end),
                "end {i}"
            );
            assert_eq!(
                NativeChassisOrientation::from_component(&component),
                Some(c.dynamic),
                "dynamic {i}"
            );
            assert_eq!(
                decode_native_direction(c.code, c.width),
                c.direction,
                "direction {i}"
            );
            assert_eq!(
                decode_native_direction_flat(c.code, c.width),
                c.flat,
                "flat {i}"
            );
            assert_eq!(
                encode_native_direction(c.up, c.width),
                c.encoded,
                "encode {i}"
            );
            assert_eq!(
                decode_native_direction_or_sentinel(c.code, c.width),
                c.sentinel,
                "sentinel {i}"
            );
            assert_eq!(native_roll_angle(c.raw, c.bits), c.angle, "roll {i}");
            assert_eq!(forward_from_up_roll(c.up, c.roll), c.forward, "forward {i}");
            assert_eq!(c.orientation.up(), c.object_up, "up {i}");
            assert_eq!(
                c.orientation.forward(),
                c.object_forward,
                "object forward {i}"
            );
        }
    }
}
