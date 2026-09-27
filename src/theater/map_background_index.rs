//! Published background identity lookup. This does not infer geometry from film data.
use super::MapBackground;
use std::collections::{BTreeMap, BTreeSet};

/// Preserve explicit playlist variants; fallback is applied only by lookup.
pub fn normalize_map_identity(name: &str) -> String {
    let joined = name.split_whitespace().collect::<Vec<_>>().join("_");
    let lower: String = joined
        .chars()
        .map(|c| {
            let table = super::map_identity_unicode::LOWER;
            table
                .binary_search_by_key(&(c as u32), |&(from, _)| from)
                .ok()
                .and_then(|i| char::from_u32(table[i].1))
                .unwrap_or(c)
        })
        .collect();
    lower.strip_suffix("_map").unwrap_or(&lower).to_owned()
}

/// A native published key can contain arbitrary filename bytes. JSON publication
/// replaces malformed UTF-8 as Go does; `as_bytes` retains the original identity.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct MapBackgroundKey(Vec<u8>);
impl MapBackgroundKey {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
    pub fn as_str(&self) -> Option<&str> {
        std::str::from_utf8(&self.0).ok()
    }
}
fn native_text(mut bytes: &[u8]) -> String {
    let mut out = String::new();
    while !bytes.is_empty() {
        match std::str::from_utf8(bytes) {
            Ok(s) => {
                out.push_str(s);
                break;
            }
            Err(e) => {
                let valid = e.valid_up_to();
                out.push_str(std::str::from_utf8(&bytes[..valid]).unwrap());
                out.push('\u{fffd}');
                bytes = &bytes[valid + 1..];
            }
        }
    }
    out
}
impl serde::Serialize for MapBackgroundKey {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&native_text(&self.0))
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MapBackgroundIndex {
    identities: BTreeMap<String, MapBackgroundKey>,
    ambiguous: BTreeMap<String, Vec<MapBackgroundKey>>,
    keys: usize,
}
impl MapBackgroundIndex {
    /// Construct from successfully decoded sidecars and their published filename keys.
    /// Each input counts as a loaded sidecar, even if all its identities are excluded.
    pub fn from_backgrounds<'a>(
        backgrounds: impl IntoIterator<Item = (&'a str, &'a MapBackground)>,
    ) -> Self {
        Self::from_background_bytes(backgrounds.into_iter().map(|(key, b)| (key.as_bytes(), b)))
    }
    pub fn from_background_bytes<'a>(
        backgrounds: impl IntoIterator<Item = (&'a [u8], &'a MapBackground)>,
    ) -> Self {
        let mut claims: BTreeMap<String, BTreeSet<MapBackgroundKey>> = BTreeMap::new();
        let mut keys = 0;
        for (key, background) in backgrounds {
            keys += 1;
            let key_text = native_text(key);
            for raw in std::iter::once(key_text.as_str())
                .chain(std::iter::once(background.module.as_str()))
                .chain(background.map_names.iter().flatten().map(String::as_str))
            {
                let identity = normalize_map_identity(raw);
                if !identity.is_empty() && identity != "map" {
                    claims
                        .entry(identity)
                        .or_default()
                        .insert(MapBackgroundKey(key.to_owned()));
                }
            }
        }
        let mut out = Self {
            keys,
            ..Self::default()
        };
        for (identity, keys) in claims {
            let keys: Vec<_> = keys.into_iter().collect();
            if keys.len() == 1 {
                out.identities.insert(identity, keys[0].clone());
            } else {
                out.ambiguous.insert(identity, keys);
            }
        }
        out
    }
    /// UTF-8 convenience view; use lookup_key to retain arbitrary filename bytes.
    pub fn lookup(&self, name: &str) -> Option<&str> {
        self.lookup_key(name).and_then(MapBackgroundKey::as_str)
    }
    pub fn lookup_key(&self, name: &str) -> Option<&MapBackgroundKey> {
        let identity = normalize_map_identity(name);
        if let Some(key) = self.identities.get(&identity) {
            return Some(key);
        }
        for suffix in ["_-_ranked", "_heavies", "_firefight"] {
            if let Some(base) = identity.strip_suffix(suffix).filter(|s| !s.is_empty()) {
                return self.identities.get(base);
            }
        }
        None
    }
    pub fn ambiguous(&self) -> &BTreeMap<String, Vec<MapBackgroundKey>> {
        &self.ambiguous
    }
    pub fn identities(&self) -> usize {
        self.identities.len()
    }
    pub fn keys(&self) -> usize {
        self.keys
    }
}
