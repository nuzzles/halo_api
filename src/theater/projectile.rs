use super::bits::Bits;
use super::combat::INPUT_END;
use super::{PlayerTrack, ProjectileTrack, Sample, SourceSpan};
pub(super) const PROJECTILE_SPAWN: &str = "0000100100000000000110100110000001100000000001011000000010111000100000110001001000010110010010110011110011111000000000000001000001110000000000110100000000010110000000000000000000000000000000000000000000100001110000000001111000";
pub(super) const PROJECTILE_DELTA: &str = "100100000000000100";
pub(super) const PROJECTILE_END: &str = "110000101100000000010000";

pub(super) fn spawn(b: Bits<'_>, o: usize) -> Option<([u32; 3], usize)> {
    if !b.is(o, PROJECTILE_SPAWN) {
        return None;
    }
    let p = o + PROJECTILE_SPAWN.len();
    Some((
        [
            b.read(p, 15)? as u32,
            b.read(p + 15, 15)? as u32,
            b.read(p + 30, 17)? as u32,
        ],
        p + 47,
    ))
}
pub(super) fn delta(b: Bits<'_>, o: usize) -> Option<([u32; 3], usize)> {
    if !b.is(o, PROJECTILE_DELTA) {
        return None;
    }
    let count = b.read(o + 18, 3)? as usize;
    if !matches!(count, 3 | 4) {
        return None;
    }
    let ids: Vec<_> = (0..count)
        .map(|i| b.read(o + 21 + 6 * i, 6))
        .collect::<Option<_>>()?;
    if !matches!(ids.as_slice(), [0, 1, 2] | [0, 1, 2, 5] | [0, 1, 2, 20]) {
        return None;
    }
    let p = o + 21 + 6 * count;
    let end = p
        + 110
        + match ids.last()? {
            5 => 29,
            20 => 9,
            _ => 0,
        };
    if b.read(p, 3)? != 0 || b.read(p + 50, 2)? != 0 || !b.is(end, INPUT_END) {
        return None;
    }
    Some((
        [
            b.read(p + 3, 15)? as u32,
            b.read(p + 18, 15)? as u32,
            b.read(p + 33, 17)? as u32,
        ],
        end + INPUT_END.len(),
    ))
}
#[derive(Default)]
pub(super) struct Candidates {
    pub spawns: Vec<Sample<[u32; 3]>>,
    pub positions: Vec<Sample<[u32; 3]>>,
    pub terminals: Vec<(u64, SourceSpan)>,
}
impl Candidates {
    pub fn finish(mut self, players: &[PlayerTrack]) -> Option<ProjectileTrack> {
        let [player] = players else {
            return None;
        };
        let [life] = player.lives.as_slice() else {
            return None;
        };
        let [throw] = player.grenades.as_slice() else {
            return None;
        };
        let [spawn] = self.spawns.as_slice() else {
            return None;
        };
        let [(end, terminal)] = self.terminals.as_slice() else {
            return None;
        };
        if self.positions.len() != 86 {
            return None;
        }
        self.positions.sort_by_key(|s| s.time_us);
        let first = self.positions.first()?;
        let delay = spawn.time_us.checked_sub(throw.time_us)?;
        if !(290_001..310_000).contains(&delay)
            || self
                .positions
                .iter()
                .any(|s| s.time_us <= spawn.time_us || s.time_us >= *end)
            || (0..3).any(|i| {
                spawn.value[i].abs_diff(first.value[i]) > 12
                    || spawn.value[i].abs_diff(life.position[i]) > 100
            })
        {
            return None;
        }
        let mut positions = self.spawns;
        positions.extend(self.positions);
        for s in &mut positions {
            s.life = life.id;
        }
        Some(ProjectileTrack {
            id: 9216,
            player: player.id,
            life: life.id,
            start_us: positions[0].time_us,
            end_us: *end,
            positions,
            terminal: *terminal,
        })
    }
}
