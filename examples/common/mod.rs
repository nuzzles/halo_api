#![allow(dead_code)]

mod oauth_setup;

use std::env;
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

use halo_api::auth::HaloAuthClient;
use halo_api::clients::hi::{HaloInfiniteClient, Player};
use xbox::{
    RelyingParty, XboxClient, XboxError,
    auth::{
        LegacyPasswordProvider, OAuthClient, OAuthRefreshTokenProvider, RefreshTokenStore,
        XblAuthProvider,
    },
    cache::CachedToken,
    models::Xuid,
};

pub type ExampleError = Box<dyn std::error::Error>;
pub type ExampleXboxClient = XboxClient<ExampleProvider>;

pub enum ExampleProvider {
    Legacy(LegacyPasswordProvider),
    OAuth(OAuthRefreshTokenProvider<FileTokenStore>),
}

#[async_trait::async_trait]
impl XblAuthProvider for ExampleProvider {
    async fn user_token(&self) -> Result<CachedToken<String>, XboxError> {
        match self {
            Self::Legacy(provider) => provider.user_token().await,
            Self::OAuth(provider) => provider.user_token().await,
        }
    }
}

pub struct FileTokenStore {
    path: PathBuf,
    lock: Mutex<()>,
}

#[async_trait::async_trait]
impl RefreshTokenStore for FileTokenStore {
    async fn load(&self) -> Result<String, XboxError> {
        std::fs::read_to_string(&self.path)
            .map(|token| token.trim().to_owned())
            .map_err(|_| XboxError::OAuthTokenStore("read refresh token file"))
    }

    async fn compare_and_swap(&self, expected: &str, replacement: &str) -> Result<bool, XboxError> {
        let _guard = self
            .lock
            .lock()
            .map_err(|_| XboxError::OAuthTokenStore("lock refresh token file"))?;
        let current = std::fs::read_to_string(&self.path)
            .map_err(|_| XboxError::OAuthTokenStore("read refresh token file"))?;
        if current.trim() != expected {
            return Ok(false);
        }
        std::fs::write(&self.path, replacement)
            .map_err(|_| XboxError::OAuthTokenStore("write refresh token file"))?;
        Ok(true)
    }
}

