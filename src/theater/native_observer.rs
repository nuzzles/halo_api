//! Shared native hooks and observation counters. Observer state is per context.
use super::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum NativeHookKind {
    Position,
    UnitReference,
    MovementState,
    Mpp,
    EquipmentCreation,
    EquipmentState,
    Probe,
    PlayerState,
    GameEngine,
    ManagedObject,
    Navpoint,
    Objective,
    ManagedProperty,
    HeldWeapon,
    ObjectParent,
    UnitEquipment,
    CamoState,
    SpartanAbility,
    AbilityNonPredicted,
    AbilityEnergy,
    GrenadeSet,
    AbilitySet,
    EmpTimer,
    GrenadeCounts,
    WeaponAmmo,
    WeaponRounds,
    DesiredWeaponSet,
    GroundWeaponAmmo,
    MobilityAction,
    RecordMask,
    ControlView,
}
#[derive(Debug, Clone, Copy)]
pub enum NativeHookPublication<'a> {
    ControlView(&'a NativeControlVerdict),
    Component(&'a FilmComponentObservation),
    MobilityAction([bool; 2]),
    RecordMask {
        indices: &'a [usize],
        payload: &'a [u8],
        after_i0: usize,
    },
}
impl NativeHookPublication<'_> {
    pub fn kind(&self) -> NativeHookKind {
        match self {
            Self::ControlView(_) => NativeHookKind::ControlView,
            Self::MobilityAction(_) => NativeHookKind::MobilityAction,
            Self::RecordMask { .. } => NativeHookKind::RecordMask,
            Self::Component(value) => match value {
                FilmComponentObservation::Position { .. } => NativeHookKind::Position,
                FilmComponentObservation::UnitReference { .. } => NativeHookKind::UnitReference,
                FilmComponentObservation::MovementState { .. } => NativeHookKind::MovementState,
                FilmComponentObservation::Mpp { .. } => NativeHookKind::Mpp,
                FilmComponentObservation::EquipmentCreation { .. } => {
                    NativeHookKind::EquipmentCreation
                }
                FilmComponentObservation::EquipmentState { .. } => NativeHookKind::EquipmentState,
                FilmComponentObservation::Probe { .. } => NativeHookKind::Probe,
                FilmComponentObservation::PlayerState { .. } => NativeHookKind::PlayerState,
                FilmComponentObservation::GameEngine { .. } => NativeHookKind::GameEngine,
                FilmComponentObservation::ManagedObject { .. } => NativeHookKind::ManagedObject,
                FilmComponentObservation::Navpoint { .. } => NativeHookKind::Navpoint,
                FilmComponentObservation::Objective { .. } => NativeHookKind::Objective,
                FilmComponentObservation::ManagedProperty { .. } => NativeHookKind::ManagedProperty,
                FilmComponentObservation::HeldWeapon { .. } => NativeHookKind::HeldWeapon,
                FilmComponentObservation::ObjectParent { .. } => NativeHookKind::ObjectParent,
                FilmComponentObservation::UnitEquipment { .. } => NativeHookKind::UnitEquipment,
                FilmComponentObservation::CamoState { .. } => NativeHookKind::CamoState,
                FilmComponentObservation::SpartanAbility { .. } => NativeHookKind::SpartanAbility,
                FilmComponentObservation::AbilityNonPredicted { .. } => {
                    NativeHookKind::AbilityNonPredicted
                }
                FilmComponentObservation::AbilityEnergy { .. } => NativeHookKind::AbilityEnergy,
                FilmComponentObservation::GrenadeSet { .. } => NativeHookKind::GrenadeSet,
                FilmComponentObservation::AbilitySet { .. } => NativeHookKind::AbilitySet,
                FilmComponentObservation::EmpTimer { .. } => NativeHookKind::EmpTimer,
                FilmComponentObservation::GrenadeCounts { .. } => NativeHookKind::GrenadeCounts,
                FilmComponentObservation::WeaponAmmo { .. } => NativeHookKind::WeaponAmmo,
                FilmComponentObservation::WeaponRounds { .. } => NativeHookKind::WeaponRounds,
                FilmComponentObservation::DesiredWeaponSet { .. } => {
                    NativeHookKind::DesiredWeaponSet
                }
                FilmComponentObservation::GroundWeaponAmmo { .. } => {
                    NativeHookKind::GroundWeaponAmmo
                }
            },
        }
    }
}
pub type NativeHook = Arc<dyn for<'a> Fn(NativeHookPublication<'a>) + Send + Sync>;
type SharedMap<K, V> = Arc<Mutex<BTreeMap<K, V>>>;
/// Scoped lookup evidence for movement publications; absent means no world context.
#[derive(Clone, Default)]
pub(crate) struct MovementBindingContext(Arc<Mutex<MovementBinding>>);
type MovementBinding = Option<(u32, Option<u32>)>;
impl MovementBindingContext {
    pub(crate) fn archetype(&self, slot: u32) -> Option<u32> {
        self.0
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .as_ref()
            .filter(|(captured, _)| *captured == slot)
            .and_then(|(_, ti)| *ti)
    }
}
pub(crate) struct MovementBindingRestoration {
    context: MovementBindingContext,
    saved: Option<(u32, Option<u32>)>,
}
impl Drop for MovementBindingRestoration {
    fn drop(&mut self) {
        *self.context.0.lock().unwrap_or_else(|e| e.into_inner()) = self.saved;
    }
}
#[derive(Clone, Default)]
struct State {
    hooks: BTreeMap<NativeHookKind, NativeHook>,
    movement_binding: MovementBindingContext,
    counters: FilmReadDiagnostics,
    widths: SharedMap<String, BTreeMap<usize, u64>>,
    anticipated: Option<SharedMap<u32, u64>>,
    absolute: Option<SharedMap<i32, u64>>,
}
#[derive(Clone, Default)]
pub struct NativeFilmObserver(Arc<Mutex<State>>);
impl std::fmt::Debug for NativeFilmObserver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state = self.0.lock().unwrap_or_else(|e| e.into_inner());
        f.debug_struct("NativeFilmObserver")
            .field("hooks", &state.hooks.keys().collect::<Vec<_>>())
            .field("counters", &state.counters)
            .finish()
    }
}
impl NativeFilmObserver {
    pub(crate) fn record_new_binding_refusal(&self, refusal: &NativeNewBindingRefusal) {
        self.0
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .counters
            .new_binding_refusals
            .push(refusal.clone());
    }

