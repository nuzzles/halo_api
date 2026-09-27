//! Optional native chain inference. Confirmed alignment is evidence for skipping
//! a transient, not proof of its archetype when several types share that alignment.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
pub const CHAIN_MAX_DEPTH: usize = 6;
pub const CHAIN_MAX_RECORDS: usize = 24;
pub const CHAIN_TRIAL_BUDGET: usize = 200_000;
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ChainInferenceOutcome {
    Immediate,
    Deep,
    NoConfirmation,
    BudgetExhausted,
    Ambiguous,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChainInference {
    pub archetype: Option<u32>,
    pub end_bit: i64,
    pub unique_archetype: bool,
    pub outcome: ChainInferenceOutcome,
    pub remaining_trials: usize,
    /// Includes rejected speculative reads, as in the native shared observer.
    #[serde(default, skip_serializing_if = "FilmReadDiagnostics::is_empty")]
    pub diagnostics: FilmReadDiagnostics,
}
struct Context<'a> {
    reader_context: Option<&'a NativeReaderContext>,
    diagnostics: std::cell::RefCell<FilmReadDiagnostics>,
    data: &'a [u8],
    registry: &'a FilmRegistry,
    world: &'a FilmWorld,
    encoding: &'a FrameEncoding,
    simulation_complete: bool,
    budget: usize,
    min_components: usize,
    confirmations: usize,
    overlay: BTreeMap<u32, Option<u32>>,
}
impl Context<'_> {
    fn trial(&self, bit: i64, ti: u32) -> Option<(i64, usize)> {
        let (result, diagnostics) = super::components::chain_delta_body_trial_contextual(
            self.data,
            bit,
            self.registry,
            ti,
            self.encoding,
            self.simulation_complete,
            self.reader_context,
        );
        self.diagnostics.borrow_mut().merge(&diagnostics);
        result
    }
    fn clean_ends(&mut self, body: i64) -> Vec<i64> {
        let mut ends = BTreeSet::new();
        for ti in 0..self.registry.archetypes.len() {
            if self.budget == 0 {
                break;
            }
            self.budget -= 1;
            if let Some((end, _)) = self.trial(body, ti as u32) {
                ends.insert(end);
            }
        }
        ends.into_iter().collect()
    }
    fn overlay_walk(
        &mut self,
        slot: u32,
        ti: Option<u32>,
        pos: i64,
        depth: usize,
        records: usize,
    ) -> bool {
        let old = self.overlay.insert(slot, ti);
        let result = self.confirm(pos, depth, records - 1);
        if let Some(v) = old {
            self.overlay.insert(slot, v);
        } else {
            self.overlay.remove(&slot);
        }
        result
    }
    fn confirm(&mut self, pos: i64, depth: usize, records: usize) -> bool {
        let length = (self.data.len() * 8) as i64;
        if records == 0 || self.budget == 0 || pos >= length {
            return false;
        }
        let bit = pos.wrapping_add(if self.encoding.extra_fields { 32 } else { 0 });
        let Some(header) =
            super::records::decode_frame_header_signed(self.data, bit, self.encoding)
        else {
            return false;
        };
        let header_end = header.end_bit;
        if header.kind == RecordKind::End {
            let rem = length.wrapping_sub(header_end);
            if !(0..=15).contains(&rem) {
                return false;
            }
            let mut reader = NativeFilmBits::new(self.data);
            reader.set_position(header_end);
            return (0..rem).all(|_| !reader.read_bit());
        }
        let Some(id) = header.id else { return false };
        let slot = id & 0x3fff_ffff;
        match header.kind {
            RecordKind::Delete => {
                let end = header_end.wrapping_add(32);
                end <= length && self.overlay_walk(slot, None, end, depth, records)
            }
            RecordKind::New => {
                // Native chain NEW does not consume HasExtraFields' optional
                // payload; only its record-prefix word participates in this path.
                let mut encoding = self.encoding.clone();
                encoding.extra_fields = false;
                let Some(r) = super::components::decode_entity_record_with_capture_slots(
                    self.data,
                    bit,
                    self.registry,
                    &encoding,
                    &EntityBindings::default(),
                    self.simulation_complete,
                    super::components::RecordCaptureSlots {
                        context: self.reader_context.cloned(),
                        movement: self
                            .reader_context
                            .and(header.id.map(|id| id & 0x3fff_ffff)),
                        position: self
                            .reader_context
                            .and(header.id.map(|id| id & 0x3fff_ffff)),
                    },
                ) else {
                    return false;
                };
                self.diagnostics.borrow_mut().merge(&r.diagnostics);
                r.archetype == Some(35)
                    && r.stop == EntityViewStop::Complete
                    && r.end_bit <= length
                    && self.overlay_walk(slot, Some(35), r.end_bit, depth, records)
            }
            RecordKind::Delta => {
                let ti = match self.overlay.get(&slot) {
                    Some(None) => return false,
                    Some(Some(ti)) => Some(*ti),
                    None => self.world.archetype(slot),
                };
                if let Some(ti) = ti {
                    if self.registry.archetype(ti as usize).is_none() {
                        return false;
                    }
                    self.budget -= 1;
                    let Some((end, comps)) = self.trial(header_end, ti) else {
                        return false;
                    };
                    if !self.overlay.contains_key(&slot)
                        && self.world.hard_bound(slot)
                        && comps >= self.min_components
                    {
                        if self.confirmations <= 1 {
                            return true;
                        }
                        self.confirmations -= 1;
                        let ok = self.confirm(end, depth, records - 1);
                        self.confirmations += 1;
                        return ok;
                    }
                    self.confirm(end, depth, records - 1)
                } else {
                    if depth == 0 {
                        return false;
                    }
                    let ends = self.clean_ends(header_end);
                    ends.into_iter()
                        .any(|end| self.confirm(end, depth - 1, records - 1))
                }
            }
            RecordKind::End => unreachable!(),
        }
    }
    fn confirmed(&mut self, ends: &[i64], depth: usize, records: usize) -> Vec<i64> {
        ends.iter()
            .copied()
            .filter(|&e| self.confirm(e, depth, records))
            .collect()
    }
    fn resolve(&mut self, ends: &[i64]) -> (Option<i64>, ChainInferenceOutcome) {
        self.min_components = 0;
        self.confirmations = 1;
        let mut candidates = self.confirmed(ends, 0, 1);
        if candidates.len() == 1 {
            return (Some(candidates[0]), ChainInferenceOutcome::Immediate);
        }
        if candidates.is_empty() {
            self.min_components = 1;
            candidates = self.confirmed(ends, CHAIN_MAX_DEPTH - 1, CHAIN_MAX_RECORDS);
            if candidates.len() == 1 {
                return (Some(candidates[0]), ChainInferenceOutcome::Deep);
            }
            if candidates.is_empty() {
                return (
                    None,
                    if self.budget == 0 {
                        ChainInferenceOutcome::BudgetExhausted
                    } else {
                        ChainInferenceOutcome::NoConfirmation
                    },
                );
            }
        }
        self.min_components = 1;
        for n in 1..=3 {
            if candidates.len() <= 1 {
                break;
            }
            self.confirmations = n;
            let next = self.confirmed(&candidates, CHAIN_MAX_DEPTH - 1, CHAIN_MAX_RECORDS);
            if !next.is_empty() {
                candidates = next;
            }
        }
        if candidates.len() == 1 {
            (Some(candidates[0]), ChainInferenceOutcome::Deep)
        } else {
            (None, ChainInferenceOutcome::Ambiguous)
        }
    }
}
/// Resolve an unbound delta's candidate mask/body alignment without mutating the
/// world. Native terminal confirmation requires an actual End marker with at most
/// 15 zero tail bits; arbitrary padding alone never establishes a record boundary.
pub fn infer_chain_archetype(
    data: &[u8],
    body_bit: impl TryInto<i64>,
    registry: &FilmRegistry,
    encoding: &FrameEncoding,
    world: &FilmWorld,
    simulation_complete: bool,
) -> ChainInference {
    infer_chain_archetype_contextual(
        data,
        body_bit,
        registry,
        encoding,
        world,
        simulation_complete,
        None,
    )
}
pub(crate) fn infer_chain_archetype_contextual(
    data: &[u8],
    body_bit: impl TryInto<i64>,
    registry: &FilmRegistry,
    encoding: &FrameEncoding,
    world: &FilmWorld,
    simulation_complete: bool,
    reader_context: Option<&NativeReaderContext>,
) -> ChainInference {
    let body_bit = body_bit
        .try_into()
        .ok()
        .expect("native signed body position");
    let _captures = reader_context
        .and_then(|c| c.observer.as_ref())
        .map(|o| o.neutralize_position_capture());
    let mut context = Context {
        reader_context,
        diagnostics: Default::default(),
        data,
        registry,
        encoding,
        world,
        simulation_complete,
        budget: CHAIN_TRIAL_BUDGET,
        min_components: 0,
        confirmations: 1,
        overlay: BTreeMap::new(),
    };
    let mut by_end = BTreeMap::<i64, Vec<u32>>::new();
    for ti in 0..registry.archetypes.len() {
        if let Some((end, _)) = context.trial(body_bit, ti as u32) {
            by_end.entry(end).or_default().push(ti as u32);
        }
    }
    let (end, outcome) = context.resolve(&by_end.keys().copied().collect::<Vec<_>>());
    let types = end.and_then(|e| by_end.get(&e));
    *context
        .diagnostics
        .borrow_mut()
        .chain_outcomes
        .entry(outcome.clone())
        .or_default() += 1;
    if let Some(observer) = reader_context.and_then(|c| c.observer.as_ref()) {
        observer.record_chain_outcome(outcome.clone());
    }
    ChainInference {
        archetype: types.map(|t| t[0]),
        end_bit: end.unwrap_or(body_bit),
        unique_archetype: types.is_some_and(|t| t.len() == 1),
        outcome,
        remaining_trials: context.budget,
        diagnostics: context.diagnostics.into_inner(),
    }
}