fn prompt_line(label: &str) -> Result<String, ExampleError> {
    print!("{label}: ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

pub fn value(name: &str, label: &str) -> Result<String, ExampleError> {
    match env::var(name) {
        Ok(value) => Ok(value),
        Err(_) => prompt_line(label),
    }
}

pub fn comma_separated(name: &str, label: &str) -> Result<Vec<String>, ExampleError> {
    let values = value(name, label)?
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if values.is_empty() {
        return Err(format!("{name} must contain at least one value").into());
    }
    Ok(values)
}

pub async fn player_xuid(halo: &HaloInfiniteClient) -> Result<(String, Player), ExampleError> {
    let gamertag = value("HALO_GAMERTAG", "Gamertag")?;
    let user = halo.user(&Player::gamertag(gamertag)).await?;
    Ok((user.gamertag, Player::xuid(user.xuid)))
}

#[derive(Debug, PartialEq, Eq)]
enum AuthMethod {
    OAuth,
    Legacy,
    Choose,
}

fn configured(name: &str) -> bool {
    env::var(name).is_ok_and(|value| !value.trim().is_empty())
}

fn auth_method(
    explicit: Option<&str>,
    oauth_ready: bool,
    legacy_ready: bool,
) -> Result<AuthMethod, ExampleError> {
    match explicit {
        Some("oauth") => Ok(AuthMethod::OAuth),
        Some("legacy") => Ok(AuthMethod::Legacy),
        Some(other) => {
            Err(format!("unknown XBOX_AUTH_METHOD: {other}; use legacy or oauth").into())
        }
        None if oauth_ready => Ok(AuthMethod::OAuth),
        None if legacy_ready => Ok(AuthMethod::Legacy),
        None => Ok(AuthMethod::Choose),
    }
}

fn selected_auth_method() -> Result<AuthMethod, ExampleError> {
    let oauth_ready = [
        "XBOX_OAUTH_CLIENT_ID",
        "XBOX_OAUTH_CLIENT_SECRET",
        "XBOX_OAUTH_REFRESH_TOKEN_FILE",
    ]
    .iter()
    .all(|name| configured(name));
    let legacy_ready = ["XBOX_USERNAME", "XBOX_PASSWORD"]
        .iter()
        .all(|name| configured(name));
    let explicit = env::var("XBOX_AUTH_METHOD").ok();
    let method = auth_method(explicit.as_deref(), oauth_ready, legacy_ready)?;
    if method != AuthMethod::Choose {
        return Ok(method);
    }
    match prompt_line("Authentication method ([1] OAuth, [2] legacy)")?
        .to_lowercase()
        .as_str()
    {
        "1" | "oauth" => Ok(AuthMethod::OAuth),
        "2" | "legacy" => Ok(AuthMethod::Legacy),
        _ => Err("choose 1 (OAuth) or 2 (legacy)".into()),
    }
}

pub async fn ensure_player_customizations_auth() -> Result<(), ExampleError> {
    if selected_auth_method()? == AuthMethod::OAuth {
        oauth_setup::ensure_local_token().await?;
    }
    Ok(())
}

pub fn xbox_client() -> Result<Arc<ExampleXboxClient>, ExampleError> {
    let provider = match selected_auth_method()? {
        AuthMethod::OAuth => {
            let client_id = value("XBOX_OAUTH_CLIENT_ID", "OAuth client ID")?;
            let client_secret = value("XBOX_OAUTH_CLIENT_SECRET", "OAuth client secret")?;
            let path = PathBuf::from(value(
                "XBOX_OAUTH_REFRESH_TOKEN_FILE",
                "Refresh token file",
            )?);
            ExampleProvider::OAuth(OAuthRefreshTokenProvider::new(
                OAuthClient::new(client_id, client_secret),
                FileTokenStore {
                    path,
                    lock: Mutex::new(()),
                },
            ))
        }
        AuthMethod::Legacy => {
            let username = value("XBOX_USERNAME", "Xbox Live email")?;
            let password = match env::var("XBOX_PASSWORD") {
                Ok(password) => password,
                Err(_) => rpassword::prompt_password("Xbox Live password: ")?,
            };
            ExampleProvider::Legacy(LegacyPasswordProvider::new(username, password))
        }
        AuthMethod::Choose => unreachable!(),
    };
    Ok(Arc::new(XboxClient::new(provider)))
}

#[cfg(test)]
mod tests {
    use super::{AuthMethod, auth_method};

    #[test]
    fn auth_selection_prefers_oauth_then_legacy_then_prompt() {
        assert_eq!(auth_method(None, true, true).unwrap(), AuthMethod::OAuth);
        assert_eq!(auth_method(None, false, true).unwrap(), AuthMethod::Legacy);
        assert_eq!(auth_method(None, false, false).unwrap(), AuthMethod::Choose);
        assert_eq!(
            auth_method(Some("legacy"), true, true).unwrap(),
            AuthMethod::Legacy
        );
    }
}

pub fn halo_infinite_client() -> Result<(Arc<ExampleXboxClient>, HaloInfiniteClient), ExampleError>
{
    let xbox = xbox_client()?;
    let auth = HaloAuthClient::from_xbox_xsts_source(xbox.clone());
    Ok((xbox, HaloInfiniteClient::new(auth)))
}

pub async fn logged_in_player(xbox: &ExampleXboxClient) -> Result<(String, Xuid), ExampleError> {
    let ticket = xbox.xsts_ticket(RelyingParty::XBOX).await?;
    let gamertag = ticket.gamertag().unwrap_or("<unknown>").to_string();
    let xuid = ticket.xuid().ok_or("Xbox ticket did not contain an XUID")?;
    Ok((gamertag, xuid))
}
