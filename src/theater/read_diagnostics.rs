//! Native read diagnostics, including speculative and padded reads when callers
//! explicitly retain those attempts. These observations do not establish valid records.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Native movement-state callback identifiers; velocity uses the native hook label.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativeMovementComponent {
    #[serde(rename = "unit-crouch-component")]
    Crouch,
    #[serde(rename = "biped-slide-component")]
    Slide,
    #[serde(rename = "biped-posture-physics-component")]
    Posture,
    #[serde(rename = "unit-control-component")]
    UnitControl,
    #[serde(rename = "object-translational-velocity-component")]
    Velocity,
    #[serde(rename = "biped-mobility-action-component")]
    Mobility,
    #[serde(rename = "biped-spartan-ability-component")]
    ActiveAbility,
}

/// Native default-state callback fields; labels match the reference reader.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativeMppField {
    #[serde(rename = "mpp-word9")]
    Word9,
    #[serde(rename = "mpp-word32")]
    Word32,
    #[serde(rename = "mpp-variant-name")]
    VariantName,
    #[serde(rename = "mpp-tail-name")]
    TailName,
}

/// Native default-state callback fields; labels match the reference reader.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativeEquipmentCreationField {
    #[serde(rename = "entity-ref-index5")]
    Reference,
    #[serde(rename = "ability-enabled-id")]
    AbilityId,
}

/// Native equipment-state callback fields, identified by registry name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativeEquipmentField {
    #[serde(rename = "equipment-deployed-component")]
    Deployed,
    #[serde(rename = "equipment-activated-component")]
    Activated,
    #[serde(rename = "equipment-creator-component")]
    Creator,
    #[serde(rename = "equipment-energy-component")]
    Energy,
    #[serde(rename = "equipment-energy-delay-ticks-left-component")]
    EnergyDelay,
    #[serde(rename = "equipment-charges-remaining-component")]
    Charges,
}

/// Native probe fields. Values remain raw; their gameplay meaning is not inferred.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativeProbeComponent {
    #[serde(rename = "managed-object-networked-splash-message-static-component")]
    SplashStatic,
    #[serde(rename = "managed-object-networked-splash-message-dynamic-component")]
    SplashDynamic,
    #[serde(rename = "high-frequency")]
    HighFrequency,
    #[serde(rename = "managed-object-property-name-component")]
    ManagedObjectPropertyName,
}

/// Native player-state callback fields, identified by registry name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativePlayerStateField {
    #[serde(rename = "player-soft-kill-timer-component")]
    SoftKill,
    #[serde(rename = "player-target-tracking-detection-component")]
    TargetTracking,
    #[serde(rename = "player-desired-respawn-player-component")]
    DesiredRespawnPlayer,
    #[serde(rename = "player-engine-loadout-component")]
    Loadout,
    #[serde(rename = "player-desired-respawn-location-component")]
    DesiredRespawnLocation,
    #[serde(rename = "player-lives-remaining-component")]
    Lives,
    #[serde(rename = "player-last-betrayer-component")]
    LastBetrayer,
    #[serde(rename = "player-control-aiming-component")]
    ControlAiming,
    #[serde(rename = "player-active-in-game-component")]
    ActiveInGame,
    #[serde(rename = "player-pending-join-in-progress-spawn-component")]
    PendingJoinInProgress,
    #[serde(rename = "player-malleable-properties-simulation-component")]
    MalleableProperties,
}

/// Native game-engine callback fields, identified by registry name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativeGameEngineField {
    #[serde(rename = "game-engine-current-state-component")]
    State,
    #[serde(rename = "game-engine-current-round-component")]
    Round,
    #[serde(rename = "game-engine-sudden-death-time-left-component")]
    SuddenDeath,
    #[serde(rename = "game-engine-grace-period-time-left-component")]
    GracePeriod,
    #[serde(rename = "game-engine-round-condition-flags-component")]
    RoundConditions,
}

/// Stable native ManagedObject hook fields, serialized using registry names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativeManagedObjectField {
    #[serde(rename = "managed-object-boundary-visibility-component")]
    BoundaryVisibility,
    #[serde(rename = "managed-object-boundary-color-component")]
    BoundaryColor,
    #[serde(rename = "managed-object-rtpc-component")]
    Rtpc,
}

/// Stable native Navpoint hook fields, serialized using registry names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativeNavpointField {
    #[serde(rename = "managed-navpoint-radial-progress")]
    RadialProgress,
    #[serde(rename = "managed-navpoint-manual-timer-initial-duration-component")]
    ManualTimerInitial,
    #[serde(rename = "managed-navpoint-manual-timer-current-duration-component")]
    ManualTimerCurrent,
}

/// Stable native Objective hook fields, serialized using registry names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativeObjectiveField {
    #[serde(rename = "managed-objective-timers-component")]
    Timers,
    #[serde(rename = "managed-objective-object-reference-component")]
    ObjectReference,
    #[serde(rename = "managed-objective-type-component")]
    Type,
    #[serde(rename = "managed-objective-progress-component")]
    Progress,
    #[serde(rename = "managed-objective-required-progress-component")]
    RequiredProgress,
    #[serde(rename = "managed-objective-state-component")]
    State,
}

