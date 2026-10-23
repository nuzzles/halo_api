//! Public data published on Halo Waypoint.

use std::time::Duration;

use reqwest::Client;
use scraper::{ElementRef, Html};
use serde::{Deserialize, Serialize};
use thiserror::Error;

const LEADERBOARD_URL: &str = "https://www.halowaypoint.com/en-nz/esports/hcs-leaderboards";
const LEADERBOARD_BLOCK: &str = "lazyblock/hcs-leaderboard";

/// One row of the Waypoint HCS leaderboard, in published order.
/// A gamertag can appear more than once in the source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HcsLeaderboardEntry {
    pub gamertag: String,
    pub points: u32,
}

/// Errors while retrieving or reading the public HCS leaderboard.
#[derive(Debug, Error)]
pub enum HcsError {
    #[error("HCS leaderboard request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("HCS leaderboard page has no Next.js page data")]
    MissingPageData,
    #[error("HCS leaderboard page data is invalid: {0}")]
    InvalidPageData(#[from] serde_json::Error),
    #[error("HCS leaderboard page has no leaderboard block or data")]
    MissingLeaderboard,
    #[error("HCS leaderboard CSV is invalid: {0}")]
    InvalidCsv(#[from] csv::Error),
    #[error("HCS leaderboard row {row} must have five fields and a gamertag")]
    InvalidRow { row: usize },
    #[error("HCS leaderboard row {row} has invalid points: {value}")]
    InvalidPoints { row: usize, value: String },
}

/// Client for public Halo Waypoint data; no Xbox authentication is required.
#[derive(Clone, Default)]
pub struct WaypointClient {
    http: Client,
}

impl WaypointClient {
    pub fn new() -> Self {
        Self::default()
    }

    /// Retrieves the currently published gamertags and points.
    ///
    /// The source is a Waypoint web page, so its structure and contents can change independently
    /// of Halo Infinite's authenticated APIs. Rows are returned in the page's order, including
    /// repeated gamertags.
    pub async fn hcs_leaderboard(&self) -> Result<Vec<HcsLeaderboardEntry>, HcsError> {
        self.fetch(LEADERBOARD_URL).await
    }

    async fn fetch(&self, url: &str) -> Result<Vec<HcsLeaderboardEntry>, HcsError> {
        let html = self
            .http
            .get(url)
            .timeout(Duration::from_secs(15))
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;
        parse_leaderboard(&html)
    }
}

#[derive(Deserialize)]
struct NextData {
    props: Props,
}

#[derive(Deserialize)]
struct Props {
    #[serde(rename = "pageProps")]
    page_props: PageProps,
}

#[derive(Deserialize)]
struct PageProps {
    page: Page,
}

#[derive(Deserialize)]
struct Page {
    blocks: Vec<Block>,
}

#[derive(Deserialize)]
struct Block {
    #[serde(rename = "blockName")]
    name: Option<String>,
    #[serde(default)]
    attrs: serde_json::Value,
}

fn parse_leaderboard(html: &str) -> Result<Vec<HcsLeaderboardEntry>, HcsError> {
    let document = Html::parse_document(html);
    let next_json = document
        .tree
        .nodes()
        .filter_map(ElementRef::wrap)
        .find(|element| {
            element.value().name() == "script" && element.attr("id") == Some("__NEXT_DATA__")
        })
        .and_then(|element| element.text().next())
        .ok_or(HcsError::MissingPageData)?;
    let next: NextData = serde_json::from_str(next_json)?;
    let data = next
        .props
        .page_props
        .page
        .blocks
        .iter()
        .find(|block| block.name.as_deref() == Some(LEADERBOARD_BLOCK))
        .and_then(|block| block.attrs.get("data"))
        .and_then(serde_json::Value::as_str)
        .ok_or(HcsError::MissingLeaderboard)?;

    let mut entries = Vec::new();
    let mut rows = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_reader(data.as_bytes());
    for (index, row) in rows.records().enumerate() {
        let row = row?;
        let row_number = index + 1;
        if row.len() != 5 {
            return Err(HcsError::InvalidRow { row: row_number });
        }
        let gamertag = row
            .get(2)
            .map(str::trim)
            .filter(|gamertag| !gamertag.is_empty())
            .ok_or(HcsError::InvalidRow { row: row_number })?;
        let points_str = row.get(4).ok_or(HcsError::InvalidRow { row: row_number })?;
        let points = points_str
            .trim()
            .parse()
            .map_err(|_| HcsError::InvalidPoints {
                row: row_number,
                value: points_str.to_owned(),
            })?;
        entries.push(HcsLeaderboardEntry {
            gamertag: gamertag.to_owned(),
            points,
        });
    }
    if entries.is_empty() {
        return Err(HcsError::MissingLeaderboard);
    }
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn page_with_rows(data: &str) -> String {
        let next = serde_json::json!({"props": {"pageProps": {"page": {"blocks": [
            {"blockName": "unrelated", "attrs": {"data": {"other": true}}},
            {"blockName": LEADERBOARD_BLOCK, "attrs": {"data": data}}
        ]}}}});
        format!(
            "<html><script type=\"application/json\" id=\"__NEXT_DATA__\">{next}</script></html>"
        )
    }

    #[test]
    fn reads_gamertags_and_points_in_page_order() {
        let html = page_with_rows(
            "GLOBAL,Frosty_BB,Itz so frosty,Shopify Rebellion,8400\nGLOBAL,test,\"Player, One\",Team,42\nGLOBAL,second,Itz so frosty,Team,10",
        );
        assert_eq!(
            parse_leaderboard(&html).unwrap(),
            vec![
                HcsLeaderboardEntry {
                    gamertag: "Itz so frosty".into(),
                    points: 8400,
                },
                HcsLeaderboardEntry {
                    gamertag: "Player, One".into(),
                    points: 42,
                },
                HcsLeaderboardEntry {
                    gamertag: "Itz so frosty".into(),
                    points: 10,
                },
            ]
        );
    }

    #[test]
    fn missing_or_malformed_page_data_is_an_error() {
        assert!(matches!(
            parse_leaderboard("<html></html>"),
            Err(HcsError::MissingPageData)
        ));
        assert!(matches!(
            parse_leaderboard(&page_with_rows("GLOBAL,test,Player,Team,no-points")),
            Err(HcsError::InvalidPoints { row: 1, .. })
        ));
        assert!(matches!(
            parse_leaderboard(&page_with_rows("GLOBAL,test,Player,42")),
            Err(HcsError::InvalidRow { row: 1 })
        ));
        assert!(matches!(
            parse_leaderboard(&page_with_rows("")),
            Err(HcsError::MissingLeaderboard)
        ));
    }

    #[tokio::test]
    async fn fetches_public_page_without_halo_authentication() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/hcs"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(page_with_rows("GLOBAL,test,Player,Team,17")),
            )
            .expect(1)
            .mount(&server)
            .await;
        let entries = WaypointClient::new()
            .fetch(&format!("{}/hcs", server.uri()))
            .await
            .unwrap();
        assert_eq!(entries[0].points, 17);
        let requests = server.received_requests().await.unwrap();
        assert!(
            requests[0]
                .headers
                .get("X-343-Authorization-Spartan")
                .is_none()
        );
    }
}
