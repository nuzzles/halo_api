use super::{Velocity, bits::Bits};

/// Six cube faces, each with a midpoint-quantized two-dimensional grid.
/// The stride includes unused codes: these must not become plausible directions.
pub(super) fn direction(code: u32) -> Option<[f32; 3]> {
    let face = code / 87_381;
    let remainder = code % 87_381;
    let u = remainder / 294;
    let v = remainder % 294;
    if face >= 6 || u >= 293 || v >= 293 {
        return None;
    }
    let u = ((u as f32) + 0.5) * 2.0 / 293.0 - 1.0;
    let v = ((v as f32) + 0.5) * 2.0 / 293.0 - 1.0;
    let mut vector = match face {
        0 => [1.0, u, v],
        1 => [u, 1.0, v],
        2 => [u, v, 1.0],
        3 => [-1.0, u, v],
        4 => [u, -1.0, v],
        5 => [u, v, -1.0],
        _ => return None,
    };
    let norm = vector.iter().map(|v| v * v).sum::<f32>().sqrt();
    for value in &mut vector {
        *value /= norm;
    }
    Some(vector)
}

/// Unsupported direction codes retain their checked length, without exporting
/// a semantic observation or discarding unrelated position/aim fields.
pub(super) fn read(b: Bits<'_>, o: usize) -> Option<(Option<Velocity>, usize)> {
    match b.read(o, 2)? {
        1 => Some((Some(Velocity::Stationary), o + 2)),
        0 => {
            let direction_code = b.read(o + 2, 19)? as u32;
            let magnitude_code = b.read(o + 21, 10)? as u16;
            let value = direction(direction_code).map(|direction| Velocity::Directed {
                direction_code,
                direction,
                magnitude_code,
            });
            Some((value, o + 31))
        }
        _ => None,
    }
}
