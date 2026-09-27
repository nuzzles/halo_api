//! Direction fields in the native derived position cache.
use super::{NativeFactsReader, NativeFactsWriter};
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct FactsPositionDirections {
    pub has_aim: bool,
    pub aim_raw: u32,
    pub has_vel: bool,
    pub vel_raw: u32,
    pub vel_scale: u32,
    pub has_aim_b: bool,
    pub yaw_raw_b: u32,
    pub pitch_raw_b: u32,
    pub aim_flag0: bool,
    pub aim_flag1: bool,
    pub aim_flag2: bool,
    pub mask_over: bool,
    pub has_roll: bool,
    pub fwd_mode: u8,
    pub aim_default: bool,
    pub roll_raw: u32,
    /// Deliberately absent from the native cache bytes. Preserved only when a
    /// caller decodes directions into an existing value; new position reads use 0.
    pub mask_bits: u64,
}
pub fn encode_facts_position_directions(w: &mut NativeFactsWriter, p: &FactsPositionDirections) {
    let mut flags = 0u8;
    for (i, v) in [
        p.has_aim,
        p.has_vel,
        p.has_aim_b,
        p.aim_flag0,
        p.aim_flag1,
        p.aim_flag2,
        p.mask_over,
        p.has_roll,
    ]
    .into_iter()
    .enumerate()
    {
        flags |= u8::from(v) << i;
    }
    w.byte(flags);
    if p.has_roll {
        w.byte((p.fwd_mode & 3) | (u8::from(p.aim_default) << 2));
        w.unsigned(u64::from(p.roll_raw));
    }
    if p.has_aim {
        w.unsigned(u64::from(p.aim_raw));
    }
    if p.has_vel {
        w.unsigned(u64::from(p.vel_raw));
        w.unsigned(u64::from(p.vel_scale));
    }
    if p.has_aim_b {
        w.unsigned(u64::from(p.yaw_raw_b));
        w.unsigned(u64::from(p.pitch_raw_b));
    }
}
pub fn decode_facts_position_directions(
    r: &mut NativeFactsReader<'_>,
    p: &mut FactsPositionDirections,
) {
    let flags = r.byte();
    p.has_aim = flags & 1 != 0;
    p.has_vel = flags & 2 != 0;
    p.has_aim_b = flags & 4 != 0;
    p.aim_flag0 = flags & 8 != 0;
    p.aim_flag1 = flags & 16 != 0;
    p.aim_flag2 = flags & 32 != 0;
    p.mask_over = flags & 64 != 0;
    p.has_roll = flags & 128 != 0;
    if p.has_roll {
        let mode = r.byte();
        p.fwd_mode = mode & 3;
        p.aim_default = mode & 4 != 0;
        p.roll_raw = r.unsigned() as u32;
    }
    if p.has_aim {
        p.aim_raw = r.unsigned() as u32;
    }
    if p.has_vel {
        p.vel_raw = r.unsigned() as u32;
        p.vel_scale = r.unsigned() as u32;
    }
    if p.has_aim_b {
        p.yaw_raw_b = r.unsigned() as u32;
        p.pitch_raw_b = r.unsigned() as u32;
    }
}
