/// A known Halo Infinite matchmaking playlist, identified by its asset GUID.
///
/// Only playlists with a confirmed asset GUID are included. `Squad` battle was referenced in
/// the codebase this crate was extracted from, but its GUID was never located — add it once
/// confirmed rather than guessing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlaylistId {
    Arena,
    Doubles,
    Slayer,
}

impl PlaylistId {
    /// The playlist's asset GUID, as used in Halo Waypoint API URLs.
    pub fn as_str(&self) -> &'static str {
        match self {
            PlaylistId::Arena => "edfef3ac-9cbe-4fa2-b949-8f29deafd483",
            PlaylistId::Doubles => "fa5aa2a3-2428-4912-a023-e1eeea7b877c",
            PlaylistId::Slayer => "dcb2e24e-05fb-4390-8076-32a0cdb4326e",
        }
    }
}

impl std::fmt::Display for PlaylistId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arena_guid_matches_known_asset_id() {
        assert_eq!(
            PlaylistId::Arena.as_str(),
            "edfef3ac-9cbe-4fa2-b949-8f29deafd483"
        );
    }

    #[test]
    fn display_matches_as_str() {
        assert_eq!(PlaylistId::Slayer.to_string(), PlaylistId::Slayer.as_str());
    }
}
