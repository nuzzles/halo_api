/// Overridable Halo authentication endpoints.
#[derive(Debug, Clone)]
pub(crate) struct AuthEndpoints {
    pub spartan_token_url: String,
    pub clearance_url: String,
    pub current_user_url: String,
}

impl Default for AuthEndpoints {
    fn default() -> Self {
        Self {
            spartan_token_url: "https://settings.svc.halowaypoint.com/spartan-token".to_string(),
            clearance_url: "https://settings.svc.halowaypoint.com/oban/flight-configurations/titles/hi/audiences/retail/players".to_string(),
            current_user_url: "https://comms.svc.halowaypoint.com/users/me".to_string(),
        }
    }
}
