//! Structural registry and bootstrap reads used by the v41 parser.

pub(crate) mod bond;
pub(crate) use bond::*;
pub(crate) mod bootstrap;
pub(crate) use bootstrap::*;
pub(crate) mod game_options;
pub(crate) use game_options::*;
pub(crate) mod models;
pub(crate) use models::*;
