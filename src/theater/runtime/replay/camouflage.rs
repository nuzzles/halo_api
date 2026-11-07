//! Causal active-camouflage state from the biped's transmitted sub[1] channel.
use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct CamouflageSample {
    pub timestamp_us: u64,
    pub active: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CamouflageObservation {
    pub timestamp_us: u64,
    pub entity_id: u32,
    pub source: PoseSource,
    pub raw_q: u16,
    pub active: Option<bool>,
}

fn active(q: u16) -> Option<bool> {
    match q {
        0 => Some(false),
        4095 => Some(true),
        _ => None,
    }
}

pub(super) fn observe(
    record: &EntityRecord,
    time: u64,
    source: PoseSource,
) -> Option<CamouflageObservation> {
    if record.stop != EntityViewStop::Complete {
        return None;
    }
    let c = component(record, "unit-active-camo-state-component")?;
    // A gated-out channel is unchanged, not an explicit inactive observation.
    if value(&c.fields, "sub[1].gate", 1) != Some(1) {
        return None;
    }
    let raw_q = value(&c.fields, "sub[1]", 12)? as u16;
    Some(CamouflageObservation {
        timestamp_us: time,
        entity_id: record.header.id?,
        source: PoseSource {
            start_bit: c.start_bit,
            end_bit: c.end_bit,
            ..source
        },
        raw_q,
        active: active(raw_q),
    })
}

pub(super) fn timeline(
    observations: &mut Vec<CamouflageObservation>,
    gaps: Vec<(u64, usize)>,
    lives: &[PlayerLife],
) -> Vec<CamouflageSample> {
    let within_life = |o: &CamouflageObservation| {
        lives.iter().any(|l| {
            l.entity_id == o.entity_id && l.start_us <= o.timestamp_us && o.timestamp_us < l.end_us
        })
    };
    let mut events = BTreeMap::<u64, Vec<(usize, Option<bool>)>>::new();
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
            .push((2 * i + 2, o.active));
    }
    observations.retain(within_life);
    events
        .into_iter()
        .map(|(timestamp_us, mut changes)| {
            changes.sort_by_key(|(order, _)| *order);
            CamouflageSample {
                timestamp_us,
                active: changes.last().unwrap().1,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests;