    pub(crate) fn movement_binding_context(&self) -> MovementBindingContext {
        self.0
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .movement_binding
            .clone()
    }
    pub(crate) fn scope_movement_binding(
        &self,
        slot: u32,
        archetype: Option<u32>,
    ) -> MovementBindingRestoration {
        let context = self.movement_binding_context();
        let saved = context
            .0
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .replace((slot, archetype));
        MovementBindingRestoration { context, saved }
    }
    pub fn same_instance(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
    /// Replace or clear one hook; return the previous callback for restoration.
    pub fn set_hook(&self, kind: NativeHookKind, hook: Option<NativeHook>) -> Option<NativeHook> {
        let mut state = self.0.lock().unwrap_or_else(|e| e.into_inner());
        match hook {
            Some(h) => state.hooks.insert(kind, h),
            None => state.hooks.remove(&kind),
        }
    }
    pub fn has_hook(&self, kind: NativeHookKind) -> bool {
        self.0
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .hooks
            .contains_key(&kind)
    }
    /// Invoke without holding the state lock, permitting callback replacement and
    /// counter inspection during callbacks. Each publication resolves its hook anew.
    pub fn publish(&self, value: NativeHookPublication<'_>) {
        let hook = self
            .0
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .hooks
            .get(&value.kind())
            .cloned();
        if let Some(hook) = hook {
            hook(value);
        }
    }
    /// Copy native scalar counters and callback identities, sharing allocated maps.
    /// Unlike Clone, later hook replacement and scalar updates are independent.
    pub fn shallow_copy(&self) -> Self {
        Self(Arc::new(Mutex::new(
            self.0.lock().unwrap_or_else(|e| e.into_inner()).clone(),
        )))
    }
    pub fn counters(&self) -> FilmReadDiagnostics {
        let state = self.0.lock().unwrap_or_else(|e| e.into_inner());
        let mut out = state.counters.clone();
        out.component_widths = state
            .widths
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone();
        out.anticipated_bindings = state
            .anticipated
            .as_ref()
            .map(|m| m.lock().unwrap_or_else(|e| e.into_inner()).clone())
            .unwrap_or_default();
        out.absolute_indices = state
            .absolute
            .as_ref()
            .map(|m| m.lock().unwrap_or_else(|e| e.into_inner()).clone())
            .unwrap_or_default();
        out
    }
    pub fn take_absolute_indices(&self) -> BTreeMap<i32, u64> {
        let mut state = self.0.lock().unwrap_or_else(|e| e.into_inner());
        let previous = state.absolute.replace(Default::default());
        previous
            .map(|m| m.lock().unwrap_or_else(|e| e.into_inner()).clone())
            .unwrap_or_default()
    }
    pub(crate) fn record_repair(&self, name: &str, widths: &[usize]) {
        let mut state = self.0.lock().unwrap_or_else(|e| e.into_inner());
        state.counters.repaired_records += 1;
        let mut map = state.widths.lock().unwrap_or_else(|e| e.into_inner());
        let counts = map.entry(name.into()).or_default();
        for width in widths {
            *counts.entry(*width).or_default() += 1;
        }
    }
    pub(crate) fn record_validated_resync(&self) {
        self.0
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .counters
            .validated_resyncs += 1;
    }
    pub(crate) fn record_chain_outcome(&self, outcome: ChainInferenceOutcome) {
        let mut state = self.0.lock().unwrap_or_else(|e| e.into_inner());
        *state.counters.chain_outcomes.entry(outcome).or_default() += 1;
    }
    pub(crate) fn record_admission(&self, admission: &FilmViewAdmission) {
        let mut state = self.0.lock().unwrap_or_else(|e| e.into_inner());
        match admission {
            FilmViewAdmission::Unbound => state.counters.rejected_unbound += 1,
            FilmViewAdmission::OtherView => state.counters.rejected_other_view += 1,
            FilmViewAdmission::Anticipated(d) => {
                *state
                    .anticipated
                    .get_or_insert_with(Default::default)
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .entry(d.archetype)
                    .or_default() += 1;
            }
            FilmViewAdmission::Allowed => {}
        }
    }
    pub(crate) fn record_absolute(&self, index: i32) {
        let mut state = self.0.lock().unwrap_or_else(|e| e.into_inner());
        *state
            .absolute
            .get_or_insert_with(Default::default)
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .entry(index)
            .or_default() += 1;
    }
    #[cfg(test)]
    pub(crate) fn absorb(&self, read: &FilmReadDiagnostics) {
        let mut counts = read.clone();
        counts.mobility_actions.clear();
        counts.mobility_offsets = None;
        counts.component_observations.clear();
        {
            let mut state = self.0.lock().unwrap_or_else(|e| e.into_inner());
            for (name, widths) in std::mem::take(&mut counts.component_widths) {
                let mut map = state.widths.lock().unwrap_or_else(|e| e.into_inner());
                for (width, n) in widths {
                    *map.entry(name.clone())
                        .or_default()
                        .entry(width)
                        .or_default() += n;
                }
            }
            for (index, n) in std::mem::take(&mut counts.absolute_indices) {
                *state
                    .absolute
                    .get_or_insert_with(Default::default)
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .entry(index)
                    .or_default() += n;
            }
            for (index, n) in std::mem::take(&mut counts.anticipated_bindings) {
                *state
                    .anticipated
                    .get_or_insert_with(Default::default)
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .entry(index)
                    .or_default() += n;
            }
            state.counters.merge(&counts);
        }
        if let Some(publications) = read.ordered_publications() {
            for p in publications {
                self.publish(match p {
                    ComponentObserverPublication::Component(c) => {
                        NativeHookPublication::Component(c)
                    }
                    ComponentObserverPublication::MobilityAction(v) => {
                        NativeHookPublication::MobilityAction(v)
                    }
                });
            }
        }
    }
}

/// Restores exactly the callbacks saved at scope entry, including absent hooks.
/// Changes made to those hooks inside the scope are replaced on restoration.
#[must_use = "hold the guard for the duration of speculative reads"]
pub struct NativeCaptureRestoration {
    observer: NativeFilmObserver,
    saved: Vec<(NativeHookKind, Option<NativeHook>)>,
}
impl NativeCaptureRestoration {
    pub fn restore(self) {
        drop(self);
    }
}
impl Drop for NativeCaptureRestoration {
    fn drop(&mut self) {
        let mut state = self.observer.0.lock().unwrap_or_else(|e| e.into_inner());
        for (kind, hook) in self.saved.drain(..) {
            if let Some(hook) = hook {
                state.hooks.insert(kind, hook);
            } else {
                state.hooks.remove(&kind);
            }
        }
    }
}
impl NativeFilmObserver {
    fn neutralize(&self, kinds: &[NativeHookKind]) -> NativeCaptureRestoration {
        let mut state = self.0.lock().unwrap_or_else(|e| e.into_inner());
        let saved = kinds
            .iter()
            .map(|&kind| (kind, state.hooks.remove(&kind)))
            .collect();
        NativeCaptureRestoration {
            observer: self.clone(),
            saved,
        }
    }
    pub fn neutralize_captures(&self) -> NativeCaptureRestoration {
        self.neutralize(&[
            NativeHookKind::Position,
            NativeHookKind::UnitReference,
            NativeHookKind::MovementState,
        ])
    }
    pub fn neutralize_position_capture(&self) -> NativeCaptureRestoration {
        self.neutralize(&[NativeHookKind::Position, NativeHookKind::MovementState])
    }
    pub fn neutralize_movement(&self) -> NativeCaptureRestoration {
        self.neutralize(&[NativeHookKind::MovementState])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn native_capture_scopes_nest_and_restore_during_unwind() {
        let observer = NativeFilmObserver::default();
        let original: NativeHook = Arc::new(|_| {});
        for kind in [
            NativeHookKind::Position,
            NativeHookKind::UnitReference,
            NativeHookKind::MovementState,
        ] {
            observer.set_hook(kind, Some(original.clone()));
        }
        let outer = observer.neutralize_position_capture();
        assert!(observer.has_hook(NativeHookKind::UnitReference));
        {
            let _inner = observer.neutralize_captures();
            assert!(!observer.has_hook(NativeHookKind::UnitReference));
            observer.set_hook(NativeHookKind::MovementState, Some(Arc::new(|_| {})));
        }
        assert!(!observer.has_hook(NativeHookKind::MovementState));
        assert!(observer.has_hook(NativeHookKind::UnitReference));
        outer.restore();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _scope = observer.neutralize_movement();
            assert!(!observer.has_hook(NativeHookKind::MovementState));
            panic!("test callback unwinding");
        }));
        assert!(result.is_err());
        for kind in [
            NativeHookKind::Position,
            NativeHookKind::UnitReference,
            NativeHookKind::MovementState,
        ] {
            assert!(Arc::ptr_eq(
                &observer.set_hook(kind, None).unwrap(),
                &original
            ));
        }
    }

    #[test]
    fn native_observer_reentrant_replacement_and_restoration() {
        let observer = NativeFilmObserver::default();
        let calls = Arc::new(AtomicUsize::new(0));
        let replacement: NativeHook = {
            let calls = calls.clone();
            Arc::new(move |_| {
                calls.fetch_add(10, Ordering::SeqCst);
            })
        };
        let initial: NativeHook = {
            let observer = observer.clone();
            let calls = calls.clone();
            Arc::new(move |_| {
                calls.fetch_add(1, Ordering::SeqCst);
                observer.set_hook(NativeHookKind::MobilityAction, Some(replacement.clone()));
                let _ = observer.counters();
                observer.publish(NativeHookPublication::MobilityAction([false, true]));
            })
        };
        assert!(
            observer
                .set_hook(NativeHookKind::MobilityAction, Some(initial.clone()))
                .is_none()
        );
        observer.publish(NativeHookPublication::MobilityAction([true, false]));
        assert_eq!(calls.load(Ordering::SeqCst), 11);
        let previous = observer
            .set_hook(NativeHookKind::MobilityAction, None)
            .unwrap();
        observer.publish(NativeHookPublication::MobilityAction([false, false]));
        assert_eq!(calls.load(Ordering::SeqCst), 11);
        observer.set_hook(NativeHookKind::MobilityAction, Some(previous));
        observer.publish(NativeHookPublication::MobilityAction([false, false]));
        assert_eq!(calls.load(Ordering::SeqCst), 21);
        // Clear hooks that capture their own shared observer before dropping it.
        observer.set_hook(NativeHookKind::MobilityAction, None);
    }

    #[test]
    fn native_observer_counters_and_histogram_are_independent_of_hooks() {
        let observer = NativeFilmObserver::default();
        let read = FilmReadDiagnostics {
            repaired_records: 2,
            absolute_indices: [(-1, 3), (7, 4)].into(),
            ..Default::default()
        };
        observer.absorb(&read);
        observer.absorb(&read);
        assert_eq!(observer.counters().repaired_records, 4);
        assert_eq!(observer.take_absolute_indices(), [(-1, 6), (7, 8)].into());
        assert!(observer.take_absolute_indices().is_empty());
        assert_eq!(observer.counters().repaired_records, 4);
        assert!(NativeFilmObserver::default().counters().is_empty());
    }
}
