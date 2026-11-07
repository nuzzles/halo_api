//! Source-linked projectile tracks from complete sequential archetype-41 records.
use super::*;

/// Known projectile tags identified from complete creation records.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectileKind {
    Frag,
    Plasma,
    Dynamo,
    Spike,
    Rocket,
}
impl ProjectileKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Frag => "frag",
            Self::Plasma => "plasma",
            Self::Dynamo => "dynamo",
            Self::Spike => "spike",
            Self::Rocket => "rocket",
        }
    }
    fn from_tag(tag: u64) -> Option<Self> {
        match tag {
            0x580b8831 => Some(Self::Frag),
            0x6071a622 => Some(Self::Plasma),
            0x1d92b3ea => Some(Self::Dynamo),
            0x49097214 => Some(Self::Spike),
            0x267e2ad4 => Some(Self::Rocket),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct ProjectilePosition {
    pub timestamp_us: u64,
    pub position: [f64; 3],
    pub source: PoseSource,
}

#[derive(Debug, Clone, Serialize)]
pub struct TrackedProjectile {
    pub entity_id: u32,
    pub owner_index: usize,
    pub kind: ProjectileKind,
    pub start_us: u64,
    /// Exclusive visibility bound. This is not a claimed detonation time.
    pub end_us: u64,
    pub positions: Vec<ProjectilePosition>,
    /// Observed deletion, which can mark an explosion but is not itself a damage event.
    pub deleted: bool,
    /// Track stopped because the replication stream became unreadable.
    pub interrupted: bool,
}
impl TrackedProjectile {
    pub fn position_at(&self, time: u64) -> Option<[f64; 3]> {
        if time < self.start_us || time >= self.end_us {
            return None;
        }
        let next = self.positions.partition_point(|p| p.timestamp_us <= time);
        let p = self.positions.get(next.checked_sub(1)?)?;
        (time.saturating_sub(p.timestamp_us) <= FRESHNESS_US).then_some(p.position)
    }
}

fn position(
    record: &EntityRecord,
    bounds: PositionBounds,
    region_index_bits: usize,
    source: PoseSource,
) -> Option<ProjectilePosition> {
    let c = component(record, "object-position-component")?;
    let fields = &c.fields;
    if value(fields, "high_precision", 1)? != 0
        || value(fields, "region.gate", 1)? != 0
        || value(fields, "region", region_index_bits)? != u64::from(bounds.region)
    {
        return None;
    }
    value(fields, "finite", 2)?;
    let widths = bounds.axis_bits()?;
    let mut raw = [0; 3];
    for axis in 0..3 {
        raw[axis] = value(fields, &format!("position[{axis}]"), widths[axis])? as u32;
    }
    Some(ProjectilePosition {
        timestamp_us: 0,
        position: bounds.dequantize(raw, widths)?,
        source: PoseSource {
            start_bit: c.start_bit,
            end_bit: c.end_bit,
            ..source
        },
    })
}

#[derive(Default)]
pub(super) struct Projectiles {
    active: BTreeMap<u32, TrackedProjectile>,
    completed: Vec<TrackedProjectile>,
}
impl Projectiles {
    fn close(&mut self, id: u32, time: u64, deleted: bool, interrupted: bool) {
        if let Some(mut track) = self.active.remove(&id) {
            let last = track.positions.last().unwrap().timestamp_us;
            track.end_us = time.min(last.saturating_add(FRESHNESS_US + 1));
            track.deleted = deleted && time <= last.saturating_add(FRESHNESS_US);
            track.interrupted = interrupted;
            if track.positions.len() >= 2 && track.end_us > track.start_us {
                self.completed.push(track);
            }
        }
    }

    pub(super) fn clear(&mut self, time: u64) {
        for id in self.active.keys().copied().collect::<Vec<_>>() {
            self.close(id, time, false, true);
        }
    }

    pub(super) fn observe(
        &mut self,
        record: &EntityRecord,
        time: u64,
        source: PoseSource,
        bounds: PositionBounds,
        region_index_bits: usize,
        player_bindings: &BTreeMap<u32, usize>,
    ) {
        let Some(id) = record.header.id else { return };
        for old in self
            .active
            .keys()
            .copied()
            .filter(|old| *old != id && (*old & 0x3fff_ffff) == (id & 0x3fff_ffff))
            .collect::<Vec<_>>()
        {
            self.close(old, time, false, false);
        }
        match record.header.kind {
            RecordKind::Delete => self.close(id, time, true, false),
            RecordKind::New => {
                self.close(id, time, false, false);
                if record.archetype != Some(41) || record.stop != EntityViewStop::Complete {
                    return;
                }
                let Some(defaults) = record
                    .default_state
                    .as_ref()
                    .filter(|d| d.status == DefaultStateStatus::Complete)
                else {
                    return;
                };
                let fields = &defaults.fields;
                let Some(kind) =
                    value(fields, "mpp.object_tag", 32).and_then(ProjectileKind::from_tag)
                else {
                    return;
                };
                if value(fields, "projectile.reference_64.present", 1) != Some(1)
                    || value(fields, "projectile.reference_64.category4", 1) != Some(1)
                {
                    return;
                }
                let (Some(reference), Some(generation)) = (
                    value(fields, "projectile.reference_64.value", 9),
                    value(fields, "projectile.reference_64.generation", 2),
                ) else {
                    return;
                };
                let owner_id = ((reference as u32) + 512) | ((generation as u32) << 30);
                let Some(&owner_index) = player_bindings.get(&owner_id) else {
                    return;
                };
                let Some(mut sample) = position(record, bounds, region_index_bits, source) else {
                    return;
                };
                sample.timestamp_us = time;
                self.active.insert(
                    id,
                    TrackedProjectile {
                        entity_id: id,
                        owner_index,
                        kind,
                        start_us: time,
                        end_us: time,
                        positions: vec![sample],
                        deleted: false,
                        interrupted: false,
                    },
                );
            }
            RecordKind::Delta => {
                if !self.active.contains_key(&id) {
                    return;
                }
                if record.stop != EntityViewStop::Complete {
                    self.close(id, time, false, false);
                    return;
                }
                if record
                    .components
                    .iter()
                    .any(|c| c.name == "object-position-component")
                {
                    let Some(mut sample) = position(record, bounds, region_index_bits, source)
                    else {
                        self.close(id, time, false, false);
                        return;
                    };
                    let last = self.active[&id].positions.last().unwrap().timestamp_us;
                    if time.saturating_sub(last) > FRESHNESS_US {
                        self.close(id, time, false, false);
                        return;
                    }
                    sample.timestamp_us = time;
                    if last == time {
                        *self
                            .active
                            .get_mut(&id)
                            .unwrap()
                            .positions
                            .last_mut()
                            .unwrap() = sample;
                    } else {
                        self.active.get_mut(&id).unwrap().positions.push(sample);
                    }
                }
            }
            RecordKind::End => {}
        }
    }

    pub(super) fn finish(mut self, duration_us: u64) -> Vec<TrackedProjectile> {
        for id in self.active.keys().copied().collect::<Vec<_>>() {
            self.close(id, duration_us, false, false);
        }
        self.completed.sort_by_key(|p| (p.start_us, p.entity_id));
        self.completed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projectile_position_uses_map_region_index_width() {
        let bounds = PositionBounds {
            region: 1,
            min: [-16.719454, -10.102863, -9.330649],
            max: [46.513535, 53.67172, 13.5679],
        };
        assert_eq!(bounds.axis_bits(), Some([12, 12, 11]));
        let fields = [
            ("high_precision", 1, 0),
            ("region.gate", 1, 0),
            ("region", 2, 1),
            ("position[0]", 12, 2048),
            ("position[1]", 12, 2048),
            ("position[2]", 11, 1024),
            ("finite", 2, 0),
        ]
        .into_iter()
        .map(|(name, width, raw)| ComponentField {
            name: name.into(),
            bit: 0,
            width,
            raw: RawBits::from_low(raw, width),
        })
        .collect();
        let record = EntityRecord {
            header: RecordHeader {
                prefix: None,
                kind: RecordKind::New,
                id: Some(0x40000400),
                start_bit: 0,
                end_bit: 0,
            },
            archetype: Some(41),
            mask: None,
            fields: Vec::new(),
            default_state: None,
            components: vec![EntityComponentRead {
                index: 0,
                name: "object-position-component".into(),
                start_bit: 0,
                end_bit: 41,
                status: ComponentReadStatus::Complete,
                fields,
                weapon_id: None,
            }],
            end_bit: 41,
            stop: EntityViewStop::Complete,
        };
        let source = PoseSource {
            chunk_index: Some(1),
            payload_byte: 0,
            start_bit: 0,
            end_bit: 41,
        };
        assert!(position(&record, bounds, 2, source).is_some());
        let mut nonzero_tail = record.clone();
        nonzero_tail.components[0].fields.last_mut().unwrap().raw = RawBits::from_low(3, 2);
        assert_eq!(
            position(&nonzero_tail, bounds, 2, source).map(|p| p.position),
            position(&record, bounds, 2, source).map(|p| p.position)
        );
        assert!(position(&record, bounds, 1, source).is_none());
    }

    #[test]
    fn exact_tags_and_bounded_samples() {
        assert_eq!(
            ProjectileKind::from_tag(0x580b8831),
            Some(ProjectileKind::Frag)
        );
        assert_eq!(
            ProjectileKind::from_tag(0x6071a622),
            Some(ProjectileKind::Plasma)
        );
        assert_eq!(
            ProjectileKind::from_tag(0x49097214),
            Some(ProjectileKind::Spike)
        );
        assert_eq!(
            ProjectileKind::from_tag(0x1d92b3ea),
            Some(ProjectileKind::Dynamo)
        );
        assert_eq!(
            ProjectileKind::from_tag(0x267e2ad4),
            Some(ProjectileKind::Rocket)
        );
        let source = PoseSource {
            chunk_index: Some(1),
            payload_byte: 0,
            start_bit: 0,
            end_bit: 1,
        };
        let track = TrackedProjectile {
            entity_id: 0x40000500,
            owner_index: 2,
            kind: ProjectileKind::Frag,
            start_us: 100,
            end_us: 300_000,
            deleted: false,
            interrupted: false,
            positions: vec![
                ProjectilePosition {
                    timestamp_us: 100,
                    position: [1., 2., 3.],
                    source,
                },
                ProjectilePosition {
                    timestamp_us: 200_000,
                    position: [4., 5., 6.],
                    source,
                },
            ],
        };
        assert_eq!(track.position_at(99), None);
        assert_eq!(track.position_at(100), Some([1., 2., 3.]));
        assert_eq!(track.position_at(100_101), None);
        assert_eq!(track.position_at(200_000), Some([4., 5., 6.]));
        assert_eq!(track.position_at(300_000), None);
    }

    #[test]
    fn film_end_does_not_look_like_an_interrupted_track() {
        let sample = |time| ProjectilePosition {
            timestamp_us: time,
            position: [0., 0., 0.],
            source: PoseSource {
                chunk_index: Some(1),
                payload_byte: 0,
                start_bit: 0,
                end_bit: 1,
            },
        };
        let track = TrackedProjectile {
            entity_id: 1,
            owner_index: 0,
            kind: ProjectileKind::Rocket,
            start_us: 0,
            end_us: 0,
            positions: vec![sample(0), sample(10_000)],
            deleted: false,
            interrupted: false,
        };
        let mut at_film_end = Projectiles::default();
        at_film_end.active.insert(1, track.clone());
        let finished = at_film_end.finish(20_000);
        assert_eq!(finished.len(), 1);
        assert!(!finished[0].interrupted);

        let mut on_refused_frame = Projectiles::default();
        on_refused_frame.active.insert(1, track);
        on_refused_frame.clear(20_000);
        let finished = on_refused_frame.finish(30_000);
        assert_eq!(finished.len(), 1);
        assert!(finished[0].interrupted);
    }
}
