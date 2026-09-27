//! Native strict emissions, bounded recovery and counter-gated equipment assembly.
use super::*;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextEquipmentRecoveryAttempt {
    pub chunk: i64,
    pub source: FilmPacket,
    pub packet_index: usize,
    pub slot: u32,
    pub bit_offset: usize,
    pub window_index: usize,
    pub probe: ContextEquipmentRecoveryProbe,
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ContextEquipmentScan {
    pub strict: ContextAbilityChannelScan,
    /// Includes refused probes with no component reads. These are speculative
    /// production-header matches, not claims that native records exist here.
    pub recovery_attempts: Vec<ContextEquipmentRecoveryAttempt>,
    /// All candidates before counter acceptance; rejected candidates stay visible.
    pub recovery_windows: Vec<EquipmentRecoveryWindow>,
    /// Counter-accepted emissions before final chain pruning.
    pub recovered: Vec<EquipmentEmission>,
    pub assembly: EquipmentChanges,
}
/// Native ScanEquipmentChanges over loaded buffers. Births are minimum raw
/// position timestamps per slot from the preceding scan phase. No birth witness
/// means no head recovery and no spawn classification, as in the reference.
pub fn scan_context_equipment_changes(
    context: &NativeFilmContext<'_>,
    births: &BTreeMap<u32, u64>,
) -> (ContextEquipmentScan, Option<ContextAbilityChannelError>) {
    let (strict, error) = scan_context_ability_emissions(context);
    let mut out = ContextEquipmentScan {
        strict,
        ..Default::default()
    };
    if error.is_some() {
        return (out, error);
    }
    let result =
        (|| -> Result<(), ContextAbilityChannelError> {
            let mut emissions = Vec::new();
            for r in &out.strict.observations {
                if let FilmComponentObservation::AbilitySet { counter, rank, .. } = r.value {
                    emissions.push(EquipmentEmission {
                        chunk_number: Some(r.chunk),
                        recovery: None,
                        ability: BipedAbilityEmission {
                            packet_index: Some(r.packet_index),
                            source: r.source,
                            slot: r.slot,
                            counter: counter as u8,
                            rank: (rank != -1).then_some(rank as u8),
                        },
                    });
                }
            }
            out.recovery_windows = equipment_recovery_windows(&emissions, births);
            if !out.recovery_windows.is_empty() {
                let layout = context.i0_layout().layout.as_ref().ok_or(
                    ContextAbilityChannelError::Layout(I0LayoutRefusal::Inconclusive),
                )?;
                let reader_context = NativeReaderContext {
                    profile: context.scan_profile()?,
                    observer: None,
                };
                let registry = context.registry().map_err(|e| *e)?;
                let arch = registry
                    .registry
                    .archetype(35)
                    .ok_or(ContextAbilityChannelError::NoArchetype)?;
                let lo = out
                    .recovery_windows
                    .iter()
                    .map(|w| w.min_chunk)
                    .min()
                    .unwrap();
                let hi = out
                    .recovery_windows
                    .iter()
                    .map(|w| w.max_chunk)
                    .max()
                    .unwrap();
                for chunk in lo..=hi {
                    let active: Vec<_> = out
                        .recovery_windows
                        .iter()
                        .enumerate()
                        .filter(|(_, w)| w.min_chunk <= chunk && chunk <= w.max_chunk)
                        .map(|(i, _)| i)
                        .collect();
                    if active.is_empty() {
                        continue;
                    }
                    let Some((data, packets)) = context.chunk_at(chunk) else {
                        continue;
                    };
                    for (packet_index, &source) in packets.iter().enumerate() {
                        if source.packet_type != 0 {
                            continue;
                        }
                        let active: Vec<_> = active
                            .iter()
                            .copied()
                            .filter(|&i| {
                                let w = &out.recovery_windows[i];
                                w.min_time_us <= source.timestamp_us
                                    && source.timestamp_us <= w.max_time_us
                            })
                            .collect();
                        if active.is_empty() {
                            continue;
                        }
                        let payload = &data
                            [source.payload_offset..source.payload_offset + source.payload_size];
                        let bits = super::bits::Bits(payload);
                        for bit_offset in 0..bits.len().saturating_sub(26) {
                            if bits.read(bit_offset, 1) != Some(1) {
                                continue;
                            }
                            let slot = bits.read(bit_offset + 1, 13).unwrap() as u32;
                            let Some(&window_index) = active
                                .iter()
                                .find(|&&i| out.recovery_windows[i].slot == slot)
                            else {
                                continue;
                            };
                            if bits.read(bit_offset + 14, 2) != Some(1)
                                || bits.read(bit_offset + 16, 1) != Some(0)
                            {
                                continue;
                            }
                            let probe = probe_context_equipment_recovery(
                                payload,
                                bit_offset,
                                layout,
                                arch,
                                &reader_context,
                            )?;
                            if let Some((counter, rank)) = probe.candidate() {
                                out.recovery_windows[window_index].candidates.push(
                                    EquipmentEmission {
                                        chunk_number: Some(chunk),
                                        ability: BipedAbilityEmission {
                                            packet_index: Some(packet_index),
                                            source,
                                            slot,
                                            counter: counter as u8,
                                            rank: (rank != -1).then_some(rank as u8),
                                        },
                                        recovery: Some(EquipmentRecoveryOrigin {
                                            bit_offset,
                                            head: false,
                                        }),
                                    },
                                );
                            }
                            out.recovery_attempts.push(ContextEquipmentRecoveryAttempt {
                                chunk,
                                source,
                                packet_index,
                                slot,
                                bit_offset,
                                window_index,
                                probe,
                            });
                        }
                    }
                }
                for window in &out.recovery_windows {
                    out.recovered.extend(accept_equipment_recovery(window));
                }
                emissions.extend(out.recovered.iter().cloned());
            }
            out.assembly = assemble_equipment_changes(&emissions, births);
            Ok(())
        })();
    (out, result.err())
}
impl ContextEquipmentScan {
    pub fn facts(&self) -> impl Iterator<Item = (&EquipmentChange, FactsEquipmentChange)> {
        self.assembly.records.iter().map(|r| {
            (
                r,
                FactsEquipmentChange {
                    timestamp_us: r.source.timestamp_us,
                    slot: r.slot,
                    counter: u32::from(r.counter),
                    rank: r.rank.map_or(-1, i64::from),
                    previous: r.previous.map_or(-1, i64::from),
                    kind: match r.kind {
                        EquipmentChangeKind::Taken => "taken",
                        EquipmentChangeKind::Spent => "spent",
                        EquipmentChangeKind::Spawned => "spawned",
                    }
                    .into(),
                    recovered: r.recovered,
                    gap: i64::from(r.gap),
                },
            )
        })
    }
    pub fn facts_stats(&self) -> FactsEquipmentChangeStats {
        let s = &self.assembly.stats;
        let w = &self.strict.stats;
        FactsEquipmentChangeStats {
            walk: FactsAbilityRankStats {
                records: w.records as i64,
                with_i48: w.with_component as i64,
                read: w.read as i64,
                unread: w.unread as i64,
                gated: w.gated as i64,
            },
            lives: s.lives as i64,
            repeats: s.repeats as i64,
            counter_jumps: s.counter_jumps as i64,
            missed_estimate: s.missed_estimate as i64,
            lives_first_off_spec: s.lives_first_off_spec as i64,
            spawned: s.spawned as i64,
            taken: s.taken as i64,
            spent: s.spent as i64,
            recovered: s.recovered as i64,
        }
    }
}
