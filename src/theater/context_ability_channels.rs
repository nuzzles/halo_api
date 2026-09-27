//! Independent ability-identity and camouflage scans over loaded source buffers.
use super::*;
use std::sync::{Arc, Mutex};

#[derive(Debug, thiserror::Error)]
pub enum ContextAbilityChannelError {
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
    #[error("composant \"unit-active-camo-state-component\" absent de l'archétype biped du film")]
    NoCamoComponent,
    #[error(
        "composant \"biped-spartan-ability-non-predicted-state-component\" absent de l'archétype biped du film"
    )]
    NoGrappleComponent,
    #[error(transparent)]
    Profile(#[from] NativeProfileResolveError),
    #[error(transparent)]
    Reader(#[from] NativeReaderProfileError),
    #[error(transparent)]
    Walk(#[from] DecodeError),
}
/// The latest channel hook after a successful target visit. All component
/// attempts, including the exact callback's component, are retained separately.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextChannelObservation {
    pub chunk: i64,
    pub source: FilmPacket,
    pub packet_index: usize,
    pub slot: u32,
    pub record_start_bit: usize,
    pub value: FilmComponentObservation,
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ContextAbilityChannelScan {
    pub stats: BipedChannelStats,
    /// Includes successfully decoded no-equipment/no-camo-channel states.
    pub observations: Vec<ContextChannelObservation>,
    pub component_attempts: Vec<BipedComponentAttempt>,
}
impl ContextAbilityChannelScan {
    /// Native ScanAbilityRanks filters only the explicit no-rank sentinel.
    /// The raw no-equipment emissions remain available for equipment changes.
    pub fn ability_ranks(
        &self,
    ) -> impl Iterator<Item = (&ContextChannelObservation, FactsAbilityRank)> {
        self.observations.iter().filter_map(|r| match &r.value {
            FilmComponentObservation::AbilitySet { rank, .. } if *rank != -1 => Some((
                r,
                FactsAbilityRank {
                    timestamp_us: r.source.timestamp_us,
                    slot: r.slot,
                    rank: i64::from(*rank),
                },
            )),
            _ => None,
        })
    }
    /// Preserve nonbinary quanta without interpreting a third camouflage state.
    pub fn camo_states(
        &self,
    ) -> impl Iterator<Item = (&ContextChannelObservation, FactsCamoState)> {
        self.observations.iter().filter_map(|r| match &r.value {
            FilmComponentObservation::CamoState { state } => state.sub[1].map(|quantum| {
                (
                    r,
                    FactsCamoState {
                        timestamp_us: r.source.timestamp_us,
                        slot: r.slot,
                        quantum,
                    },
                )
            }),
            _ => None,
        })
    }
}
#[derive(Clone, Copy)]
enum Channel {
    Ability,
    Camo,
    Grapple,
}
/// Native walkAbilityEmissions plus ScanAbilityRanks' filtered view. Targets
/// index 48 as the pinned parser does, and keeps every no-equipment emission.
pub fn scan_context_ability_emissions(
    context: &NativeFilmContext<'_>,
) -> (
    ContextAbilityChannelScan,
    Option<ContextAbilityChannelError>,
) {
    scan_channel(context, Channel::Ability)
}
/// Native ScanCamoStates. Resolves the first component by name independently of
/// the ability scan, and does not require a decoded ability rank for publication.
pub fn scan_context_camo_states(
    context: &NativeFilmContext<'_>,
) -> (
    ContextAbilityChannelScan,
    Option<ContextAbilityChannelError>,
) {
    scan_channel(context, Channel::Camo)
}
fn scan_channel(
    context: &NativeFilmContext<'_>,
    channel: Channel,
) -> (
    ContextAbilityChannelScan,
    Option<ContextAbilityChannelError>,
) {
    let mut out = ContextAbilityChannelScan::default();
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
        // Native ability setup resolves the profile BEFORE its registry. Camo
        // resolves it AFTER both registry and component-name admission.
        let early_profile = match channel {
            Channel::Ability => Some(context.scan_profile()?),
            Channel::Camo | Channel::Grapple => None,
        };
        let registry = context.registry().map_err(|e| *e)?;
        let arch = registry
            .registry
            .archetype(35)
            .ok_or(ContextAbilityChannelError::NoArchetype)?;
        let (target, kind) = match channel {
            Channel::Ability => (48, NativeHookKind::AbilitySet),
            Channel::Grapple => (
                super::ability_states::component_index(
                    arch,
                    "biped-spartan-ability-non-predicted-state",
                )
                .ok_or(ContextAbilityChannelError::NoGrappleComponent)?,
                NativeHookKind::AbilityNonPredicted,
            ),
            Channel::Camo => (
                arch.components
                    .iter()
                    .position(|n| n == "unit-active-camo-state-component")
                    .ok_or(ContextAbilityChannelError::NoCamoComponent)? as u8,
                NativeHookKind::CamoState,
            ),
        };
        let profile = match early_profile {
            Some(p) => p,
            None => context.scan_profile()?,
        };
        let latest = Arc::new(Mutex::new(None));
        let captured = latest.clone();
        let observer = NativeFilmObserver::default();
        observer.set_hook(
            kind,
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
            let found = match super::context_delta_walk::walk_context_record_to(
                &r,
                arch,
                &reader_context,
                target,
                &mut out.component_attempts,
            ) {
                Ok(found) => found,
                Err(e) => {
                    reader_error = Some(e);
                    return;
                }
            };
            let value = latest.lock().unwrap().take();
            if (!found && !matches!(channel, Channel::Grapple)) || value.is_none() {
                out.stats.unread += 1;
                return;
            }
            let value = value.unwrap();
            out.stats.read += 1;
            out.stats.gated += usize::from(match &value {
                FilmComponentObservation::AbilitySet { rank, .. } => *rank == -1,
                FilmComponentObservation::CamoState { state } => state.sub[1].is_none(),
                _ => false,
            });
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
        Ok(())
    })();
    (out, result.err())
}

/// Native ScanGrappleReads accepts a published hook even when its component
/// later fails dispatch/bounds checks. Broken known bodies remain observations.
pub fn scan_context_grapple_reads(
    context: &NativeFilmContext<'_>,
) -> (
    ContextAbilityChannelScan,
    Option<ContextAbilityChannelError>,
) {
    scan_channel(context, Channel::Grapple)
}
impl ContextAbilityChannelScan {
    pub fn grapple_reads(
        &self,
    ) -> impl Iterator<Item = (&ContextChannelObservation, FactsGrappleRead)> {
        self.observations.iter().filter_map(|r| match &r.value {
            FilmComponentObservation::AbilityNonPredicted { state }
                if state.tag == 3 && state.body_ok && matches!(state.inner, Some(1 | 2)) =>
            {
                Some((
                    r,
                    FactsGrappleRead {
                        timestamp_us: r.source.timestamp_us,
                        slot: r.slot,
                        heavy: state.inner == Some(2),
                        position_quanta: state.position,
                    },
                ))
            }
            _ => None,
        })
    }
    pub fn grapple_stats(&self) -> GrappleStats {
        let mut out = GrappleStats {
            records: self.stats.records,
            with_component: self.stats.with_component,
            read: self.stats.read,
            unread: self.stats.unread,
            ..Default::default()
        };
        for r in &self.observations {
            if let FilmComponentObservation::AbilityNonPredicted { state } = &r.value
                && state.tag == 3
            {
                out.tag_three += 1;
                out.body_broken += usize::from(!state.body_ok);
            }
        }
        out
    }
}
