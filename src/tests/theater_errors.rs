use super::{FailingSpartanTokenSource, UnusedClearanceTokenSource, test_client};
use crate::auth::{AuthError, HaloAuthClient};
use crate::clients::hi::models::{FilmChunk, FilmCustomData, FilmManifest};
use crate::clients::hi::{HaloInfiniteClient, InfiniteClientError, TheaterError};
use std::error::Error;
use std::sync::Arc;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn missing_film_is_typed_and_propagates_to_highlights() {
    let server = MockServer::start().await;
    let (halo, _) = test_client(&server).await;
    Mock::given(method("GET"))
        .and(path("/hi/films/matches/local-match/spectate"))
        .and(header("X-343-Authorization-Spartan", "fake-spartan-token"))
        .and(header("343-Clearance", "fake-clearance"))
        .respond_with(ResponseTemplate::new(404).set_body_json("No film found for that match"))
        .expect(2)
        .mount(&server)
        .await;

    for error in [
        halo.match_film("local-match").await.unwrap_err(),
        halo.match_highlight_events("local-match")
            .await
            .unwrap_err(),
    ] {
        assert!(matches!(
            error,
            InfiniteClientError::Theater(TheaterError::FilmUnavailable { match_id })
                if match_id == "local-match"
        ));
    }
}

#[tokio::test]
async fn film_http_errors_preserve_status_body_and_retries() {
    for status in [401, 403, 429, 500] {
        let server = MockServer::start().await;
        let (halo, _) = test_client(&server).await;
        let attempts = match status {
            401 => 2,
            429 => 4,
            _ => 1,
        };
        Mock::given(method("GET"))
            .and(path("/hi/films/matches/test-match/spectate"))
            .respond_with(
                ResponseTemplate::new(status)
                    .insert_header("Retry-After", "0")
                    .set_body_string("upstream error"),
            )
            .expect(attempts)
            .mount(&server)
            .await;

        let error = halo.match_film("test-match").await.unwrap_err();
        assert!(matches!(
            error,
            InfiniteClientError::HttpStatus { status: actual, body, url }
                if actual.as_u16() == status && body == "upstream error"
                    && url == format!("{}/hi/films/matches/test-match/spectate", server.uri())
        ));
    }
}

#[tokio::test]
async fn film_authentication_errors_remain_distinguishable() {
    let halo = HaloInfiniteClient::new(HaloAuthClient::with_sources(
        Arc::new(FailingSpartanTokenSource),
        Arc::new(UnusedClearanceTokenSource),
    ));
    assert!(matches!(
        halo.match_film("test-match").await.unwrap_err(),
        InfiniteClientError::Auth(AuthError::SpartanTokenProvider(_))
    ));
}

#[tokio::test]
async fn malformed_manifest_is_not_an_unavailable_film() {
    let server = MockServer::start().await;
    let (halo, _) = test_client(&server).await;
    Mock::given(method("GET"))
        .and(path("/hi/films/matches/test-match/spectate"))
        .respond_with(ResponseTemplate::new(200).set_body_string("invalid JSON"))
        .expect(1)
        .mount(&server)
        .await;
    assert!(matches!(
        halo.match_film("test-match").await.unwrap_err(),
        InfiniteClientError::Decode { .. }
    ));
}

#[tokio::test]
async fn unrelated_404_is_not_a_theater_error() {
    let server = MockServer::start().await;
    let (halo, _) = test_client(&server).await;
    Mock::given(method("GET"))
        .and(path("/hi/matches/test-match/stats"))
        .respond_with(ResponseTemplate::new(404))
        .expect(1)
        .mount(&server)
        .await;
    assert!(matches!(
        halo.match_stats("test-match").await.unwrap_err(),
        InfiniteClientError::HttpStatus {
            status: reqwest::StatusCode::NOT_FOUND,
            ..
        }
    ));
}

fn film_manifest(server: &MockServer) -> FilmManifest {
    FilmManifest {
        status: 1,
        asset_id: "film-asset".into(),
        blob_storage_path_prefix: server.uri(),
        custom_data: FilmCustomData {
            film_length: 1000,
            has_game_ended: true,
            manifest_refresh_seconds: 60,
            match_id: "test-match".into(),
            film_major_version: 41,
            chunks: vec![FilmChunk {
                index: 0,
                start_time_offset_ms: 0,
                duration_ms: 1000,
                size: 0,
                file_relative_path: "film/chunk.bin".into(),
                chunk_type: 3,
            }],
        },
    }
}

#[tokio::test]
async fn corrupt_chunk_returns_theater_error_with_io_source() {
    let server = MockServer::start().await;
    let (halo, _) = test_client(&server).await;
    Mock::given(method("GET"))
        .and(path("/film/chunk.bin"))
        .respond_with(ResponseTemplate::new(200).set_body_string("invalid zlib"))
        .expect(1)
        .mount(&server)
        .await;
    let error = halo.film_chunks(&film_manifest(&server)).await.unwrap_err();
    assert!(matches!(
        &error,
        InfiniteClientError::Theater(TheaterError::Decompression(_))
    ));
    assert!(error.source().is_some());
}

#[tokio::test]
async fn missing_chunk_preserves_http_error_for_an_existing_film() {
    let server = MockServer::start().await;
    let (halo, _) = test_client(&server).await;
    Mock::given(method("GET"))
        .and(path("/film/chunk.bin"))
        .respond_with(ResponseTemplate::new(404))
        .expect(1)
        .mount(&server)
        .await;
    let error = halo.film_chunks(&film_manifest(&server)).await.unwrap_err();
    assert!(matches!(
        error,
        InfiniteClientError::HttpStatus {
            status: reqwest::StatusCode::NOT_FOUND,
            ..
        }
    ));
}
