//! Private v41 decoding configuration.

mod map;
pub(crate) use map::*;

pub(crate) mod profile;
pub(crate) use profile::MovementProfile;

pub(crate) mod scan_profile;
pub(crate) use scan_profile::{KeyframeReadLayout, ScanGrammar, ScanProfile};

mod values;
pub(crate) use values::*;