pub const CHAIN_COMPONENT_MAX_STUB_BITS: usize = 640;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChainComponentRepair {
    pub component_name: String,
    /// Every extra width that produces the selected confirmed alignment.
    pub matching_widths: Vec<usize>,
    pub selected_width: usize,
    pub record: EntityRecord,
}
/// Try the native inclusive 0..640 extra-width sweep for the last unsupported
/// component. Preset widths are not overridden. No mutation or inferred binding
/// escapes; a successful record explicitly retains its unknown tail as raw bits.
pub fn repair_chain_component(
    data: &[u8],
    failed: &EntityRecord,
    registry: &FilmRegistry,
    profile: &KillWalkProfile,
    world: &FilmWorld,
    preset_widths: &BTreeMap<String, usize>,
) -> Option<ChainComponentRepair> {
    repair_chain_component_observed(data, failed, registry, profile, world, preset_widths).result
}
/// Retain diagnostics from every width trial, including failed repair attempts.
pub fn repair_chain_component_observed(
    data: &[u8],
    failed: &EntityRecord,
    registry: &FilmRegistry,
    profile: &KillWalkProfile,
    world: &FilmWorld,
    preset_widths: &BTreeMap<String, usize>,
) -> FilmReadAttempt<ChainComponentRepair> {
    repair_chain_component_contextual(data, failed, registry, profile, world, preset_widths, None)
}

