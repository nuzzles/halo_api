//! Reference data models.
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
        position_kind: PositionKind,
        vector_bits: [u32; 3],
        bit: i64,
        slot: u32,
    },
    UnitReference {
        reference: UnitReference,
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
        state: Box<UnitEquipmentRead>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum UnitReferenceKind {
    VariableWidth,
    GatedWord32,
    Word32,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UnitReference {
    pub kind: UnitReferenceKind,
    pub start_bit: i64,
    pub end_bit: i64,
    pub present: bool,
    /// Raw index or full word; domain-specific bases are not guessed.
    pub value: u32,
    pub tail: u32,
    /// Reference category-one flag; remains true even when the presence gate is closed.
    pub probe: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UnitEquipmentEntry {
    pub value: u32,
    pub tail: u32,
    /// A closed entry remains in the list, with value and tail zero.
    pub present: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UnitEquipmentRead {
    pub head: u32,
    pub entries: Vec<UnitEquipmentEntry>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PositionKind {
    /// Reference PosKindRaw: re-emits the saved baseline, never the 96 copied wire bits.
    Baseline,
    Absolute,
    AbsoluteFallback,
    Delta8,
    DeltaAxis,
}
