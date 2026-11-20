pub(crate) mod cache;
mod clearance;
mod client;
mod error;
mod spartan;

pub(crate) mod endpoints;

pub(crate) use clearance::{ClearanceTokenSource, WaypointClearanceProvider};
pub use client::HaloAuthClient;
pub(crate) use client::HaloCredentials;
pub use error::AuthError;
pub use spartan::XstsTicketSource;
pub(crate) use spartan::{SpartanTokenSource, WaypointSpartanTokenProvider};
