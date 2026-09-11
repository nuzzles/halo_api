use super::bits::{Bits, pattern};
use super::{Firing, ZoomStage};

pub(super) const CLOCKS: [u64; 3] = [
    pattern("10100000011110110100001000000"),
    pattern("10100000011111000100001000000"),
    pattern("10100000011111010100001000000"),
];
pub(super) const AUX: &str = "0100000000000100100001001100000";
pub(super) const FIRING_GUARD: &str = "0010110010010110011110011111";
pub(super) const INPUT_END: &str = "0001000000001101";
#[derive(Debug, Clone)]
pub(super) struct Event<T> {
    pub serial: u16,
    pub player: Option<u8>,
    pub value: T,
    pub end: usize,
}
/// Two-bit generation tags cycle 01, 10, 11, 00 over the observed four uses
/// of a wire ID. A later repeat of the same wire/tag remains ambiguous.
pub(super) fn serial(bits: Bits<'_>, wire: usize, generation: usize) -> Option<u16> {
    let generation_tag = bits.read(generation, 2)?;
    Some((bits.read(wire, 8)? + 256 * ((generation_tag + 3) % 4)) as u16)
}
pub(super) fn clock(bits: Bits<'_>, offset: usize, strict: bool) -> Option<(u32, u8)> {
    let word = bits.read(offset, 29)?;
    let mask = if strict { (1 << 29) - 1 } else { (1 << 26) - 1 };
    if !CLOCKS.iter().any(|c| word & mask == c & mask) {
        return None;
    }
    if !strict && !matches!(word >> 26, 1 | 5 | 7) {
        return None;
    }
    Some((
        (word & ((1 << 26) - 1)) as u32,
        bits.read(offset + 29, 8)? as u8,
    ))
}
pub(super) fn firing(bits: Bits<'_>, o: usize) -> Option<Event<Firing>> {
    if !bits.is(o + 1, "10100100110")
        || !bits.is(o + 22, "0000")
        || bits.read(o + 34, 1)? != 0
        || !bits.is(o + 80, FIRING_GUARD)
    {
        return None;
    }
    Some(Event {
        serial: serial(bits, o + 12, o + 20)?,
        player: Some(bits.read(o + 35, 5)? as u8),
        value: Firing {
            sequence: (bits.read(o + 26, 7)? + 128 * bits.read(o + 33, 1)?) as u8,
            weapon_window: bits.read(o + 40, 40)?,
        },
        end: o + 108,
    })
}
/// Checked hit-notification prefix immediately following a firing record.
/// The victim identity is decoded; damage amounts and the remaining payload are unknown.
pub(super) fn weapon_damage(bits: Bits<'_>, o: usize) -> Option<Event<()>> {
    let preceding = o.checked_sub(108)?;
    firing(bits, preceding)?;
    if !bits.is(o + 1, "1010101010") || !bits.is(o + 21, "000010000000000000") {
        return None;
    }
    Some(Event {
        serial: serial(bits, o + 11, o + 19)?,
        player: None,
        value: (),
        end: o + 39,
    })
}
pub(super) fn melee(bits: Bits<'_>, o: usize) -> Option<Event<u64>> {
    let c = o.checked_sub(31)?;
    if !bits.is(o + 1, "101010001")
        || !bits.is(o + 20, "000010")
        || !bits.is(o + 66, FIRING_GUARD)
        || !bits.is(c + 1, "101001101")
        || !bits.is(c + 20, "0011000")
        || bits.read(c + 10, 10)? != bits.read(o + 10, 10)?
    {
        return None;
    }
    Some(Event {
        serial: serial(bits, o + 10, o + 18)?,
        player: Some(bits.read(c + 27, 5)? as u8),
        value: bits.read(o + 26, 40)?,
        end: o + 94,
    })
}
pub(super) fn grenade(bits: Bits<'_>, o: usize) -> Option<Event<()>> {
    if !bits.is(o + 1, "101001111") || !bits.is(o + 20, "0000010") {
        return None;
    }
    let word = bits.read(o + 32, 28)?;
    if !CLOCKS.iter().any(|c| c & ((1 << 28) - 1) == word) || bits.read(o + 60, 8).is_none() {
        return None;
    }
    Some(Event {
        serial: serial(bits, o + 10, o + 18)?,
        player: Some(bits.read(o + 27, 5)? as u8),
        value: (),
        end: o + 68,
    })
}
pub(super) fn reload(bits: Bits<'_>, o: usize) -> Option<Event<()>> {
    if !bits.is(o + 1, "101001101") || !bits.is(o + 20, "0001000") {
        return None;
    }
    let mut next = o + 31;
    if bits.is(next + 1, AUX) {
        next += 64;
    }
    clock(bits, next, false)?;
    Some(Event {
        serial: serial(bits, o + 10, o + 18)?,
        player: Some(bits.read(o + 27, 5)? as u8),
        value: (),
        end: next + 37,
    })
}
fn scope_header(bits: Bits<'_>, o: usize) -> Option<(u16, ZoomStage)> {
    if !bits.is(o + 1, "1001010110") || !bits.is(o + 21, "00") {
        return None;
    }
    let level = match bits.read(o + 23, 2)? {
        0 => ZoomStage::Unscoped,
        1 => ZoomStage::First,
        2 => ZoomStage::Second,
        _ => return None,
    };
    Some((serial(bits, o + 11, o + 19)?, level))
}
pub(super) fn zoom(bits: Bits<'_>, o: usize) -> Option<Event<ZoomStage>> {
    let (serial, value) = scope_header(bits, o)?;
    let mut next = o + 24;
    for _ in 0..2 {
        if scope_header(bits, next).is_none() {
            break;
        }
        next += 24;
    }
    if bits.is(next + 1, AUX) {
        next += 64;
    }
    clock(bits, next, false)?;
    Some(Event {
        serial,
        player: None,
        value,
        end: next + 37,
    })
}