/// Stable native ManagedProperty hook fields, serialized using registry names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativeManagedPropertyField {
    #[serde(rename = "managed-object-property-component")]
    Scalar,
    #[serde(rename = "managed-object-player-masked-property-component")]
    PerPlayer,
}

/// Positional object-parent fields from the native observer; no parent identity
/// is inferred from the quantized word or free-reference index.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeObjectParentState {
    pub archetype: u32,
    pub parameter: u32,
    pub start_bit: i64,
    pub end_bit: i64,
    pub attached: bool,
    pub quantized_word: u32,
    pub word: u32,
    pub optional_word: Option<u32>,
    pub flags: [bool; 2],
    pub matrix: [u32; 3],
    pub velocity: Option<u32>,
    pub byte: u32,
    pub flag_c: bool,
    pub free_read: bool,
    pub free_bits: usize,
    pub free_id: Option<u64>,
    pub alternate: Option<u32>,
    pub tail_sign: bool,
    pub tail6: Option<u32>,
    pub tail_bit: bool,
    pub tail3: Option<u32>,
}

/// Raw camo hook state; absent gated values are distinct from zero.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeCamoState {
    pub state: u8,
    pub flag0: bool,
    pub flag1: Option<bool>,
    pub fraction: Option<u16>,
    pub sub: [Option<u16>; 6],
}

/// Native non-predicted ability publication, including unsupported body prefixes.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeAbilityNonPredictedState {
    pub tag: u32,
    pub body_walked: bool,
    pub body_ok: bool,
    /// None is native Inner=-1: the body was not read.
    pub inner: Option<u32>,
    pub flags: u32,
    pub position: [u32; 3],
    pub mid: u32,
    pub value8: Option<u32>,
    /// Direction and magnitude quanta; None denotes the constant-vector gate.
    pub vectors: [Option<[u32; 2]>; 3],
    pub packed: u32,
    pub tail: u32,
}

/// Native component-hook publications. Values describe reads, including padded
/// and speculative attempts; they are not evidence of accepted gameplay events.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FilmComponentObservation {
    /// IEEE-754 bits retain exact native values, including non-finite values.
    Position {
        position_kind: crate::theater::NativePositionKind,
        vector_bits: [u32; 3],
        bit: i64,
        slot: u32,
    },
    UnitReference {
        reference: crate::theater::NativeUnitReference,
    },
    MovementState {
        component: NativeMovementComponent,
        slot: u32,
        values: Vec<u64>,
    },
    Mpp {
        field: NativeMppField,
        value: u64,
        present: bool,
    },
    EquipmentCreation {
        field: NativeEquipmentCreationField,
        value: u64,
        present: bool,
    },
    EquipmentState {
        field: NativeEquipmentField,
        value: u64,
        present: bool,
    },
    Probe {
        archetype: u32,
        component: NativeProbeComponent,
        values: Vec<u64>,
    },
    PlayerState {
        field: NativePlayerStateField,
        values: Vec<u64>,
        present: bool,
    },
    GameEngine {
        field: NativeGameEngineField,
        values: Vec<u64>,
        present: bool,
    },
    ManagedObject {
        field: NativeManagedObjectField,
        values: Vec<u64>,
    },
    Navpoint {
        field: NativeNavpointField,
        values: Vec<u64>,
    },
    Objective {
        field: NativeObjectiveField,
        values: Vec<u64>,
    },
    ManagedProperty {
        field: NativeManagedPropertyField,
        values: Vec<u64>,
    },

    HeldWeapon {
        id_high: u32,
        id_low: u32,
    },
    ObjectParent {
        state: Box<NativeObjectParentState>,
    },
    UnitEquipment {
        state: Box<crate::theater::UnitEquipmentRead>,
    },
    CamoState {
        state: Box<NativeCamoState>,
    },
    SpartanAbility {
        tag: u64,
        sub: u64,
        reference: u64,
        has_reference: bool,
    },
    AbilityNonPredicted {
        state: Box<NativeAbilityNonPredictedState>,
    },
    AbilityEnergy {
        mask: u32,
        charges: [i32; 3],
    },
    GrenadeSet {
        mask: u32,
        selection: i32,
    },
    AbilitySet {
        counter: u64,
        rank: i32,
        width: usize,
    },
    EmpTimer {
        quantum: u32,
    },
    GrenadeCounts {
        count: u64,
        values: Vec<u64>,
    },
    WeaponAmmo {
        magazine: Option<u32>,
        fraction: Option<u32>,
    },
    WeaponRounds {
        rounds: u32,
    },
    DesiredWeaponSet {
        selection: u32,
    },
    GroundWeaponAmmo {
        a: u32,
        b: u32,
        c: u32,
    },
}

