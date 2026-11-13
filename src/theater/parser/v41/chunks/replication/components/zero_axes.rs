use super::{
    cursor::ComponentCursor,
    reader::{ComponentReader, ComponentWidths},
    widths::read_component,
};
use crate::theater::parser::v41::chunks::replication::source::EntityViewStop;
use crate::theater::parser::v41::chunks::replication::source::components::field::RawBits;
use crate::theater::{film::catalog::maps::zero_fixtures, parser::v41::context::V41DecodeConfig};

#[derive(Default)]
struct Wire(Vec<bool>);
impl Wire {
    fn put(&mut self, n: usize, v: u64) {
        self.0.extend((0..n).rev().map(|i| v & (1 << i) != 0));
    }
    fn append(&mut self, source: &[u8], n: usize) {
        for i in 0..n {
            self.put(1, ((source[i / 8] >> (7 - i % 8)) & 1).into());
        }
    }
    fn bytes(&self) -> Vec<u8> {
        let mut b = vec![0; self.0.len().div_ceil(8)];
        for (i, v) in self.0.iter().enumerate() {
            b[i / 8] |= u8::from(*v) << (7 - i % 8)
        }
        b
    }
    fn cut(&self, n: usize) -> (Vec<u8>, usize) {
        let shift = (8 - n % 8) % 8;
        let mut w = Wire(vec![false; shift]);
        w.0.extend(&self.0[..n]);
        (w.bytes(), shift)
    }
}
fn config(c: &zero_fixtures::Case) -> V41DecodeConfig {
    let mut config = V41DecodeConfig::default();
    config.profile.movement.world_object.index_bits = 2;
    config.profile.movement.world_object.region = 3;
    config.profile.movement.world_object.axis_bits = c.widths.map(|w| w as u64);
    config.profile.movement.world_object.regions = [(3, c.widths.map(|w| w as u64))].into();
    config
}
fn body(c: &zero_fixtures::Case, dynamic: bool) -> Wire {
    let mut w = Wire::default();
    w.put(if dynamic { 3 } else { 1 }, 0);
    w.append(&c.bytes(), c.bit_len);
    w.put(2, 3);
    w
}
fn name(dynamic: bool) -> &'static str {
    if dynamic {
        "object-position-dynamic-precision-component"
    } else {
        "object-position-component"
    }
}

#[test]
fn zero_axes_keep_explicit_fields_boundaries_and_every_bit_truncation() {
    for c in zero_fixtures::cases()
        .into_iter()
        .filter(|c| c.precision == 16)
    {
        for dynamic in [false, true] {
            let config = config(&c);
            let enc = config.profile.component_encoding().unwrap();
            assert!(enc.valid());
            let read = |data: &[u8], at: usize| {
                let mut r = ComponentReader {
                    category_references: true,
                    snapshot: false,
                    reference_widths: Some(ComponentWidths {
                        movement: &config.profile.movement,
                        mpp: config.profile.mpp,
                        maximum: u64::MAX,
                    }),
                    width_error: None,
                    context_error: None,
                    live_grammar: None,
                    movement_slot: None,
                    references: vec![],
                    diagnostics: Default::default(),
                    cursor: ComponentCursor::guarded(data, at as i64),
                    fields: vec![],
                    position_encoding: Some(&enc),
                };
                let status = read_component(&mut r, name(dynamic), 1, 42, None, (true, None));
                (status, r.cursor.position, r.fields)
            };
            let mut w = body(&c, dynamic);
            let end = w.0.len();
            w.put(8, 0xd3);
            let (status, at, fields) = read(&w.bytes(), 0);
            assert_eq!(status, Some(true), "{}", c.name);
            assert_eq!(at as usize, end);
            for axis in 0..3 {
                let f = fields
                    .iter()
                    .find(|f| f.name == format!("position[{axis}]"))
                    .unwrap();
                assert_eq!(
                    (f.width, f.raw.low_u64()),
                    (c.widths[axis], u64::from(c.codes[axis]))
                );
                if f.width == 0 {
                    assert_eq!(f.raw, RawBits::from_low(0, 0));
                }
            }
            let prefix = if dynamic { 3 } else { 1 };
            assert_eq!(
                fields
                    .iter()
                    .skip(prefix)
                    .take(5)
                    .map(|f| (f.bit - prefix, f.width))
                    .collect::<Vec<_>>(),
                c.trace
            );
            assert_eq!(
                RawBits::from_source(&w.bytes(), end, 8).unwrap().low_u64(),
                211
            );
            for cut in 0..end {
                let (b, shift) = w.cut(cut);
                let (status, at, got) = read(&b, shift);
                assert_eq!(status, None, "{} cut {cut}", c.name);
                assert!(at as usize <= b.len() * 8);
                // A zero-width axis is retained only after all earlier reads succeed.
                let expected: Vec<_> = fields
                    .iter()
                    .take_while(|f| f.bit + f.width <= cut)
                    .collect();
                assert_eq!(got.len(), expected.len(), "{} cut {cut}", c.name);
                for (g, e) in got.iter().zip(expected) {
                    assert_eq!(
                        (&g.name, g.bit, g.width, &g.raw),
                        (&e.name, e.bit + shift, e.width, &e.raw)
                    );
                }
            }
        }
    }
}

#[test]
fn zero_axes_keep_following_delta_components_and_unknown_region_refusals() {
    use crate::theater::parser::{
        test_support::component_registry,
        v41::chunks::replication::replication_stream::frame::records::{
            RecordBinding, RecordReadContext, decode_entity_record,
        },
    };
    for c in zero_fixtures::cases()
        .into_iter()
        .filter(|c| c.precision == 16)
    {
        for dynamic in [false, true] {
            for unknown in [false, true] {
                let mut config = config(&c);
                if unknown {
                    config.profile.movement.world_object.regions = [(2, [1; 3])].into();
                }
                let registry = component_registry(&[(
                    42,
                    vec![
                        (name(dynamic).into(), 1),
                        ("equipment-being-hacked-component".into(), 1),
                    ],
                )]);
                let mut w = Wire::default();
                for (n, v) in [(1, 1), (13, 43), (2, 0), (1, 0), (1, 1), (64, 3)] {
                    w.put(n, v)
                }
                let header = w.0.len();
                w.0.extend(body(&c, dynamic).0);
                let marker = w.0.len();
                w.put(8, 0xd3);
                let r = decode_entity_record(
                    &w.bytes(),
                    0,
                    &registry,
                    &config.record_layout(),
                    Some(RecordBinding {
                        id: 43,
                        archetype: 42,
                    }),
                    true,
                    RecordReadContext {
                        context: Some(&config.profile),
                        movement: None,
                    },
                )
                .unwrap();
                if unknown {
                    assert!(matches!(
                        r.stop,
                        EntityViewStop::UnsupportedComponent { .. }
                    ));
                    assert_eq!(r.components.len(), 1);
                    assert_eq!(r.end_bit as usize, header + if dynamic { 6 } else { 4 });
                    assert!(
                        !r.components[0]
                            .fields
                            .iter()
                            .any(|f| f.name.starts_with("position["))
                    );
                } else {
                    assert_eq!(r.stop, EntityViewStop::Complete);
                    assert_eq!(r.end_bit as usize, marker + 8);
                    assert_eq!(r.components.len(), 2);
                    assert_eq!(r.components[1].fields[0].raw.low_u64(), 211);
                }
            }
        }
    }
}
