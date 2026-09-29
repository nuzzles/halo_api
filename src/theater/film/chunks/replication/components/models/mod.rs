//! Component observations and structural decoder diagnostics.

pub mod observations;
pub use observations::{
    AbilityNonPredictedState, CamoState, EquipmentCreationField, EquipmentField,
    FilmComponentObservation, GameEngineField, ManagedObjectField, ManagedPropertyField,
    MovementComponent, MppField, NavpointField, ObjectParentState, ObjectiveField,
    PlayerStateField, ProbeComponent,
};

pub mod diagnostics;
pub use diagnostics::{
    FilmReadDiagnostics, ReadOperation, ReadRefusal, WidthAdjustment, WidthPurpose, WidthRefusal,
};
