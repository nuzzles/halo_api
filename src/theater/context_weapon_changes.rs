//! Held weapon publication through the loaded native scan context.
use super::*;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, thiserror::Error)]
pub enum ContextWeaponScanError {
    #[error("aucun chunk de donnees dans le film")]
    NoChunks,
    #[error("aucun slot biped (ti=35) dans les keyframes du film")]
    NoBipeds,
    #[error("découpage i0 illisible : {0:?}")]
    Layout(I0LayoutRefusal),
    #[error(transparent)]
    Registry(#[from] NativeContextRegistryError),
    #[error("archétype biped 35 absent du registre")]
    NoArchetype,
    #[error("aucun weapon-state-type-info dans l'archétype biped du film")]
    NoWeaponComponents,
    #[error(transparent)]
    Profile(#[from] NativeProfileResolveError),
    #[error(transparent)]
    Reader(#[from] NativeReaderProfileError),
    #[error(transparent)]
    Walk(#[from] DecodeError),
}

/// Native ScanHeldWeaponChanges. Resolve the context in native admission order,
/// then walk every announced component with a fresh reader and a scan-local
/// observer. Failed and post-weapon attempts remain available. Hook state survives
/// a failed walk until a successful visitor consumes/clears it, as in the native
/// implementation. Return retained data alongside any Rust reader-domain error.
pub fn scan_context_held_weapon_changes(
    context: &NativeFilmContext<'_>,
    loadouts: &[KeyframeLoadout],
) -> (HeldWeaponChangeStream, Option<ContextWeaponScanError>) {
    let mut out = HeldWeaponChangeStream::default();
    let result = (|| -> Result<(), ContextWeaponScanError> {
        let chunks = context.chunk_numbers();
        if chunks.is_empty() {
            return Err(ContextWeaponScanError::NoChunks);
        }
        let [lo, hi] = context
            .biped_slot_band()
            .ok_or(ContextWeaponScanError::NoBipeds)?;
        let slots = FilmSlotBand::from_slots(lo..=hi)?;
        let detected = context.i0_layout();
        if let Some(refusal) = &detected.refusal {
            return Err(ContextWeaponScanError::Layout(refusal.clone()));
        }
        let layout = detected
            .layout
            .as_ref()
            .ok_or(ContextWeaponScanError::Layout(
                I0LayoutRefusal::Inconclusive,
            ))?;
        let registry = context.registry().map_err(|e| *e)?;
        let arch = registry
            .registry
            .archetype(35)
            .ok_or(ContextWeaponScanError::NoArchetype)?;
        let profile = context.scan_profile()?;
        let weapons: Vec<u8> = arch
            .components
            .iter()
            .take(64)
            .enumerate()
            .filter_map(|(i, n)| (n == "weapon-state-type-info").then_some(i as u8))
            .collect();
        if weapons.is_empty() {
            return Err(ContextWeaponScanError::NoWeaponComponents);
        }
        let mut by_slot = BTreeMap::<u32, Vec<&KeyframeLoadout>>::new();
        for sample in loadouts {
            by_slot.entry(sample.slot).or_default().push(sample);
        }
        let last = Arc::new(Mutex::new(None));
        let observer = NativeFilmObserver::default();
        let captured = last.clone();
        observer.set_hook(
            NativeHookKind::HeldWeapon,
            Some(Arc::new(move |value| {
                if let NativeHookPublication::Component(FilmComponentObservation::HeldWeapon {
                    id_high,
                    id_low,
                }) = value
                {
                    *captured.lock().unwrap() = Some((*id_high, *id_low));
                }
            })),
        );
        let reader_context = NativeReaderContext {
            profile,
            observer: Some(observer),
        };
        let mut previous = BTreeMap::new();
        let mut reader_error = None;
        walk_context_delta_bipeds(context, chunks, &slots, layout, |r| {
            if reader_error.is_some() {
                return;
            }
            out.stats.records += 1;
            if !r.component_indices.iter().any(|i| weapons.contains(i)) {
                return;
            }
            out.stats.with_component += 1;
            let mut at = (r.position_end_bit as i64).wrapping_add(2);
            for &id in r.component_indices.iter().skip(1) {
                let Some(name) = arch
                    .components
                    .get(usize::from(id))
                    .filter(|n| !n.is_empty())
                else {
                    break;
                };
                let mut reader = NativeFilmReader::with_context(r.payload, reader_context.clone());
                reader.set_native_bit_position(at);
                let (status, component) = match reader.read_component(
                    name,
                    arch.levels.get(usize::from(id)).copied().unwrap_or(0),
                    35,
                ) {
                    Ok(v) => v,
                    Err(e) => {
                        reader_error = Some(e);
                        break;
                    }
                };
                let in_bounds =
                    component.end_bit >= 0 && component.end_bit <= (r.payload.len() * 8) as i64;
                at = component.end_bit;
                let attempt = BipedComponentAttempt {
                    component_index: id,
                    status,
                    in_bounds,
                    read: BipedChannelRead {
                        packet_index: Some(r.packet_index),
                        source: r.source,
                        slot: r.slot,
                        record_start_bit: r.start_bit,
                        component,
                    },
                };
                if status != Some(true) || !in_bounds {
                    out.rejected_components.push(attempt.clone());
                }
                out.component_attempts.push(attempt);
                if status != Some(true) || !in_bounds {
                    break;
                }
                let got = last.lock().unwrap().take();
                if !weapons.contains(&id) {
                    continue;
                }
                let Some((family, low)) = got else { continue };
                let prev = previous.insert((r.slot, id), family);
                out.stats.emissions += 1;
                out.stats.repeats += usize::from(prev == Some(family));
                let mut change = HeldWeaponChange {
                    timestamp_us: r.source.timestamp_us,
                    chunk: r.chunk,
                    slot: r.slot,
                    slot_index: id,
                    family,
                    low,
                    previous: prev.unwrap_or(u32::MAX),
                    kind: HeldWeaponChangeKind::Taken,
                };
                change.kind = super::weapon_changes::classify(
                    &change,
                    prev.is_some(),
                    by_slot.get(&r.slot).map(Vec::as_slice).unwrap_or_default(),
                );
                out.records.push(change);
            }
        })?;
        if let Some(e) = reader_error {
            return Err(e.into());
        }
        Ok(())
    })();
    (out, result.err())
}

/// Native cache omits the source file number and low variant identity; both
/// remain in the scan result. Cache kind values follow the native lower case.
impl From<&HeldWeaponChange> for FactsWeaponChange {
    fn from(c: &HeldWeaponChange) -> Self {
        Self {
            timestamp_us: c.timestamp_us,
            slot: c.slot,
            slot_index: i64::from(c.slot_index),
            family: c.family,
            previous: c.previous,
            kind: match c.kind {
                HeldWeaponChangeKind::Taken => b"taken".to_vec(),
                HeldWeaponChangeKind::Dropped => b"dropped".to_vec(),
                HeldWeaponChangeKind::Swapped => b"swapped".to_vec(),
                HeldWeaponChangeKind::Restated => b"restated".to_vec(),
            },
        }
    }
}
