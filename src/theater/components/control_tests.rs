use super::super::Cursor;
use super::*;
use crate::theater::*;
use serde_json::{Value, json};
use std::io::Read;
fn bytes(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
fn action_json(a: &NativeActionBlock) -> Value {
    json!({"Present":a.present,"Gachettes":a.triggers,"Barillets":a.barrels,"Arme":a.weapons})
}
fn view_json(v: &DecodedFrameView) -> Value {
    let entries:Vec<_>=v.control_entries.iter().map(|e|{
        let absent=|v:Option<u8>|v.map_or(-1,i32::from);
        let action=e.action.as_ref().map(action_json).unwrap_or(json!({"Present":false,"Gachettes":[0,0],"Barillets":[0,0],"Arme":[0,0]}));
        json!({"Index":e.index,"Bloc":e.action.is_some(),"Action":action,
            "Champs":{"Cdc04":absent(e.baseline),"Court":absent(e.short),"Troisieme":absent(e.third_analog),"Champ10":absent(e.extra),"Drapeaux":absent(e.flags),"Analogique":e.analog.map_or([-1,-1],|a|a.map(i32::from))}})
    }).collect();
    json!(entries)
}
fn check_view(v: &DecodedFrameView, expected: &Value, end: i64) {
    assert_eq!(v.end_bit, end);
    assert_eq!(
        v.stop == FrameViewStop::Complete,
        expected["Porte"].as_bool().unwrap()
    );
    let stop = match &v.stop {
        FrameViewStop::Complete => 0,
        FrameViewStop::Truncated => 1,
        FrameViewStop::RecordLimit => 4,
        FrameViewStop::Unsupported { reason } => {
            if reason == "secondary control block" {
                3
            } else {
                2
            }
        }
    };
    assert_eq!(json!(stop), expected["Arret"]);
    assert_eq!(
        json!(v.kinds),
        expected["Kinds"].as_array().map_or(json!([]), |v| json!(v))
    );
    assert_eq!(
        view_json(v),
        expected["Entrees"]
            .as_array()
            .map_or(json!([]), |v| json!(v))
    );
}
#[test]
fn native_action_control_d61443e() {
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(
        include_bytes!("../fixtures/action-control-d61443e-v41.json.zlib").as_slice(),
    )
    .read_to_end(&mut raw)
    .unwrap();
    let cases: Vec<Value> = serde_json::from_slice(&raw).unwrap();
    assert_eq!(cases.len(), 768);
    let mut modes = [0; 4];
    let mut targets = [0; 4];
    let mut weapons = [0; 3];
    for (i, c) in cases.iter().enumerate() {
        let data = bytes(c["input"].as_str().unwrap());
        let start = c["start"].as_u64().unwrap() as usize;
        let encoding: PositionEncoding = serde_json::from_value(c["encoding"].clone()).unwrap();
        let mut r = Reader {
            native_widths: None,
            width_error: None,
            live_observer: None,
            live_grammar: None,
            position_capture: None,
            position_start: 0,
            position_slot: 0,
            position_fallback: false,
            movement_slot: None,
            references: vec![],
            diagnostics: Default::default(),
            cursor: Cursor::new(&data, start).unwrap(),
            fields: vec![],
            position_encoding: Some(&encoding),
        };
        let ActionBlockRead::Decoded(block) = actions(&mut r).unwrap() else {
            panic!("fixture supplies position context");
        };
        assert_eq!(action_json(&block), c["action"], "action {i}");
        assert_eq!(json!(r.cursor.position), c["end"], "end {i}");
        assert_eq!(
            crate::theater::bits::Bits(&data).read(r.cursor.position as usize, 8),
            c["next"].as_u64()
        );
        let mut at = start as i64;
        for f in &r.fields {
            assert_eq!(f.bit, at);
            assert_eq!(
                crate::theater::bits::Bits(&data).read(f.bit as usize, f.width as usize),
                Some(f.raw)
            );
            at += f.width as i64;
            if f.name == "actions.aim.vector_mode" {
                modes[f.raw as usize] += 1;
            }
            if f.name == "actions.tail.mode" {
                targets[f.raw as usize] += 1;
            }
        }
        assert_eq!(at, r.cursor.position);
        let indices: Option<std::collections::BTreeMap<i32, u64>> =
            serde_json::from_value(c["indices"].clone()).unwrap();
        assert_eq!(
            r.diagnostics.absolute_indices,
            indices.clone().unwrap_or_default()
        );
        for w in block.weapons {
            weapons[if w == -2 {
                0
            } else if w == -1 {
                1
            } else {
                2
            }] += 1;
        }
        let (ported, component) = decode_native_component(
            &data,
            start,
            "unit-actor-control-component",
            c["level"].as_u64().unwrap() as u32,
            35,
            Some(&encoding),
        );
        assert_eq!(ported, Some(c["ported"].as_bool().unwrap()));
        assert_eq!(
            json!(component.end_bit),
            c["component_end"],
            "component {i}"
        );
        let control = bytes(c["control"].as_str().unwrap());
        let view = super::super::views::decode_control_view_contextual(
            &control,
            start as i64,
            Some(&encoding),
            None,
        );
        check_view(&view, &c["view"], c["control_end"].as_i64().unwrap());
        let observer = NativeFilmObserver::default();
        let mut context = NativeReaderContext {
            observer: Some(observer.clone()),
            ..Default::default()
        };
        context.profile.movement.world_object.index_bits = encoding.index_bits as u64;
        context.profile.movement.world_object.axis_bits =
            encoding.world_axis_bits.unwrap().map(|v| v as u64);
        context.profile.movement.full_precision = encoding.full_precision;
        context.profile.grammar.baseline_scope = encoding.baseline_scope;
        let contextual = super::super::views::decode_control_view_contextual(
            &control,
            start as i64,
            Some(&encoding),
            Some(&context),
        );
        assert_eq!(contextual, view);
        assert_eq!(
            observer.take_absolute_indices(),
            indices.unwrap_or_default()
        );

        assert_eq!(
            serde_json::from_slice::<DecodedFrameView>(&serde_json::to_vec(&view).unwrap())
                .unwrap(),
            view
        );
        if let Some(mode) = r
            .fields
            .iter()
            .find(|f| f.name == "actions.aim.vector_mode" && f.raw == 0)
            .cloned()
        {
            let prefix = r
                .fields
                .iter()
                .take_while(|f| f.bit <= mode.bit)
                .cloned()
                .collect::<Vec<_>>();
            r.position_encoding = None;
            r.cursor = Cursor::new(&data, start).unwrap();
            r.fields.clear();
            assert!(matches!(
                actions(&mut r),
                Some(ActionBlockRead::MissingPositionContext)
            ));
            assert_eq!(r.cursor.position, mode.bit + mode.width as i64);
            assert_eq!(r.fields, prefix);
        }
        for p in c["prefixes"].as_array().unwrap() {
            let cut = p["cut"].as_u64().unwrap() as usize;
            let v = super::super::views::decode_control_view_contextual(
                &control[..cut],
                start as i64,
                Some(&encoding),
                None,
            );
            check_view(&v, &p["view"], p["end"].as_i64().unwrap());
        }
    }
    assert!(modes.iter().all(|&n| n > 0));
    assert!(targets.iter().all(|&n| n > 0));
    assert!(weapons.iter().all(|&n| n > 0));
}

#[test]
fn actor_action_missing_context_is_not_truncated_source() {
    // Hand-built recorded prefix: stationary actor, absent word reference and
    // fraction, followed by an action aim with absolute-position mode (0).
    let actor = "000 0 0 000000 1 0000000000000000000 0 1";
    let aim = "1 0 0 1 00 0 0 00";
    let prefix: String = format!("{actor} {aim}")
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    let mut data = vec![0; 64];
    for (i, bit) in prefix.bytes().enumerate() {
        if bit == b'1' {
            data[i / 8] |= 1 << (7 - i % 8);
        }
    }
    let name = "unit-actor-control-component";
    let (status, parsed) = super::super::decode_component_attempt(&data, 0, name, 1, 35, None);
    assert_eq!(status, Some(false));
    assert_eq!(parsed.end_bit, prefix.len() as i64);
    assert_eq!(
        parsed.fields.last().unwrap().name,
        "actions.aim.vector_mode"
    );
    assert_eq!(
        decode_component(&data, 0, name, 1, 35),
        ComponentDecode::Unsupported
    );
    let (native_status, native) = decode_native_component(&data, 0, name, 1, 35, None);
    assert_eq!(native_status, Some(false));
    assert_eq!(native, parsed);
    // Removing recorded bytes before the mode really is truncation.
    assert!(matches!(
        decode_component(&data[..2], 0, name, 1, 35),
        ComponentDecode::Truncated { .. }
    ));
}
