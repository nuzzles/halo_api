//! Object-relative positions use the referenced object's frame, not map quantization bounds.
use super::*;

#[derive(Default)]
pub(super) struct ReferenceFrames(BTreeMap<u32, ReferenceFrame>);

struct ReferenceFrame {
    origin: [f64; 3],
    axes: [[f64; 3]; 3],
}

impl ReferenceFrames {
    pub(super) fn baseline(&mut self, record: &KeyframeRecord) {
        self.0
            .retain(|id, _| id & 0x3fff_ffff != record.id & 0x3fff_ffff);
        if record.stop != KeyframeStop::Complete {
            return;
        }
        let fields = |name: &str| {
            let mut matches = record.components.iter().filter(|c| c.name == name);
            let c = matches.next()?;
            (matches.next().is_none() && c.status == ComponentReadStatus::Complete)
                .then_some(c.fields.as_slice())
        };
        let Some(frame) = (|| {
            let p = fields("object-position-component")?;
            if value(p, "high_precision", 1)? != 0
                || value(fields("object-parent-state-component")?, "attached", 1)? != 0
                || value(fields("object-scale-component")?, "default", 1)? != 1
            {
                return None;
            }
            Some(ReferenceFrame {
                origin: raw_vector(p, "vector_bits", 0)?,
                axes: orientation(fields("object-forward-and-up-dynamic-precision-component")?)?,
            })
        })() else {
            return;
        };
        self.0.insert(record.id, frame);
    }

    pub(super) fn observe(&mut self, record: &EntityRecord) {
        let Some(id) = record.header.id else {
            return;
        };
        // A changed transform or a reused slot needs another proven full-state baseline.
        if record.header.kind != RecordKind::Delta
            || record.components.iter().any(|c| {
                matches!(
                    c.name.as_str(),
                    "object-position-component"
                        | "object-forward-and-up-dynamic-precision-component"
                        | "object-parent-state-component"
                        | "object-scale-component"
                )
            })
        {
            self.0
                .retain(|key, _| key & 0x3fff_ffff != id & 0x3fff_ffff);
        }
    }

    pub(super) fn position(&self, record: &EntityRecord) -> Option<[f64; 3]> {
        let f = &component(record, "object-position-dynamic-precision-component")?.fields;
        if value(f, "use_prediction", 1)? != 0
            || value(f, "delta", 1)? != 0
            || value(f, "precision_high", 1)? != 1
            || value(f, "position_handle.present", 1)? != 1
        {
            return None;
        }
        // FUN_140d10bb0 sets the category-0 reference base to 0x200.
        let slot = value(f, "position_handle.value", 13)? as u32 + 0x200;
        let generation = value(f, "position_handle.generation", 2)? as u32;
        let frame = self.0.get(&(slot | generation << 30))?;
        value(f, "finite", 2)?;
        let mut local = [0.; 3];
        for (axis, coordinate) in local.iter_mut().enumerate() {
            let raw = value(f, &format!("bounded_position[{axis}]"), 14)?;
            *coordinate = -100. + (raw as f64 + 0.5) * 200. / 16384.;
        }
        Some(std::array::from_fn(|axis| {
            frame.origin[axis] + (0..3).map(|i| local[i] * frame.axes[i][axis]).sum::<f64>()
        }))
    }
}

fn raw_vector(fields: &[ComponentField], prefix: &str, offset: usize) -> Option<[f64; 3]> {
    let mut result = [0.; 3];
    for (i, v) in result.iter_mut().enumerate() {
        let raw = value(fields, &format!("{prefix}[{}]", offset + i), 32)? as u32;
        *v = f64::from(f32::from_bits(raw.swap_bytes()));
        if !v.is_finite() {
            return None;
        }
    }
    Some(result)
}

fn orientation(fields: &[ComponentField]) -> Option<[[f64; 3]; 3]> {
    if value(fields, "raw_vectors", 1)? != 0 {
        let forward = raw_vector(fields, "orientation_vector_bits", 0)?;
        let up = raw_vector(fields, "orientation_vector_bits", 3)?;
        let left = cross(up, forward);
        normalize(left)?;
        return Some([forward, left, up]);
    }
    if value(fields, "delta", 1)? != 0 {
        return None;
    }
    let width = if value(fields, "config_precision", 1) == Some(1) {
        30
    } else {
        19
    };
    let up = if value(fields, "up_direction.gate", 1)? == 1 {
        [0., 0., 1.]
    } else {
        direction(value(fields, "up_direction", width)? as u32, width)?
    };
    let roll_width = if width == 30 { 30 } else { 8 };
    let roll = (value(fields, "roll", roll_width)? as f64 + 0.5) * std::f64::consts::TAU
        / (1_u64 << roll_width) as f64
        - std::f64::consts::PI;
    let base = if up[0].abs() < up[1].abs() {
        cross(up, [1., 0., 0.])
    } else {
        cross([0., 1., 0.], up)
    };
    let base = normalize(base)?;
    let perpendicular = cross(up, base);
    let forward = normalize(std::array::from_fn(|i| {
        base[i] * roll.cos() + perpendicular[i] * roll.sin()
    }))?;
    Some([forward, cross(up, forward), up])
}

