mod clearance;
mod client;
#[path = "../../../src/auth/error.rs"]
mod error;
mod spartan;

#[path = "../../../src/auth/endpoints.rs"]
pub(crate) mod endpoints;

pub(crate) use clearance::{ClearanceTokenSource, WaypointClearanceProvider};
pub use client::HaloAuthClient;
pub(crate) use client::HaloCredentials;
pub use error::AuthError;
pub(crate) use spartan::{SpartanTokenSource, XboxSpartanTokenProvider};
