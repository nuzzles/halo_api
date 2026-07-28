pub mod clearance;
pub mod client;
pub mod error;
pub mod spartan;

pub(crate) mod endpoints;

pub use clearance::{ClearanceTokenSource, WaypointClearanceProvider};
pub use client::{AuthClient, HaloAuth, HaloCredentials};
pub use error::AuthError;
pub use spartan::{SpartanTokenSource, XboxSpartanTokenProvider};
