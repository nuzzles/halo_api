//! Reference data models.
use std::collections::BTreeMap;
/// Reference movement-state callback identifiers; velocity uses the reference hook label.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MovementComponent {
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

/// Reference default-state callback fields; labels match the reference reader.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MppField {
    #[serde(rename = "mpp-word9")]
    Word9,
    #[serde(rename = "mpp-word32")]
    Word32,
    #[serde(rename = "mpp-variant-name")]
    VariantName,
    #[serde(rename = "mpp-tail-name")]
    TailName,
}

/// Reference default-state callback fields; labels match the reference reader.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum EquipmentCreationField {
    #[serde(rename = "entity-ref-index5")]
    Reference,
    #[serde(rename = "ability-enabled-id")]
    AbilityId,
}

/// Reference equipment-state callback fields, identified by registry name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum EquipmentField {
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

/// Reference probe fields. Values remain raw; their gameplay meaning is not inferred.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ProbeComponent {
    #[serde(rename = "managed-object-networked-splash-message-static-component")]
    SplashStatic,
    #[serde(rename = "managed-object-networked-splash-message-dynamic-component")]
    SplashDynamic,
    #[serde(rename = "high-frequency")]
    HighFrequency,
    #[serde(rename = "managed-object-property-name-component")]
    ManagedObjectPropertyName,
}

/// Reference player-state callback fields, identified by registry name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PlayerStateField {
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

/// Reference game-engine callback fields, identified by registry name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum GameEngineField {
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

/// Stable reference ManagedObject hook fields, serialized using registry names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ManagedObjectField {
    #[serde(rename = "managed-object-boundary-visibility-component")]
    BoundaryVisibility,
    #[serde(rename = "managed-object-boundary-color-component")]
    BoundaryColor,
    #[serde(rename = "managed-object-rtpc-component")]
    Rtpc,
}

/// Stable reference Navpoint hook fields, serialized using registry names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum NavpointField {
    #[serde(rename = "managed-navpoint-radial-progress")]
    RadialProgress,
    #[serde(rename = "managed-navpoint-manual-timer-initial-duration-component")]
    ManualTimerInitial,
    #[serde(rename = "managed-navpoint-manual-timer-current-duration-component")]
    ManualTimerCurrent,
}

/// Stable reference Objective hook fields, serialized using registry names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ObjectiveField {
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

/// Stable reference ManagedProperty hook fields, serialized using registry names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ManagedPropertyField {
    #[serde(rename = "managed-object-property-component")]
    Scalar,
    #[serde(rename = "managed-object-player-masked-property-component")]
    PerPlayer,
}

/// Positional object-parent fields from the reference observer; no parent identity
/// is inferred from the quantized word or free-reference index.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ObjectParentState {
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
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CamoState {
    pub state: u8,
    pub flag0: bool,
    pub flag1: Option<bool>,
    pub fraction: Option<u16>,
    pub sub: [Option<u16>; 6],
}

/// Reference non-predicted ability publication, including unsupported body prefixes.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AbilityNonPredictedState {
    pub tag: u32,
    pub body_walked: bool,
    pub body_ok: bool,
    /// None is reference Inner=-1: the body was not read.
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

