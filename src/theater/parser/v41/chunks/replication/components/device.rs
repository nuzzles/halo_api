//! Map device state, including animation and dispenser updates.
use super::reader::ComponentReader;

pub(crate) fn component(r: &mut ComponentReader<'_>, name: &str) -> Option<bool> {
    match name {
        "device-position-animation-name-component" => {
            r.r("animation", 32)?;
            r.r("position", 10)?;
        }
        "device-position-animation-control-component" => r.words("control", 4, 64)?,
        "device-position-group-component" => {
            r.r("group", 32)?;
            r.gate("index", 8, true)?;
        }
        "device-power-component" | "device-power-group-component" => {
            r.r("power", 14)?;
        }
        "device-interaction-in-progress-component" | "device-exclusive-user-component" => {
            r.optional_handle("user", 1)?
        }
        "device-interaction-hold-time-component"
        | "device-interaction-start-time-override-component" => {
            r.r("time", 8)?;
        }
        "device-control-action-string-override-component" => r.words("action", 2, 32)?,
        "device-health-station-charges-component" => {
            r.bit("flag")?;
            r.r("charges", 6)?;
        }
        "device-health-station-in-use-component"
        | "device-in-primary-mode-component"
        | "device-dispenser-require-los-component" => {
            r.bit("enabled")?;
        }
        "device-dispenser-monitors-changed-component" => {
            let count = r.r("count", 8)?;
            if count > 8 {
                return Some(false);
            }
            for i in 0..count {
                r.optional_handle(&format!("monitors[{i}]"), 1)?;
            }
        }
        "device-dispenser-state-flags-component" => {
            r.r("flags", 5)?;
        }
        "device-animation-layer-settings-component" => {
            if r.bit("present")? {
                for i in 0..8 {
                    let p = format!("layers[{i}]");
                    if r.bit(&format!("{p}.present"))? {
                        r.r(&format!("{p}.animation"), 32)?;
                        r.words(&format!("{p}.range"), 2, 14)?;
                        r.r(&format!("{p}.weight"), 10)?;
                        r.r(&format!("{p}.position"), 14)?;
                        r.r(&format!("{p}.flags"), 2)?;
                    }
                }
            }
        }
        "device-animation-layer-state-component" => {
            if r.bit("present")? {
                for i in 0..8 {
                    let p = format!("layers[{i}]");
                    if r.bit(&format!("{p}.present"))? && r.r(&format!("{p}.weight"), 10)? != 0 {
                        r.r(&format!("{p}.position"), 14)?;
                    }
                }
            }
        }
        "device-dispenser-state-component" => {
            for i in 0..2 {
                let p = format!("slots[{i}]");
                if r.bit(&format!("{p}.present"))? {
                    r.optional_handle(&format!("{p}.object"), 0)?;
                }
                r.r(&format!("{p}.state"), 3)?;
            }
        }
        "device-object-dispenser-timer-component" => {
            for i in 0..2 {
                let p = format!("timers[{i}]");
                if r.bit(&format!("{p}.present"))? {
                    r.words(&format!("{p}.values"), 2, 10)?;
                    r.r(&format!("{p}.flags"), 5)?;
                    r.r(&format!("{p}.remaining"), 10)?;
                }
            }
        }
        "device-position-transition-velocity-component" => {
            r.r("velocity", 18)?;
        }
        "device-machine-flags-component" => {
            r.r("flags", 9)?;
        }
        _ => return Some(false),
    }
    Some(true)
}

#[cfg(test)]
mod tests {
    use super::super::cursor::ComponentCursor;
    use super::*;

    fn reader(bytes: &[u8]) -> ComponentReader<'_> {
        ComponentReader {
            reference_widths: None,
            width_error: None,
            live_grammar: None,
            movement_slot: None,
            references: Vec::new(),
            diagnostics: Default::default(),
            cursor: ComponentCursor::new(bytes, 0).unwrap(),
            fields: Vec::new(),
            position_encoding: None,
        }
    }

    #[test]
    fn zero_weight_layer_omits_position_without_consuming_the_next_layer() {
        // Present collection, first layer with zero weight, second with weight 1
        // and position 7, then six absent layers and a trailing sentinel.
        let bits = format!("11{:010b}1{:010b}{:014b}00000010101", 0, 1, 7);
        let mut bytes = vec![0; bits.len().div_ceil(8)];
        for (i, b) in bits.bytes().enumerate() {
            bytes[i / 8] |= (b - b'0') << (7 - i % 8);
        }
        let mut reader = reader(&bytes);
        assert_eq!(
            component(&mut reader, "device-animation-layer-state-component"),
            Some(true)
        );
        assert_eq!(reader.cursor.position, 43);
        assert!(!reader.fields.iter().any(|f| f.name == "layers[0].position"));
        assert_eq!(
            reader
                .fields
                .iter()
                .find(|f| f.name == "layers[1].position")
                .unwrap()
                .raw
                .low_u64(),
            7
        );
        assert_eq!(reader.r("sentinel", 5), Some(0b10101));
    }

    #[test]
    fn monitor_count_cannot_exceed_the_eight_slots() {
        let mut invalid = reader(&[9, 0]);
        assert_eq!(
            component(&mut invalid, "device-dispenser-monitors-changed-component"),
            Some(false)
        );
        assert_eq!(invalid.cursor.position, 8);
        let mut valid = reader(&[8, 0]);
        assert_eq!(
            component(&mut valid, "device-dispenser-monitors-changed-component"),
            Some(true)
        );
        assert_eq!(valid.cursor.position, 16);
    }
}