fn direction(code: u32, width: usize) -> Option<[f64; 3]> {
    let (face_size, grid) = match width {
        19 => (87_381, 294),
        30 => (178_956_970, 13_376),
        _ => return None,
    };
    let coord = |i: u32| {
        if i * 2 == grid - 2 {
            0.
        } else {
            2. * (f64::from(i) + 0.5) / f64::from(grid - 1) - 1.
        }
    };
    let rem = code % face_size;
    let (a, b) = (coord(rem / grid), coord(rem % grid));
    normalize(match code / face_size {
        0 => [1., a, b],
        1 => [a, 1., b],
        2 => [a, b, 1.],
        3 => [-1., a, b],
        4 => [a, -1., b],
        5 => [a, b, -1.],
        _ => return None,
    })
}
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn normalize(v: [f64; 3]) -> Option<[f64; 3]> {
    let norm = v.iter().map(|x| x * x).sum::<f64>().sqrt();
    (norm > 0. && norm.is_finite()).then(|| v.map(|x| x / norm))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theater::parser::test_support::recharge_device_baseline;

    fn position_record() -> EntityRecord {
        let fields = [
            ("use_prediction", 1, 0),
            ("delta", 1, 0),
            ("precision_high", 1, 1),
            ("bounded_position[0]", 14, 8288),
            ("bounded_position[1]", 14, 8138),
            ("bounded_position[2]", 14, 8257),
            ("position_handle.present", 1, 1),
            ("position_handle.value", 13, 768),
            ("position_handle.generation", 2, 1),
            ("position_region.gate", 1, 1),
            ("position_region", 11, 1),
            ("finite", 2, 0),
        ]
        .into_iter()
        .map(|(name, width, raw)| ComponentField {
            name: name.into(),
            bit: 0,
            width,
            raw: RawBits::from_low(raw, width),
        })
        .collect();
        EntityRecord {
            header: RecordHeader {
                prefix: None,
                kind: RecordKind::Delta,
                id: Some(1073742356),
                start_bit: 822,
                end_bit: 838,
            },
            archetype: None,
            mask: Some(1),
            fields: Vec::new(),
            default_state: None,
            components: vec![EntityComponentRead {
                index: 0,
                name: "object-position-dynamic-precision-component".into(),
                start_bit: 867,
                end_bit: 942,
                status: ComponentReadStatus::Complete,
                fields,
                weapon_id: None,
            }],
            end_bit: 942,
            stop: EntityViewStop::Complete,
        }
    }

    #[test]
    fn bounded_position_uses_recorded_frame_and_reference_generation() {
        let mut frames = ReferenceFrames::default();
        let mut record = position_record();
        assert!(frames.position(&record).is_none());
        frames.baseline(&recharge_device_baseline());
        let position = frames.position(&record).unwrap();
        assert!((position[0] - 20.809).abs() < 0.01);
        assert!((position[1] - 11.644).abs() < 0.01);
        assert!((position[2] - 2.8795604705810547).abs() < 1e-9);
        let generation = record.components[0]
            .fields
            .iter_mut()
            .find(|f| f.name == "position_handle.generation")
            .unwrap();
        generation.raw = RawBits::from_low(2, 2);
        assert!(frames.position(&record).is_none());
    }

    #[test]
    fn reused_or_changed_reference_frames_are_not_reused() {
        let mut frames = ReferenceFrames::default();
        let baseline = recharge_device_baseline();
        let position = position_record();
        for kind in [RecordKind::New, RecordKind::Delete, RecordKind::Delta] {
            frames.baseline(&baseline);
            let mut change = position_record();
            change.header.id = Some(baseline.id);
            change.header.kind = kind;
            change.components[0].name = "object-position-component".into();
            frames.observe(&change);
            assert!(frames.position(&position).is_none());
        }
    }

    #[test]
    fn unhandled_parent_scale_and_nonfinite_baselines_stay_unknown() {
        for (component, name, raw, width) in [
            ("object-parent-state-component", "attached", 1, 1),
            ("object-scale-component", "default", 0, 1),
            (
                "object-position-component",
                "vector_bits[0]",
                u64::from(f32::NAN.to_bits().swap_bytes()),
                32,
            ),
        ] {
            let mut baseline = recharge_device_baseline();
            let field = baseline
                .components
                .iter_mut()
                .find(|c| c.name == component)
                .unwrap()
                .fields
                .iter_mut()
                .find(|f| f.name == name)
                .unwrap();
            field.raw = RawBits::from_low(raw, width);
            let mut frames = ReferenceFrames::default();
            frames.baseline(&baseline);
            assert!(frames.position(&position_record()).is_none());
        }
    }

    #[test]
    fn raw_orientation_rejects_degenerate_frames() {
        let fields =
            |forward: [f32; 3], up: [f32; 3]| {
                let mut fields = vec![ComponentField {
                    name: "raw_vectors".into(),
                    bit: 0,
                    width: 1,
                    raw: RawBits::from_low(1, 1),
                }];
                fields.extend(forward.into_iter().chain(up).enumerate().map(|(i, v)| {
                    ComponentField {
                        name: format!("orientation_vector_bits[{i}]"),
                        bit: 0,
                        width: 32,
                        raw: RawBits::from_low(u64::from(v.to_bits().swap_bytes()), 32),
                    }
                }));
                fields
            };
        assert_eq!(
            orientation(&fields([1., 0., 0.], [0., 0., 1.])),
            Some([[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]])
        );
        assert!(orientation(&fields([0.; 3], [0., 0., 1.])).is_none());
        assert!(orientation(&fields([0., 0., 1.], [0., 0., 1.])).is_none());
        assert!(orientation(&fields([f32::NAN, 0., 0.], [0., 0., 1.])).is_none());
    }
}