/// Signed native skip that rewinds, compacts padding, or cannot fit the source-offset model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeWidthAdjustment {
    pub component: String,
    pub calibrated: bool,
    /// None denotes the legacy calibrated/stub override. Mobility and New-record
    /// tails are identified separately rather than reported as unported stubs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub purpose: Option<NativeWidthPurpose>,
    pub bit: i64,
    pub width: i64,
    /// For a compact positive skip, source bits retained as scalar fields. The
    /// remaining skipped bits are synthetic zero padding, not recorded data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retained_bits: Option<usize>,
    /// Checked address-sized projection of the native wrapping target. None
    /// includes negative targets and positive targets outside usize; use
    /// `native_end_bit` rather than treating this as missing native data.
    pub end_bit: Option<usize>,
}
impl NativeWidthAdjustment {
    /// Target of the native signed 64-bit Skip, reconstructed from retained
    /// inputs. This also works for old exports without adding redundant fields.
    /// A negative result is a cursor outcome, not a source bit range. It does not
    /// imply that a subsequent source read or header can decode at this target.
    /// The Option return is retained for compatibility; signed inputs always
    /// produce Some, including when addition wraps.
    pub fn native_end_bit(&self) -> Option<i64> {
        Some(self.bit.wrapping_add(self.width))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativeWidthPurpose {
    RecordPrefix,
    MobilityExtra,
    NewRecordTail,
    NewRecordDefault,
}

/// A native field was reached, but its raw width exceeds this reader's address
/// domain. This is a decoder limitation, not evidence of missing source bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeWidthRefusal {
    pub field: String,
    pub bit: i64,
    pub raw_width: u64,
    pub maximum: u64,
}

/// A bounded operation refused before advancing the cursor. This is not a
/// fabricated zero-valued field and does not include native padded reads.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeReadRefusal {
    pub field: String,
    pub bit: i64,
    pub width: u64,
    pub source_bits: usize,
    pub operation: NativeReadOperation,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativeReadOperation {
    /// The scalar cursor returned None (source bounds or bounded width/domain).
    Scalar,
    /// The native grouped source guard failed before any member was read.
    GroupGuard,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct FilmReadDiagnostics {
    /// Native NEW binding refusals, in read order; parsed records are retained.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub new_binding_refusals: Vec<crate::theater::NativeNewBindingRefusal>,
    /// Reads assuming vehicle object byte +0x818 is set, as in LevelUp d61443e.
    /// This byte is runtime state, not a recorded presence bit. Counts include
    /// refused attempts and are kept separate from raw component fields.
    #[serde(skip_serializing_if = "is_zero_count")]
    pub vehicle_type_physics_assumed: u64,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub read_refusals: Vec<NativeReadRefusal>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub width_refusals: Vec<NativeWidthRefusal>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub width_adjustments: Vec<NativeWidthAdjustment>,
    pub repaired_records: u64,
    pub validated_resyncs: u64,
    pub component_widths: BTreeMap<String, BTreeMap<usize, u64>>,
    pub chain_outcomes: BTreeMap<crate::theater::ChainInferenceOutcome, u64>,
    pub rejected_unbound: u64,
    pub rejected_other_view: u64,
    pub anticipated_bindings: BTreeMap<u32, u64>,
    /// Native Observation.IndexAbsolus; -1 denotes the build's default region.
    pub absolute_indices: BTreeMap<i32, u64>,
    /// Ordered native MobilityActionHook flags, including failed/speculative reads.
    /// Unlike movement-state capture, this historical hook is never suppressed.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub mobility_actions: Vec<[bool; 2]>,
    /// Number of component publications preceding each mobility publication.
    /// None with nonempty mobility_actions means ordering was not retained.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mobility_offsets: Option<Vec<usize>>,
    /// Ordered publications from named component hooks. Position, movement and
    /// reference callbacks obey their independent native capture policies.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub component_observations: Vec<FilmComponentObservation>,
}
fn is_zero_count(value: &u64) -> bool {
    *value == 0
}

impl FilmReadDiagnostics {
    pub(crate) fn publish_mobility(&mut self, flags: [bool; 2]) {
        if self.mobility_actions.is_empty() {
            self.mobility_offsets = Some(Vec::new());
        }
        if let Some(offsets) = &mut self.mobility_offsets {
            offsets.push(self.component_observations.len());
        }
        self.mobility_actions.push(flags);
    }

    fn valid_mobility_offsets(&self) -> Option<&[usize]> {
        if self.mobility_actions.is_empty() {
            return Some(&[]);
        }
        self.mobility_offsets.as_deref().filter(|offsets| {
            offsets.len() == self.mobility_actions.len()
                && offsets
                    .iter()
                    .all(|&n| n <= self.component_observations.len())
                && offsets.windows(2).all(|w| w[0] <= w[1])
        })
    }

    /// Borrow the combined native publication stream. Old or inconsistent
    /// diagnostics without sufficient order evidence return None.
    pub fn ordered_publications(
        &self,
    ) -> Option<impl Iterator<Item = crate::theater::ComponentObserverPublication<'_>>> {
        let offsets = self.valid_mobility_offsets()?;
        let mut mobility = 0;
        let mut component = 0;
        Some(std::iter::from_fn(move || {
            if offsets.get(mobility) == Some(&component) {
                let flags = self.mobility_actions[mobility];
                mobility += 1;
                return Some(crate::theater::ComponentObserverPublication::MobilityAction(flags));
            }
            let value = self.component_observations.get(component)?;
            component += 1;
            Some(crate::theater::ComponentObserverPublication::Component(
                value,
            ))
        }))
    }

    fn retain_components(&mut self, mut keep: impl FnMut(&FilmComponentObservation) -> bool) {
        if self.mobility_actions.is_empty() {
            self.component_observations.retain(keep);
            self.mobility_offsets = None;
            return;
        }
        let offsets = self.valid_mobility_offsets().map(<[usize]>::to_vec);
        let mut prefix = vec![0];
        let mut retained = 0;
        self.component_observations.retain(|o| {
            let yes = keep(o);
            retained += usize::from(yes);
            prefix.push(retained);
            yes
        });
        self.mobility_offsets = if self.mobility_actions.is_empty() {
            None
        } else {
            offsets.map(|offsets| offsets.into_iter().map(|n| prefix[n]).collect())
        };
    }

    pub(crate) fn suppress_positions(&mut self) {
        self.retain_components(|o| !matches!(o, FilmComponentObservation::Position { .. }));
    }
    pub(crate) fn suppress_unit_references(&mut self) {
        self.retain_components(|o| !matches!(o, FilmComponentObservation::UnitReference { .. }));
    }

    fn merge_publications(&mut self, other: &Self) {
        // Validate shape/boundary in constant time. An internally unsorted vector
        // stays unsorted after concatenation and is rejected by ordered_publications.
        // Avoid rescanning or copying the accumulated publication history per merge.
        let known = |d: &Self| {
            d.mobility_actions.is_empty()
                || d.mobility_offsets.as_ref().is_some_and(|v| {
                    v.len() == d.mobility_actions.len()
                        && v.last()
                            .is_some_and(|&n| n <= d.component_observations.len())
                })
        };
        if known(self) && known(other) {
            if self.mobility_actions.is_empty() {
                self.mobility_offsets = Some(Vec::new());
            }
            if !other.mobility_actions.is_empty() {
                self.mobility_offsets.as_mut().unwrap().extend(
                    other
                        .mobility_offsets
                        .as_ref()
                        .unwrap()
                        .iter()
                        // Imported malformed offsets may overflow when shifted.
                        // Saturation keeps them out of bounds (and order unknown)
                        // without panicking or wrapping into a plausible offset.
                        .map(|n| n.saturating_add(self.component_observations.len())),
                );
            }
        } else {
            self.mobility_offsets = None;
        }
        self.mobility_actions
            .extend_from_slice(&other.mobility_actions);
        self.component_observations
            .extend_from_slice(&other.component_observations);
        if self.mobility_actions.is_empty() {
            self.mobility_offsets = None;
        }
    }

    pub fn is_empty(&self) -> bool {
        self.new_binding_refusals.is_empty()
            && self.vehicle_type_physics_assumed == 0
            && self.read_refusals.is_empty()
            && self.width_refusals.is_empty()
            && self.width_adjustments.is_empty()
            && self.repaired_records == 0
            && self.validated_resyncs == 0
            && self.component_widths.is_empty()
            && self.absolute_indices.is_empty()
            && self.chain_outcomes.is_empty()
            && self.rejected_unbound == 0
            && self.rejected_other_view == 0
            && self.anticipated_bindings.is_empty()
            && self.mobility_actions.is_empty()
            && self.component_observations.is_empty()
    }
    pub fn merge(&mut self, other: &Self) {
        self.new_binding_refusals
            .extend_from_slice(&other.new_binding_refusals);
        self.vehicle_type_physics_assumed += other.vehicle_type_physics_assumed;
        self.read_refusals.extend_from_slice(&other.read_refusals);
        self.width_refusals.extend_from_slice(&other.width_refusals);
        self.width_adjustments
            .extend_from_slice(&other.width_adjustments);
        self.merge_publications(other);
        self.repaired_records += other.repaired_records;
        self.validated_resyncs += other.validated_resyncs;
        for (name, widths) in &other.component_widths {
            let target = self.component_widths.entry(name.clone()).or_default();
            for (&width, &count) in widths {
                *target.entry(width).or_default() += count;
            }
        }
        for (outcome, count) in &other.chain_outcomes {
            *self.chain_outcomes.entry(outcome.clone()).or_default() += count;
        }
        self.rejected_unbound += other.rejected_unbound;
        self.rejected_other_view += other.rejected_other_view;
        for (&ti, &count) in &other.anticipated_bindings {
            *self.anticipated_bindings.entry(ti).or_default() += count;
        }
        for (&index, &count) in &other.absolute_indices {
            *self.absolute_indices.entry(index).or_default() += count;
        }
    }
    /// Native prendreIndexAbsolus semantics: snapshot and clear the histogram.
    pub fn take_absolute_indices(&mut self) -> BTreeMap<i32, u64> {
        std::mem::take(&mut self.absolute_indices)
    }
    pub(super) fn absolute(&mut self, index: i32) {
        *self.absolute_indices.entry(index).or_default() += 1;
    }
}

/// Result of a native probing operation; diagnostics survive an unsuccessful attempt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilmReadAttempt<T> {
    pub result: Option<T>,
    pub diagnostics: FilmReadDiagnostics,
}

