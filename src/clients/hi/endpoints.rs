//! Halo Waypoint API origin constants. Every endpoint lives on `<origin>.svc.halowaypoint.com`.
//!
//! Only origins used by a currently implemented endpoint are listed here.

const SERVICE_DOMAIN: &str = "svc.halowaypoint.com";

const SKILL_ORIGIN: &str = "skill";
const HALOSTATS_ORIGIN: &str = "halostats";
const PROFILE_ORIGIN: &str = "profile";
const GAME_CMS_ORIGIN: &str = "gamecms-hacs";
const UGC_ORIGIN: &str = "discovery-infiniteugc";
const BAN_ORIGIN: &str = "banprocessor";
const ECONOMY_ORIGIN: &str = "economy";

/// Builds a full `https://<origin>.svc.halowaypoint.com` base URL for the given origin.
fn base_url(origin: &str) -> String {
    format!("https://{origin}.{SERVICE_DOMAIN}")
}

/// Overridable base URLs for the Halo Waypoint endpoints this crate calls.
///
/// Exists primarily so tests (and callers proxying/mocking Halo Waypoint) can point this crate
/// at something other than the real service. [`Default`] points at the real endpoints.
#[derive(Debug, Clone)]
pub(crate) struct HaloEndpoints {
    pub skill_base_url: String,
    pub halostats_base_url: String,
    pub current_user_url: String,
    pub profile_base_url: String,
    pub game_cms_base_url: String,
    pub ugc_base_url: String,
    pub settings_base_url: String,
    pub ban_base_url: String,
    pub economy_base_url: String,
}

impl Default for HaloEndpoints {
    fn default() -> Self {
        Self {
            skill_base_url: base_url(SKILL_ORIGIN),
            halostats_base_url: base_url(HALOSTATS_ORIGIN),
            current_user_url: "https://comms.svc.halowaypoint.com/users/me".to_string(),
            profile_base_url: base_url(PROFILE_ORIGIN),
            game_cms_base_url: base_url(GAME_CMS_ORIGIN),
            ugc_base_url: base_url(UGC_ORIGIN),
            settings_base_url: "https://settings.svc.halowaypoint.com".to_string(),
            ban_base_url: base_url(BAN_ORIGIN),
            economy_base_url: base_url(ECONOMY_ORIGIN),
        }
    }
}
