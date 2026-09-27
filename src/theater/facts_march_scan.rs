//! Native march-to-cache projection; keeps absent scan distinct from empty scan.
use super::*;
impl From<&NativePrecisionDescriptor> for FactsPrecisionDescriptor {
    fn from(v: &NativePrecisionDescriptor) -> Self {
        Self {
            index_width: v.index_bits,
            axis_widths: v.axis_bits,
            region: v.region,
        }
    }
}
impl From<&NativeScanProfile> for FactsScanProfile {
    fn from(v: &NativeScanProfile) -> Self {
        let m = &v.movement;
        let g = &v.grammar;
        let widths = |w: &Option<NativeSharedWidths>| {
            w.as_ref().map(|w| {
                w.snapshot()
                    .into_iter()
                    .map(|(k, v)| (k.into_bytes(), v))
                    .collect()
            })
        };
        Self {
            movement: FactsMovementProfile {
                traversal: FactsPrecisionDescriptor::from(&m.traversal),
                world_object: FactsPrecisionDescriptor::from(&m.world_object),
                range: m.range.map(|r| FactsAxisRange {
                    min: r[0],
                    max: r[1],
                }),
                delta_quantum: m.delta_quantum,
                delta_axis_width: m.delta_axis_width,
                full_precision: m.full_precision,
                delta_has_handle_tail: m.delta_has_handle_tail,
                calibrated_skip: m.calibrated_skip,
                mobility_action_extra_bits: m.mobility_action_extra_bits,
            },
            keyframe: FactsKeyframeProfile {
                header_bits: v.keyframe.header_bits,
                size_word_bits: v.keyframe.size_word_bits,
            },
            mpp: FactsMppWidths {
                lead: v.mpp.lead,
                index: v.mpp.index,
            },
            grammar: FactsScanGrammar {
                corruption_check: g.corruption_check,
                new_record_tail_bits: g.new_record_tail_bits,
                default_state_by_archetype: g.default_state_by_archetype,
                simulation_complete: g.simulation_complete,
                baseline_scope: g.baseline_scope,
                writer_absolute: g.writer_absolute,
                mobility_action_body: g.mobility_action_body,
                ability_anchor_body: g.ability_anchor_body,
                chain_inference: g.chain_inference,
                generation_strict: g.generation_strict,
                view_tables: g.view_tables,
                view_classes: g.view_classes,
                calibrated_widths: widths(&g.calibrated_widths),
                stub_widths: widths(&g.stub_widths),
            },
        }
    }
}
impl From<&NativeFrameMetadata> for FactsDeathFrame {
    fn from(v: &NativeFrameMetadata) -> Self {
        Self {
            has_extra_fields: v.extra_fields,
            id_low_bits: v.id_low_bits,
            id_base: v.id_base,
            new_default_state_bits: v.new_default_state_bits,
            packet_preamble_bits: v.packet_preamble_bits,
            profile: FactsScanProfile::from(&v.profile),
        }
    }
}
/// Native ScanMarchFacts allocates coverage maps even on an early refusal.
/// Pass the returned scan when available; None describes a refused scan, not an
/// unattempted channel (whose complete FactsObjectDeaths remains Default).
pub fn facts_from_march_scan(
    scan: Option<&FilmMarchFacts>,
    archetype: Option<u32>,
) -> FactsObjectDeaths {
    let empty = FilmMarchFacts::default();
    let m = scan.unwrap_or(&empty);
    let c = &m.facts.coverage;
    let counts = |v: &std::collections::BTreeMap<u32, u64>| {
        Some(v.iter().map(|(&k, &n)| (k, n as i64)).collect())
    };
    let deaths: Vec<_> = m
        .facts
        .deaths
        .iter()
        .filter(|d| archetype.is_none_or(|ti| ti == d.type_index))
        .cloned()
        .collect();
    FactsObjectDeaths {
        deaths: (!deaths.is_empty()).then_some(deaths),
        frame: m
            .native_config
            .as_ref()
            .map(FactsDeathFrame::from)
            .unwrap_or_default(),
        stats: FactsObjectDeathStats {
            default_frame: m.calibration.as_ref().is_some_and(|c| c.retained_default),
            frame_located: m.calibration.as_ref().map_or(0, |c| c.best.located as i64),
            frame_runner: m
                .calibration
                .as_ref()
                .map_or(0, |c| c.runner_up.located as i64),
            frame_events: m.calibration.as_ref().map_or(0, |c| c.best.events as i64),
            keyframes: m.keyframes as i64,
            deltas: m.deltas as i64,
            packets: m.packets as i64,
            event_packets: m.event_packets as i64,
            located_packets: m.located_packets as i64,
            records: counts(&c.records),
            clean_records: counts(&c.clean_records),
            mask_declared: counts(&c.mask_declared),
            mask_declared_desync: counts(&c.mask_declared_desync),
        },
    }
}
