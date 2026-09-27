//! Mode-2 wire grammar from the pinned reference's `tlv_mode2.go`.
//! Retain all wire bytes, including schema skips and uninterpreted field bodies.
use super::Reader;

const MAX_FIELDS: u64 = 1 << 12;
const MAX_BODY_BYTES: u64 = 1 << 20;
const MAX_DEPTH: usize = 8;

fn varint(r: &mut Reader<'_>, name: &str) -> Option<u64> {
    let mut value = 0;
    for shift in (0..64).step_by(7) {
        let byte = r.r(&format!("{name}.byte[{}]", shift / 7), 8)?;
        value |= (byte & 0x7f) << shift;
        if byte < 0x80 {
            break;
        }
    }
    Some(value)
}

fn field(r: &mut Reader<'_>, name: &str, wire: u64, depth: usize) -> Option<()> {
    match wire {
        2 | 3 | 14 => {
            r.r(name, 8)?;
        }
        7 => {
            r.r(name, 32)?;
        }
        8 => {
            r.r(name, 64)?;
        }
        4..=6 | 15..=17 => {
            varint(r, name)?;
        }
        9 | 10 | 18 => {
            let count = varint(r, &format!("{name}.length"))?.min(MAX_BODY_BYTES);
            let bits = count as usize * if wire == 18 { 16 } else { 8 };
            for offset in (0..bits).step_by(64) {
                r.r(
                    &format!("{name}.body[{}]", offset / 64),
                    (bits - offset).min(64),
                )?;
            }
        }
        11 | 12 => {
            let header = r.r(&format!("{name}.header"), 8)?;
            let count = if header & 0xe0 != 0 {
                (header >> 5) - 1
            } else {
                varint(r, &format!("{name}.count"))?
            };
            if depth < MAX_DEPTH {
                for i in 0..count.min(MAX_FIELDS) {
                    field(r, &format!("{name}[{i}]"), header & 31, depth + 1)?;
                }
            }
        }
        13 => {
            let key = r.r(&format!("{name}.key_wire"), 8)?;
            let value = r.r(&format!("{name}.value_wire"), 8)?;
            let count = varint(r, &format!("{name}.count"))?;
            if depth < MAX_DEPTH {
                for i in 0..count.min(MAX_FIELDS) {
                    field(r, &format!("{name}[{i}].key"), key, depth + 1)?;
                    field(r, &format!("{name}[{i}].value"), value, depth + 1)?;
                }
            }
        }
        _ => {}
    }
    Some(())
}

pub(super) fn multiplayer_properties(r: &mut Reader<'_>) -> Option<()> {
    if r.bit("tlv.absent")? {
        return Some(());
    }
    r.r("tlv.variant", 5)?;
    varint(r, "tlv.header")?;
    for i in 0..MAX_FIELDS {
        let name = format!("tlv.field[{i}]");
        let tag = r.r(&format!("{name}.tag"), 8)?;
        match tag & 0xe0 {
            0xe0 => {
                r.r(&format!("{name}.schema_skip"), 16)?;
            }
            0xc0 => {
                r.r(&format!("{name}.schema_skip"), 8)?;
            }
            _ => {}
        }
        let wire = tag & 31;
        if wire <= 1 {
            break;
        }
        field(r, &name, wire, 0)?;
    }
    Some(())
}