pub(crate) fn repair_chain_component_contextual(
    data: &[u8],
    failed: &EntityRecord,
    registry: &FilmRegistry,
    profile: &KillWalkProfile,
    world: &FilmWorld,
    preset_widths: &BTreeMap<String, usize>,
    reader_context: Option<&NativeReaderContext>,
) -> FilmReadAttempt<ChainComponentRepair> {
    let observer = reader_context.and_then(|c| c.observer.as_ref());
    let diagnostics = std::cell::RefCell::new(FilmReadDiagnostics::default());
    let result = (|| {
        let last = failed.attempts.last()?;
        if reader_context
            .and_then(|c| c.profile.grammar.stub_widths.as_ref())
            .and_then(|w| w.get(&last.span.name))
            .is_some()
        {
            return None;
        }
        if last.status != Some(false)
            || preset_widths.contains_key(&last.span.name)
            || profile
                .encoding
                .component_widths
                .stubs
                .contains_key(&last.span.name)
        {
            return None;
        }
        if !matches!(failed.header.kind, RecordKind::New | RecordKind::Delta) {
            return None;
        }
        let _captures = observer.map(NativeFilmObserver::neutralize_captures);
        let local_widths = NativeSharedWidths::default();
        let local_context = reader_context.cloned().map(|mut c| {
            c.profile.grammar.stub_widths = Some(local_widths.clone());
            c
        });
        let name = &last.span.name;
        let id = failed.header.id?;
        let mut bindings = EntityBindings::default();
        if let Some(ti) = world.archetype(id & 0x3fff_ffff) {
            bindings.bind(id, ti);
        }
        let redecode = |width| {
            local_widths.insert(name.clone(), width as i64);
            let record = super::components::decode_entity_record_with_stub(
                data,
                failed,
                registry,
                profile,
                &bindings,
                (name.as_str(), width),
                local_context.as_ref(),
            );
            if let Some(r) = &record {
                diagnostics.borrow_mut().merge(&r.diagnostics);
            }
            record
        };
        let mut by_end = BTreeMap::<i64, Vec<usize>>::new();
        for width in 0..=CHAIN_COMPONENT_MAX_STUB_BITS {
            if let Some(record) = redecode(width)
                && record.stop == EntityViewStop::Complete
                && record.end_bit <= (data.len() * 8) as i64
            {
                by_end.entry(record.end_bit).or_default().push(width);
            }
        }
        local_widths.remove(name);
        if by_end.is_empty() {
            return None;
        }
        let mut confirmation_encoding = profile.encoding.clone();
        confirmation_encoding.component_widths.stubs.clear();
        let mut ctx = Context {
            reader_context: local_context.as_ref(),
            diagnostics: Default::default(),
            data,
            registry,
            world,
            encoding: &confirmation_encoding,
            simulation_complete: profile.simulation_complete,
            budget: CHAIN_TRIAL_BUDGET,
            min_components: 0,
            confirmations: 1,
            overlay: BTreeMap::new(),
        };
        let (end, outcome) = ctx.resolve(&by_end.keys().copied().collect::<Vec<_>>());
        diagnostics
            .borrow_mut()
            .merge(&ctx.diagnostics.into_inner());
        if end.is_none() {
            if let Some(observer) = observer {
                observer.record_chain_outcome(outcome.clone());
            }
            *diagnostics
                .borrow_mut()
                .chain_outcomes
                .entry(outcome)
                .or_default() += 1;
        }
        let end = end?;
        let matching_widths = by_end.remove(&end)?;
        let selected_width = matching_widths[0];
        let mut record = redecode(selected_width)?;
        local_widths.remove(name);
        if reader_context.is_none() {
            record.diagnostics.suppress_unit_references();
        }
        if record.stop != EntityViewStop::Complete || record.end_bit != end {
            return None;
        }
        diagnostics.borrow_mut().repaired_records += 1;
        if let Some(observer) = observer {
            observer.record_repair(name, &matching_widths);
        }
        for &width in &matching_widths {
            *diagnostics
                .borrow_mut()
                .component_widths
                .entry(name.clone())
                .or_default()
                .entry(width)
                .or_default() += 1;
        }
        Some(ChainComponentRepair {
            component_name: name.clone(),
            matching_widths,
            selected_width,
            record,
        })
    })();
    let mut diagnostics = diagnostics.into_inner();
    if reader_context.is_none() {
        diagnostics.suppress_unit_references();
    }
    FilmReadAttempt {
        result,
        diagnostics,
    }
}

