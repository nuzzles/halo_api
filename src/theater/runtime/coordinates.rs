//! World-coordinate conversion with bounds selected by [`crate::theater::film::maps`].
//!
//! Film position fields contain quantized values, not map bounds. A completed
//! component read alone does not establish that its map-specific widths were correct.
use super::{SourceRef, world::ComponentState};

pub use crate::theater::film::maps::PositionBounds;

/// One supported absolute position read and its canonical record reference.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldPosition {
    pub value: [f64; 3],
    pub quantized: [u32; 3],
    pub source: SourceRef,
}

impl PositionBounds {
    /// Resolve the ordinary absolute branch of the dynamic position component.
    /// Prediction, delta, default-region and high-precision branches require their
    /// own state/context and are refused here. This never fabricates a baseline.
    pub fn absolute_component(&self, component: &ComponentState) -> Option<WorldPosition> {
        if !component.complete || component.name != "object-position-dynamic-precision-component" {
            return None;
        }
        let field = |name: &str| {
            let mut matches = component.fields.iter().filter(|f| f.name == name);
            let first = matches.next()?;
            (matches.next().is_none()
                && (1..=32).contains(&first.width)
                && first.raw.bit_len == first.width
                && first.raw.bytes.len() == first.width.div_ceil(8))
            .then_some(first)
        };
        for name in [
            "use_prediction",
            "delta",
            "precision_high",
            "default_region",
        ] {
            let gate = field(name)?;
            if gate.width != 1 || gate.raw.low_u64() != 0 {
                return None;
            }
        }
        if field("region_index")?.raw.low_u64() != u64::from(self.region) {
            return None;
        }
        // The two-bit tail belongs to the absolute-position grammar; its value
        // does not encode whether the three quantized coordinates are valid.
        let finite = field("finite")?;
        if finite.width != 2 {
            return None;
        }
        let mut raw = [0; 3];
        let mut widths = [0; 3];
        for axis in 0..3 {
            let value = field(&format!("position[{axis}]"))?;
            raw[axis] = value.raw.low_u64() as u32;
            widths[axis] = value.width;
        }
        Some(WorldPosition {
            value: self.dequantize(raw, widths)?,
            quantized: raw,
            source: component.source,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theater::{
        film::{ComponentField, RawBits},
        runtime::RecordRef,
    };
    const BOUNDS: PositionBounds = PositionBounds {
        region: 0,
        min: [-1.; 3],
        max: [1.; 3],
    };
    fn component() -> ComponentState {
        let fields = [
            ("use_prediction", 1, 0),
            ("delta", 1, 0),
            ("precision_high", 1, 0),
            ("default_region", 1, 0),
            ("region_index", 1, 0),
            ("position[0]", 7, 0),
            ("position[1]", 7, 64),
            ("position[2]", 7, 127),
            ("finite", 2, 0),
        ]
        .into_iter()
        .map(|(name, width, value)| ComponentField {
            name: name.into(),
            bit: 0,
            width,
            raw: RawBits::from_low(value, width),
        })
        .collect();
        ComponentState {
            name: "object-position-dynamic-precision-component".into(),
            fields,
            complete: true,
            source: SourceRef {
                chunk: 1,
                packet: 2,
                record: RecordRef::Entity { index: 0 },
            },
        }
    }
    #[test]
    fn module_bounds_match_independently_recorded_fixture_widths() {
        // LevelUp d61443e, data/titles/halo_infinite/reference/map_quant_bounds.json.
        // These widths also match the independently investigated film partitions.
        let octagon = PositionBounds {
            region: 0,
            min: [-231.00145, -227.08232, -946.215],
            max: [231.63503, 226.34886, 242.32658],
        };
        let recharge = PositionBounds {
            region: 0,
            min: [-993.9214, -1320.3536, -250.0],
            max: [1713.4697, 1320.3536, 250.0],
        };
        assert_eq!(octagon.axis_bits(), Some([15, 15, 17]));
        assert_eq!(octagon.axis_bits_at_precision(15), Some([14, 14, 16]));
        assert_eq!(octagon.axis_bits_at_precision(12), Some([11, 11, 13]));
        assert_eq!(recharge.axis_bits(), Some([18, 18, 15]));
        // The current parser's default map profile must never be used for either.
        assert!(octagon.dequantize([0; 3], [13, 13, 14]).is_none());
        assert!(recharge.dequantize([0; 3], [13, 13, 14]).is_none());
    }

    #[test]
    fn lower_precision_recomputes_widths_before_the_quantization_cap() {
        let bounds = PositionBounds {
            region: 0,
            min: [0.; 3],
            max: [1_000_000.; 3],
        };
        assert_eq!(bounds.axis_bits(), Some([22; 3]));
        assert_eq!(bounds.axis_bits_at_precision(15), Some([22; 3]));
        assert_eq!(bounds.axis_bits_at_precision(12), Some([22; 3]));
    }

    #[test]
    fn midpoint_bounds_and_source_are_retained() {
        let c = component();
        let p = BOUNDS.absolute_component(&c).unwrap();
        assert_eq!(p.value, [-0.9921875, 0.0078125, 0.9921875]);
        assert_eq!(p.source, c.source);
        assert_eq!(p.quantized, [0, 64, 127]);
    }
    #[test]
    fn absolute_position_accepts_nonzero_tail() {
        let mut c = component();
        let original = BOUNDS.absolute_component(&c).unwrap();
        c.fields.last_mut().unwrap().raw = RawBits::from_low(3, 2);
        assert_eq!(BOUNDS.absolute_component(&c).unwrap(), original);
    }
    #[test]
    fn wrong_map_widths_and_out_of_range_values_are_refused() {
        assert!(BOUNDS.dequantize([0; 3], [13, 13, 14]).is_none());
        assert!(BOUNDS.dequantize([128, 0, 0], [7; 3]).is_none());
        assert!(
            PositionBounds {
                max: [f32::NAN; 3],
                ..BOUNDS
            }
            .axis_bits()
            .is_none()
        );
    }
    #[test]
    fn unsupported_branches_and_partial_reads_cannot_become_positions() {
        for gate in 0..4 {
            let mut c = component();
            c.fields[gate].raw = RawBits::from_low(1, 1);
            assert!(BOUNDS.absolute_component(&c).is_none());
        }
        let mut c = component();
        c.complete = false;
        assert!(BOUNDS.absolute_component(&c).is_none());
        c.complete = true;
        c.fields.push(c.fields[5].clone());
        assert!(BOUNDS.absolute_component(&c).is_none());
    }
}
