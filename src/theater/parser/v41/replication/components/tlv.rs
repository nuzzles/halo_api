//! Mode-2 wire grammar from the pinned reference's `tlv_mode2.go`.
//! Retain all wire bytes, including schema skips and uninterpreted field bodies.
use super::reader::ComponentReader;
use std::fmt::Write;

const MAX_FIELDS: u64 = 1 << 12;
const MAX_BODY_BYTES: u64 = 1 << 20;
const MAX_DEPTH: usize = 8;

// Resource limits are parser admission limits, not wire lengths. Never clamp a
// declared body/count or silently leave a nested scope and report completion.
fn refuse<T>(r: &mut ComponentReader<'_>, reason: &'static str) -> Option<T> {
    r.context_error = Some(reason);
    None
}

fn with_part<T>(
    name: &mut String,
    part: &str,
    read: impl FnOnce(&mut String) -> Option<T>,
) -> Option<T> {
    let len = name.len();
    name.push_str(part);
    let result = read(name);
    name.truncate(len);
    result
}

fn with_index<T>(
    name: &mut String,
    index: u64,
    read: impl FnOnce(&mut String) -> Option<T>,
) -> Option<T> {
    let len = name.len();
    write!(name, "[{index}]").ok()?;
    let result = read(name);
    name.truncate(len);
    result
}

fn read_part(
    r: &mut ComponentReader<'_>,
    name: &mut String,
    part: &str,
    width: usize,
) -> Option<u64> {
    with_part(name, part, |name| r.r(name, width))
}

fn varint(r: &mut ComponentReader<'_>, name: &mut String) -> Option<u64> {
    let mut value = 0;
    for shift in (0..64).step_by(7) {
        let byte = with_part(name, ".byte", |name| {
            with_index(name, shift / 7, |name| r.r(name, 8))
        })?;
        if shift == 63 && byte > 1 {
            return refuse(r, "TLV varint exceeds 64 bits");
        }
        value |= (byte & 0x7f) << shift;
        if byte < 0x80 {
            return Some(value);
        }
    }
    unreachable!("the tenth byte must terminate or be refused")
}

fn field(r: &mut ComponentReader<'_>, name: &mut String, wire: u64, depth: usize) -> Option<()> {
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
            let count = with_part(name, ".length", |name| varint(r, name))?;
            if count > MAX_BODY_BYTES {
                return refuse(r, "TLV body exceeds resource limit");
            }
            let bits = count as usize * if wire == 18 { 16 } else { 8 };
            for offset in (0..bits).step_by(64) {
                with_part(name, ".body", |name| {
                    with_index(name, (offset / 64) as u64, |name| {
                        r.r(name, (bits - offset).min(64))
                    })
                })?;
            }
        }
        11 | 12 => {
            let header = read_part(r, name, ".header", 8)?;
            let count = if header & 0xe0 != 0 {
                (header >> 5) - 1
            } else {
                with_part(name, ".count", |name| varint(r, name))?
            };
            if count > MAX_FIELDS || (count != 0 && depth >= MAX_DEPTH) {
                return refuse(r, "TLV list exceeds resource limit");
            }
            for i in 0..count {
                with_index(name, i, |name| field(r, name, header & 31, depth + 1))?;
            }
        }
        13 => {
            let key = read_part(r, name, ".key_wire", 8)?;
            let value = read_part(r, name, ".value_wire", 8)?;
            let count = with_part(name, ".count", |name| varint(r, name))?;
            if count > MAX_FIELDS || (count != 0 && depth >= MAX_DEPTH) {
                return refuse(r, "TLV map exceeds resource limit");
            }
            for i in 0..count {
                with_index(name, i, |name| {
                    with_part(name, ".key", |name| field(r, name, key, depth + 1))
                })?;
                with_index(name, i, |name| {
                    with_part(name, ".value", |name| field(r, name, value, depth + 1))
                })?;
            }
        }
        _ => {}
    }
    Some(())
}

pub(crate) fn multiplayer_properties(r: &mut ComponentReader<'_>) -> Option<()> {
    if r.bit("tlv.absent")? {
        return Some(());
    }
    r.r("tlv.variant", 5)?;
    let mut name = String::with_capacity(64);
    name.push_str("tlv.header");
    varint(r, &mut name)?;
    for i in 0..MAX_FIELDS {
        name.clear();
        write!(name, "tlv.field[{i}]").ok()?;
        let tag = read_part(r, &mut name, ".tag", 8)?;
        match tag & 0xe0 {
            0xe0 => {
                read_part(r, &mut name, ".schema_skip", 16)?;
            }
            0xc0 => {
                read_part(r, &mut name, ".schema_skip", 8)?;
            }
            _ => {}
        }
        let wire = tag & 31;
        if wire <= 1 {
            return Some(());
        }
        field(r, &mut name, wire, 0)?;
    }
    refuse(
        r,
        "TLV message exceeds resource limit before its terminator",
    )
}
