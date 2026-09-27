//! Distance arithmetic for pinned native replay geometry.
/// Native planDist: subtract in f32, then evaluate the scaled f64 hypot formula.
/// The explicit fused step matches the pinned Go reference and avoids differing
/// platform hypot implementations on Rust hosts and WebAssembly.
pub fn replay_plan_distance(a: [f32; 2], b: [f32; 2]) -> f64 {
    let mut p = f64::from(a[0] - b[0]).abs();
    let mut q = f64::from(a[1] - b[1]).abs();
    if p.is_infinite() || q.is_infinite() {
        return f64::INFINITY;
    }
    if p.is_nan() || q.is_nan() {
        // math.NaN's canonical payload, returned by native math.Hypot.
        return f64::from_bits(0x7ff8_0000_0000_0001);
    }
    if p < q {
        std::mem::swap(&mut p, &mut q);
    }
    if p == 0.0 {
        return 0.0;
    }
    q /= p;
    p * q.mul_add(q, 1.0).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn native_geometry_plan_distance() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            include_bytes!("fixtures/geometry-distance-v41.json.zlib").as_slice(),
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(rows.len(), 4661);
        for (i, row) in rows.iter().enumerate() {
            let a = std::array::from_fn(|j| f32::from_bits(row["a"][j].as_u64().unwrap() as u32));
            let b = std::array::from_fn(|j| f32::from_bits(row["b"][j].as_u64().unwrap() as u32));
            assert_eq!(
                replay_plan_distance(a, b).to_bits(),
                row["plan"].as_u64().unwrap(),
                "case {i}: {row}"
            );
        }
    }

    #[test]
    fn native_geometry_grenade_attribution() {
        use crate::theater::*;
        use std::num::NonZeroU64;
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            include_bytes!("fixtures/geometry-distance-grenades-v41.json.zlib").as_slice(),
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(rows.len(), 4);
        for (i, row) in rows.into_iter().enumerate() {
            let positions: Vec<ReplayPlayerPosition> =
                serde_json::from_value(row["positions"].clone()).unwrap();
            let throws: Vec<FilmGrenadeThrow> =
                serde_json::from_value(row["throws"].clone()).unwrap();
            let owners = serde_json::from_value(row["owners"].clone()).unwrap();
            let projectiles: Vec<WorldObjectTrack> =
                serde_json::from_value(row["projectiles"].clone()).unwrap();
            let links = serde_json::from_value(row["links"].clone()).unwrap();
            let expected: ReplayGrenadePublication =
                serde_json::from_value(row["output"].clone()).unwrap();
            assert_eq!(
                build_replay_grenades(
                    &positions,
                    &throws,
                    0,
                    NonZeroU64::new(100_000).unwrap(),
                    &owners,
                    &projectiles,
                    &links
                ),
                expected,
                "case {i}"
            );
        }
    }
}
