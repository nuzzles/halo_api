use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::time::Duration;

use reqwest::Url;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use xbox::auth::OAuthClient;

use super::ExampleError;

/// Create a separate local token only on the first OAuth run. Nothing here accesses S3.
pub async fn ensure_local_token() -> Result<(), ExampleError> {
    let file = env::var("XBOX_OAUTH_REFRESH_TOKEN_FILE")?;
    if file.starts_with("s3://") || file.starts_with("https://") || file.starts_with("http://") {
        return Err("XBOX_OAUTH_REFRESH_TOKEN_FILE must be a local filesystem path".into());
    }
    let path = Path::new(&file);
    if path.exists() {
        if !path.is_file() || path.metadata()?.len() == 0 {
            return Err("OAuth refresh token path must be a nonempty file".into());
        }
        return Ok(());
    }

    let redirect = env::var("XBOX_OAUTH_REDIRECT_URI")
        .map_err(|_| "Set XBOX_OAUTH_REDIRECT_URI to a registered loopback redirect URI for first-time OAuth sign-in")?;
    let uri = Url::parse(&redirect)?;
    if uri.scheme() != "http"
        || !matches!(uri.host_str(), Some("localhost" | "127.0.0.1"))
        || uri.port().is_none()
        || uri.query().is_some()
        || uri.fragment().is_some()
    {
        return Err("XBOX_OAUTH_REDIRECT_URI must be an http://localhost:PORT/path or http://127.0.0.1:PORT/path URI".into());
    }
    let port = uri.port().expect("validated port");
    let listener = TcpListener::bind(("127.0.0.1", port)).await?;
    let state = uuid::Uuid::new_v4().to_string();
    let client = OAuthClient::new(
        env::var("XBOX_OAUTH_CLIENT_ID")?,
        env::var("XBOX_OAUTH_CLIENT_SECRET")?,
    );
    let authorize = client.authorization_url(&redirect, &state)?;

    println!(
        "Complete Microsoft sign-in in your browser. If it does not open, visit:\n{authorize}"
    );
    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open")
        .arg(authorize.as_str())
        .status();

    let code = tokio::time::timeout(Duration::from_secs(300), async {
        loop {
            let (mut stream, peer) = listener.accept().await?;
            if !peer.ip().is_loopback() {
                continue;
            }
            let mut buffer = [0_u8; 8192];
            let count = stream.read(&mut buffer).await?;
            let request = std::str::from_utf8(&buffer[..count]).unwrap_or("");
            let target = request
                .lines()
                .next()
                .and_then(|line| line.strip_prefix("GET "))
                .and_then(|line| line.split_once(' '))
                .map(|(target, _)| target);
            let Some(target) = target else { continue };
            let callback = Url::parse(&format!("http://localhost:{port}{target}"))?;
            if callback.path() != uri.path() {
                continue;
            }
            let params = callback.query_pairs();
            let returned_state = params.clone().find(|(key, _)| key == "state").map(|(_, value)| value.into_owned());
            let code = params.clone().find(|(key, _)| key == "code").map(|(_, value)| value.into_owned());
            let valid = returned_state.as_deref() == Some(state.as_str()) && code.is_some();
            let body = if valid {
                "Sign-in received. You may close this tab."
            } else {
                "Sign-in failed or state did not match. Return to the terminal."
            };
            let reply = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len());
            stream.write_all(reply.as_bytes()).await?;
            if !valid {
                return Err::<String, ExampleError>("OAuth callback was rejected".into());
            }
            return Ok(code.expect("validated code"));
        }
    })
    .await
    .map_err(|_| "Timed out waiting for Microsoft sign-in")??;

    let tokens = client.exchange_code(&redirect, &code).await?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path)?;
    file.write_all(tokens.refresh_token.as_bytes())?;
    file.sync_all()?;
    println!("Local OAuth refresh token saved.");
    Ok(())
}
