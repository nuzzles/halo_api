//! Inventory publication through the loaded native scan context.
use super::*;
use std::sync::{Arc, Mutex};

#[derive(Debug, thiserror::Error)]
pub enum ContextInventoryScanError {
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
    #[error("aucun composant d'inventaire dans l'archétype biped du film")]
    NoInventoryComponents,
    #[error(transparent)]
    Profile(#[from] NativeProfileResolveError),
    #[error(transparent)]
    Reader(#[from] NativeReaderProfileError),
    #[error(transparent)]
    Walk(#[from] DecodeError),
}

/// Native ScanInventoryDeltas over loaded source buffers. Grenade hooks publish
/// before the component bounds verdict; ammo/rounds require a successful visitor
/// to associate them with a weapon slot. Keep those distinct publication gates.
/// Statistics and all attempted components survive global ammo refusal.
pub fn scan_context_inventory_deltas(
    context: &NativeFilmContext<'_>,
) -> (InventoryDeltaStream, Option<ContextInventoryScanError>) {
    use super::inventory_delta::{
        InventoryRecordValues, Role, publish_inventory_record, refuse_inventory_ammo,
    };
    #[derive(Default)]
    struct Capture {
        values: InventoryRecordValues,
        ammo: Option<(Option<u32>, Option<u32>)>,
        rounds: Option<u32>,
    }
    let mut out = InventoryDeltaStream::default();
    let result = (|| -> Result<(), ContextInventoryScanError> {
        let chunks = context.chunk_numbers();
        if chunks.is_empty() {
            return Err(ContextInventoryScanError::NoChunks);
        }
        let [lo, hi] = context
            .biped_slot_band()
            .ok_or(ContextInventoryScanError::NoBipeds)?;
        let slots = FilmSlotBand::from_slots(lo..=hi)?;
        let detected = context.i0_layout();
        if let Some(refusal) = &detected.refusal {
            return Err(ContextInventoryScanError::Layout(refusal.clone()));
        }
        let layout = detected
            .layout
            .as_ref()
            .ok_or(ContextInventoryScanError::Layout(
                I0LayoutRefusal::Inconclusive,
            ))?;
        let registry = context.registry().map_err(|e| *e)?;
        let arch = registry
            .registry
            .archetype(35)
            .ok_or(ContextInventoryScanError::NoArchetype)?;
        let profile = context.scan_profile()?;

        let roles = super::inventory_delta::roles(arch);
        if roles.is_empty() {
            return Err(ContextInventoryScanError::NoInventoryComponents);
        }
        let capture = Arc::new(Mutex::new(Capture::default()));
        let observer = NativeFilmObserver::default();
        for kind in [
            NativeHookKind::GrenadeCounts,
            NativeHookKind::GrenadeSet,
            NativeHookKind::WeaponAmmo,
            NativeHookKind::WeaponRounds,
        ] {
            let captured = capture.clone();
            observer.set_hook(
                kind,
                Some(Arc::new(move |value| {
                    let NativeHookPublication::Component(value) = value else {
                        return;
                    };
                    let mut c = captured.lock().unwrap();
                    match value {
                        FilmComponentObservation::GrenadeCounts { count, values } => {
                            c.values.counts =
                                Some((*count, values.iter().map(|v| *v as u32).collect()))
                        }
                        FilmComponentObservation::GrenadeSet { mask, selection } => {
                            c.values.selection = Some((*mask, *selection as u8))
                        }
                        FilmComponentObservation::WeaponAmmo { magazine, fraction } => {
                            c.ammo = Some((*magazine, *fraction))
                        }
                        FilmComponentObservation::WeaponRounds { rounds } => {
                            c.rounds = Some(*rounds)
                        }
                        _ => {}
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
            let wanted: Vec<_> = r
                .component_indices
                .iter()
                .skip(1)
                .filter_map(|id| roles.get(id).map(|role| (*id, *role)))
                .collect();
            let Some(&(last, _)) = wanted.last() else {
                return;
            };
            *capture.lock().unwrap() = Capture::default();
            for &(_, role) in &wanted {
                match role {
                    Role::Grenades => out.stats.with_i22 += 1,
                    Role::Selection => out.stats.with_i47 += 1,
                    Role::Ammo(_) => out.stats.with_ammo += 1,
                    Role::Rounds(_) => out.stats.with_rounds += 1,
                }
            }
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
                out.component_attempts.push(attempt.clone());
                if status != Some(true) || !in_bounds {
                    break;
                }
                if let Some(role) = roles.get(&id) {
                    let mut c = capture.lock().unwrap();
                    match role {
                        Role::Ammo(slot) => c.values.ammo[*slot] = c.ammo.take(),
                        Role::Rounds(slot) => c.values.rounds[*slot] = c.rounds.take(),
                        _ => {}
                    }
                    out.components.push(attempt.read);
                }
                if id >= last {
                    break;
                }
            }
            let values = std::mem::take(&mut capture.lock().unwrap().values);
            publish_inventory_record(
                &mut out,
                InventoryDeltaRead {
                    chunk_number: Some(r.chunk),
                    source: r.source,
                    packet_index: Some(r.packet_index),
                    slot: r.slot,
                    grenades: None,
                    selection: None,
                    ammo: vec![],
                },
                values,
            );
        })?;
        if let Some(e) = reader_error {
            return Err(e.into());
        }
        refuse_inventory_ammo(&mut out);
        Ok(())
    })();
    (out, result.err())
}

/// Native cache omits delta ammo, locations and refusal diagnostics. They remain
/// in InventoryDeltaStream; this projection does not erase them from the scan.
impl From<&InventoryDeltaRead> for FactsInventoryDelta {
    fn from(r: &InventoryDeltaRead) -> Self {
        Self {
            timestamp_us: r.source.timestamp_us,
            slot: r.slot,
            grenades: r.grenades.map(|g| g.to_vec()),
            selection_read: r.selection.is_some(),
            selection: r
                .selection
                .as_ref()
                .map_or(0, |s| s.rank.map_or(-1, i64::from)),
            mask: r.selection.as_ref().map_or(0, |s| s.mask),
        }
    }
}
