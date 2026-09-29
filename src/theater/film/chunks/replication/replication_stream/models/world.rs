//! Reference data models.
use crate::theater::film::*;
/// A decoded NEW contradicted an existing hard parser binding. Its raw record
/// remains retained; only the proposed binding mutation is refused.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NewBindingRefusal {
    pub record_bit: i64,
    pub id: u32,
    pub slot: u32,
    pub existing_archetype: u32,
    pub proposed_archetype: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FilmViewAdmission {
    Allowed,
    Anticipated(AnticipatedDeclaration),
    Unbound,
    OtherView,
}
