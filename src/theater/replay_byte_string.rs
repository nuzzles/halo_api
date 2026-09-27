//! Raw native string identities used by cached replay publication.
use serde::{Deserialize, Deserializer, Serialize, Serializer};
/// Bytewise identity and ordering are retained until JSON serialization.
/// JSON replaces each invalid UTF-8 byte with U+FFFD, like Go encoding/json.
/// Deserializing JSON cannot recover invalid bytes or distinguish colliding keys;
/// retain this structure or the original facts cache for lossless inspection.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReplayByteString(pub Vec<u8>);
impl Serialize for ReplayByteString {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut writer = super::facts_json_write::FactsJsonWriter::default();
        writer.string(&self.0);
        let normalized: String =
            serde_json::from_slice(&writer.bytes).map_err(serde::ser::Error::custom)?;
        serializer.serialize_str(&normalized)
    }
}
impl<'de> Deserialize<'de> for ReplayByteString {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer).map(|s| Self(s.into_bytes()))
    }
}

impl AsRef<[u8]> for ReplayByteString {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}
impl std::borrow::Borrow<[u8]> for ReplayByteString {
    fn borrow(&self) -> &[u8] {
        &self.0
    }
}
impl From<String> for ReplayByteString {
    fn from(value: String) -> Self {
        Self(value.into_bytes())
    }
}
impl From<&str> for ReplayByteString {
    fn from(value: &str) -> Self {
        Self(value.as_bytes().to_vec())
    }
}
impl ReplayByteString {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    /// Native JSON string replacement, for display/logging only. Raw identity
    /// and sorting continue to use the stored bytes.
    pub fn json_text(&self) -> String {
        let mut writer = super::facts_json_write::FactsJsonWriter::default();
        writer.string(&self.0);
        serde_json::from_slice(&writer.bytes).expect("native JSON string writer")
    }
}
