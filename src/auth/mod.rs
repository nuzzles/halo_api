mod clearance;
mod client;
mod error;
mod spartan;

pub(crate) mod endpoints;

pub(crate) use clearance::{ClearanceTokenSource, WaypointClearanceProvider};
pub use client::HaloAuthClient;
pub(crate) use client::HaloCredentials;
pub use error::AuthError;
pub(crate) use spartan::{SpartanTokenSource, XboxSpartanTokenProvider};
