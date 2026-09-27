use super::bits::Bits;
use super::combat::INPUT_END;
use super::{PlayerTrack, ProjectileTrack, Sample, SourceSpan};
pub(super) const PROJECTILE_SPAWN: &str = "0000100100000000000110100110000001100000000001011000000010111000100000110001001000010110010010110011110011111000000000000001000001110000000000110100000000010110000000000000000000000000000000000000000000100001110000000001111000";
#[cfg(test)]
pub(super) const PROJECTILE_DELTA: &str = "100100000000000100";
pub(super) const PROJECTILE_END: &str = "110000101100000000010000";

#[cfg(test)]
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
#[cfg(test)]
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

pub(super) struct Birth {
    pub id: u16,
    pub generation: u8,
    pub player: u8,
    pub life: u16,
    pub xyz: [u32; 3],
    pub velocity: (usize, usize, super::Velocity),
}

pub(super) fn birth(
    b: Bits<'_>,
    o: usize,
    layout: super::CoordinateLayout,
) -> Option<(Birth, usize)> {
    // The middle contains a roster5 and wire8/generation2 thrower reference.
    // All surrounding default-state and position-prefix bits must match.
    if b.read(o, 2)? == 3
        || b.read(o + 2, 2)? != 0
        || !b.is(o + 20, &PROJECTILE_SPAWN[20..124])
        || !b.is(o + 129, &PROJECTILE_SPAWN[129..133])
        || !b.is(o + 143, &PROJECTILE_SPAWN[143..])
    {
        return None;
    }
    let id = b.read(o + 4, 14)? as u16;
    if id >> 8 != 36 {
        return None;
    }
    let mut p = o + 226;
    let mut xyz = [0; 3];
    for (v, width) in xyz.iter_mut().zip(layout.axis_bits()) {
        *v = b.read(p, width)? as u32;
        p += width;
    }
    if b.read(p, 2)? != 0 {
        return None;
    }
    p += 2;
    let (value, end) = super::velocity::packed(b, p)?;
    Some((
        Birth {
            id,
            generation: b.read(o + 18, 2)? as u8,
            player: b.read(o + 124, 5)? as u8,
            life: super::combat::serial(b, o + 133, o + 141)?,
            xyz,
            velocity: (p, end, value),
        },
        end,
    ))
}

pub(super) struct Record {
    pub id: u16,
    pub generation: u8,
    pub position: Option<(usize, usize, [u32; 3])>,
    pub velocity: (usize, usize, super::Velocity),
    pub rest: Option<(usize, usize, bool)>,
    pub end: usize,
}

fn fields(b: Bits<'_>, o: usize, layout: super::CoordinateLayout) -> Option<Record> {
    let id = b.read(o, 14)? as u16;
    if id >> 8 != 36 || b.read(o + 16, 2)? != 0 {
        return None;
    }
    let count = b.read(o + 18, 3)? as usize;
    if !(2..=4).contains(&count) {
        return None;
    }
    let ids: Vec<_> = (0..count)
        .map(|i| b.read(o + 21 + i * 6, 6))
        .collect::<Option<_>>()?;
    if !matches!(
        ids.as_slice(),
        [0, 1, 2] | [0, 1, 2, 5] | [0, 1, 2, 20] | [0, 1, 2, 18] | [1, 2] | [1, 2, 20] | [1, 2, 18]
    ) {
        return None;
    }
    let mut p = o + 21 + count * 6;
    let position = if ids[0] == 0 {
        let start = p;
        if b.read(p, 3)? != 0 {
            return None;
        }
        p += 3;
        let mut xyz = [0; 3];
        for (v, width) in xyz.iter_mut().zip(layout.axis_bits()) {
            *v = b.read(p, width)? as u32;
            p += width;
        }
        if b.read(p, 2)? != 0 {
            return None;
        }
        p += 2;
        Some((start, p, xyz))
    } else {
        None
    };
    let start = p;
    let (velocity, end) = super::velocity::packed(b, p)?;
    p = end;
    // Component 2: forward direction and roll. Boundaries are checked, but
    // orientation/roll semantics are not exported as a decoded rotation.
    if b.read(p, 1)? == 0 {
        super::velocity::direction(b.read(p + 1, 19)? as u32)?;
        p += 20;
    } else {
        p += 1;
    }
    b.read(p, 8)?;
    p += 8;
    let mut rest = None;
    match ids.last()? {
        5 => {
            if b.read(p, 29)? != 0 {
                return None;
            }
            p += 29;
        }
        18 => {
            let a = p;
            let value = b.read(p, 1)? != 0;
            let extra = b.read(p + 1, 1)? != 0;
            p += 2;
            if extra {
                super::velocity::direction(b.read(p, 19)? as u32)?;
                p += 19;
            }
            rest = Some((a, p, value));
        }
        20 => {
            let width = if b.read(p, 1)? == 1 { 9 } else { 1 };
            b.read(p, width)?;
            p += width;
        }
        _ => {}
    }
    Some(Record {
        id,
        generation: b.read(o + 14, 2)? as u8,
        position,
        velocity: (start, end, velocity),
        rest,
        end: p,
    })
}

