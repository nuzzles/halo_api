use super::InputAxes;
use super::bits::Bits;

#[derive(Debug)]
pub(super) struct Command {
    pub player: u8,
    pub axes: InputAxes,
    pub crouch: bool,
    pub start: usize,
    pub buttons: usize,
    pub end: usize,
}

/// Checked terminal command records for roster slots 0/1. Replication records
/// before this suffix can remain opaque; no component lengths are guessed.
/// The two header tags and four button forms are captured encodings, not a
/// generalized button mask or a physical stance/slide component.
pub(super) fn terminal(b: Bits<'_>) -> Option<Vec<Command>> {
    // Two maximum-width commands, their preceding zero guard, and byte padding.
    const MAX_BITS: usize = 2 * (25 + 11) + 3 + 7;
    let mut found: Option<Vec<Command>> = None;
    for guard in b.len().saturating_sub(MAX_BITS)..b.len().saturating_sub(32) {
        if b.read(guard, 3) != Some(0) {
            continue;
        }
        let Some(rows) = sequence(b, guard + 3) else {
            continue;
        };
        if let Some(previous) = &found {
            // The second player's record also forms a valid suffix. Keep the
            // longest sequence; reject overlapping, incompatible interpretations.
            if !previous.iter().any(|r| r.start == rows[0].start) {
                return None;
            }
        } else {
            found = Some(rows);
        }
    }
    found
}

fn sequence(b: Bits<'_>, mut p: usize) -> Option<Vec<Command>> {
    let mut rows: Vec<Command> = Vec::with_capacity(2);
    for _ in 0..2 {
        let start = p;
        if b.read(p, 1)? != 1 {
            return None;
        }
        let player = b.read(p + 1, 8)? as u8;
        if player > 1
            || rows.last().is_some_and(|r| r.player >= player)
            || !matches!(b.read(p + 9, 4)?, 13 | 14)
        {
            return None;
        }
        let forward = b.read(p + 13, 6)? as u8;
        let left = b.read(p + 19, 6)? as u8;
        if forward > 62 || left > 62 {
            return None;
        }
        p += 25;
        let buttons = p;
        let (crouch, width) = if b.is(p, "00000") {
            (false, 5)
        } else if b.is(p, "00100001000") {
            (true, 11)
        } else if b.is(p, "00100100000") {
            (false, 11) // Jump without crouch in the independent jump control.
        } else if b.is(p, "00100101000") {
            (true, 11) // Combined jump/crouch, also captured in Octagon.
        } else {
            return None;
        };
        p += width;
        rows.push(Command {
            player,
            axes: InputAxes { forward, left },
            crouch,
            start,
            buttons,
            end: p,
        });
        if p.div_ceil(8) * 8 == b.len() {
            return (b.read(p, b.len().checked_sub(p)?)? == 0).then_some(rows);
        }
    }
    None
}
