//! Reference option names and mode-script defaults. These are not recorded
//! Bond values, resolved variant overrides, or evidence for a writer/build.

mod data;
use super::Catalog;
use data::MODES;
pub mod known_match_options;
pub use known_match_options::KnownMatchOption;

/// A variant option's lowercased-name hash, distinct from asset or service IDs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OptionHash(pub u32);

impl OptionHash {
    pub const fn raw(self) -> u32 {
        self.0
    }

    /// Hash an ASCII option name using the game's ASCII lowercase convention.
    /// This computes a hash even when the name is absent from the catalogue.
    pub fn from_name(name: &str) -> Self {
        Self(murmur3(name.to_ascii_lowercase().as_bytes()))
    }
}

/// A literal script default. It does not supply a missing recorded value.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OptionValue {
    Number(f64),
    Bool(bool),
}

/// One option in a mode's reference Options script.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct ModeOption {
    pub name: &'static str,
    /// All aliases in source order, including older names absent from hash lookup.
    pub keys: &'static [&'static str],
    /// The script's literal default; None includes nonliteral defaults.
    pub default: Option<OptionValue>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct ModeScriptOptions {
    pub prefix: &'static str,
    /// Lowercased Options script filename.
    pub script: &'static str,
    /// Options in source script order.
    pub options: &'static [ModeOption],
}

impl Catalog {
    /// Original hash/name entries in hash order. Alias lists are separate.
    pub fn option_names(self) -> impl ExactSizeIterator<Item = (OptionHash, &'static str)> {
        known_match_options::ALL
            .iter()
            .map(|option| (option.hash(), option.name()))
    }

    pub fn option_name(self, hash: OptionHash) -> Option<&'static str> {
        self.match_option(hash).map(KnownMatchOption::name)
    }

    /// Classify a recorded option hash without applying script defaults.
    pub fn match_option(self, hash: OptionHash) -> Option<KnownMatchOption> {
        known_match_options::ALL
            .binary_search_by_key(&hash.raw(), |option| option.hash().raw())
            .ok()
            .map(|i| known_match_options::ALL[i])
    }

    /// Reference scripts only; this does not select a recorded mode or apply defaults.
    pub fn mode_options(self) -> &'static [ModeScriptOptions] {
        MODES
    }
}

/// Murmur3 x86 32-bit, seed zero. Input is hashed verbatim without lowercasing.
pub fn murmur3(data: &[u8]) -> u32 {
    let (c1, c2) = (0xcc9e_2d51u32, 0x1b87_3593u32);
    let mut h = 0u32;
    let (chunks, tail) = data.as_chunks::<4>();
    for w in chunks {
        let k = u32::from_le_bytes(*w)
            .wrapping_mul(c1)
            .rotate_left(15)
            .wrapping_mul(c2);
        h = (h ^ k)
            .rotate_left(13)
            .wrapping_mul(5)
            .wrapping_add(0xe654_6b64);
    }
    if !tail.is_empty() {
        let k = tail
            .iter()
            .enumerate()
            .fold(0u32, |k, (i, &b)| k | (b as u32) << (8 * i));
        h ^= k.wrapping_mul(c1).rotate_left(15).wrapping_mul(c2);
    }
    h ^= data.len() as u32;
    h ^= h >> 16;
    h = h.wrapping_mul(0x85eb_ca6b);
    h ^= h >> 13;
    h = h.wrapping_mul(0xc2b2_ae35);
    h ^ (h >> 16)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theater::film::MatchOptionName;

    #[test]
    fn known_option_names_are_unique_sorted_and_hash_verified() {
        let scripts: &[ModeScriptOptions] = Catalog.mode_options();
        assert!(!scripts.is_empty());
        let options = known_match_options::ALL;
        assert_eq!(options.len(), 98);
        assert!(
            options
                .windows(2)
                .all(|pair| pair[0].hash().raw() < pair[1].hash().raw())
        );
        let mut names = std::collections::HashSet::new();
        for &option in options {
            assert!(names.insert(option.name()));
            assert_eq!(OptionHash::from_name(option.name()), option.hash());
            assert_eq!(Catalog.match_option(option.hash()), Some(option));
            assert_eq!(Catalog.option_name(option.hash()), Some(option.name()));
            let classified = MatchOptionName::from(option.hash());
            assert_eq!(classified, MatchOptionName::Known(option));
            assert_eq!(classified.hash(), option.hash());
            assert_eq!(classified.as_str(), Some(option.name()));
            let json = serde_json::to_value(classified).unwrap();
            assert_eq!(json, serde_json::json!({"Known": option.name()}));
            assert_eq!(
                serde_json::from_value::<MatchOptionName>(json).unwrap(),
                classified
            );
        }
        assert_eq!(
            Catalog.option_names().collect::<Vec<_>>(),
            options
                .iter()
                .map(|o| (o.hash(), o.name()))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn unknown_option_hashes_round_trip_without_defaulting() {
        for name_hash in [0, 0x8d9d0ac5, u32::MAX] {
            assert_eq!(Catalog.match_option(OptionHash(name_hash)), None);
            let name = MatchOptionName::from(OptionHash(name_hash));
            assert_eq!(name, MatchOptionName::Unknown { name_hash });
            assert_eq!(name.hash().raw(), name_hash);
            assert_eq!(name.as_str(), None);
            let json = serde_json::to_value(name).unwrap();
            assert_eq!(
                json,
                serde_json::json!({"Unknown": {"name_hash": name_hash}})
            );
            assert_eq!(
                serde_json::from_value::<MatchOptionName>(json).unwrap(),
                name
            );
        }
        for invalid in [
            serde_json::json!({"Known": "NotAnOption"}),
            serde_json::json!({"Unknown": {}}),
            serde_json::json!({"Unknown": {"name_hash": -1}}),
            serde_json::json!({"Unknown": {"name_hash": 4294967296_u64}}),
        ] {
            assert!(serde_json::from_value::<MatchOptionName>(invalid).is_err());
        }
    }
}
