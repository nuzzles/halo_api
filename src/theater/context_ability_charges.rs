//! Loaded-context energy reads. Unarmed slots have no transmitted charge value.
use super::*;
use std::sync::{Arc, Mutex};
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ContextChargeScan {
    pub stats: AbilityChargeStats,
    pub component_attempts: Vec<BipedComponentAttempt>,
    /// Full hook publications in traversal order, including mask-zero reads.
    pub observations: Vec<ContextChannelObservation>,
    /// Stable time/slot/emplacement order, referencing the original observation.
    pub charges: Vec<ContextChargeRead>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextChargeRead {
    pub observation_index: usize,
    pub emplacement: u8,
    pub charges: u8,
    pub low: u8,
}
impl ContextChargeScan {
    pub fn facts(&self) -> impl Iterator<Item = (&ContextChannelObservation, FactsAbilityCharge)> {
        self.charges.iter().map(|v| {
            let r = &self.observations[v.observation_index];
            (
                r,
                FactsAbilityCharge {
                    timestamp_us: r.source.timestamp_us,
                    slot: r.slot,
                    emplacement: i64::from(v.emplacement),
                    charges: i64::from(v.charges),
                    low: i64::from(v.low),
                },
            )
        })
    }
}
/// Native ScanAbilityCharges: hook publication survives a failed target visit.
pub fn scan_context_ability_charges(
    context: &NativeFilmContext<'_>,
) -> (ContextChargeScan, Option<ContextAbilityChannelError>) {
    let mut out = ContextChargeScan::default();
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
        let Some(target) =
            super::ability_states::component_index(arch, "biped-spartan-ability-energy")
        else {
            out.stats.absent = true;
            out.stats.scanned = true;
            return Ok(());
        };
        let latest = Arc::new(Mutex::new(None));
        let captured = latest.clone();
        let observer = NativeFilmObserver::default();
        observer.set_hook(
            NativeHookKind::AbilityEnergy,
            Some(Arc::new(move |v| {
                if let NativeHookPublication::Component(value) = v {
                    *captured.lock().unwrap() = Some(value.clone());
                }
            })),
        );
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
            if !r.component_indices.contains(&target) {
                return;
            }
            out.stats.with_component += 1;
            *latest.lock().unwrap() = None;
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
            let Some(value) = latest.lock().unwrap().take() else {
                out.stats.unread += 1;
                return;
            };
            out.stats.read += 1;
            if let FilmComponentObservation::AbilityEnergy { mask, charges } = &value {
                for (i, raw) in charges.iter().enumerate() {
                    if mask & (1 << i) == 0 {
                        continue;
                    }
                    out.stats.armed += 1;
                    out.charges.push(ContextChargeRead {
                        observation_index: out.observations.len(),
                        emplacement: i as u8,
                        charges: ((raw >> 4) & 15) as u8,
                        low: (raw & 15) as u8,
                    });
                }
            }
            out.observations.push(ContextChannelObservation {
                chunk: r.chunk,
                source: r.source,
                packet_index: r.packet_index,
                slot: r.slot,
                record_start_bit: r.start_bit,
                value,
            });
        })?;
        if let Some(e) = reader_error {
            return Err(e.into());
        }
        out.charges.sort_by_key(|v| {
            let r = &out.observations[v.observation_index];
            (r.source.timestamp_us, r.slot, v.emplacement)
        });
        out.stats.scanned = true;
        Ok(())
    })();
    (out, result.err())
}
impl From<&AbilityChargeStats> for FactsAbilityChargeStats {
    fn from(s: &AbilityChargeStats) -> Self {
        Self {
            records: s.records as i64,
            with_i56: s.with_component as i64,
            read: s.read as i64,
            unread: s.unread as i64,
            armed: s.armed as i64,
            absent: s.absent,
            scanned: s.scanned,
        }
    }
}
