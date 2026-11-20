pub(crate) mod cache;
mod clearance;
mod client;
mod error;
mod spartan;
#[cfg(feature = "xbox")]
mod xbox;

pub(crate) mod endpoints;

pub(crate) use clearance::{ClearanceTokenSource, WaypointClearanceProvider};
pub use client::HaloAuthClient;
pub(crate) use client::HaloCredentials;
pub use error::AuthError;
pub use spartan::XstsTicketSource;
pub(crate) use spartan::{SpartanTokenSource, WaypointSpartanTokenProvider};
#[cfg(feature = "xbox")]
pub use xbox::{HALO_RELYING_PARTY, XboxXstsTicketSource};