pub(super) fn record(b: Bits<'_>, o: usize, layout: super::CoordinateLayout) -> Option<Record> {
    let r = fields(b, o, layout)?;
    let p = r.end;
    // A complete supported record ends at End/input, another parsed projectile,
    // or a checked common auxiliary header/payload. Later unrelated components
    // need not be decoded to establish this record's boundary.
    if b.is(p, INPUT_END) || fields(b, p, layout).is_some() {
        return Some(r);
    }
    let class = b.read(p, 6)?;
    if !matches!(class, 0b100101 | 0b101001 | 0b101100) || b.read(p + 16, 5)? != 1 {
        return None;
    }
    let width = match b.read(p + 21, 6)? {
        18 => 15,
        1 => 24,
        _ => return None,
    };
    b.read(p + 27, width)?;
    Some(r)
}

#[derive(Default)]
pub(super) struct Tracks {
    pub births: Vec<Sample<Birth>>,
    pub records: Vec<Sample<Record>>,
    pub terminals: Vec<(u64, SourceSpan)>,
}
impl Tracks {
    pub fn finish(mut self, players: &[PlayerTrack]) -> Vec<ProjectileTrack> {
        self.births.sort_by_key(|s| {
            (
                s.time_us,
                s.source.chunk,
                s.source.payload_byte,
                s.source.bit,
            )
        });
        self.records.sort_by_key(|s| {
            (
                s.time_us,
                s.source.chunk,
                s.source.payload_byte,
                s.source.bit,
            )
        });
        let mut by_identity = std::collections::BTreeMap::<_, Vec<_>>::new();
        for record in &self.records {
            by_identity
                .entry((record.value.id, record.value.generation))
                .or_default()
                .push(record);
        }
        let mut out = Vec::new();
        for (i, spawn) in self.births.iter().enumerate() {
            let b = &spawn.value;
            let Some(player) = players.iter().find(|p| p.id == b.player) else {
                continue;
            };
            let Some(life) = player.lives.iter().find(|l| {
                l.id == b.life && l.start_us <= spawn.time_us && spawn.time_us < l.end_us
            }) else {
                continue;
            };
            let until = self.births[i + 1..]
                .iter()
                .find(|s| s.value.id == b.id)
                .map_or(u64::MAX, |s| s.time_us);
            let source_sample = |time_us, source, value| Sample {
                time_us,
                life: life.id,
                source,
                value,
            };
            let mut positions = vec![source_sample(spawn.time_us, spawn.source, b.xyz)];
            let mut velocities = Vec::new();
            let mut at_rest = Vec::new();
            let vs = SourceSpan {
                bit: b.velocity.0,
                end_bit: b.velocity.1,
                ..spawn.source
            };
            velocities.push(Sample {
                time_us: spawn.time_us,
                life: life.id,
                source: vs,
                value: b.velocity.2,
            });
            let mut last = spawn.time_us;
            for s in by_identity
                .get(&(b.id, b.generation))
                .into_iter()
                .flatten()
                .filter(|s| spawn.time_us < s.time_us && s.time_us < until)
            {
                // A discontinuous or unbound segment is not silently joined.
                if s.time_us.saturating_sub(last) > 100_000 {
                    break;
                }
                if let Some((a, end, xyz)) = s.value.position {
                    if positions.len() == 1
                        && (0..3).any(|axis| xyz[axis].abs_diff(b.xyz[axis]) > 128)
                    {
                        break;
                    }
                    positions.push(source_sample(
                        s.time_us,
                        SourceSpan {
                            bit: a,
                            end_bit: end,
                            ..s.source
                        },
                        xyz,
                    ));
                }
                let (a, end, value) = s.value.velocity;
                velocities.push(Sample {
                    time_us: s.time_us,
                    life: life.id,
                    source: SourceSpan {
                        bit: a,
                        end_bit: end,
                        ..s.source
                    },
                    value,
                });
                if let Some((a, end, value)) = s.value.rest {
                    at_rest.push(Sample {
                        time_us: s.time_us,
                        life: life.id,
                        source: SourceSpan {
                            bit: a,
                            end_bit: end,
                            ..s.source
                        },
                        value,
                    });
                }
                last = s.time_us;
            }
            if positions.len() < 3 {
                continue;
            }
            let mut end_us = last.saturating_add(1);
            let mut terminal = None;
            // Preserve the independently checked terminal in the two controls.
            if players.len() == 1
                && player.lives.len() == 1
                && player.grenades.len() == 1
                && self.births.len() == 1
                && positions.len() == 87
                && b.id == 9216
                && b.generation == 1
                && let [(end, source)] = self.terminals.as_slice()
                && *end > last
                && *end - last < 100_000
            {
                end_us = *end;
                terminal = Some(*source);
            }
            out.push(ProjectileTrack {
                id: b.id,
                generation: b.generation,
                player: player.id,
                life: life.id,
                start_us: spawn.time_us,
                end_us,
                positions,
                velocities,
                at_rest,
                terminal,
            });
        }
        out
    }
}
