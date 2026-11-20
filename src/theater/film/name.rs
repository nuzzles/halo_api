//! Shared names retained by decoded film observations.
use std::sync::Arc;

/// A component or field name shared across observations of a film.
///
/// It serializes as a string and can be read as `&str` without copying.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct FilmName(Arc<String>);

impl FilmName {
    #[inline]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl From<String> for FilmName {
    #[inline]
    fn from(name: String) -> Self {
        Self(Arc::new(name))
    }
}

impl From<&str> for FilmName {
    #[inline]
    fn from(name: &str) -> Self {
        Self::from(name.to_owned())
    }
}

impl AsRef<str> for FilmName {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::ops::Deref for FilmName {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl std::fmt::Display for FilmName {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::FilmName;

    #[test]
    fn shared_name_is_pointer_sized_and_serializes_as_text() {
        assert_eq!(
            std::mem::size_of::<FilmName>(),
            std::mem::size_of::<usize>()
        );
        let name = FilmName::from("biped-position-component");
        assert_eq!(name.as_str(), "biped-position-component");
        let json = serde_json::to_string(&name).unwrap();
        assert_eq!(json, "\"biped-position-component\"");
        assert_eq!(serde_json::from_str::<FilmName>(&json).unwrap(), name);
    }
}
