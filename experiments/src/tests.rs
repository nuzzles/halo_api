// Reuse the baseline integration tests, then exercise the experimental timeout.
include!("../../src/tests.rs");

#[tokio::test]
async fn applies_custom_timeout_to_each_halo_auth_request() {
    use std::time::Duration as StdDuration;

    let clearance_path = "/clearance/xuid(123456789)/active";
    for delayed_path in ["/spartan-token", "/users/me", clearance_path] {
        let server = MockServer::start().await;
        for (verb, route, body) in [
            ("POST", "/xsts/authorize", xsts_body()),
            (
                "POST",
                "/spartan-token",
                spartan_token_body(Utc::now() + Duration::hours(1)),
            ),
            (
                "GET",
                "/users/me",
                serde_json::json!({ "xuid": "123456789" }),
            ),
            (
                "GET",
                clearance_path,
                serde_json::json!({ "FlightConfigurationId": "fake-clearance" }),
            ),
        ] {
            let mut response = ResponseTemplate::new(200).set_body_json(body);
            if route == delayed_path {
                response = response.set_delay(StdDuration::from_millis(200));
            }
            Mock::given(method(verb))
                .and(path(route))
                .respond_with(response)
                .mount(&server)
                .await;
        }
        let xbox = Arc::new(XboxClient::with_endpoints(
            FakeAuthProvider,
            reqwest::Client::new(),
            XboxEndpoints {
                xsts_authorize_url: format!("{}/xsts/authorize", server.uri()),
                peoplehub_base_url: server.uri(),
            },
        ));
        let endpoints = AuthEndpoints {
            spartan_token_url: format!("{}/spartan-token", server.uri()),
            current_user_url: format!("{}/users/me", server.uri()),
            clearance_url: format!("{}/clearance", server.uri()),
        };
        let short = HaloAuthClient::from_xbox_client_with_endpoints_and_timeout(
            xbox.clone(),
            &endpoints,
            StdDuration::from_millis(50),
        );
        let error = short.credentials(true).await.unwrap_err();
        assert!(
            matches!(&error, AuthError::Network(source)
            if source.is_timeout() && source.url().is_some_and(|url| url.path() == delayed_path)),
            "expected timeout at {delayed_path}, got {error:?}"
        );

        let longer = HaloAuthClient::from_xbox_client_with_endpoints_and_timeout(
            xbox,
            &endpoints,
            StdDuration::from_secs(2),
        );
        let credentials = longer.credentials(true).await.unwrap();
        assert_eq!(credentials.spartan_token, "fake-spartan-token");
        assert_eq!(credentials.clearance.as_deref(), Some("fake-clearance"));
    }
}
