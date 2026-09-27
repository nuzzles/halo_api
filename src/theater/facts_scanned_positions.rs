//! Project loaded-source position scans into the native derived facts cache.
use super::*;

/// Preserve scan order and the already-filtered world coordinates. Pass the
/// actual scan's `capture_dirs` setting: native masks are unset when it is false,
/// even though the Rust source record retains its component index list.
///
/// This is the reference cache projection, not a lossless recording model:
/// source ranges, packet indices and vitality flags remain in `report`, because
/// the native facts DTO has no fields for them. MaskBits survives in the DTO but
/// the native cache encoder omits it. No filtering or dequantization occurs here.
pub fn facts_from_world_position_scan(
    report: &SourceWorldPositionReport,
    capture_dirs: bool,
) -> Vec<FactsBipedPosition> {
    report
        .positions
        .iter()
        .map(|p| position_fact(&p.position, Some(p.world), capture_dirs))
        .collect()
}

/// Preserve quanta without claiming world coordinates. The native cache byte
/// format only writes Q when HasWorld is true; this in-memory projection keeps Q
/// even though encoding a quanta-only DTO cannot preserve those values.
pub fn facts_from_quantized_position_scan(
    report: &SourceQuantizedPositionReport,
    capture_dirs: bool,
) -> Vec<FactsBipedPosition> {
    report
        .positions
        .iter()
        .map(|p| position_fact(p, None, capture_dirs))
        .collect()
}

fn position_fact(
    p: &SourceQuantizedPosition,
    world: Option<[f32; 3]>,
    capture_dirs: bool,
) -> FactsBipedPosition {
    let mut out = FactsBipedPosition {
        timestamp_us: p.source.timestamp_us,
        slot: p.record.slot,
        quantized: p.record.quantized,
        has_world: world.is_some(),
        world: world.unwrap_or_default(),
        ..Default::default()
    };
    if !capture_dirs {
        return out;
    }
    let c = &p.record.companions;
    let (mask_bits, mask_over) =
        native_component_mask(p.record.component_indices.iter().map(|&i| i64::from(i)));
    let [yaw_raw, pitch_raw] = c.aim.unwrap_or_default();
    out.has_yaw = c.aim.is_some();
    out.yaw_raw = yaw_raw;
    out.pitch_raw = pitch_raw;
    let [vel_raw, vel_scale] = c.velocity.unwrap_or_default();
    let [yaw_raw_b, pitch_raw_b] = c.aim_b.unwrap_or_default();
    out.directions = FactsPositionDirections {
        has_aim: c.forward.is_some(),
        aim_raw: c.forward.unwrap_or_default(),
        has_vel: c.velocity.is_some(),
        vel_raw,
        vel_scale,
        has_aim_b: c.aim_b.is_some(),
        yaw_raw_b,
        pitch_raw_b,
        aim_flag0: c.aim_flag_0,
        aim_flag1: c.aim_flag_1,
        aim_flag2: c.aim_flag_2,
        mask_bits,
        mask_over,
        has_roll: c.chassis.as_ref().is_some_and(|o| o.roll.is_some()),
        roll_raw: c.chassis.as_ref().and_then(|o| o.roll).unwrap_or_default(),
        fwd_mode: c.chassis.as_ref().map_or(0, |o| o.mode),
        aim_default: c.chassis.as_ref().is_some_and(|o| o.direction_default),
    };
    if let Some(body) = &c.body {
        out.has_body = true;
        out.health = body.health;
    }
    if let Some(shield) = &c.shield {
        out.has_shield = true;
        out.shield = shield.shield;
        out.shield_quantum = shield.quantum;
    }
    out
}
