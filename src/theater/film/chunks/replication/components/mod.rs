//! Native component models.

pub mod native_scan_profile;
pub use native_scan_profile::{
    NativeKeyframeLayout, NativeScanGrammar, NativeScanProfile, NativeSharedWidths,
};

pub mod profile;
pub use profile::FilmMapBounds;

pub mod native_profile;
pub use native_profile::{NativeMovementProfile, NativePrecisionDescriptor};

pub mod unit_equipment;
pub use unit_equipment::{UnitEquipmentEntry, UnitEquipmentRead};

pub mod read_diagnostics;
pub use read_diagnostics::{
    FilmComponentObservation, FilmReadDiagnostics, NativeAbilityNonPredictedState, NativeCamoState,
    NativeEquipmentCreationField, NativeEquipmentField, NativeGameEngineField,
    NativeManagedObjectField, NativeManagedPropertyField, NativeMovementComponent, NativeMppField,
    NativeNavpointField, NativeObjectParentState, NativeObjectiveField, NativePlayerStateField,
    NativeProbeComponent, NativeReadOperation, NativeReadRefusal, NativeWidthAdjustment,
    NativeWidthPurpose, NativeWidthRefusal,
};

pub mod profile_values;
pub use profile_values::{FilmMppWidths, FilmQuantizationRange};

pub mod unit_references;
pub use unit_references::{NativeUnitReference, NativeUnitReferenceKind};

pub mod control;
pub use control::NativeActionBlock;

pub mod keyframes;
pub use keyframes::{KeyframeComponentSpan, KeyframeRecord, KeyframeStop};

pub mod views;
pub use views::{DecodedFrameView, FrameViewStop, NativeControlEntry};

pub mod keyframe_chain;
pub use keyframe_chain::{KeyframeChainAttempt, KeyframeChainStop, NativeKeyframeTable};

pub mod frames;
pub use frames::{
    BindingOrigin, EntityComponentAttempt, EntityComponentSpan, EntityRecord, EntityViewStop,
};

pub mod position;
pub use position::NativePositionKind;

pub use read_diagnostics::ChainInferenceOutcome;

pub mod field;
pub use field::ComponentField;
