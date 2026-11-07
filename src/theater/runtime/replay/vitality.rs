//! Source-linked vitality scalars, independent of pose updates and custom maxima.
use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct VitalitySample {
    pub timestamp_us: u64,
    /// Nonnegative serialized body vitality, with 1 as the normal full reference.
    pub health: Option<f32>,
    /// Serialized shield vitality in [0, 4]; overfill is preserved.
    pub shields: Option<f32>,
}

impl VitalitySample {
    /// LevelUp's measured raw shield q > 64 rule, equivalent to decoded shields > 1.
    /// A threshold inference, not a separately recorded activation flag.
    pub fn is_overshield_active(&self) -> Option<bool> {
        self.shields.map(|value| value > 1.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum VitalityKind {
    Health,
    Shields,
    /// Retained for audit only: these modifiers are not a calibrated HUD maximum.
    MaximumModifiers,
}

#[derive(Debug, Clone, Serialize)]
pub struct VitalityObservation {
    pub timestamp_us: u64,
    pub entity_id: u32,
    pub source: PoseSource,
    pub kind: VitalityKind,
    /// Includes flags, optional regeneration words and maximum modifiers verbatim.
    pub fields: Vec<ComponentField>,
    pub raw_q: Option<u8>,
    /// Negative body values are preserved here but not interpreted as health.
    pub decoded: Option<f32>,
    pub value: Option<f32>,
}

// Endpoint-exact quantizer from LevelUp 413971bcc485be53a2fa8cf229e170539a5146f8,
// grammar/quantize_endpoint.go (FUN_1406d84b4). Preserve float32 operation order.
fn scalar(q: u8, kind: VitalityKind) -> Option<f32> {
    let (min, max, levels) = match kind {
        VitalityKind::Health if q < 255 => (-1.0_f32, 1.0_f32, 255_u16),
        VitalityKind::Shields => (0.0, 4.0, 256),
        _ => return None,
    };
    Some(if q == 0 {
        min
    } else if u16::from(q) == levels - 1 {
        max
    } else if kind == VitalityKind::Health && q == 127 {
        0.0
    } else {
        let step = (max - min) / f32::from(levels - 2);
        f32::from(q - 1) * step + (step * 0.5 + min)
    })
}

fn quantum(fields: &[ComponentField], kind: VitalityKind) -> Option<u8> {
    let (field, flags, mut count) = match kind {
        VitalityKind::Health => ("health", 3, 4),
        VitalityKind::Shields => ("shield", 4, 7),
        VitalityKind::MaximumModifiers => return None,
    };
    for i in 0..flags {
        value(fields, &format!("flags[{i}]"), 1)?;
    }
    if kind == VitalityKind::Shields {
        value(fields, "block64", 16)?;
        if value(fields, "regeneration", 1)? == 1 {
            for name in ["regeneration_a", "regeneration_b"] {
                count += 1;
                if value(fields, &format!("{name}.gate"), 1)? == 1 {
                    value(fields, name, 12)?;
                    count += 1;
                }
            }
        }
    }
    (fields.len() == count)
        .then(|| value(fields, field, 8))
        .flatten()
        .map(|q| q as u8)
}

pub(super) fn observe(
    record: &EntityRecord,
    time: u64,
    source: PoseSource,
) -> Vec<VitalityObservation> {
    if record.stop != EntityViewStop::Complete {
        return vec![];
    }
    let Some(entity_id) = record.header.id else {
        return vec![];
    };
    [
        ("object-body-vitality-component", VitalityKind::Health),
        ("object-shield-vitality-component", VitalityKind::Shields),
        (
            "object-maximum-vitalities-component",
            VitalityKind::MaximumModifiers,
        ),
    ]
    .into_iter()
    .filter_map(|(name, kind)| {
        let c = component(record, name)?;
        let raw_q = quantum(&c.fields, kind);
        let decoded = raw_q.and_then(|q| scalar(q, kind));
        Some(VitalityObservation {
            timestamp_us: time,
            entity_id,
            source: PoseSource {
                start_bit: c.start_bit,
                end_bit: c.end_bit,
                ..source
            },
            kind,
            fields: c.fields.clone(),
            raw_q,
            decoded,
            value: decoded.filter(|v| *v >= 0.0),
        })
    })
    .collect()
}

pub(super) fn timeline(
    observations: &mut Vec<VitalityObservation>,
    gaps: Vec<(u64, usize)>,
    lives: &[PlayerLife],
) -> Vec<VitalitySample> {
    let within_life = |o: &VitalityObservation| {
        lives.iter().any(|l| {
            l.entity_id == o.entity_id && l.start_us <= o.timestamp_us && o.timestamp_us < l.end_us
        })
    };
    // None resets both channels; a present change updates only its own channel.
    let mut events = BTreeMap::<u64, Vec<(usize, Option<(VitalityKind, Option<f32>)>)>>::new();
    for life in lives {
        for time in [life.start_us, life.end_us] {
            events.entry(time).or_default().push((0, None));
        }
    }
    for (time, preceding) in gaps {
        events
            .entry(time)
            .or_default()
            .push((2 * preceding + 1, None));
    }
    for (i, o) in observations
        .iter()
        .enumerate()
        .filter(|(_, o)| within_life(o))
    {
        events
            .entry(o.timestamp_us)
            .or_default()
            .push((2 * i + 2, Some((o.kind, o.value))));
    }
    observations.retain(within_life);
    let mut state = VitalitySample {
        timestamp_us: 0,
        health: None,
        shields: None,
    };
    events
        .into_iter()
        .map(|(time, mut changes)| {
            changes.sort_by_key(|(order, _)| *order);
            for (_, change) in changes {
                match change {
                    None => {
                        state.health = None;
                        state.shields = None;
                    }
                    Some((VitalityKind::Health, v)) => state.health = v,
                    Some((VitalityKind::Shields, v)) => state.shields = v,
                    Some((VitalityKind::MaximumModifiers, _)) => {}
                }
            }
            state.timestamp_us = time;
            state
        })
        .collect()
}

#[cfg(test)]
mod tests;