/// Find the first target delta whose continuation passes the native chain check.
/// This is an explicit probe: default frame decoding does not enable resync.
/// Candidate reads and continuation trials leave the supplied world unchanged.
pub fn validated_chain_resync(
    data: &[u8],
    from: impl TryInto<i64>,
    registry: &FilmRegistry,
    profile: &KillWalkProfile,
    world: &FilmWorld,
    targets: &BTreeSet<u32>,
) -> Option<i64> {
    validated_chain_resync_observed(data, from, registry, profile, world, targets).result
}
/// Native resync diagnostics include all attempted bound deltas, even non-targets.
pub fn validated_chain_resync_observed(
    data: &[u8],
    from: impl TryInto<i64>,
    registry: &FilmRegistry,
    profile: &KillWalkProfile,
    world: &FilmWorld,
    targets: &BTreeSet<u32>,
) -> FilmReadAttempt<i64> {
    validated_chain_resync_contextual(data, from, registry, profile, world, targets, None)
}

pub(crate) fn validated_chain_resync_contextual(
    data: &[u8],
    from: impl TryInto<i64>,
    registry: &FilmRegistry,
    profile: &KillWalkProfile,
    world: &FilmWorld,
    targets: &BTreeSet<u32>,
    reader_context: Option<&NativeReaderContext>,
) -> FilmReadAttempt<i64> {
    let observer = reader_context.and_then(|c| c.observer.as_ref());
    let _captures = observer.map(NativeFilmObserver::neutralize_captures);
    let mut diagnostics = FilmReadDiagnostics::default();
    let Ok(from) = from.try_into() else {
        return FilmReadAttempt {
            result: None,
            diagnostics,
        };
    };
    if !profile.encoding.valid() {
        return FilmReadAttempt {
            result: None,
            diagnostics,
        };
    }
    let mut context = Context {
        reader_context,
        diagnostics: Default::default(),
        data,
        registry,
        world,
        encoding: &profile.encoding,
        simulation_complete: profile.simulation_complete,
        budget: CHAIN_TRIAL_BUDGET,
        min_components: 1,
        confirmations: 1,
        overlay: BTreeMap::new(),
    };
    let result = (|| {
        let frame_len = (data.len() as i64).wrapping_mul(8);
        for start in from..frame_len.wrapping_sub(24) {
            // TryDeltaAt checks availability before skipping the optional prefix.
            if frame_len.wrapping_sub(start) < 24 {
                continue;
            }
            let header_bit = start.wrapping_add(if profile.encoding.extra_fields { 32 } else { 0 });
            let Some(header) =
                super::records::decode_frame_header_signed(data, header_bit, &profile.encoding)
            else {
                continue;
            };
            if header.kind != RecordKind::Delta {
                continue;
            }
            let Some(record) = super::inference_frame::read_bound_record_contextual(
                data,
                header_bit,
                &header,
                registry,
                profile,
                world,
                super::components::RecordCaptureSlots {
                    context: reader_context.cloned(),
                    movement: reader_context.and(header.id.map(|id| id & 0x3fff_ffff)),
                    position: reader_context.and(header.id.map(|id| id & 0x3fff_ffff)),
                },
            ) else {
                continue;
            };
            diagnostics.merge(&record.diagnostics);
            if record.stop != EntityViewStop::Complete
                || record.components.is_empty()
                || !header
                    .id
                    .is_some_and(|id| targets.contains(&(id & 0x3fff_ffff)))
            {
                continue;
            }
            if context.budget == 0 {
                return None;
            }
            if context.confirm(record.end_bit, CHAIN_MAX_DEPTH - 1, CHAIN_MAX_RECORDS) {
                return Some(start);
            }
        }
        None
    })();
    diagnostics.merge(&context.diagnostics.into_inner());
    if result.is_some() {
        diagnostics.validated_resyncs += 1;
        if let Some(observer) = observer {
            observer.record_validated_resync();
        }
    }
    if reader_context.is_none() {
        diagnostics.suppress_unit_references();
    }
    FilmReadAttempt {
        result,
        diagnostics,
    }
}

