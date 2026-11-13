//! Known game reference data used to interpret film identifiers.
//!
//! These tables are external to the recording. An absent entry stays unknown,
//! and catalogue metadata never supplies a missing wire field or selects a
//! parser layout without separately established build/schema context.

pub mod damage;
pub mod items;
pub mod maps;
pub mod medals;
pub mod options;
pub mod physics;
pub mod weapons;

/// The bundled reference tables. Lookup does not establish a film's game build.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Catalog;
