//! Reference data models.
/// Reference movement-state callback identifiers; velocity uses the reference hook label.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MovementComponent {
    Crouch,
    Slide,
    Posture,
    UnitControl,
    Velocity,
    Mobility,
    ActiveAbility,
}

/// Reference default-state callback fields; labels match the reference reader.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MppField {
    Word9,
    Word32,
    VariantName,
    TailName,
}

/// Reference default-state callback fields; labels match the reference reader.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipmentCreationField {
    Reference,
    AbilityId,
}

/// Reference equipment-state callback fields, identified by registry name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipmentField {
    Deployed,
    Activated,
    Creator,
    Energy,
    EnergyDelay,
    Charges,
}

/// Reference probe fields. Values remain raw; their gameplay meaning is not inferred.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeComponent {
    SplashStatic,
    SplashDynamic,
    HighFrequency,
    ManagedObjectPropertyName,
}

/// Reference player-state callback fields, identified by registry name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerStateField {
    SoftKill,
    TargetTracking,
    DesiredRespawnPlayer,
    Loadout,
    DesiredRespawnLocation,
    Lives,
    LastBetrayer,
    ControlAiming,
    ActiveInGame,
    PendingJoinInProgress,
    MalleableProperties,
}

/// Reference game-engine callback fields, identified by registry name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameEngineField {
    State,
    Round,
    SuddenDeath,
    GracePeriod,
    RoundConditions,
}

/// Stable reference ManagedObject hook fields, serialized using registry names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManagedObjectField {
    BoundaryVisibility,
    BoundaryColor,
    Rtpc,
}

/// Stable reference Navpoint hook fields, serialized using registry names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavpointField {
    RadialProgress,
    ManualTimerInitial,
    ManualTimerCurrent,
}

/// Stable reference Objective hook fields, serialized using registry names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectiveField {
    Timers,
    ObjectReference,
    Type,
    Progress,
    RequiredProgress,
    State,
}

/// Stable reference ManagedProperty hook fields, serialized using registry names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManagedPropertyField {
    Scalar,
    PerPlayer,
}

/// Positional object-parent fields from the reference observer; no parent identity
/// is inferred from the quantized word or free-reference index.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
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
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CamoState {
    pub state: u8,
    pub flag0: bool,
    pub flag1: Option<bool>,
    pub fraction: Option<u16>,
    pub sub: [Option<u16>; 6],
}

/// Source-read non-predicted ability publication, including unsupported prefixes.
/// Position ranges preserve their recorded form; they are not world coordinates.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AbilityNonPredictedState {
    pub tag: u32,
    pub body_walked: bool,
    pub body_ok: bool,
    /// None is reference Inner=-1: the body was not read.
    pub inner: Option<u32>,
    pub reference: Option<UnitReference>,
    pub position: Option<crate::theater::parser::v41::packet::BitRange>,
    pub flags: Option<u32>,
    pub source: Option<UnitReference>,
    pub target: Option<UnitReference>,
    pub destination: Option<crate::theater::parser::v41::packet::BitRange>,
    pub value8: Option<u32>,
    /// Direction and magnitude quanta; None is constant or not reached/read.
    pub vectors: [Option<[u32; 2]>; 3],
    pub flag: Option<bool>,
    pub packed: Option<u32>,
    pub tail: Option<u32>,
}

/// Reference component-hook publications. Values describe reads, including padded
/// and speculative attempts; they are not evidence of accepted gameplay events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilmComponentObservation {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitReferenceKind {
    VariableWidth,
    GatedWord32,
    Word32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitEquipmentEntry {
    pub value: u32,
    pub tail: u32,
    /// A closed entry remains in the list, with value and tail zero.
    pub present: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitEquipmentRead {
    pub head: u32,
    pub entries: Vec<UnitEquipmentEntry>,
}