pub(super) fn harvest_confirmer<'a>(
    data: &'a [u8],
    registry: &'a FilmRegistry,
    profile: &'a KillWalkProfile,
    world: &'a FilmWorld,
    reader_context: Option<&'a NativeReaderContext>,
) -> impl FnMut(i64) -> (bool, FilmReadDiagnostics) + 'a {
    let mut context = Context {
        reader_context,
        diagnostics: Default::default(),
        data,
        registry,
        world,
        encoding: &profile.encoding,
        simulation_complete: profile.simulation_complete,
        budget: 1 << 30,
        min_components: 1,
        confirmations: 1,
        overlay: BTreeMap::new(),
    };
    move |after| {
        let confirmed = context.confirm(after, CHAIN_MAX_DEPTH - 1, CHAIN_MAX_RECORDS);
        (
            confirmed,
            std::mem::take(&mut *context.diagnostics.borrow_mut()),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn native_new_repair_preserves_simulation_prefix() {
        compare_new_repairs(include_bytes!("fixtures/chain-new-repair-v41.json.zlib"));
    }
    #[test]
    fn native_default_hook_repair() {
        compare_new_repairs(include_bytes!(
            "fixtures/default-hook-repair-d61443e-v41.json.zlib"
        ));
    }
    fn compare_new_repairs(fixture: &[u8]) {
        #[derive(Deserialize)]
        struct Case {
            hex: String,
            created_archetype: Option<usize>,
            observations: Option<Vec<FilmComponentObservation>>,
            mpp: [usize; 2],
            corruption: bool,
            extra: bool,
            encoding: PositionEncoding,
            prefix: usize,
            ok: bool,
            end: usize,
            trace_end: usize,
            widths: Vec<usize>,
        }
        let mut json = String::new();
        flate2::read::ZlibDecoder::new(fixture)
            .read_to_string(&mut json)
            .unwrap();
        let cases: Vec<Case> = serde_json::from_str(&json).unwrap();
        assert!(matches!(cases.len(), 128 | 512));
        for (i, c) in cases.into_iter().enumerate() {
            let data: Vec<u8> = (0..c.hex.len())
                .step_by(2)
                .map(|n| u8::from_str_radix(&c.hex[n..n + 2], 16).unwrap())
                .collect();
            let created_archetype = c.created_archetype.unwrap_or(35);
            let mut registry = FilmRegistry {
                archetypes: (0..if c.created_archetype.is_some() {
                    44
                } else {
                    36
                })
                    .map(|index| FilmArchetype {
                        index,
                        levels: vec![],
                        components: vec![],
                    })
                    .collect(),
                major_version: 41,
                format_version: 27,
                end_byte: 0,
                truncated: false,
            };
            registry.archetypes[created_archetype].components = vec![
                "simulation-state".into(),
                "biped-emp-timer-component".into(),
            ];
            registry.archetypes[created_archetype].levels = vec![0, 0];
            let profile = KillWalkProfile {
                encoding: FrameEncoding {
                    keyframe_layout: Default::default(),
                    keyframe_simulation_complete: None,
                    native_id_low_bits: None,
                    component_widths: Default::default(),
                    new_record: Default::default(),
                    position_capture: None,
                    ids: RecordIdLayout {
                        low_bits: 11,
                        base: 0,
                    },
                    mpp_widths: c.mpp,
                    position: Some(c.encoding),
                    extra_fields: c.extra,
                    corruption_check: c.corruption,
                },
                simulation_complete: false,
            };
            let mut world = FilmWorld::default();
            world.bind_full(50, 0);
            let before = world.clone();
            let mut record_encoding = profile.encoding.clone();
            record_encoding.extra_fields = false;
            let failed = super::super::components::decode_entity_record_at_with_simulation_policy(
                &data,
                if c.extra { 32 } else { 0 },
                &registry,
                &record_encoding,
                &EntityBindings::default(),
                false,
            )
            .unwrap();
            assert_eq!(
                serde_json::json!(
                    failed.attempts[0].span.end_bit - failed.attempts[0].span.start_bit
                ),
                serde_json::json!(c.prefix),
                "prefix {i}"
            );
            let observed = repair_chain_component_observed(
                &data,
                &failed,
                &registry,
                &profile,
                &world,
                &BTreeMap::new(),
            );
            if let Some(expected) = c.observations {
                let mut actual = failed.diagnostics.component_observations.clone();
                actual.extend(observed.diagnostics.component_observations.iter().cloned());
                assert_eq!(actual, expected, "new repair observations {i}");
            }
            let result = observed.result;
            assert_eq!(result.is_some(), c.ok, "repair {i}");
            if let Some(result) = result {
                assert_eq!(
                    serde_json::json!(result.record.end_bit),
                    serde_json::json!(c.end),
                    "end {i}"
                );
                assert_eq!(
                    serde_json::json!(result.record.end_bit),
                    serde_json::json!(c.trace_end)
                );
                assert_eq!(result.matching_widths, c.widths, "widths {i}");
                let span = &result.record.attempts[0].span;
                assert_eq!(
                    serde_json::json!(span.end_bit - span.start_bit),
                    serde_json::json!(c.prefix + result.selected_width)
                );
                for field in &result.record.fields {
                    assert_eq!(
                        serde_json::json!(
                            super::super::bits::Bits(&data).read(field.bit, field.width)
                        ),
                        serde_json::json!(Some(field.raw))
                    );
                }
            }
            assert_eq!(world, before);
        }
    }
    #[test]
    fn native_chain_biped_new_overlay() {
        compare_new_chain(include_bytes!("fixtures/chain-new-v41.json.zlib"));
    }
    #[test]
    fn native_default_hook_chain() {
        compare_new_chain(include_bytes!(
            "fixtures/default-hook-chain-d61443e-v41.json.zlib"
        ));
    }
    fn compare_new_chain(fixture: &[u8]) {
        #[derive(Deserialize)]
        struct Case {
            hex: String,
            created_archetype: Option<usize>,
            observations: Option<Vec<FilmComponentObservation>>,
            mpp: [usize; 2],
            encoding: Option<PositionEncoding>,
            corruption: bool,
            soft: bool,
            ti: u32,
            end: usize,
            unique: bool,
            ok: bool,
            outcome: ChainInferenceOutcome,
        }
        let mut json = String::new();
        flate2::read::ZlibDecoder::new(fixture)
            .read_to_string(&mut json)
            .unwrap();
        let cases: Vec<Case> = serde_json::from_str(&json).unwrap();
        assert!(matches!(cases.len(), 128 | 512));
        for (i, c) in cases.into_iter().enumerate() {
            let data: Vec<u8> = (0..c.hex.len())
                .step_by(2)
                .map(|n| u8::from_str_radix(&c.hex[n..n + 2], 16).unwrap())
                .collect();
            let mut registry = FilmRegistry {
                archetypes: (0..if c.created_archetype.is_some() {
                    44
                } else {
                    36
                })
                    .map(|index| FilmArchetype {
                        index,
                        levels: vec![],
                        components: vec![],
                    })
                    .collect(),
                major_version: 41,
                format_version: 27,
                end_byte: 0,
                truncated: false,
            };
            registry.archetypes[0].components = vec!["biped-emp-timer-component".into()];
            registry.archetypes[0].levels = vec![0];
            let encoding = FrameEncoding {
                keyframe_layout: Default::default(),
                keyframe_simulation_complete: None,
                native_id_low_bits: None,
                component_widths: Default::default(),
                new_record: Default::default(),
                position_capture: None,
                ids: RecordIdLayout {
                    low_bits: 11,
                    base: 0,
                },
                mpp_widths: c.mpp,
                position: c.encoding,
                extra_fields: false,
                corruption_check: c.corruption,
            };
            let mut world = FilmWorld::default();
            if c.soft {
                world.bind_soft(50, 0)
            } else {
                world.bind_full(50, 0)
            };
            world.bind_full(100, 0);
            let before = world.clone();
            let got = infer_chain_archetype(&data, 0, &registry, &encoding, &world, false);
            if let Some(expected) = c.observations {
                assert_eq!(
                    got.diagnostics.component_observations, expected,
                    "new chain observations {i}"
                );
            }
            assert_eq!(got.archetype, c.ok.then_some(c.ti), "type {i}");
            assert_eq!(got.end_bit, c.end as i64, "end {i}");
            assert_eq!(got.unique_archetype, c.unique, "unique {i}");
            assert_eq!(got.outcome, c.outcome, "outcome {i}");
            assert_eq!(world, before);
        }
    }
    #[test]
    fn native_validated_resync() {
        compare_resync(include_bytes!("fixtures/validated-resync-v41.json.zlib"));
    }
    #[test]
    fn native_position_hook_validated_resync() {
        compare_resync(include_bytes!(
            "fixtures/position-hook-validated-resync-v41.json.zlib"
        ));
    }
    fn compare_resync(fixture: &[u8]) {
        #[derive(Deserialize)]
        struct Case {
            capture: Option<crate::theater::PositionCaptureEncoding>,
            name: Option<String>,
            position: Option<PositionEncoding>,
            diagnostics: Option<FilmReadDiagnostics>,
            hex: String,
            extra: bool,
            targets: BTreeSet<u32>,
            from: usize,
            start: i64,
            ok: bool,
        }
        let mut json = String::new();
        flate2::read::ZlibDecoder::new(fixture)
            .read_to_string(&mut json)
            .unwrap();
        let cases: Vec<Case> = serde_json::from_str(&json).unwrap();
        assert_eq!(cases.len(), 512);
        for (i, c) in cases.into_iter().enumerate() {
            let data: Vec<u8> = (0..c.hex.len())
                .step_by(2)
                .map(|n| u8::from_str_radix(&c.hex[n..n + 2], 16).unwrap())
                .collect();
            let registry = FilmRegistry {
                archetypes: vec![FilmArchetype {
                    index: 0,
                    levels: vec![0],
                    components: vec![
                        c.name
                            .unwrap_or_else(|| "object-dead-state-component".into()),
                    ],
                }],
                major_version: 41,
                format_version: 27,
                end_byte: 0,
                truncated: false,
            };
            let profile = KillWalkProfile {
                encoding: FrameEncoding {
                    keyframe_layout: Default::default(),
                    keyframe_simulation_complete: None,
                    native_id_low_bits: None,
                    component_widths: Default::default(),
                    new_record: Default::default(),
                    position_capture: c.capture,
                    ids: RecordIdLayout {
                        low_bits: 11,
                        base: 0,
                    },
                    mpp_widths: [9, 5],
                    position: c.position,
                    extra_fields: c.extra,
                    corruption_check: false,
                },
                simulation_complete: false,
            };
            let mut world = FilmWorld::default();
            world.bind_full(50, 0);
            if c.diagnostics.is_some() {
                world.bind_full(51, 0);
            }
            let before = world.clone();
            let attempt = validated_chain_resync_observed(
                &data, c.from, &registry, &profile, &world, &c.targets,
            );
            let observer = NativeFilmObserver::default();
            for kind in [
                NativeHookKind::Position,
                NativeHookKind::UnitReference,
                NativeHookKind::MovementState,
            ] {
                observer.set_hook(
                    kind,
                    Some(std::sync::Arc::new(|_| {
                        panic!("speculative capture escaped")
                    })),
                );
            }
            let context = NativeReaderContext {
                observer: Some(observer.clone()),
                ..Default::default()
            };
            for pass in 1..=2 {
                let live = validated_chain_resync_contextual(
                    &data,
                    c.from,
                    &registry,
                    &profile,
                    &world,
                    &c.targets,
                    Some(&context),
                );
                assert_eq!(live.result, c.ok.then_some(c.start), "live landing {i}");
                assert_eq!(
                    observer.counters().validated_resyncs,
                    if c.ok { pass } else { 0 }
                );
                for kind in [
                    NativeHookKind::Position,
                    NativeHookKind::UnitReference,
                    NativeHookKind::MovementState,
                ] {
                    assert!(observer.has_hook(kind));
                }
            }
            if let Some(expected) = c.diagnostics {
                assert_eq!(attempt.diagnostics, expected, "diagnostics {i}");
            }
            let restored: FilmReadAttempt<i64> =
                serde_json::from_slice(&serde_json::to_vec(&attempt).unwrap()).unwrap();
            assert_eq!(restored, attempt);
            assert_eq!(attempt.result, c.ok.then_some(c.start), "landing {i}");
            assert_eq!(world, before);
        }
    }
    #[test]
    fn native_resync_diagnostics() {
        compare_resync(include_bytes!("fixtures/resync-diagnostics-v41.json.zlib"));
    }
    #[test]
    fn native_chain_component_repair() {
        compare_repairs(include_bytes!("fixtures/chain-repair-v41.json.zlib"));
    }
    #[test]
    fn native_repair_diagnostics() {
        compare_repairs(include_bytes!("fixtures/repair-diagnostics-v41.json.zlib"));
    }
    #[test]
    fn native_mobility_repair() {
        compare_repairs(include_bytes!("fixtures/mobility-repair-v41.json.zlib"));
    }
    #[test]
    fn native_movement_hook_repair() {
        compare_repairs(include_bytes!(
            "fixtures/movement-hook-repair-v41.json.zlib"
        ));
    }
    #[test]
    fn native_position_hook_repair() {
        compare_repairs(include_bytes!(
            "fixtures/position-hook-repair-v41.json.zlib"
        ));
    }
    #[test]
    fn native_component_hook_repair() {
        compare_repairs(include_bytes!(
            "fixtures/component-hook-repair-v41.json.zlib"
        ));
    }
    #[test]
    fn native_ability_hook_repair() {
        compare_repairs(include_bytes!("fixtures/ability-hook-repair-v41.json.zlib"));
    }
    #[test]
    fn native_unit_reference_hook_repair() {
        compare_repairs(include_bytes!(
            "fixtures/unit-reference-hook-repair-v41.json.zlib"
        ));
    }
    #[test]
    fn native_object_hook_repair() {
        compare_repairs(include_bytes!("fixtures/object-hook-repair-v41.json.zlib"));
    }
    #[test]
    fn native_managed_hook_repair() {
        compare_repairs(include_bytes!("fixtures/managed-hook-repair-v41.json.zlib"));
    }
    #[test]
    fn native_engine_hook_repair() {
        compare_repairs(include_bytes!("fixtures/engine-hook-repair-v41.json.zlib"));
    }
    #[test]
    fn native_player_hook_repair() {
        compare_repairs(include_bytes!("fixtures/player-hook-repair-v41.json.zlib"));
    }
    #[test]
    fn native_equipment_hook_repair() {
        compare_repairs(include_bytes!(
            "fixtures/equipment-hook-repair-v41.json.zlib"
        ));
    }
    #[test]
    fn native_probe_hook_repair() {
        compare_repairs(include_bytes!("fixtures/probe-hook-repair-v41.json.zlib"));
    }
    #[test]
    fn native_width_override_repair() {
        compare_repairs(include_bytes!(
            "fixtures/width-override-repair-v41.json.zlib"
        ));
    }
    fn compare_repairs(fixture: &[u8]) {
        #[derive(Deserialize)]
        struct Case {
            #[serde(default)]
            component_widths: ComponentWidthOverrides,
            capture: Option<crate::theater::PositionCaptureEncoding>,
            position: Option<PositionEncoding>,
            diagnostics: Option<FilmReadDiagnostics>,
            hex: String,
            names: Vec<String>,
            preset: bool,
            ok: bool,
            end: usize,
            trace_end: usize,
            widths: Vec<usize>,
        }
        let mut json = String::new();
        flate2::read::ZlibDecoder::new(fixture)
            .read_to_string(&mut json)
            .unwrap();
        let cases: Vec<Case> = serde_json::from_str(&json).unwrap();
        assert_eq!(cases.len(), 512);
        for (i, c) in cases.into_iter().enumerate() {
            let data: Vec<u8> = (0..c.hex.len())
                .step_by(2)
                .map(|n| u8::from_str_radix(&c.hex[n..n + 2], 16).unwrap())
                .collect();
            let registry = FilmRegistry {
                archetypes: vec![
                    FilmArchetype {
                        index: 0,
                        levels: vec![0; c.names.len()],
                        components: c.names,
                    },
                    FilmArchetype {
                        index: 1,
                        levels: vec![],
                        components: vec![],
                    },
                ],
                major_version: 41,
                format_version: 27,
                end_byte: 0,
                truncated: false,
            };
            let profile = KillWalkProfile {
                encoding: FrameEncoding {
                    keyframe_layout: Default::default(),
                    keyframe_simulation_complete: None,
                    native_id_low_bits: None,
                    component_widths: c.component_widths,
                    new_record: Default::default(),
                    position_capture: c.capture,
                    ids: RecordIdLayout {
                        low_bits: 11,
                        base: 0,
                    },
                    mpp_widths: [9, 5],
                    position: c.position,
                    extra_fields: false,
                    corruption_check: false,
                },
                simulation_complete: false,
            };
            let mut world = FilmWorld::default();
            world.bind_full(50, 0);
            world.bind_full(60, 1);
            let before = world.clone();
            let mut bindings = EntityBindings::default();
            bindings.bind(50, 0);
            let mut initial_encoding = profile.encoding.clone();
            initial_encoding.component_widths = Default::default();
            let failed = super::super::components::decode_entity_record_at_with_simulation_policy(
                &data,
                0,
                &registry,
                &initial_encoding,
                &bindings,
                false,
            )
            .unwrap();
            let presets = if c.preset {
                [("not-ported".into(), 3)].into()
            } else {
                BTreeMap::new()
            };
            let attempt = repair_chain_component_observed(
                &data, &failed, &registry, &profile, &world, &presets,
            );
            if let Some(expected) = c.diagnostics {
                assert_eq!(attempt.diagnostics, expected, "diagnostics {i}");
            }
            let restored: FilmReadAttempt<ChainComponentRepair> =
                serde_json::from_slice(&serde_json::to_vec(&attempt).unwrap()).unwrap();
            assert_eq!(restored, attempt);
            let result = attempt.result;
            assert_eq!(result.is_some(), c.ok, "repair {i}");
            if let Some(result) = result {
                assert_eq!(
                    serde_json::json!(result.record.end_bit),
                    serde_json::json!(c.end),
                    "end {i}"
                );
                assert_eq!(
                    serde_json::json!(result.record.end_bit),
                    serde_json::json!(c.trace_end)
                );
                assert_eq!(result.matching_widths, c.widths, "widths {i}");
                assert_eq!(result.selected_width, c.widths[0]);
                assert_eq!(
                    result
                        .record
                        .fields
                        .iter()
                        .filter(|f| f.name.starts_with("inferred_stub["))
                        .map(|f| usize::try_from(f.width).unwrap())
                        .sum::<usize>(),
                    result.selected_width
                );
                for field in &result.record.fields {
                    assert_eq!(
                        serde_json::json!(
                            super::super::bits::Bits(&data).read(field.bit, field.width)
                        ),
                        serde_json::json!(Some(field.raw))
                    );
                }
            }
            assert_eq!(world, before);
        }
    }
    #[test]
    fn native_chain_inference() {
        #[derive(Deserialize)]
        struct Case {
            hex: String,
            names: Vec<Vec<String>>,
            ti: u32,
            end: usize,
            unique: bool,
            ok: bool,
            outcome: ChainInferenceOutcome,
            candidates: Vec<usize>,
            budget: usize,
            remaining: usize,
            resolved: bool,
            resolved_end: usize,
            resolved_outcome: ChainInferenceOutcome,
        }
        let mut json = String::new();
        flate2::read::ZlibDecoder::new(
            &include_bytes!("fixtures/chain-inference-v41.json.zlib")[..],
        )
        .read_to_string(&mut json)
        .unwrap();
        let cases: Vec<Case> = serde_json::from_str(&json).unwrap();
        assert_eq!(cases.len(), 1024);
        for (i, c) in cases.into_iter().enumerate() {
            let data: Vec<u8> = (0..c.hex.len())
                .step_by(2)
                .map(|n| u8::from_str_radix(&c.hex[n..n + 2], 16).unwrap())
                .collect();
            let registry = FilmRegistry {
                archetypes: c
                    .names
                    .into_iter()
                    .enumerate()
                    .map(|(index, components)| FilmArchetype {
                        index,
                        levels: vec![0; components.len()],
                        components,
                    })
                    .collect(),
                major_version: 41,
                format_version: 27,
                end_byte: 0,
                truncated: false,
            };
            let encoding = FrameEncoding {
                keyframe_layout: Default::default(),
                keyframe_simulation_complete: None,
                native_id_low_bits: None,
                component_widths: Default::default(),
                new_record: Default::default(),
                position_capture: None,
                ids: RecordIdLayout {
                    low_bits: 11,
                    base: 0,
                },
                mpp_widths: [9, 5],
                position: None,
                extra_fields: false,
                corruption_check: false,
            };
            let mut world = FilmWorld::default();
            world.bind_full(50, 0);
            world.bind_soft(51, 0);
            let before = world.clone();
            let got = infer_chain_archetype(&data, 0, &registry, &encoding, &world, false);
            assert_eq!(got.archetype, c.ok.then_some(c.ti), "type {i}");
            assert_eq!(got.end_bit, c.end as i64, "end {i}");
            assert_eq!(got.unique_archetype, c.unique, "unique {i}");
            assert_eq!(got.outcome, c.outcome, "outcome {i}");
            assert_eq!(world, before);
            let mut ctx = Context {
                reader_context: None,
                diagnostics: Default::default(),
                data: &data,
                registry: &registry,
                encoding: &encoding,
                world: &world,
                simulation_complete: false,
                budget: c.budget,
                min_components: 0,
                confirmations: 1,
                overlay: BTreeMap::new(),
            };
            let (end, outcome) =
                ctx.resolve(&c.candidates.iter().map(|&v| v as i64).collect::<Vec<_>>());
            assert_eq!(
                end,
                c.resolved.then_some(c.resolved_end as i64),
                "resolved end {i}"
            );
            assert_eq!(outcome, c.resolved_outcome, "resolved outcome {i}");
            assert_eq!(ctx.budget, c.remaining, "budget {i}");
            assert!(ctx.overlay.is_empty());
        }
    }
}
