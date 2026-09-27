//! Native predicted/non-predicted impulse scan over loaded context buffers.
use super::*;
use std::sync::{Arc, Mutex};
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ContextImpulseScan {
    pub stats: AbilityImpulseStats,
    pub component_attempts: Vec<BipedComponentAttempt>,
    /// Every announced channel whose hook fired, including non-impulse tags.
    pub observations: Vec<ContextChannelObservation>,
    /// Tag-one observations in stable native time/slot/predicted-first order.
    pub impulses: Vec<ContextChannelObservation>,
}
impl ContextImpulseScan {
    pub fn facts(&self) -> impl Iterator<Item = (&ContextChannelObservation, FactsAbilityImpulse)> {
        self.impulses.iter().map(|r| {
            (
                r,
                FactsAbilityImpulse {
                    timestamp_us: r.source.timestamp_us,
                    slot: r.slot,
                    predicted: matches!(r.value, FilmComponentObservation::SpartanAbility { .. }),
                },
            )
        })
    }
}
/// Native ScanAbilityImpulses. Either named channel may be absent; absence of
/// both is a successful Scanned/Absent verdict. Hook publication, rather than a
/// completed target walk, governs reads. Both co-transmitted channels contribute.
pub fn scan_context_ability_impulses(
    context: &NativeFilmContext<'_>,
) -> (ContextImpulseScan, Option<ContextAbilityChannelError>) {
    let mut out = ContextImpulseScan::default();
    let result = (|| -> Result<(), ContextAbilityChannelError> {
        let chunks = context.chunk_numbers();
        if chunks.is_empty() {
            return Err(ContextAbilityChannelError::NoChunks);
        }
        let [lo, hi] = context
            .biped_slot_band()
            .ok_or(ContextAbilityChannelError::NoBipeds)?;
        let slots = FilmSlotBand::from_slots(lo..=hi)?;
        let detection = context.i0_layout();
        if let Some(reason) = &detection.refusal {
            return Err(ContextAbilityChannelError::Layout(reason.clone()));
        }
        let layout = detection
            .layout
            .as_ref()
            .ok_or(ContextAbilityChannelError::Layout(
                I0LayoutRefusal::Inconclusive,
            ))?;
        let profile = context.scan_profile()?;
        let registry = context.registry().map_err(|e| *e)?;
        let arch = registry
            .registry
            .archetype(35)
            .ok_or(ContextAbilityChannelError::NoArchetype)?;
        let targets = [
            super::ability_states::component_index(arch, "biped-spartan-ability"),
            super::ability_states::component_index(
                arch,
                "biped-spartan-ability-non-predicted-state",
            ),
        ];
        if targets.iter().all(Option::is_none) {
            out.stats.absent = true;
            out.stats.scanned = true;
            return Ok(());
        }
        let latest = Arc::new(Mutex::new([None, None]));
        let observer = NativeFilmObserver::default();
        for (index, kind) in [
            NativeHookKind::SpartanAbility,
            NativeHookKind::AbilityNonPredicted,
        ]
        .into_iter()
        .enumerate()
        {
            let captured = latest.clone();
            observer.set_hook(
                kind,
                Some(Arc::new(move |v| {
                    if let NativeHookPublication::Component(value) = v {
                        captured.lock().unwrap()[index] = Some(value.clone());
                    }
                })),
            );
        }
        let reader_context = NativeReaderContext {
            profile,
            observer: Some(observer),
        };
        let mut reader_error = None;
        walk_context_delta_bipeds(context, chunks, &slots, layout, |r| {
            if reader_error.is_some() {
                return;
            }
            out.stats.records += 1;
            let announced = targets.map(|id| id.filter(|id| r.component_indices.contains(id)));
            let Some(target) = announced.iter().flatten().copied().max() else {
                return;
            };
            out.stats.with_predicted += usize::from(announced[0].is_some());
            out.stats.with_non_predicted += usize::from(announced[1].is_some());
            *latest.lock().unwrap() = [None, None];
            if let Err(e) = super::context_delta_walk::walk_context_record_to(
                &r,
                arch,
                &reader_context,
                target,
                &mut out.component_attempts,
            ) {
                reader_error = Some(e);
                return;
            }
            let reads = std::mem::take(&mut *latest.lock().unwrap());
            for (index, value) in reads.into_iter().enumerate() {
                if announced[index].is_none() {
                    continue;
                }
                let Some(value) = value else {
                    out.stats.unread += 1;
                    continue;
                };
                out.stats.read += 1;
                let tag = match &value {
                    FilmComponentObservation::SpartanAbility { tag, .. } => *tag,
                    FilmComponentObservation::AbilityNonPredicted { state } => u64::from(state.tag),
                    _ => unreachable!(),
                };
                let observation = ContextChannelObservation {
                    chunk: r.chunk,
                    source: r.source,
                    packet_index: r.packet_index,
                    slot: r.slot,
                    record_start_bit: r.start_bit,
                    value,
                };
                if tag == 1 {
                    out.stats.tag_one += 1;
                    out.impulses.push(observation.clone());
                }
                out.observations.push(observation);
            }
        })?;
        if let Some(e) = reader_error {
            return Err(e.into());
        }
        out.impulses.sort_by_key(|r| {
            (
                r.source.timestamp_us,
                r.slot,
                !matches!(r.value, FilmComponentObservation::SpartanAbility { .. }),
            )
        });
        out.stats.scanned = true;
        Ok(())
    })();
    (out, result.err())
}
impl From<&AbilityImpulseStats> for FactsAbilityImpulseStats {
    fn from(s: &AbilityImpulseStats) -> Self {
        Self {
            records: s.records as i64,
            with_i57: s.with_predicted as i64,
            with_i59: s.with_non_predicted as i64,
            read: s.read as i64,
            unread: s.unread as i64,
            tag1: s.tag_one as i64,
            absent: s.absent,
            scanned: s.scanned,
        }
    }
}
