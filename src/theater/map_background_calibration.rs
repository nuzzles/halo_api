//! Native image-to-world calibration, independent of the background image format.
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct MapBackgroundCalibration {
    pub meters_per_pixel: f64,
    pub origin_x: f64,
    pub origin_y: f64,
    pub width_px: i64,
    pub height_px: i64,
    pub convention: String,
}
impl Serialize for MapBackgroundCalibration {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        if ![self.meters_per_pixel, self.origin_x, self.origin_y]
            .iter()
            .all(|v| v.is_finite())
        {
            return Err(serde::ser::Error::custom(
                "nonfinite map calibration cannot be published as JSON",
            ));
        }
        let mut value = serializer.serialize_struct("MapBackgroundCalibration", 6)?;
        value.serialize_field("metersPerPixel", &self.meters_per_pixel)?;
        value.serialize_field("originX", &self.origin_x)?;
        value.serialize_field("originY", &self.origin_y)?;
        value.serialize_field("widthPx", &self.width_px)?;
        value.serialize_field("heightPx", &self.height_px)?;
        value.serialize_field("convention", &self.convention)?;
        value.end()
    }
}
impl MapBackgroundCalibration {
    /// Native MondeVersPixel. Conversion truncates toward zero, including for
    /// negative fractional coordinates; this is intentionally not floor().
    /// Dimensions and returned coordinates retain native 64-bit int width on WASM.
    pub fn world_to_pixel(&self, x: f64, y: f64) -> (i64, i64, bool) {
        if self.meters_per_pixel <= 0.0 {
            return (0, 0, false);
        }
        // Rust's saturating float-to-int conversion matches the pinned arm64
        // reference, including infinities and NaN. No finite-input guard is added.
        let px = ((x - self.origin_x) / self.meters_per_pixel) as i64;
        let py = ((self.origin_y - y) / self.meters_per_pixel) as i64;
        (
            px,
            py,
            px >= 0 && px < self.width_px && py >= 0 && py < self.height_px,
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn native_map_background_world_to_pixel() {
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(
            include_bytes!("fixtures/map-background-calibration-v41.json.zlib").as_slice(),
        )
        .read_to_end(&mut raw)
        .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&raw).unwrap();
        assert_eq!(rows.len(), 3465);
        for (i, row) in rows.iter().enumerate() {
            let n = |j| f64::from_bits(row["bits"][j].as_u64().unwrap());
            let c = MapBackgroundCalibration {
                meters_per_pixel: n(0),
                origin_x: n(1),
                origin_y: n(2),
                width_px: row["width"].as_i64().unwrap(),
                height_px: row["height"].as_i64().unwrap(),
                convention: row["convention"].as_str().unwrap().into(),
            };
            let expected = (
                row["px"].as_i64().unwrap(),
                row["py"].as_i64().unwrap(),
                row["ok"].as_bool().unwrap(),
            );
            assert_eq!(c.world_to_pixel(n(3), n(4)), expected, "case {i}");
            assert_eq!(
                serde_json::to_value(&c).is_err(),
                row["marshal_error"].as_bool().unwrap(),
                "JSON refusal {i}"
            );
            if [c.meters_per_pixel, c.origin_x, c.origin_y]
                .iter()
                .all(|x| x.is_finite())
            {
                let restored: MapBackgroundCalibration =
                    serde_json::from_value(serde_json::to_value(&c).unwrap()).unwrap();
                assert_eq!(restored, c, "storage {i}");
                let native: MapBackgroundCalibration =
                    serde_json::from_value(row["calibration"].clone()).unwrap();
                assert_eq!(restored, native, "native fields {i}");
                for (actual, expected) in [
                    restored.meters_per_pixel,
                    restored.origin_x,
                    restored.origin_y,
                ]
                .into_iter()
                .zip([native.meters_per_pixel, native.origin_x, native.origin_y])
                {
                    assert_eq!(
                        actual.to_bits(),
                        expected.to_bits(),
                        "native float bits {i}"
                    );
                }
                assert_eq!(
                    restored.world_to_pixel(n(3), n(4)),
                    expected,
                    "restored {i}"
                );
            }
        }
    }
}
