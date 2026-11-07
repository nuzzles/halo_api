//! Carried grenade counts from sequentially decoded biped components.
use super::*;

/// v41 inventory rank order, independently established in the pinned reference:
/// fragmentation, plasma, dynamo, spike. Counts are bytes, not capped at two.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct GrenadeCounts {
    pub frag: u8,
    pub plasma: u8,
    pub dynamo: u8,
    pub spike: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct GrenadeSample {
    pub timestamp_us: u64,
    /// None means unknown; an observed all-zero inventory is Some.
    pub counts: Option<GrenadeCounts>,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct GrenadeObservation {
    pub timestamp_us: u64,
    pub entity_id: u32,
    pub source: PoseSource,
    /// Exact decoded byte counts, including values rejected by the admission limit.
    pub raw_counts: Option<GrenadeCounts>,
    /// An unsupported array shape or out-of-limit value invalidates previous counts.
    pub counts: Option<GrenadeCounts>,
}

fn counts(fields: &[ComponentField]) -> Option<GrenadeCounts> {
    if fields.len() != 5 || value(fields, "count", 3)? != 4 {
        return None;
    }
    Some(GrenadeCounts {
        frag: value(fields, "grenades[0]", 8)? as u8,
        plasma: value(fields, "grenades[1]", 8)? as u8,
        dynamo: value(fields, "grenades[2]", 8)? as u8,
        spike: value(fields, "grenades[3]", 8)? as u8,
    })
}

fn admitted_counts(raw: Option<GrenadeCounts>, maximum: u8) -> Option<GrenadeCounts> {
    raw.filter(|c| {
        [c.frag, c.plasma, c.dynamo, c.spike]
            .into_iter()
            .all(|count| count <= maximum)
    })
}

pub(super) fn observe(
    record: &EntityRecord,
    time: u64,
    maximum: u8,
    source: PoseSource,
) -> Option<GrenadeObservation> {
    if record.stop != EntityViewStop::Complete {
        return None;
    }
    let c = component(record, "unit-grenade-counts-component")?;
    let raw_counts = counts(&c.fields);
    Some(GrenadeObservation {
        timestamp_us: time,
        entity_id: record.header.id?,
        source: PoseSource {
            start_bit: c.start_bit,
            end_bit: c.end_bit,
            ..source
        },
        raw_counts,
        counts: admitted_counts(raw_counts, maximum),
    })
}

pub(super) fn timeline(
    observations: &mut Vec<GrenadeObservation>,
    gaps: Vec<(u64, usize)>,
    lives: &[PlayerLife],
) -> Vec<GrenadeSample> {
    let within_life = |o: &GrenadeObservation| {
        lives.iter().any(|l| {
            l.entity_id == o.entity_id && l.start_us <= o.timestamp_us && o.timestamp_us < l.end_us
        })
    };
    let mut events = BTreeMap::<u64, Vec<(usize, Option<GrenadeCounts>)>>::new();
    for life in lives {
        for time in [life.start_us, life.end_us] {
            events.entry(time).or_default().push((0, None));
        }
    }
    // Boundaries precede observations, but later packets at the same clock
    // keep their order. A later gap must invalidate an earlier reading.
    for (time, preceding_observations) in gaps {
        events
            .entry(time)
            .or_default()
            .push((2 * preceding_observations + 1, None));
    }
    for (i, o) in observations
        .iter()
        .enumerate()
        .filter(|(_, o)| within_life(o))
    {
        events
            .entry(o.timestamp_us)
            .or_default()
            .push((2 * i + 2, o.counts));
    }
    observations.retain(within_life);
    events
        .into_iter()
        .map(|(time, changes)| GrenadeSample {
            timestamp_us: time,
            counts: changes
                .into_iter()
                .max_by_key(|(order, _)| *order)
                .unwrap()
                .1,
        })
        .collect()
}

#[cfg(test)]
mod tests;
