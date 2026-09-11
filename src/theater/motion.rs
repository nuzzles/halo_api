use super::bits::Bits;
use super::combat::{INPUT_END, clock};
use super::{Aim, BodyVitality, CoordinateLayout, InputAxes, Magazine, ShieldVitality};

pub(super) const SPAWN_BODY: &str = "1000111000011011000110000111011010111101101000000";
// Observed prefixes precede coordinates; they do not identify playlists or
// uniquely determine widths. The full Octagon match shares Aquarius's prefix,
// but only the Octagon coordinate partition has motion evidence.
const COORD_PREFIXES: [&str; 3] = [
    "110000010000110010111111000000011011000011111001101110001001000",
    "110000010000110010111111000000000011000011111001101110001001000",
    "110000010000110010011111000000011011000011111001101110001001000",
];
pub(super) const COORD_SUFFIX: &str = "000011101010010111001110000001";

pub(super) struct Spawn {
    pub serial: u16,
    pub player: u8,
    pub xyz: [u32; 3],
    pub layout: CoordinateLayout,
    pub end: usize,
}
pub(super) fn spawn(b: Bits<'_>, o: usize) -> Option<Spawn> {
    if !b.is(o + 20, SPAWN_BODY) {
        return None;
    }
    // The first two bits are record flags, not part of the wire identity.
    // 01 occurs on chained spawns in the raid; 00/10 occur in prior controls.
    if b.read(o, 2)? == 3 || b.read(o + 2, 8)? != 0x22 {
        return None;
    }
    let serial = super::combat::serial(b, o + 10, o + 18)?;
    let mut found = None;
    for shift in [0, 32] {
        if !COORD_PREFIXES
            .iter()
            .any(|prefix| b.is(o + 192 + shift, prefix))
        {
            continue;
        }
        for layout in [
            CoordinateLayout::X15Y15Z17,
            CoordinateLayout::X17Y17Z16,
            CoordinateLayout::X18Y18Z15,
        ] {
            let suffix = o + 263 + shift + layout.coordinate_bits();
            // Both captured suffixes preserve the same coordinate boundary.
            // Their differing flag is opaque; it does not select an axis layout.
            if !b.is(suffix, COORD_SUFFIX) && !b.is(suffix, "000011101010010111001110000011") {
                continue;
            }
            if found.is_some() {
                return None;
            }
            let p = o + 263 + shift;
            let xyz = coordinates(b, p, layout)?;
            found = Some(Spawn {
                serial,
                player: b.read(o + 69, 5)? as u8,
                xyz,
                layout,
                end: p + layout.coordinate_bits() + COORD_SUFFIX.len(),
            });
        }
    }
    found
}
#[derive(Debug, Default)]
pub(super) struct Delta {
    pub ids: Vec<u8>,
    pub end: usize,
    pub position: Option<[u32; 3]>,
    pub aim: Option<Aim>,
    pub body: Option<BodyVitality>,
    pub shield: Option<ShieldVitality>,
    pub magazines: Vec<Magazine>,
    pub selection: Option<u8>,
}
pub(super) fn pawn_header(b: Bits<'_>, o: usize) -> Option<(u16, Vec<u8>, usize)> {
    if !b.is(o, "100010") || b.read(o + 16, 2)? != 0 {
        return None;
    }
    let serial = super::combat::serial(b, o + 6, o + 14)?;
    let count = b.read(o + 18, 3)? as usize;
    if count == 0 {
        return None;
    }
    let mut ids = Vec::with_capacity(count);
    for i in 0..count {
        let id = b.read(o + 21 + 6 * i, 6)? as u8;
        if ids.last().is_some_and(|p| *p >= id) {
            return None;
        }
        ids.push(id);
    }
    Some((serial, ids, o + 21 + 6 * count))
}
pub(super) fn body(b: Bits<'_>, o: usize) -> Option<BodyVitality> {
    let raw = b.read(o + 1, 7)? as u8;
    let state = b.read(o + 8, 3)? as u8;
    if b.read(o, 1)? != 1 || raw > 126 || !matches!(state, 0 | 4 | 6) {
        return None;
    }
    Some(BodyVitality { raw, state })
}
pub(super) fn shield(b: Bits<'_>, o: usize) -> Option<ShieldVitality> {
    let raw = b.read(o, 8)? as u8;
    let delay_ticks = b.read(o + 16, 9)? as u16;
    let state = b.read(o + 25, 4)? as u8;
    if raw > 64 || b.read(o + 8, 8)? != 0 || delay_ticks > 300 || !matches!(state, 0 | 1 | 12) {
        return None;
    }
    Some(ShieldVitality {
        raw,
        delay_ticks,
        state,
    })
}
fn coordinates(b: Bits<'_>, mut p: usize, layout: CoordinateLayout) -> Option<[u32; 3]> {
    let mut xyz = [0; 3];
    for (value, width) in xyz.iter_mut().zip(layout.axis_bits()) {
        *value = b.read(p, width)? as u32;
        p += width;
    }
    Some(xyz)
}
fn position(b: Bits<'_>, o: usize, layout: CoordinateLayout) -> Option<[u32; 3]> {
    if b.read(o, 5)? != 0 || b.read(o + 5 + layout.coordinate_bits(), 2)? != 0 {
        return None;
    }
    coordinates(b, o + 5, layout)
}
fn aim(b: Bits<'_>, o: usize) -> Option<Aim> {
    if b.read(o, 1)? != 1 || b.read(o + 24, 1)? != 0 {
        return None;
    }
    Some(Aim {
        yaw: b.read(o + 1, 12)? as u16,
        pitch: b.read(o + 13, 11)? as u16,
    })
}
/// Prefix ends at the checked command tick. Unknown later components stay unparsed.
pub(super) fn clocked_delta(
    b: Bits<'_>,
    o: usize,
    layout: CoordinateLayout,
    tick: u8,
) -> Option<Delta> {
    let d = clocked_fields(b, o, layout, tick)?;
    // A supported prefix is followed by another record or the checked End/input
    // boundary. This is a packet-grammar check, independent of axis widths.
    if d.ids.last() == Some(&25) && b.read(d.end, 1)? != 1 && !b.is(d.end, INPUT_END) {
        return None;
    }
    Some(d)
}
fn clocked_fields(b: Bits<'_>, o: usize, layout: CoordinateLayout, tick: u8) -> Option<Delta> {
    let (_, ids, mut p) = pawn_header(b, o)?;
    if !ids.contains(&25) {
        return None;
    }
    let mut d = Delta {
        ids: ids.clone(),
        ..Delta::default()
    };
    for id in ids {
        match id {
            0 => {
                d.position = Some(position(b, p, layout)?);
                p += layout.position_bits();
            }
            1 => {
                p += match b.read(p, 2)? {
                    0 => 31,
                    1 => 2,
                    _ => return None,
                };
            }
            4 => {
                if !d.ids.contains(&5) {
                    return None;
                }
                d.body = body(b, p);
                b.read(p, 11)?;
                p += 11;
            }
            5 => {
                if b.read(p, 1)? != 0 {
                    return None;
                }
                d.shield = shield(b, p);
                b.read(p, 29)?;
                p += 29;
            }
            21 => {
                d.aim = Some(aim(b, p)?);
                p += 25;
            }
            25 => {
                if b.read(p, 10)? != ((1 << 9) | u64::from(tick) << 1) {
                    return None;
                }
                d.end = p + 10;
                return Some(d);
            }
            _ => return None,
        }
    }
    None
}
pub(super) fn isolated_vitality(b: Bits<'_>, o: usize) -> Option<Delta> {
    let (serial, ids, mut p) = pawn_header(b, o)?;
    if serial >= 128 || !matches!(ids.as_slice(), [5] | [4, 5]) {
        return None;
    }
    let mut d = Delta {
        ids: ids.clone(),
        ..Delta::default()
    };
    for id in ids {
        match id {
            4 => {
                d.body = Some(body(b, p)?);
                p += 11;
            }
            5 => {
                d.shield = Some(shield(b, p)?);
                p += 29;
            }
            _ => return None,
        }
    }
    if !b.is(p, INPUT_END) {
        return None;
    }
    d.end = p;
    Some(d)
}
/// Weapon forms end at End/input or an independently checked neighboring pawn.
/// Unsupported inventory and energy-ammo forms stay unknown.
pub(super) fn weapon_delta(
    b: Bits<'_>,
    o: usize,
    layout: CoordinateLayout,
    tick: Option<u8>,
) -> Option<Delta> {
    let (_, ids, mut p) = pawn_header(b, o)?;
    if !ids.iter().any(|i| matches!(i, 30 | 33 | 42)) {
        return None;
    }
    let mut d = Delta {
        ids: ids.clone(),
        ..Delta::default()
    };
    for id in ids {
        match id {
            0 => {
                position(b, p, layout)?;
                p += layout.position_bits();
            }
            1 => {
                p += match b.read(p, 2)? {
                    0 => 31,
                    1 => 2,
                    _ => return None,
                };
            }
            4 => {
                body(b, p)?;
                p += 11;
            }
            5 => {
                shield(b, p)?;
                p += 29;
            }
            21 => {
                aim(b, p)?;
                p += 25;
            }
            25 => {
                if b.read(p, 10)? != ((1 << 9) | u64::from(tick?) << 1) {
                    return None;
                }
                p += 10;
            }
            30 | 33 => {
                let rounds = if b.read(p, 2)? == 3 {
                    p += 2;
                    0
                } else {
                    if b.read(p, 1)? != 0 || b.read(p + 9, 1)? != 1 {
                        return None;
                    }
                    let amount = b.read(p + 1, 8)? as u8;
                    // The AR control and multiplayer BR sequences extend the
                    // literal form through 36; energy/heat forms differ.
                    if !(1..=36).contains(&amount) {
                        return None;
                    }
                    p += 10;
                    amount
                };
                d.magazines.push(Magazine {
                    slot: (id - 30) / 3,
                    rounds,
                });
            }
            31 | 34 => {
                // Captured reload refills pair the magazine with an 11-bit
                // rounds-inventory component. Its value is not exported yet.
                if !d.ids.contains(&(id - 1)) {
                    return None;
                }
                b.read(p, 11)?;
                p += 11;
            }
            35 => {
                // The Stalker control bounds this nine-bit overheated-state
                // form with 496 independent continuations: upper seven bits
                // 0..=21, trailing 00. Consume only those observed forms to
                // reach combined weapon fields; do not export it as ammo.
                let raw = b.read(p, 9)?;
                if raw & 3 != 0 || raw >> 2 > 21 {
                    return None;
                }
                p += 9;
            }
            42 => {
                // Paired switch/back-switch control: these exact seven-bit
                // forms select primary/secondary. Their first three bits are
                // constant, not the slot index.
                d.selection = Some(match b.read(p, 7)? {
                    0b0010001 => 0,
                    0b0010011 => 1,
                    _ => return None,
                });
                p += 7;
            }
            _ => return None,
        }
    }
    if !b.is(p, INPUT_END)
        && !tick.is_some_and(|t| clocked_delta(b, p, layout, t).is_some())
        && isolated_vitality(b, p).is_none()
    {
        return None;
    }
    d.end = p;
    Some(d)
}
#[derive(Debug, Default)]
pub(super) struct Chain {
    pub position: Option<[u32; 3]>,
    pub aim: Option<Aim>,
    pub input: Option<InputAxes>,
    pub end: usize,
    pub copied: Option<[u32; 3]>,
    pub secondary: bool,
    pub auxiliaries: usize,
}
/// Checked wire-0/generation-1 chain, including its auxiliary and input boundaries.
/// This record grammar is independent of coordinate layout and gameplay category.
pub(super) fn input_chain(b: Bits<'_>, layout: CoordinateLayout) -> Option<Chain> {
    let (_, tick) = clock(b, 0, true)?;
    let mut p = 37;
    let mut c = Chain::default();
    if b.is(p, "10001000000000") {
        let (serial, ids, _) = pawn_header(b, p)?;
        if serial != 0 || !ids.iter().all(|id| matches!(id, 0 | 1 | 21 | 25)) {
            return None;
        }
        let d = clocked_fields(b, p, layout, tick)?;
        c.position = d.position;
        c.aim = d.aim;
        p = d.end;
    }
    if b.is(p, "10001000000001") && b.is(p + 14, "010000100010") {
        b.read(p + 26, 30)?;
        c.secondary = true;
        p += 56;
    }
    while b.is(p, "10010100010010") || b.is(p, "10010100010100") {
        if !b.is(p + 14, "0100001000001000001") {
            return None;
        }
        b.read(p + 33, 18)?;
        c.auxiliaries += 1;
        p += 51;
    }
    if b.is(p, "10010100110101") {
        if !b.is(p + 14, "0100001000000000") {
            return None;
        }
        let xyz = coordinates(b, p + 30, layout)?;
        if c.position.is_some_and(|v| v != xyz)
            || b.read(p + 30 + layout.coordinate_bits(), 2)? != 0
        {
            return None;
        }
        c.copied = Some(xyz);
        p += 32 + layout.coordinate_bits();
    }
    if b.read(p, 3)? != 0 {
        return None;
    }
    p += 3;
    b.read(p, 1)?;
    if b.is(p, "1000000001101") {
        let forward = b.read(p + 13, 6)? as u8;
        let left = b.read(p + 19, 6)? as u8;
        if forward > 62 || left > 62 {
            return None;
        }
        c.input = Some(InputAxes { forward, left });
        p += 25;
    }
    c.end = p;
    Some(c)
}

/// Two checked command-tail forms after the independently parsed input axes.
/// This is a crouch command, not proof of the resulting physical posture/slide.
/// The exact final-byte padding rejects additional players/buttons and unknown forms.
pub(super) fn crouch_input(b: Bits<'_>, o: usize) -> Option<(bool, usize)> {
    let (value, width) = if b.is(o, "00100001000") {
        (true, 11)
    } else if b.is(o, "00000") {
        (false, 5)
    } else {
        return None;
    };
    let end = o.checked_add(width)?;
    if end.div_ceil(8) * 8 != b.len() || b.read(end, b.len().checked_sub(end)?)? != 0 {
        return None;
    }
    Some((value, end))
}