/// Reference component-hook publications. Values describe reads, including padded
/// and speculative attempts; they are not evidence of accepted gameplay events.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FilmComponentObservation {
    /// IEEE-754 bits retain exact reference values, including non-finite values.
    Position {
        position_kind: crate::theater::film::PositionKind,
        vector_bits: [u32; 3],
        bit: i64,
        slot: u32,
    },
    UnitReference {
        reference: crate::theater::film::UnitReference,
    },
    MovementState {
        component: MovementComponent,
        slot: u32,
        values: Vec<u64>,
    },
    Mpp {
        field: MppField,
        value: u64,
        present: bool,
    },
    EquipmentCreation {
        field: EquipmentCreationField,
        value: u64,
        present: bool,
    },
    EquipmentState {
        field: EquipmentField,
        value: u64,
        present: bool,
    },
    Probe {
        archetype: u32,
        component: ProbeComponent,
        values: Vec<u64>,
    },
    PlayerState {
        field: PlayerStateField,
        values: Vec<u64>,
        present: bool,
    },
    GameEngine {
        field: GameEngineField,
        values: Vec<u64>,
        present: bool,
    },
    ManagedObject {
        field: ManagedObjectField,
        values: Vec<u64>,
    },
    Navpoint {
        field: NavpointField,
        values: Vec<u64>,
    },
    Objective {
        field: ObjectiveField,
        values: Vec<u64>,
    },
    ManagedProperty {
        field: ManagedPropertyField,
        values: Vec<u64>,
    },

    HeldWeapon {
        id_high: u32,
        id_low: u32,
    },
    ObjectParent {
        state: Box<ObjectParentState>,
    },
    UnitEquipment {
        state: Box<crate::theater::film::UnitEquipmentRead>,
    },
    CamoState {
        state: Box<CamoState>,
    },
    SpartanAbility {
        tag: u64,
        sub: u64,
        reference: u64,
        has_reference: bool,
    },
    AbilityNonPredicted {
        state: Box<AbilityNonPredictedState>,
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

/// Signed reference skip that rewinds, compacts padding, or cannot fit the source-offset model.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct WidthAdjustment {
    pub component: String,
    pub calibrated: bool,
    /// None denotes the legacy calibrated/stub override. Mobility and New-record
    /// tails are identified separately rather than reported as unported stubs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub purpose: Option<WidthPurpose>,
    pub bit: i64,
    pub width: i64,
    /// For a compact positive skip, source bits retained as scalar fields. The
    /// remaining skipped bits are synthetic zero padding, not recorded data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retained_bits: Option<usize>,
    /// Checked address-sized projection of the reference wrapping target. None
    /// includes negative targets and positive targets outside usize; use
    /// `reference_end_bit` rather than treating this as missing reference data.
    pub end_bit: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum WidthPurpose {
    RecordPrefix,
    MobilityExtra,
    NewRecordTail,
    NewRecordDefault,
}

/// A reference field was reached, but its raw width exceeds this reader's address
/// domain. This is a decoder limitation, not evidence of missing source bytes.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct WidthRefusal {
    pub field: String,
    pub bit: i64,
    pub raw_width: u64,
    pub maximum: u64,
}

/// A bounded operation refused before advancing the cursor. This is not a
/// fabricated zero-valued field and does not include reference padded reads.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ReadRefusal {
    pub field: String,
    pub bit: i64,
    pub width: u64,
    pub source_bits: usize,
    pub operation: ReadOperation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ReadOperation {
    /// The scalar cursor returned None (source bounds or bounded width/domain).
    Scalar,
    /// The reference grouped source guard failed before any member was read.
    GroupGuard,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct FilmReadDiagnostics {
    /// Reference NEW binding refusals, in read order; parsed records are retained.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub new_binding_refusals: Vec<crate::theater::film::NewBindingRefusal>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub read_refusals: Vec<ReadRefusal>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub width_refusals: Vec<WidthRefusal>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub width_adjustments: Vec<WidthAdjustment>,
    pub component_widths: BTreeMap<String, BTreeMap<usize, u64>>,
    pub rejected_unbound: u64,
    pub rejected_other_view: u64,
    pub anticipated_bindings: BTreeMap<u32, u64>,
    /// Reference Observation.IndexAbsolus; -1 denotes the build's default region.
    pub absolute_indices: BTreeMap<i32, u64>,
    /// Ordered reference MobilityActionHook flags, including failed/speculative reads.
    /// Unlike movement-state capture, this historical hook is never suppressed.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub mobility_actions: Vec<[bool; 2]>,
    /// Number of component publications preceding each mobility publication.
    /// None with nonempty mobility_actions means ordering was not retained.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mobility_offsets: Option<Vec<usize>>,
    /// Ordered publications from named component hooks. Position, movement and
    /// reference callbacks obey their independent reference capture policies.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub component_observations: Vec<FilmComponentObservation>,
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

    pub(crate) fn is_empty(&self) -> bool {
        self.new_binding_refusals.is_empty()
            && self.read_refusals.is_empty()
            && self.width_refusals.is_empty()
            && self.width_adjustments.is_empty()
            && self.component_widths.is_empty()
            && self.absolute_indices.is_empty()
            && self.rejected_unbound == 0
            && self.rejected_other_view == 0
            && self.anticipated_bindings.is_empty()
            && self.mobility_actions.is_empty()
            && self.component_observations.is_empty()
    }
    pub(crate) fn merge(&mut self, other: &Self) {
        self.new_binding_refusals
            .extend_from_slice(&other.new_binding_refusals);
        self.read_refusals.extend_from_slice(&other.read_refusals);
        self.width_refusals.extend_from_slice(&other.width_refusals);
        self.width_adjustments
            .extend_from_slice(&other.width_adjustments);
        self.merge_publications(other);
        for (name, widths) in &other.component_widths {
            let target = self.component_widths.entry(name.clone()).or_default();
            for (&width, &count) in widths {
                *target.entry(width).or_default() += count;
            }
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
    pub(crate) fn absolute(&mut self, index: i32) {
        *self.absolute_indices.entry(index).or_default() += 1;
    }
}
