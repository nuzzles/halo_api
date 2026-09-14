//! Local API variant for Theater experiments, with configurable authentication timeouts.
//! Unchanged clients are shared with the main crate; research code lives here.

pub mod auth;
#[path = "../../src/theater/mod.rs"]
pub mod theater;
// Keep the shared baseline untouched by this experimental package. Its existing
// chunks_exact calls trigger a newer Clippy style lint, not a correctness error.
#[allow(clippy::chunks_exact_to_as_chunks)]
#[path = "../../src/clients/mod.rs"]
pub mod clients;
pub use clients::hi::{HaloInfiniteClient, InfiniteClientError};

/// Research-only raw response capture, preserving fields absent from typed models.
/// Credentials are sent only to Halo's HTTPS service origins.
pub async fn fetch_waypoint_json(
    auth: &auth::HaloAuthClient,
    http: &reqwest::Client,
    url: &str,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let parsed = reqwest::Url::parse(url)?;
    if parsed.scheme() != "https"
        || !parsed
            .host_str()
            .is_some_and(|h| h.ends_with(".svc.halowaypoint.com"))
    {
        return Err("Expected a Halo Waypoint HTTPS service URL".into());
    }
    let credentials = auth.credentials(true).await?;
    let mut request = http
        .get(parsed)
        .header("Accept", "application/json")
        .header("X-343-Authorization-Spartan", &credentials.spartan_token)
        .header(
            "User-Agent",
            "SHIVA-2043073184/6.10021.18539.0 (release; PC)",
        );
    if let Some(clearance) = &credentials.clearance {
        request = request.header("343-Clearance", clearance);
    }
    Ok(request.send().await?.error_for_status()?.json().await?)
}

#[cfg(test)]
mod tests;
