//! A player identifier accepted by Halo Infinite endpoints.

/// An Xbox user ID stored as a numeric string.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Xuid(pub String);

impl Xuid {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Xuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<String> for Xuid {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for Xuid {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

/// A player identifier: either a gamertag or a numeric XUID.
///
/// Construction is explicit ([`Self::gamertag`], [`Self::xuid`], or `From<Xuid>`) rather than via
/// `From<&str>`/`From<String>`. A raw XUID is itself just a numeric string, so a blanket string
/// conversion would let a caller holding a raw XUID string silently produce [`Player::Gamertag`]
/// instead, defeating the point of this type.
///
/// Endpoints that require a numeric XUID resolve a [`Player::Gamertag`] transparently (one extra
/// lookup); endpoints that accept Halo's `gt(...)`/`xuid(...)` forms interchangeably use whichever
/// variant was given directly, with no extra network call.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Player {
    Gamertag(String),
    Xuid(Xuid),
}

impl Player {
    pub fn gamertag(gamertag: impl Into<String>) -> Self {
        Self::Gamertag(gamertag.into())
    }

    pub fn xuid(xuid: impl Into<Xuid>) -> Self {
        Self::Xuid(xuid.into())
    }
}

impl From<Xuid> for Player {
    fn from(xuid: Xuid) -> Self {
        Self::Xuid(xuid)
    }
}

impl From<&Xuid> for Player {
    fn from(xuid: &Xuid) -> Self {
        Self::Xuid(xuid.clone())
    }
}

impl std::fmt::Display for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Gamertag(gamertag) => write!(f, "{gamertag}"),
            Self::Xuid(xuid) => write!(f, "{xuid}"),
        }
    }
}