/// Native raw-resync observer ownership. A capture scan shallow-copies its caller:
/// an existing absolute-index map is shared, while a newly allocated map is local.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct RawResyncDiagnostics {
    pub caller_visible: FilmReadDiagnostics,
    /// Reads retained by Rust that native leaves in a temporary capture observer.
    pub detached_scans: FilmReadDiagnostics,
    pub absolute_indices_initialized: bool,
}
impl RawResyncDiagnostics {
    pub fn with_initialized_histogram() -> Self {
        Self {
            absolute_indices_initialized: true,
            ..Self::default()
        }
    }
    pub fn absorb_direct(&mut self, reads: &FilmReadDiagnostics) {
        self.absolute_indices_initialized |= !reads.absolute_indices.is_empty();
        self.caller_visible.merge(reads);
    }
    pub fn absorb_scan(&mut self, reads: &FilmReadDiagnostics) {
        self.absolute_indices_initialized |= !self.caller_visible.absolute_indices.is_empty();
        if self.absolute_indices_initialized {
            self.caller_visible.merge(reads);
        } else {
            // Hook closures remain shared even when the absolute-index map was
            // allocated only in the temporary observer.
            self.caller_visible.merge_publications(reads);
            let mut detached = reads.clone();
            detached.mobility_actions.clear();
            detached.mobility_offsets = None;
            detached.component_observations.clear();
            self.detached_scans.merge(&detached);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theater::PositionEncoding;
    use std::io::Read;
    fn unhex(s: &str) -> Vec<u8> {
        s.as_bytes()
            .as_chunks::<2>()
            .0
            .iter()
            .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap())
            .collect()
    }
    #[test]
    fn native_read_diagnostics() {
        check_read_diagnostics(
            include_bytes!("fixtures/read-diagnostics-v41.json.zlib"),
            false,
        );
    }
    #[test]
    fn native_mobility_reads() {
        check_read_diagnostics(
            include_bytes!("fixtures/mobility-reads-v41.json.zlib"),
            true,
        );
    }
    #[test]
    fn native_component_hook_reads() {
        check_read_diagnostics(
            include_bytes!("fixtures/component-hook-reads-v41.json.zlib"),
            true,
        );
    }
    #[test]
    fn native_ability_hook_reads() {
        check_read_diagnostics(
            include_bytes!("fixtures/ability-hook-reads-v41.json.zlib"),
            true,
        );
    }
    #[test]
    fn native_unit_reference_hook_reads() {
        check_read_diagnostics(
            include_bytes!("fixtures/unit-reference-hook-reads-d61443e-v41.json.zlib"),
            true,
        );
    }
    #[test]
    fn native_object_hook_reads() {
        check_read_diagnostics(
            include_bytes!("fixtures/object-hook-reads-v41.json.zlib"),
            true,
        );
    }
    #[test]
    fn native_managed_hook_reads() {
        check_read_diagnostics(
            include_bytes!("fixtures/managed-hook-reads-v41.json.zlib"),
            true,
        );
    }
    #[test]
    fn native_engine_hook_reads() {
        check_read_diagnostics(
            include_bytes!("fixtures/engine-hook-reads-v41.json.zlib"),
            true,
        );
    }
    #[test]
    fn native_player_hook_reads() {
        check_read_diagnostics(
            include_bytes!("fixtures/player-hook-reads-v41.json.zlib"),
            true,
        );
    }
    #[test]
    fn native_movement_hook_reads() {
        check_read_diagnostics(
            include_bytes!("fixtures/movement-hook-reads-v41.json.zlib"),
            true,
        );
    }
    #[test]
    fn native_equipment_hook_reads() {
        check_read_diagnostics(
            include_bytes!("fixtures/equipment-hook-reads-v41.json.zlib"),
            true,
        );
    }
    #[test]
    fn native_default_hook_keyframes() {
        check_read_diagnostics(
            include_bytes!("fixtures/default-hook-keyframes-d61443e-v41.json.zlib"),
            true,
        );
    }
    #[test]
    fn native_probe_hook_reads() {
        check_read_diagnostics(
            include_bytes!("fixtures/probe-hook-reads-v41.json.zlib"),
            true,
        );
    }
    #[test]
    fn native_position_hook_keyframes() {
        check_read_diagnostics(
            include_bytes!("fixtures/position-hook-keyframes-v41.json.zlib"),
            true,
        );
    }
    #[test]
    fn native_body_profile_reads() {
        let fixture = include_bytes!("fixtures/body-profile-reads-v41.json.zlib");
        // These readers do not consume the indexed absolute-position header.
        check_read_diagnostics(fixture, true);
        let mut bytes = Vec::new();
        flate2::read::ZlibDecoder::new(&fixture[..])
            .read_to_end(&mut bytes)
            .unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&bytes).unwrap();
        let mut disabled_anchors = 0;
        let mut active_skips = 0;
        for row in rows {
            let bodies = &row["encoding"]["bodies"];
            for step in row["steps"].as_array().unwrap() {
                for observation in step["observations"].as_array().unwrap() {
                    if observation["kind"] == "ability_non_predicted"
                        && observation["state"]["tag"] == 3
                        && observation["state"]["body_walked"] == false
                    {
                        disabled_anchors += 1;
                    }
                }
                if bodies["mobility"] == false
                    && bodies["mobility_extra_bits"].as_i64().unwrap() > 0
                    && step["actions"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|a| a[0] == true)
                {
                    active_skips += 1;
                }
            }
        }
        assert_eq!(disabled_anchors, 978);
        assert_eq!(active_skips, 312);
    }
    #[test]
    fn native_calibrated_position_keyframes() {
        check_read_diagnostics(
            include_bytes!("fixtures/calibrated-position-keyframes-v41.json.zlib"),
            false,
        );
    }
    #[test]
    fn native_width_override_keyframes() {
        check_read_diagnostics(
            include_bytes!("fixtures/width-override-keyframes-v41.json.zlib"),
            false,
        );
    }
    #[test]
    fn native_baseline_scope_keyframes() {
        check_read_diagnostics(
            include_bytes!("fixtures/baseline-scope-keyframes-v41.json.zlib"),
            false,
        );
    }
    #[test]
    fn native_keyframe_simulation_policy() {
        check_read_diagnostics(
            include_bytes!("fixtures/keyframe-simulation-policy-v41.json.zlib"),
            true,
        );
    }
    #[test]
    fn native_keyframe_layout() {
        check_read_diagnostics(
            include_bytes!("fixtures/keyframe-layout-v41.json.zlib"),
            true,
        );
    }
    fn check_read_diagnostics(fixture: &[u8], mobility: bool) {
        #[derive(Deserialize)]
        struct Step {
            archetype: Option<u32>,
            movement_slot: Option<u32>,
            #[serde(default)]
            suppress_references: bool,
            #[serde(default)]
            level: u32,
            hex: String,
            name: String,
            start: usize,
            end: usize,
            ported: bool,
            hist: BTreeMap<i32, u64>,
            take: bool,
            #[serde(default)]
            actions: Vec<[bool; 2]>,
            observations: Option<Vec<FilmComponentObservation>>,
        }
        #[derive(Deserialize)]
        struct Keyframe {
            desync: Option<i64>,
            observations: Option<Vec<FilmComponentObservation>>,
            #[serde(default)]
            actions: Vec<[bool; 2]>,
            hex: String,
            hist: BTreeMap<i32, u64>,
            end: usize,
        }
        #[derive(Deserialize)]
        struct Case {
            #[serde(default)]
            keyframe_layout: crate::theater::KeyframeLayout,
            #[serde(default)]
            corruption_check: bool,
            keyframe_simulation_complete: Option<bool>,
            #[serde(default)]
            component_widths: crate::theater::ComponentWidthOverrides,
            capture: Option<crate::theater::PositionCaptureEncoding>,
            archetype: Option<usize>,
            mpp: Option<[usize; 2]>,
            #[serde(default)]
            native: bool,
            levels: Option<Vec<u32>>,
            names: Option<Vec<String>>,
            keyframe: Keyframe,
            encoding: PositionEncoding,
            steps: Vec<Step>,
        }
        let mut bytes = Vec::new();
        flate2::read::ZlibDecoder::new(fixture)
            .read_to_end(&mut bytes)
            .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&bytes).unwrap();
        let mut count = 0;
        for (i, c) in cases.into_iter().enumerate() {
            use crate::theater::{
                FilmArchetype, FilmRegistry, KeyframeRecord, decode_keyframe_record,
            };
            let archetype = c.archetype.unwrap_or(0);
            let mut archetypes: Vec<_> = (0..=archetype)
                .map(|index| FilmArchetype {
                    index,
                    components: Vec::new(),
                    levels: Vec::new(),
                })
                .collect();
            let registry = FilmRegistry {
                major_version: 41,
                format_version: 27,
                end_byte: 0,
                truncated: false,
                archetypes: {
                    archetypes[archetype] = FilmArchetype {
                        index: archetype,
                        components: c.names.unwrap_or_else(|| {
                            vec![
                                if mobility {
                                    "biped-mobility-action-component"
                                } else {
                                    "object-position-dynamic-precision-component"
                                }
                                .into(),
                                if mobility {
                                    "biped-mobility-action"
                                } else {
                                    "simulation-state-component"
                                }
                                .into(),
                            ]
                        }),
                        levels: c.levels.unwrap_or_else(|| vec![0, 0]),
                    };
                    archetypes
                },
            };
            let data = unhex(&c.keyframe.hex);
            let record = if c.native {
                crate::theater::components::decode_native_keyframe_record(
                    &data,
                    0,
                    &registry,
                    &crate::theater::FrameEncoding {
                        keyframe_layout: c.keyframe_layout,
                        keyframe_simulation_complete: c.keyframe_simulation_complete,
                        native_id_low_bits: None,
                        component_widths: c.component_widths.clone(),
                        new_record: Default::default(),
                        position_capture: c.capture.clone(),
                        mpp_widths: c.mpp.unwrap_or([9, 5]),
                        position: Some(c.encoding.clone()),
                        ids: crate::theater::RecordIdLayout {
                            low_bits: 11,
                            base: 0,
                        },
                        extra_fields: false,
                        corruption_check: c.corruption_check,
                    },
                )
            } else {
                decode_keyframe_record(
                    &data,
                    0,
                    &registry,
                    c.mpp.unwrap_or([9, 5]),
                    Some(&c.encoding),
                    false,
                )
            }
            .unwrap();
            if let Some(desync) = c.keyframe.desync {
                let actual = match record.stop {
                    crate::theater::KeyframeStop::Complete => -1,
                    crate::theater::KeyframeStop::UnsupportedComponent { index, .. } => {
                        index as i64
                    }
                    _ => -2,
                };
                assert_eq!(actual, desync, "keyframe desync {i}");
                let mut shifted = vec![0u8; data.len() + 1];
                for bit in 0..data.len() * 8 {
                    if data[crate::theater::bits::native_address(bit / 8)] & (1 << (7 - bit % 8))
                        != 0
                    {
                        shifted[(bit + 1) / 8] |= 1 << (7 - (bit + 1) % 8);
                    }
                }
                let encoding = crate::theater::FrameEncoding {
                    keyframe_layout: c.keyframe_layout,
                    keyframe_simulation_complete: c.keyframe_simulation_complete,
                    native_id_low_bits: None,
                    component_widths: c.component_widths.clone(),
                    new_record: Default::default(),
                    position_capture: c.capture.clone(),
                    ids: crate::theater::RecordIdLayout {
                        low_bits: 11,
                        base: 0,
                    },
                    mpp_widths: c.mpp.unwrap_or([9, 5]),
                    position: Some(c.encoding.clone()),
                    extra_fields: false,
                    corruption_check: c.corruption_check,
                };
                let mut bindings = crate::theater::EntityBindings::default();
                let table = crate::theater::decode_keyframe_table(
                    &shifted,
                    &registry,
                    &encoding,
                    &mut bindings,
                );
                assert_eq!(table.records[0].stop, record.stop, "table stop {i}");
                assert_eq!(
                    table.records[0].end_bit,
                    record.end_bit + 1,
                    "table boundary {i}"
                );
                assert_eq!(
                    bindings.slots.contains_key(&(record.id & 0x3fff_ffff)),
                    desync == -1,
                    "table binding {i}"
                );
            }
            assert_eq!(record.diagnostics.mobility_actions, c.keyframe.actions);
            if let Some(expected) = c.keyframe.observations {
                assert_eq!(
                    record.diagnostics.component_observations, expected,
                    "keyframe observations {i}"
                );
            }
            assert_eq!(
                serde_json::json!(record.end_bit),
                serde_json::json!(c.keyframe.end),
                "keyframe end {i}"
            );
            assert_eq!(
                record.diagnostics.absolute_indices, c.keyframe.hist,
                "keyframe hist {i}"
            );
            if c.capture.is_some() && record.end_bit <= (data.len() * 8) as i64 {
                let bounded = crate::theater::decode_keyframe_record_with_encoding(
                    &data,
                    0,
                    &registry,
                    &crate::theater::FrameEncoding {
                        keyframe_layout: c.keyframe_layout,
                        keyframe_simulation_complete: c.keyframe_simulation_complete,
                        native_id_low_bits: None,
                        component_widths: c.component_widths.clone(),
                        new_record: Default::default(),
                        position_capture: c.capture.clone(),
                        mpp_widths: c.mpp.unwrap_or([9, 5]),
                        position: Some(c.encoding.clone()),
                        ids: crate::theater::RecordIdLayout {
                            low_bits: 11,
                            base: 0,
                        },
                        extra_fields: false,
                        corruption_check: c.corruption_check,
                    },
                )
                .unwrap();
                assert_eq!(bounded, record, "bounded/native captured keyframe {i}");
            }
            let restored: KeyframeRecord =
                serde_json::from_slice(&serde_json::to_vec(&record).unwrap()).unwrap();
            assert_eq!(restored, record);
            let mut diagnostics = FilmReadDiagnostics::default();
            let mut expected_actions = Vec::new();
            for (j, s) in c.steps.into_iter().enumerate() {
                let data = unhex(&s.hex);
                let (status, component) = crate::theater::decode_native_component_with_capture(
                    &data,
                    s.start,
                    &s.name,
                    s.level,
                    s.archetype.unwrap_or(35),
                    Some(&c.encoding),
                    crate::theater::NativeComponentCapture {
                        position: None,
                        movement_slot: s.movement_slot,
                        unit_references: !s.suppress_references,
                    },
                );
                if s.movement_slot == Some(0) && !s.suppress_references {
                    let direct = crate::theater::decode_native_component(
                        &data,
                        s.start,
                        &s.name,
                        s.level,
                        s.archetype.unwrap_or(35),
                        Some(&c.encoding),
                    );
                    assert_eq!(direct, (status, component.clone()), "fresh reader {i}/{j}");
                }
                assert_eq!(status == Some(true), s.ported, "status {i}/{j}");
                assert_eq!(
                    component.diagnostics.mobility_actions, s.actions,
                    "actions {i}/{j}"
                );
                assert_eq!(
                    serde_json::json!(component.end_bit),
                    serde_json::json!(s.end),
                    "end {i}/{j}"
                );
                if let Some(expected) = s.observations {
                    assert_eq!(
                        component.diagnostics.component_observations, expected,
                        "observations {i}/{j}"
                    );
                }
                diagnostics.merge(&component.diagnostics);
                expected_actions.extend_from_slice(&s.actions);
                assert_eq!(diagnostics.mobility_actions, expected_actions);
                count += component.diagnostics.absolute_indices.values().sum::<u64>();
                assert_eq!(diagnostics.absolute_indices, s.hist, "hist {i}/{j}");
                let restored: crate::theater::DecodedComponent =
                    serde_json::from_slice(&serde_json::to_vec(&component).unwrap()).unwrap();
                assert_eq!(restored, component);
                if s.take {
                    assert_eq!(diagnostics.take_absolute_indices(), s.hist);
                    assert!(diagnostics.absolute_indices.is_empty());
                    assert_eq!(
                        diagnostics.is_empty(),
                        expected_actions.is_empty()
                            && diagnostics.component_observations.is_empty()
                    );
                }
            }
        }
        if !mobility {
            assert!(count > 100);
        }
        eprintln!("matched {count} absolute-index increments");
    }
    #[test]
    fn native_chain_diagnostics() {
        check_chain_diagnostics(
            include_bytes!("fixtures/chain-diagnostics-v41.json.zlib"),
            false,
        );
    }
    #[test]
    fn native_mobility_chain() {
        check_chain_diagnostics(
            include_bytes!("fixtures/mobility-chain-v41.json.zlib"),
            true,
        );
    }
    #[test]
    fn native_movement_hook_chain() {
        check_chain_diagnostics(
            include_bytes!("fixtures/movement-hook-chain-v41.json.zlib"),
            true,
        );
    }
    #[test]
    fn native_position_hook_chain() {
        check_chain_diagnostics(
            include_bytes!("fixtures/position-hook-chain-v41.json.zlib"),
            true,
        );
    }
    #[test]
    fn native_component_hook_chain() {
        check_chain_diagnostics(
            include_bytes!("fixtures/component-hook-chain-v41.json.zlib"),
            true,
        );
    }
    #[test]
    fn native_ability_hook_chain() {
        check_chain_diagnostics(
            include_bytes!("fixtures/ability-hook-chain-v41.json.zlib"),
            true,
        );
    }
    #[test]
    fn native_unit_reference_hook_chain() {
        check_chain_diagnostics(
            include_bytes!("fixtures/unit-reference-hook-chain-v41.json.zlib"),
            true,
        );
    }
    #[test]
    fn native_object_hook_chain() {
        check_chain_diagnostics(
            include_bytes!("fixtures/object-hook-chain-v41.json.zlib"),
            true,
        );
    }
    #[test]
    fn native_managed_hook_chain() {
        check_chain_diagnostics(
            include_bytes!("fixtures/managed-hook-chain-v41.json.zlib"),
            true,
        );
    }
    #[test]
    fn native_engine_hook_chain() {
        check_chain_diagnostics(
            include_bytes!("fixtures/engine-hook-chain-v41.json.zlib"),
            true,
        );
    }
    #[test]
    fn native_player_hook_chain() {
        check_chain_diagnostics(
            include_bytes!("fixtures/player-hook-chain-v41.json.zlib"),
            true,
        );
    }
    #[test]
    fn native_equipment_hook_chain() {
        check_chain_diagnostics(
            include_bytes!("fixtures/equipment-hook-chain-v41.json.zlib"),
            true,
        );
    }
    #[test]
    fn native_probe_hook_chain() {
        check_chain_diagnostics(
            include_bytes!("fixtures/probe-hook-chain-v41.json.zlib"),
            true,
        );
    }
    fn check_chain_diagnostics(fixture: &[u8], mobility: bool) {
        use crate::theater::*;
        #[derive(Deserialize)]
        struct Case {
            capture: Option<crate::theater::PositionCaptureEncoding>,
            hex: String,
            names: Vec<Vec<String>>,
            encoding: PositionEncoding,
            hist: BTreeMap<i32, u64>,
            ti: u32,
            end: usize,
            unique: bool,
            ok: bool,
            outcome: ChainInferenceOutcome,
            observations: Option<Vec<FilmComponentObservation>>,
            #[serde(default)]
            actions: Vec<[bool; 2]>,
        }
        let mut bytes = Vec::new();
        flate2::read::ZlibDecoder::new(fixture)
            .read_to_end(&mut bytes)
            .unwrap();
        let cases: Vec<Case> = serde_json::from_slice(&bytes).unwrap();
        let mut counts = 0;
        for (i, c) in cases.into_iter().enumerate() {
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
                position_capture: c.capture,
                ids: RecordIdLayout {
                    low_bits: 11,
                    base: 0,
                },
                mpp_widths: [9, 5],
                position: Some(c.encoding),
                extra_fields: false,
                corruption_check: false,
            };
            let mut world = FilmWorld::default();
            world.bind_full(50, 0);
            world.bind_soft(51, 1);
            let got = infer_chain_archetype(&unhex(&c.hex), 0, &registry, &encoding, &world, true);
            assert_eq!(got.archetype, c.ok.then_some(c.ti), "type {i}");
            assert_eq!(got.end_bit, c.end as i64, "end {i}");
            assert_eq!(got.unique_archetype, c.unique, "unique {i}");
            assert_eq!(got.outcome, c.outcome, "outcome {i}");
            assert_eq!(got.diagnostics.absolute_indices, c.hist, "hist {i}");
            assert_eq!(got.diagnostics.mobility_actions, c.actions, "actions {i}");
            counts += if let Some(expected) = c.observations {
                assert_eq!(
                    got.diagnostics.component_observations, expected,
                    "observations {i}"
                );
                expected.len() as u64
            } else if mobility {
                c.actions.len() as u64
            } else {
                c.hist.values().sum::<u64>()
            };
            let restored: ChainInference =
                serde_json::from_slice(&serde_json::to_vec(&got).unwrap()).unwrap();
            assert_eq!(restored, got);
        }
        assert!(counts > 100);
        eprintln!("matched {counts} speculative observations (mobility={mobility})");
    }
}
