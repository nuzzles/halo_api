//! Halo Waypoint API origin constants. Every endpoint lives on `<origin>.svc.halowaypoint.com`.
//!
//! Only origins used by a currently-implemented endpoint are listed here. Additional origins
//! this API surface uses (`profile`, `gamecms-hacs`, `discovery-infiniteugc`, `settings`,
//! `banprocessor`, `comms`) should be added alongside the endpoint that needs them, not ahead
//! of time.

pub const SERVICE_DOMAIN: &str = "svc.halowaypoint.com";

pub const SKILL_ORIGIN: &str = "skill";
pub const HALOSTATS_ORIGIN: &str = "halostats";
pub const SETTINGS_ORIGIN: &str = "settings";

/// Builds a full `https://<origin>.svc.halowaypoint.com` base URL for the given origin.
pub fn base_url(origin: &str) -> String {
    format!("https://{origin}.{SERVICE_DOMAIN}")
}

/// Overridable base URLs for the Halo Waypoint endpoints this crate calls.
///
/// Exists primarily so tests (and callers proxying/mocking Halo Waypoint) can point this crate
/// at something other than the real service. [`Default`] points at the real endpoints.
#[derive(Debug, Clone)]
pub struct HaloEndpoints {
    pub skill_base_url: String,
    pub halostats_base_url: String,
    pub spartan_token_url: String,
}

impl Default for HaloEndpoints {
    fn default() -> Self {
        Self {
            skill_base_url: base_url(SKILL_ORIGIN),
            halostats_base_url: base_url(HALOSTATS_ORIGIN),
            spartan_token_url: format!("{}/spartan-token", base_url(SETTINGS_ORIGIN)),
        }
    }
}
