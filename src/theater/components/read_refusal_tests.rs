use super::*;
use crate::theater::{NativeReadOperation, NativeReadRefusal};

fn pack(bits: &str) -> Vec<u8> {
    let mut data = vec![0; bits.len().div_ceil(8)];
    for (i, b) in bits.bytes().enumerate() {
        data[i / 8] |= (b - b'0') << (7 - i % 8);
    }
    data
}
#[test]
fn bounded_read_refusals_preserve_component_prefix_and_request() {
    let (status, component) =
        decode_component_attempt(&[0xab], 0, "player-engine-loadout-component", 0, 0, None);
    assert_eq!(status, None);
    assert_eq!(component.end_bit, 8);
    assert_eq!(component.fields.len(), 1);
    assert_eq!(component.fields[0].raw, 0xab);
    assert_eq!(
        component.diagnostics.read_refusals,
        [NativeReadRefusal {
            field: "loadout[1]".into(),
            bit: 8,
            width: 8,
            source_bits: 8,
            operation: NativeReadOperation::Scalar,
        }]
    );
    let mut merged = crate::theater::FilmReadDiagnostics::default();
    assert!(merged.is_empty());
    merged.merge(&component.diagnostics);
    assert!(!merged.is_empty());
    assert_eq!(merged, component.diagnostics);
    assert_eq!(
        component,
        serde_json::from_slice(&serde_json::to_vec(&component).unwrap()).unwrap()
    );
}
#[test]
fn bounded_read_refusals_distinguish_control_groups() {
    let scalar = decode_control_view(&[0x80], 0);
    assert_eq!(scalar.end_bit, 4);
    assert_eq!(
        scalar.diagnostics.as_ref().unwrap().read_refusals,
        [NativeReadRefusal {
            field: "control.index_and_input".into(),
            bit: 4,
            width: 6,
            source_bits: 8,
            operation: NativeReadOperation::GroupGuard,
        }]
    );
    // present, kind=0, baseline gate=0, index=0, input=1, second-field gate=0.
    // The following two analog values plus presence bit require 13 bits together.
    let grouped = decode_control_view(&pack("10000000010"), 0);
    assert_eq!(grouped.end_bit, 11);
    assert_eq!(
        grouped.diagnostics.as_ref().unwrap().read_refusals,
        [NativeReadRefusal {
            field: "control.input.analog_group".into(),
            bit: 11,
            width: 13,
            source_bits: 16,
            operation: NativeReadOperation::GroupGuard,
        }]
    );
    assert!(
        grouped
            .fields
            .iter()
            .all(|f| !f.name.starts_with("control.input.analog"))
    );
    for view in [scalar, grouped] {
        assert_eq!(view.stop, FrameViewStop::Truncated);
        assert_eq!(
            view,
            serde_json::from_slice(&serde_json::to_vec(&view).unwrap()).unwrap()
        );
        let mut old = serde_json::to_value(&view).unwrap();
        old.as_object_mut().unwrap().remove("diagnostics");
        assert!(
            serde_json::from_value::<DecodedFrameView>(old)
                .unwrap()
                .diagnostics
                .is_none()
        );
    }
    let unsupported = decode_message_view(&[0x80], 0);
    assert!(matches!(
        unsupported.stop,
        FrameViewStop::Unsupported { .. }
    ));
    assert!(unsupported.diagnostics.is_none());
    let complete = decode_message_view(&[0], 0);
    assert_eq!(complete.stop, FrameViewStop::Complete);
    assert!(complete.diagnostics.is_none());
}
