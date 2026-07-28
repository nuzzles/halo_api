#![allow(dead_code)]

use std::env;
use std::io::{self, Write};
use std::sync::Arc;

use halo_api::auth::AuthClient;
use halo_api::clients::hi::HaloInfiniteClient;
use xbox::{XboxClient, auth::LegacyPasswordProvider, models::Xuid};

pub type ExampleError = Box<dyn std::error::Error>;
pub type ExampleXboxClient = XboxClient<LegacyPasswordProvider>;

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

pub fn xuid() -> Result<Xuid, ExampleError> {
    Ok(Xuid::from(value("HALO_XUID", "Player XUID")?))
}

pub fn xbox_client() -> Result<Arc<ExampleXboxClient>, ExampleError> {
    let username = value("XBOX_USERNAME", "Xbox Live email")?;
    let password = match env::var("XBOX_PASSWORD") {
        Ok(password) => password,
        Err(_) => rpassword::prompt_password("Xbox Live password: ")?,
    };
    Ok(Arc::new(XboxClient::new(LegacyPasswordProvider::new(
        username, password,
    ))))
}

pub fn halo_infinite_client() -> Result<(Arc<ExampleXboxClient>, HaloInfiniteClient), ExampleError>
{
    let xbox = xbox_client()?;
    let auth = AuthClient::from_xbox_client(xbox.clone());
    Ok((xbox, HaloInfiniteClient::new(auth)))
}
