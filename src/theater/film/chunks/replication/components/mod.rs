//! Reference component models.

pub mod unit_equipment;
pub use unit_equipment::{UnitEquipmentEntry, UnitEquipmentRead};

pub mod unit_references;
pub use unit_references::{UnitReference, UnitReferenceKind};

pub mod control;
pub use control::ActionBlock;

pub mod position;
pub use position::PositionKind;

pub mod field;
pub use field::{ComponentField, RawBits};
